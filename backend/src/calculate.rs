use crate::weapons::{WeaponBaseStats, WeaponType, BulletType, Module, ModuleType, ModuleBonusType, Roll, RollType};
use crate::{ModuleCombinations, OptimizationCache};
use crate::modules::*;
use itertools::Itertools;
use tokio::fs::{self, File};
use tokio::io::{BufReader, BufWriter, AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::Arc;
use rayon::prelude::*;
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
pub struct OptimizationConfig {
    pub valby: bool,
    pub gley: bool,
    pub gley_duration: f64,
}

fn calculate_dps(stats: &WeaponBaseStats, weak_point_hit_chance: f64, gley: bool) -> f64 {
    let time_to_empty_mag = (stats.magazine_capacity) / (stats.fire_rate) * 60.0;
    let mut cycle_time = if gley {
        // config.gley_duration + 1.0
        9.0 + 1.0
    } else {
        time_to_empty_mag + stats.reload_time
    };
    let mut bullets_per_cycle = if gley {
        // (stats.fire_rate / 60.0 * config.gley_duration).floor()
        (stats.fire_rate / 60.0 * 9.0).floor()
    } else {
        (stats.magazine_capacity).floor()
    };
    if stats.firing_fiesta == 1.0 {
        cycle_time += 3.0;
        bullets_per_cycle += (stats.fire_rate/60.0) * 3.0
    }

    let ele_damage = (stats.base_atk * (stats.ele_enhancement) + stats.flat_ele_atk) *
        (1.0 + stats.crit_chance * (stats.crit_damage - 1.0)) *
        stats.ele_multiplier;

    let avg_damage_per_bullet = (stats.base_atk + stats.colossus_atk) *
        (1.0 + stats.crit_chance * (stats.crit_damage - 1.0)) * 
        (1.0 + weak_point_hit_chance * (stats.weak_point_damage - 1.0)) +
        ele_damage;

    let total_damage_per_cycle = avg_damage_per_bullet * stats.bullets_per_shot * bullets_per_cycle;
    
    total_damage_per_cycle / cycle_time
}

fn calculate_dpbullet(stats: &WeaponBaseStats, weak_point_hit_chance: f64) -> f64 {
    let base_damage = (stats.base_atk + stats.colossus_atk) * 
        (1.0 + stats.crit_chance * (stats.crit_damage - 1.0)) * 
        (1.0 + weak_point_hit_chance * (stats.weak_point_damage - 1.0));
    let ele_damage = (stats.base_atk * (1.0 + stats.ele_enhancement) + stats.flat_ele_atk) *
        (1.0 + stats.crit_chance * (stats.crit_damage - 1.0));
    
    base_damage + ele_damage
}

pub fn generate_module_combinations(modules: &[Module]) -> Vec<Vec<usize>> {
    let total_modules = modules.len();
    if total_modules <= 10 {
        return vec![modules.iter().enumerate().map(|(i, _)| i).collect()];
    }

    let mut valid_combinations = Vec::new();
    let module_types: Vec<ModuleType> = modules.iter().map(|m| m.module_type).collect();

    // Generate combinations of 9 modules (first module is always included)
    for combination in (1..total_modules).combinations(9) {
        let mut full_combination = vec![0]; // Always include the first module
        full_combination.extend(combination);
        
        if is_valid_combination(&full_combination, &module_types) {
            valid_combinations.push(full_combination);
        }
    }

    valid_combinations
}

fn is_valid_combination(combo: &[usize], module_types: &[ModuleType]) -> bool {
    let mut used_types = 0u64;

    for &index in combo {
        let module_type = module_types[index];
        if module_type != ModuleType::None {
            let type_bit = 1u64 << (module_type as u64);
            if used_types & type_bit != 0 {
                return false; // Duplicate non-None type found
            }
            used_types |= type_bit;
        }
    }

    true
}

fn apply_rolls_and_modules(
    base_stats: &WeaponBaseStats,
    roll_indices: &[usize],
    module_indices: &[usize],
    available_rolls: &[Roll],
    available_modules: &[Module],
    valby: bool
) -> WeaponBaseStats {
    let mut new_stats = *base_stats;
    let mut bonus_multipliers: HashMap<ModuleBonusType, f64> = HashMap::new();
    
    // Initialize bonus multipliers
    bonus_multipliers.insert(ModuleBonusType::Atk, 0.0);
    bonus_multipliers.insert(ModuleBonusType::FireRate, 0.0);
    bonus_multipliers.insert(ModuleBonusType::Crit, 0.0);
    bonus_multipliers.insert(ModuleBonusType::CritDamage, 0.0);
    bonus_multipliers.insert(ModuleBonusType::WeakPointDamage, 0.0);
    bonus_multipliers.insert(ModuleBonusType::RoundsPerMagazine, 0.0);
    bonus_multipliers.insert(ModuleBonusType::ReloadTime, 0.0);
    bonus_multipliers.insert(ModuleBonusType::EleMult, 0.0);
    bonus_multipliers.insert(ModuleBonusType::ShellCapacity, 0.0);
    
    // Accumulate roll bonuses
    for &i in roll_indices {
        let roll = &available_rolls[i];
        match roll.roll_type {
            RollType::Atk => {
                *bonus_multipliers.entry(ModuleBonusType::Atk).or_insert(0.0) += roll.value;
            },
            RollType::ElementAtk => {
                new_stats.flat_ele_atk += roll.value;
            },
            RollType::ColossusDamage => {
                new_stats.colossus_atk += roll.value;
            },
            RollType::WeakPointDamage => {
                *bonus_multipliers.entry(ModuleBonusType::WeakPointDamage).or_insert(0.0) += roll.value;
            },
            RollType::Crit => {
                *bonus_multipliers.entry(ModuleBonusType::Crit).or_insert(0.0) += roll.value;
            },
            RollType::CritDamage => {
                *bonus_multipliers.entry(ModuleBonusType::CritDamage).or_insert(0.0) += roll.value;
            },
            RollType::RoundsPerMagazine => {
                *bonus_multipliers.entry(ModuleBonusType::RoundsPerMagazine).or_insert(0.0) += roll.value;
            },
        }
    }
    
    // Accumulate module bonuses
    for &i in module_indices {
        let module = &available_modules[i];
        for effect in &module.effects {
            match effect.effect_type {
                ModuleBonusType::EleEnhancement => {
                    new_stats.ele_enhancement = effect.value;
                },
                ModuleBonusType::EleMult => {
                    new_stats.ele_multiplier += effect.value;
                },
                ModuleBonusType::FiringFiesta => {
                    new_stats.firing_fiesta = effect.value;
                },
                _ => {
                    *bonus_multipliers.entry(effect.effect_type).or_insert(0.0) += effect.value;
                }
            }
        }
    }
    
    // Apply accumulated bonuses
    if valby {
        new_stats.crit_chance += 0.2;
    }
    new_stats.base_atk *= 1.0 + bonus_multipliers[&ModuleBonusType::Atk];
    new_stats.fire_rate *= 1.0 + bonus_multipliers[&ModuleBonusType::FireRate];
    new_stats.crit_chance *= 1.0 + bonus_multipliers[&ModuleBonusType::Crit];
    new_stats.crit_damage *= 1.0 + bonus_multipliers[&ModuleBonusType::CritDamage];
    new_stats.magazine_capacity *= 1.0 + bonus_multipliers[&ModuleBonusType::RoundsPerMagazine];
    new_stats.reload_time *= 1.0 + bonus_multipliers[&ModuleBonusType::ReloadTime] * 0.8;
    new_stats.weak_point_damage *= 1.0 + bonus_multipliers[&ModuleBonusType::WeakPointDamage];
    new_stats.bullets_per_shot *= 1.0 + bonus_multipliers[&ModuleBonusType::ShellCapacity];
    new_stats.weak_point_damage += 0.5;
    new_stats.bullets_per_shot = new_stats.bullets_per_shot.floor();
    
    
    // Max crit chance is 100%
    if new_stats.crit_chance > 1.0 {
        new_stats.crit_chance = 1.0;
    }

    new_stats
}

pub async fn generate_or_load_module_combinations(modules: &[Module], bullet_type: BulletType, weapon_type: WeaponType) -> Vec<Vec<usize>> {
    let file_name = if bullet_type == BulletType::HighPowerRounds {
        format!("module_combinations/{:?}_valid_combinations.bin", weapon_type)
    } else {
        format!("module_combinations/{:?}_valid_combinations.bin", bullet_type)
    };

    println!("Attempting to load combinations from file: {}", file_name);

    if let Ok(data) = fs::read(&file_name).await {
        if let Ok(combinations) = bincode::deserialize::<ModuleCombinations>(&data) {
            println!("Loaded {} combinations from file for {:?} {:?}", combinations.combinations.len(), bullet_type, weapon_type);
            return combinations.combinations;
        }
    }
    
    if bullet_type == BulletType::HighPowerRounds {
        println!("Generating new combinations for {:?}", weapon_type);
    } else {
        println!("Generating new combinations for {:?}", bullet_type);
    }
    let combinations = generate_module_combinations(modules);
    println!("Generated {} combinations for {:?} {:?}", combinations.len(), bullet_type, weapon_type);

    let module_combinations = ModuleCombinations { combinations: combinations.clone() };
    
    // Asynchronously write to file
    let file_name_clone = file_name.clone();
    tokio::spawn(async move {
        if let Ok(data) = bincode::serialize(&module_combinations) {
            if let Err(e) = fs::write(&file_name_clone, data).await {
                eprintln!("Failed to write combinations to file: {}", e);
            } else {
                println!("Successfully wrote combinations to file: {}", file_name_clone);
            }
        } else {
            eprintln!("Failed to serialize combinations");
        }
    });
    
    combinations
}

async fn load_valid_combinations(bullet_type: BulletType, weapon_type: WeaponType) -> std::io::Result<Option<Vec<Vec<usize>>>> {
    let filename = if bullet_type != BulletType::HighPowerRounds {
        format!("{:?}_valid_combinations.bin", bullet_type)
    } else {
        format!("{:?}_valid_combinations.bin", weapon_type)
    };

    if !Path::new(&filename).exists() {
        return Ok(None);
    }

    let file = File::open(filename).await?;
    let mut reader = BufReader::new(file);

    let mut combinations = Vec::new();
    let mut buffer = [0u8; 8];

    // Read the number of combinations
    reader.read_exact(&mut buffer).await?;
    let num_combinations = u64::from_le_bytes(buffer);

    // Read each combination
    for _ in 0..num_combinations {
        reader.read_exact(&mut buffer[0..4]).await?;
        let combo_len = u32::from_le_bytes(buffer[0..4].try_into().unwrap()) as usize;

        let mut combo = Vec::with_capacity(combo_len);
        for _ in 0..combo_len {
            reader.read_exact(&mut buffer[0..4]).await?;
            combo.push(u32::from_le_bytes(buffer[0..4].try_into().unwrap()) as usize);
        }

        combinations.push(combo);
    }

    Ok(Some(combinations))
}

async fn save_valid_combinations(combinations: &[Vec<usize>], bullet_type: BulletType, weapon_type: WeaponType) -> std::io::Result<()> {
    let filename = if bullet_type != BulletType::HighPowerRounds {
        format!("{:?}_valid_combinations.bin", bullet_type)
    } else {
        format!("{:?}_valid_combinations.bin", weapon_type)
    };
    let file = File::create(filename).await?;
    let mut writer = BufWriter::new(file);

    // Write the number of combinations
    writer.write_all(&(combinations.len() as u64).to_le_bytes()).await?;

    // Write each combination
    for combo in combinations {
        writer.write_all(&(combo.len() as u32).to_le_bytes()).await?;
        for &index in combo {
            writer.write_all(&(index as u32).to_le_bytes()).await?;
        }
    }

    writer.flush().await?;
    Ok(())
}

// // progress version?
// use futures::stream::{self, StreamExt};
// use tokio::sync::Mutex as TokioMutex;
// pub async fn optimize_weapon(
//     cache: &OptimizationCache,
//     base_stats: WeaponBaseStats,
//     available_rolls: Vec<Roll>,
//     available_modules: Vec<Module>,
//     weak_point_hit_chance: f64,
//     config: OptimizationConfig,
//     progress_sender: mpsc::Sender<f32>,
// ) -> (f64, Vec<Roll>, Vec<Module>) {
//     println!("Starting optimization for {:?}", base_stats.weapon_type);
//     let start_time = std::time::Instant::now();

//     let best_dps = Arc::new(AtomicU64::new(0));
//     let best_combo = Arc::new(TokioMutex::new((Vec::new(), Vec::new())));
    
//     let roll_combinations: Vec<Vec<usize>> = (0..available_rolls.len()).combinations(4).collect();
//     println!("Generated {} roll combinations", roll_combinations.len());

//     let valid_module_combinations = cache.get_or_compute_module_combinations(base_stats.bullet_type, base_stats.weapon_type).await;
//     println!("Retrieved {} valid module combinations", valid_module_combinations.len());

//     let total_combinations = roll_combinations.len() * valid_module_combinations.len();
//     println!("Total combinations to evaluate: {}", total_combinations);

//     let combinations_tried = Arc::new(AtomicUsize::new(0));

//     let base_stats = Arc::new(base_stats);
//     let available_rolls = Arc::new(available_rolls);
//     let available_modules = Arc::new(available_modules);
//     let config = Arc::new(config);

//     let chunk_size = 1000;
//     let num_chunks = (roll_combinations.len() + chunk_size - 1) / chunk_size;

//     stream::iter(0..num_chunks)
//         .for_each_concurrent(None, |chunk_index| {
//             let start = chunk_index * chunk_size;
//             let end = (start + chunk_size).min(roll_combinations.len());
//             let chunk = &roll_combinations[start..end];
            
//             let best_dps = best_dps.clone();
//             let best_combo = best_combo.clone();
//             let combinations_tried = combinations_tried.clone();
//             let progress_sender = progress_sender.clone();
//             let base_stats = base_stats.clone();
//             let available_rolls = available_rolls.clone();
//             let available_modules = available_modules.clone();
//             let config = config.clone();
//             let valid_module_combinations = valid_module_combinations.clone();

//             async move {
//                 for roll_combo in chunk {
//                     for module_indices in &valid_module_combinations {
//                         let final_stats = apply_rolls_and_modules(
//                             &base_stats,
//                             roll_combo,
//                             module_indices,
//                             &available_rolls,
//                             &available_modules,
//                             config.valby,
//                         );

//                         let final_dps = if base_stats.weapon_type == WeaponType::SniperRifle {
//                             calculate_dpbullet(&final_stats, weak_point_hit_chance)
//                         } else {
//                             calculate_dps(&final_stats, weak_point_hit_chance, config.gley)
//                         };

//                         let current_best = best_dps.load(Ordering::Relaxed);
//                         if (final_dps * 1e6) as u64 > current_best {
//                             best_dps.store((final_dps * 1e6) as u64, Ordering::Relaxed);
//                             let mut best = best_combo.lock().await;
//                             *best = (
//                                 roll_combo.iter().map(|&i| available_rolls[i].clone()).collect(),
//                                 module_indices.iter().map(|&i| available_modules[i].clone()).collect()
//                             );
//                         }

//                         let tried = combinations_tried.fetch_add(1, Ordering::Relaxed);
//                         if tried % 1_000_000 == 0 {
//                             let progress = tried as f32 / total_combinations as f32;
//                             let _ = progress_sender.try_send(progress);
//                             println!("Progress: {:.2}%, Time elapsed: {:?}", progress * 100.0, start_time.elapsed());
//                         }
//                     }
//                 }
//             }
//         })
//         .await;

//     let (best_rolls, best_modules) = best_combo.lock().await.clone();
//     let final_dps = (best_dps.load(Ordering::Relaxed) as f64) / 1e6;
//     println!("Optimization complete. Best DPS: {}, Total time: {:?}", final_dps, start_time.elapsed());

//     (final_dps, best_rolls, best_modules)
// }

// working version
pub async fn optimize_weapon(
    cache: &OptimizationCache,
    base_stats: WeaponBaseStats,
    available_rolls: Vec<Roll>,
    available_modules: Vec<Module>,
    weak_point_hit_chance: f64,
    config: OptimizationConfig,
) -> (f64, Vec<Roll>, Vec<Module>) {
    println!("Starting optimization for {:?}", base_stats.weapon_type);
    let start_time = std::time::Instant::now();

    let best_dps = Arc::new(AtomicU64::new(0));
    let best_combo = Arc::new(Mutex::new((Vec::new(), Vec::new())));
    
    let roll_combinations: Vec<Vec<usize>> = (0..available_rolls.len()).combinations(4).collect();
    println!("Generated {} roll combinations", roll_combinations.len());

    let valid_module_combinations = cache.get_or_compute_module_combinations(base_stats.bullet_type, base_stats.weapon_type).await;
    println!("Retrieved {} valid module combinations", valid_module_combinations.len());

    let total_combinations = roll_combinations.len() * valid_module_combinations.len();
    println!("Total combinations to evaluate: {}", total_combinations);

    let base_stats = Arc::new(base_stats);
    let available_rolls = Arc::new(available_rolls);
    let available_modules = Arc::new(available_modules);
    let config = Arc::new(config);

    roll_combinations.par_iter().for_each(|roll_combo| {
        for module_indices in &valid_module_combinations {
            let final_stats = apply_rolls_and_modules(
                &base_stats,
                roll_combo,
                module_indices,
                &available_rolls,
                &available_modules,
                config.valby,
            );

            let final_dps = if base_stats.weapon_type == WeaponType::SniperRifle {
                calculate_dpbullet(&final_stats, weak_point_hit_chance)
            } else {
                calculate_dps(&final_stats, weak_point_hit_chance, config.gley)
            };

            let current_best = best_dps.load(Ordering::Relaxed);
            if (final_dps * 1e6) as u64 > current_best {
                best_dps.store((final_dps * 1e6) as u64, Ordering::Relaxed);
                let mut best = best_combo.blocking_lock();
                *best = (
                    roll_combo.iter().map(|&i| available_rolls[i].clone()).collect(),
                    module_indices.iter().map(|&i| available_modules[i].clone()).collect()
                );
            }
        }
    });

    let (best_rolls, best_modules) = best_combo.lock().await.clone();
    let final_dps = (best_dps.load(Ordering::Relaxed) as f64) / 1e6;
    println!("Optimization complete. Best DPS: {}, Total time: {:?}", final_dps, start_time.elapsed());

    (final_dps, best_rolls, best_modules)
}

pub fn get_available_modules(bullet_type: BulletType, weapon_type: WeaponType) -> Vec<Module> {
    match bullet_type {
        BulletType::GeneralRounds => general_rounds_modules::get_modules(),
        BulletType::SpecialRounds => special_rounds_modules::get_modules(),
        BulletType::ImpactRounds => impact_rounds_modules::get_modules(),
        BulletType::HighPowerRounds => match weapon_type {
            WeaponType::SniperRifle => sniper_modules::get_modules(),
            WeaponType::Shotgun => shotgun_modules::get_modules(),
            WeaponType::Launcher => launcher_modules::get_modules(),
            _ => panic!("Invalid WeaponType with HighPowerRounds")
        }
    }
}

pub fn get_available_rolls(weapon_type: WeaponType) -> Vec<Roll> {
    match weapon_type {
        WeaponType::Handgun => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1232.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.133 },
            Roll { roll_type: RollType::CritDamage, value: 0.368 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 2465.0 },
        ],
        WeaponType::HandCannon => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 5838.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.114 },
            Roll { roll_type: RollType::CritDamage, value: 0.215 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 11676.0 },
        ],
        WeaponType::Shotgun => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1806.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.098 },
            Roll { roll_type: RollType::CritDamage, value: 0.111 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 3612.0 },
        ],
        WeaponType::SubmachineGun => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1226.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.133 },
            Roll { roll_type: RollType::CritDamage, value: 0.368 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 2453.0 },
        ],
        WeaponType::MachineGun => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1702.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.143 },
            Roll { roll_type: RollType::CritDamage, value: 0.411 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 3404.0 },
        ],
        WeaponType::AssaultRifle => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1679.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.152 },
            Roll { roll_type: RollType::CritDamage, value: 0.449 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 3357.0 },
        ],
        WeaponType::TacticalRifle => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1731.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.122 },
            Roll { roll_type: RollType::CritDamage, value: 0.247 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 3462.0 },
        ],
        WeaponType::ScoutRifle => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 4747.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.133 },
            Roll { roll_type: RollType::CritDamage, value: 0.327 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 9494.0 },
        ],
        WeaponType::SniperRifle => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 15196.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.108 },
            Roll { roll_type: RollType::CritDamage, value: 0.184 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 30393.0 },
        ],
        WeaponType::Launcher => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 14828.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.108 },
            Roll { roll_type: RollType::CritDamage, value: 0.184 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 29654.0 },
        ],
        WeaponType::BeamRifle => vec![
            Roll { roll_type: RollType::Atk, value: 0.122 },
            Roll { roll_type: RollType::ElementAtk, value: 1986.0 },
            Roll { roll_type: RollType::WeakPointDamage, value: 0.12 },
            Roll { roll_type: RollType::Crit, value: 0.133 },
            Roll { roll_type: RollType::CritDamage, value: 0.368 },
            Roll { roll_type: RollType::RoundsPerMagazine, value: 0.12 },
            Roll { roll_type: RollType::ColossusDamage, value: 3972.0 },
        ]
    }
}
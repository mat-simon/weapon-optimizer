use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::{collections::HashMap, sync::atomic::AtomicU64};
use std::sync::Arc;
use tokio::sync::Mutex;
use itertools::Itertools;
use rayon::prelude::*;

use crate::weapons::{Module, ModuleBonusType, ModuleType, Roll, RollType, WeaponBaseStats, WeaponType};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ModuleCombinations {
    pub combinations: Vec<Vec<usize>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OptimizationConfig {
    pub valby: bool,
    pub gley: bool,
    pub gley_duration: f64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        OptimizationConfig {
            valby: false,
            gley: false,
            gley_duration: 9.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OptimizationResult {
    pub max_dps: f64,
    pub best_rolls: Vec<Roll>,
    pub best_modules: Vec<Module>,
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
        let multiplier = (cycle_time / 10.0).min(1.0);
        cycle_time += 3.0 * multiplier;
        bullets_per_cycle += (stats.fire_rate / 60.0) * 3.0 * multiplier;
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
    // new_stats.bullets_per_shot = new_stats.bullets_per_shot.floor();
    
    
    // Max crit chance is 100%
    if new_stats.crit_chance > 1.0 {
        new_stats.crit_chance = 1.0;
    }

    new_stats
}

pub async fn optimize_weapon(
    base_stats: WeaponBaseStats,
    available_rolls: Vec<Roll>,
    available_modules: Vec<Module>,
    module_combinations: Vec<Vec<usize>>,
    weak_point_hit_chance: f64,
    config: OptimizationConfig,
) -> OptimizationResult {
    println!("Starting optimization for {:?}", base_stats.weapon_type);
    let start_time = std::time::Instant::now();

    let best_dps = Arc::new(AtomicU64::new(0));
    let best_combo = Arc::new(Mutex::new((Vec::new(), Vec::new())));
    
    let roll_combinations: Vec<Vec<usize>> = (0..available_rolls.len()).combinations(4).collect();
    println!("Generated {} roll combinations", roll_combinations.len());

    println!("Using {} pre-calculated module combinations", module_combinations.len());

    let total_combinations = roll_combinations.len() * module_combinations.len();
    println!("Total combinations to evaluate: {}", total_combinations);

    let base_stats = Arc::new(base_stats);
    let available_rolls = Arc::new(available_rolls);
    let available_modules = Arc::new(available_modules);
    let config = Arc::new(config);

    roll_combinations.par_iter().for_each(|roll_combo| {
        for module_indices in &module_combinations {
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

    OptimizationResult {
        max_dps: final_dps,
        best_rolls,
        best_modules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson;

    #[test]
    fn test_optimization_result_serialization() {
        let result = OptimizationResult {
            max_dps: 1000.0,
            best_rolls: vec![/* ... */],
            best_modules: vec![/* ... */],
        };

        let bson = bson::to_bson(&result).unwrap();
        let doc = bson.as_document().unwrap();

        assert!(doc.contains_key("max_dps"));
        assert!(doc.contains_key("best_rolls"));
        assert!(doc.contains_key("best_modules"));

        let deserialized: OptimizationResult = bson::from_bson(bson).unwrap();
        assert_eq!(result.max_dps, deserialized.max_dps);
    }
}
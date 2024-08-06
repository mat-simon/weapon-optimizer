use mongodb::{Client, options::ClientOptions, bson::doc};
use mongodb::{options::ServerApi, options::ServerApiVersion};
use structopt::StructOpt;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use dotenv::dotenv;
use log::info;
use std::hash::{Hash, Hasher};
use tokio::time::Duration;
use std::fs::{File, create_dir_all};
use std::io::{Write, Read};
use std::path::Path;
use bincode;
use reqwest;

mod weapons;
mod calculate;
mod modules;

use crate::weapons::{Weapon, WeaponType, BulletType, WeaponBaseStats, get_available_modules, get_available_rolls};
use crate::calculate::{OptimizationConfig, OptimizationResult, ModuleCombinations, generate_module_combinations, optimize_weapon};

#[derive(Debug, Clone, PartialEq)]
struct OptimizationKey {
    weapon: String,
    weak_point_hit_chance: f64,
    valby: bool,
}

impl Eq for OptimizationKey {}

impl Hash for OptimizationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.weapon.hash(state);
        let bits: u64 = self.weak_point_hit_chance.to_bits();
        bits.hash(state);
        self.valby.hash(state);
    }
}

#[derive(Debug)]
struct WorkerResults {
    module_combinations: HashMap<String, ModuleCombinations>,
    weapon_results: HashMap<OptimizationKey, OptimizationResult>,
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    UpdateWeapons { names: Vec<String> },
    UpdateWeaponType { weapon_type: String },
    UpdateBulletType { bullet_type: String },
    UpdateModules { module_type: String },
    UpdateAll,
}

async fn create_mongo_client() -> Result<Client, Box<dyn std::error::Error>> {
    dotenv().ok();
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    info!("Connecting to MongoDB...");
    
    let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
    
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    
    client_options.connect_timeout = Some(Duration::from_secs(30));
    client_options.server_selection_timeout = Some(Duration::from_secs(30));
    client_options.max_pool_size = Some(50);
    client_options.min_pool_size = Some(1);
    client_options.retry_writes = Some(true);
    client_options.retry_reads = Some(true);

    let client = Client::with_options(client_options)?;

    // Test the connection
    client.database("admin").run_command(doc! {"ping": 1}, None).await?;

    info!("Connected to MongoDB successfully!");
    Ok(client)
}

fn generate_all_module_combinations(module_type: Option<&str>) -> HashMap<String, ModuleCombinations> {
    info!("Generating module combinations...");
    let mut combinations = HashMap::new();
    let module_types = [
        ("GeneralRounds", BulletType::GeneralRounds, None),
        ("SpecialRounds", BulletType::SpecialRounds, None),
        ("ImpactRounds", BulletType::ImpactRounds, None),
        ("SniperRifle", BulletType::HighPowerRounds, Some(WeaponType::SniperRifle)),
        ("Shotgun", BulletType::HighPowerRounds, Some(WeaponType::Shotgun)),
        ("Launcher", BulletType::HighPowerRounds, Some(WeaponType::Launcher)),
    ];

    for (key, bullet_type, weapon_type) in module_types.iter() {
        if module_type.is_none() || module_type == Some(key) {
            let modules = get_available_modules(*bullet_type, weapon_type.unwrap_or(WeaponType::AssaultRifle));
            let module_combinations = generate_module_combinations(&modules);
            combinations.insert(key.to_string(), ModuleCombinations { combinations: module_combinations });
        }
    }

    if let Err(e) = save_module_combinations(&combinations, "module_combinations") {
        eprintln!("Failed to save module combinations: {}", e);
    }

    combinations
}

async fn store_results_in_db(db: &mongodb::Database, results: WorkerResults) -> Result<(), Box<dyn std::error::Error>> {
    // Store module combinations
    let module_collection = db.collection::<ModuleCombinations>("module_combinations");
    for (key, combinations) in results.module_combinations {
        module_collection.update_one(
            doc! { "key": key },
            doc! { "$set": mongodb::bson::to_bson(&combinations).unwrap() },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        ).await?;
    }

    // Store weapon results
    let weapon_collection = db.collection::<OptimizationResult>("weapon_results");
    for (key, result) in results.weapon_results {
        weapon_collection.update_one(
            doc! {
                "weapon": key.weapon,
                "weak_point_hit_chance": key.weak_point_hit_chance,
                "valby": key.valby,
            },
            doc! { "$set": mongodb::bson::to_bson(&result).unwrap() },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        ).await?;
    }

    Ok(())
}

async fn update_weapons(db: &mongodb::Database, module_combinations: &HashMap<String, ModuleCombinations>, names: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for name in names {
        if let Ok(weapon) = Weapon::from_str(&name) {
            update_weapon(db, module_combinations, weapon, &OptimizationConfig::default()).await?;
            update_weapon(db, module_combinations, weapon, &OptimizationConfig { valby: true, ..Default::default() }).await?;
        }
    }
    Ok(())
}

async fn update_bullet_type(db: &mongodb::Database, module_combinations: &HashMap<String, ModuleCombinations>, bullet_type: String) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(bt) = BulletType::from_str(&bullet_type) {
        for weapon in Weapon::all().iter().filter(|w| WeaponBaseStats::get(**w).bullet_type == bt) {
            update_weapon(db, module_combinations, *weapon, &OptimizationConfig::default()).await?;
            update_weapon(db, module_combinations, *weapon, &OptimizationConfig { valby: true, ..Default::default() }).await?;
        }
    }
    Ok(())
}

async fn update_weapon_type(db: &mongodb::Database, module_combinations: &HashMap<String, ModuleCombinations>, weapon_type: String) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(wt) = WeaponType::from_str(&weapon_type) {
        for weapon in Weapon::all().iter().filter(|w| WeaponBaseStats::get(**w).weapon_type == wt) {
            update_weapon(db, module_combinations, *weapon, &OptimizationConfig::default()).await?;
            update_weapon(db, module_combinations, *weapon, &OptimizationConfig { valby: true, ..Default::default() }).await?;
        }
    }
    Ok(())
}

async fn update_modules(module_combinations: &HashMap<String, ModuleCombinations>, module_type: String) -> Result<(), Box<dyn std::error::Error>> {
    let combinations = module_combinations.get(&module_type)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Module combinations not found for {}", module_type)))?;

    // Save the updated combinations
    let mut updated_combinations = HashMap::new();
    updated_combinations.insert(module_type.clone(), combinations.clone());
    save_module_combinations(&updated_combinations, "module_combinations")?;

    info!("Updated module combinations for {}", module_type);
    info!("Number of combinations: {}", combinations.combinations.len());

    Ok(())
}

async fn update_all(db: &mongodb::Database, module_combinations: &HashMap<String, ModuleCombinations>) -> Result<(), Box<dyn std::error::Error>> {
    let mut results = WorkerResults {
        module_combinations: module_combinations.clone(),
        weapon_results: HashMap::new(),
    };

    // Weapon DPS simulations
    for weapon in Weapon::all() {
        let base_stats = WeaponBaseStats::get(*weapon);
        let available_rolls = get_available_rolls(base_stats.weapon_type);
        
        // Determine which module combinations to use
        let module_key = match (base_stats.bullet_type, base_stats.weapon_type) {
            (_, WeaponType::SniperRifle) => "SniperRifle",
            (_, WeaponType::Shotgun) => "Shotgun",
            (_, WeaponType::Launcher) => "Launcher",
            (BulletType::ImpactRounds, _) => "ImpactRounds",
            (BulletType::GeneralRounds, _) => "GeneralRounds",
            (BulletType::SpecialRounds, _) => "SpecialRounds",
            _ => panic!("Unexpected combination of bullet type and weapon type"),
        };
        
        let module_combinations = results.module_combinations.get(module_key)
            .expect("Module combinations not found")
            .combinations.clone();

        let available_modules = get_available_modules(base_stats.bullet_type, base_stats.weapon_type);
        
        for valby in [false, true] {
            let config = OptimizationConfig { valby, ..Default::default() };
            for &weak_point_hit_chance in &[0.0, 0.25, 0.33, 0.5, 0.67, 0.75, 1.0] {
                let result = optimize_weapon(
                    base_stats,
                    available_rolls.clone(),
                    available_modules.clone(),
                    module_combinations.clone(),
                    weak_point_hit_chance,
                    config.clone(),
                ).await;

                let key = OptimizationKey {
                    weapon: weapon.to_string(),
                    weak_point_hit_chance,
                    valby,
                };
                results.weapon_results.insert(key, result);
            }
        }
    }

    store_results_in_db(db, results).await?;

    Ok(())
}

async fn update_weapon(db: &mongodb::Database, module_combinations: &HashMap<String, ModuleCombinations>, weapon: Weapon, config: &OptimizationConfig) -> Result<(), Box<dyn std::error::Error>> {
    let base_stats = WeaponBaseStats::get(weapon);
    let available_rolls = get_available_rolls(base_stats.weapon_type);
    
    let module_key = match (base_stats.bullet_type, base_stats.weapon_type) {
        (_, WeaponType::SniperRifle) => "SniperRifle",
        (_, WeaponType::Shotgun) => "Shotgun",
        (_, WeaponType::Launcher) => "Launcher",
        (BulletType::ImpactRounds, _) => "ImpactRounds",
        (BulletType::GeneralRounds, _) => "GeneralRounds",
        (BulletType::SpecialRounds, _) => "SpecialRounds",
        _ => panic!("Unexpected combination of bullet type and weapon type"),
    };

    let weapon_module_combinations = module_combinations.get(module_key)
        .expect("Module combinations not found")
        .combinations.clone();

    let available_modules = get_available_modules(base_stats.bullet_type, base_stats.weapon_type);

    let collection = db.collection::<OptimizationResult>("weapon_results");

    for weak_point_hit_chance in [0.25, 0.5, 1.0].iter() {
        let result = optimize_weapon(
            base_stats,
            available_rolls.clone(),
            available_modules.clone(),
            weapon_module_combinations.clone(),
            *weak_point_hit_chance,
            config.clone(),
        ).await;

        collection.update_one(
            doc! {
                "weapon": weapon.to_string(),
                "weak_point_hit_chance": weak_point_hit_chance,
                "valby": config.valby,
            },
            doc! { "$set": mongodb::bson::to_bson(&result).unwrap() },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        ).await?;
    }

    Ok(())
}

fn save_module_combinations(combinations: &HashMap<String, ModuleCombinations>, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(dir)?;
    for (key, combination) in combinations {
        let path = Path::new(dir).join(format!("{}.bin", key));
        let encoded: Vec<u8> = bincode::serialize(&combination)?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
    }
    Ok(())
}

fn load_module_combinations(dir: &str) -> Result<HashMap<String, ModuleCombinations>, Box<dyn std::error::Error>> {
    let mut combinations = HashMap::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("bin") {
            let mut file = File::open(&path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let combination: ModuleCombinations = bincode::deserialize(&buffer)?;
            let key = path.file_stem().and_then(|s| s.to_str()).unwrap().to_string();
            combinations.insert(key, combination);
        }
    }
    Ok(combinations)
}

async fn clear_api_cache(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Clearing api cache...");
    let client = reqwest::Client::new();
    let response = client.post("https://weapon-optimizer-api.onrender.com/clear-cache-and-fetch")
        .query(&[("target", target)])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to clear cache: {}", response.status())
        )));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();
    info!("Starting worker...");
    let opts = Cli::from_args();
    info!("Received command: {:?}", opts.cmd);

    let client = create_mongo_client().await?;
    let db = client.database("weapon_optimizer");

    let module_combinations = match &opts.cmd {
        Command::UpdateAll => generate_all_module_combinations(None),
        Command::UpdateModules { module_type } => generate_all_module_combinations(Some(module_type)),
        _ => load_module_combinations("module_combinations")?,
    };

    match opts.cmd {
        Command::UpdateAll => {
            update_all(&db, &module_combinations).await?;
            // clear_api_cache("all").await?;
        },
        Command::UpdateWeapons { names } => {
            update_weapons(&db, &module_combinations, names.clone()).await?;
            for name in names {
                // clear_api_cache(&name).await?;
            }
        },
        Command::UpdateWeaponType { weapon_type } => {
            update_weapon_type(&db, &module_combinations, weapon_type.clone()).await?;
            // clear_api_cache(&weapon_type).await?;
        },
        Command::UpdateBulletType { bullet_type } => {
            update_bullet_type(&db, &module_combinations, bullet_type.clone()).await?;
            // clear_api_cache(&bullet_type).await?;
        },
        Command::UpdateModules { module_type } => {
            update_modules(&module_combinations, module_type.clone()).await?;
            // clear_api_cache(&module_type).await?;
        },
    }

    Ok(())
}
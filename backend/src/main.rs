use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::future::Future;
use std::sync::Arc;
use tokio::{fs, task};
use tokio::sync::{RwLock, Mutex};
use crate::weapons::{Weapon, WeaponBaseStats, Module, BulletType, WeaponType};
use crate::calculate::{optimize_weapon, get_available_rolls, get_available_modules, OptimizationConfig, generate_or_load_module_combinations};
use tokio::sync::Semaphore;
mod weapons;
mod calculate;
mod modules {
    pub mod general_rounds_modules;
    pub mod special_rounds_modules;
    pub mod impact_rounds_modules;
    pub mod sniper_modules;
    pub mod shotgun_modules;
    pub mod launcher_modules;
}

#[derive(Clone, Serialize, Deserialize)]
struct ModuleCombinations {
    combinations: Vec<Vec<usize>>,
}

#[derive(Deserialize, Debug)]
struct OptimizationRequest {
    weapon: Weapon,
    weak_point_hit_chance: f64,
    valby: bool,
    gley: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct OptimizationResult {
    max_dps: f64,
    best_rolls: Vec<weapons::Roll>,
    best_modules: Vec<Module>,
}

struct OptimizationCache {
    module_combinations: RwLock<HashMap<(BulletType, Option<WeaponType>), ModuleCombinations>>,
    results: RwLock<HashMap<String, OptimizationResult>>,
}

impl OptimizationCache {
    fn new() -> Self {
        OptimizationCache {
            module_combinations: RwLock::new(HashMap::new()),
            results: RwLock::new(HashMap::new()),
        }
    }

    async fn get_or_compute_module_combinations(
        &self,
        bullet_type: BulletType,
        weapon_type: WeaponType,
    ) -> Vec<Vec<usize>> {
        let mut cache = self.module_combinations.write().await;
        let key = if bullet_type == BulletType::HighPowerRounds {
            (bullet_type, Some(weapon_type))
        } else {
            (bullet_type, None)
        };

        if let Some(combinations) = cache.get(&key) {
            combinations.combinations.clone()
        } else {
            let modules = get_available_modules(bullet_type, weapon_type);
            let combinations = generate_or_load_module_combinations(&modules, bullet_type, weapon_type).await;
            
            let module_combinations = ModuleCombinations { combinations: combinations.clone() };
            cache.insert(key, module_combinations);
            
            combinations
        }
    }

    async fn get_or_compute_result<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> OptimizationResult
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = OptimizationResult>,
    {
        let mut cache = self.results.write().await;
        if let Some(result) = cache.get(key) {
            result.clone()
        } else {
            let result = compute().await;
            cache.insert(key.to_string(), result.clone());
            result
        }
    }
}

struct AppState {
    cache: Arc<OptimizationCache>,
    module_combinations_ready: Arc<RwLock<bool>>,
    request_semaphore: Arc<Semaphore>,
}

async fn optimize_weapon_handler(req: web::Json<OptimizationRequest>, state: web::Data<AppState>) -> impl Responder {
    let _permit = state.request_semaphore.acquire().await.unwrap();
    println!("Received optimization request: {:?}", req);
    if !*state.module_combinations_ready.read().await {
        return HttpResponse::ServiceUnavailable().json(json!({"error": "Module combinations not ready"}));
    }

    let cache = &state.cache;
    let cache_key = format!("{:?}_{}_{}_{}", req.weapon, req.weak_point_hit_chance, req.valby, req.gley);

    let result = cache.get_or_compute_result(&cache_key, || async {
        let base_stats = WeaponBaseStats::get(req.weapon);
        let available_rolls = get_available_rolls(base_stats.weapon_type);
        let available_modules = get_available_modules(base_stats.bullet_type, base_stats.weapon_type);
        let config = OptimizationConfig {
            valby: req.valby,
            gley: req.gley,
            gley_duration: 9.0,
        };

        let (max_dps, best_rolls, best_modules) = optimize_weapon(
            cache,
            base_stats,
            available_rolls,
            available_modules,
            req.weak_point_hit_chance,
            config,
        ).await;

        OptimizationResult {
            max_dps,
            best_rolls,
            best_modules,
        }
    }).await;

    HttpResponse::Ok().json(result)
}

async fn get_weapons() -> impl Responder {
    HttpResponse::Ok().json(Weapon::all())
}

async fn pre_compute_module_combinations(cache: Arc<OptimizationCache>, ready: Arc<RwLock<bool>>) {
    println!("Starting pre-computation of module combinations...");
    if let Err(e) = create_dir_all("module_combinations") {
        eprintln!("Failed to create module_combinations directory: {}", e);
        return;
    }
    let bullet_types = [BulletType::GeneralRounds, BulletType::SpecialRounds, BulletType::ImpactRounds];
    let high_power_weapon_types = [WeaponType::SniperRifle, WeaponType::Shotgun, WeaponType::Launcher];

    let mut tasks = Vec::new();

    for &bt in &bullet_types {
        let cache = cache.clone();
        tasks.push(task::spawn(async move {
            println!("Computing combinations for {:?}", bt);
            let combinations = cache.get_or_compute_module_combinations(bt, WeaponType::AssaultRifle).await;
            println!("Finished computing combinations for {:?}", bt);
            (bt, None, combinations)
        }));
    }

    for &wt in &high_power_weapon_types {
        let cache = cache.clone();
        tasks.push(task::spawn(async move {
            println!("Computing combinations for {:?}", wt);
            let combinations = cache.get_or_compute_module_combinations(BulletType::HighPowerRounds, wt).await;
            println!("Finished computing combinations for {:?}", wt);
            (BulletType::HighPowerRounds, Some(wt), combinations)
        }));
    }

    let results = join_all(tasks).await;

    println!("All module combinations computed. Writing to files...");

    for result in results {
        match result {
            Ok((bullet_type @ (BulletType::GeneralRounds | BulletType::SpecialRounds | BulletType::ImpactRounds), _, combinations)) => {
                let file_name = format!("module_combinations/{:?}_valid_combinations.bin", bullet_type);
                tokio::spawn(async move {
                    if let Ok(data) = bincode::serialize(&ModuleCombinations { combinations }) {
                        if let Err(e) = fs::write(&file_name, data).await {
                            eprintln!("Failed to write file {}: {}", file_name, e);
                        } else {
                            println!("Successfully wrote file: {}", file_name);
                        }
                    } else {
                        eprintln!("Failed to serialize combinations for {:?}", bullet_type);
                    }
                });
            }
            Ok((BulletType::HighPowerRounds, Some(wt), combinations)) => {
                let file_name = format!("module_combinations/{:?}_valid_combinations.bin", wt);
                tokio::spawn(async move {
                    if let Ok(data) = bincode::serialize(&ModuleCombinations { combinations }) {
                        if let Err(e) = fs::write(&file_name, data).await {
                            eprintln!("Failed to write file {}: {}", file_name, e);
                        } else {
                            println!("Successfully wrote file: {}", file_name);
                        }
                    } else {
                        eprintln!("Failed to serialize combinations for {:?}", wt);
                    }
                });
            }
            Err(e) => eprintln!("Task failed: {:?}", e),
            _ => eprintln!("Unexpected result format"),
        }
    }

    println!("All module combinations written to files.");
    {
        let mut ready_guard = ready.write().await;
        *ready_guard = true;
        println!("Module combinations ready flag set to true.");
    }
    println!("Pre-computation completed successfully.");
}

async fn delete_module_combination_files() -> std::io::Result<()> {
    let mut dir = fs::read_dir("module_combinations").await?;
    while let Some(entry) = dir.next_entry().await? {
        if entry.file_type().await?.is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "bin" {
                    fs::remove_file(entry.path()).await?;
                }
            }
        }
    }
    Ok(())
}

async fn delete_module_combinations_handler() -> impl Responder {
    match delete_module_combination_files().await {
        Ok(_) => HttpResponse::Ok().body("Module combination files deleted successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to delete files: {}", e)),
    }
}

async fn get_module_combinations_status(state: web::Data<AppState>) -> impl Responder {
    println!("Received request for module combinations status");
    let status = *state.module_combinations_ready.read().await;
    println!("Module combinations status: {}", status);
    HttpResponse::Ok().json(json!({"ready": status}))
}

async fn server_ready_handler() -> impl Responder {
    HttpResponse::Ok().json(json!({"ready": true}))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server initialization...");

    let cache = Arc::new(OptimizationCache::new());
    let module_combinations_ready = Arc::new(RwLock::new(false));

    println!("Pre-computing module combinations...");
    pre_compute_module_combinations(cache.clone(), module_combinations_ready.clone()).await;

    // Wait until module combinations are ready
    while !*module_combinations_ready.read().await {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("Module combinations loaded. Starting server...");

    let app_state = web::Data::new(AppState {
        cache: cache.clone(),
        module_combinations_ready: module_combinations_ready.clone(),
        request_semaphore: Arc::new(Semaphore::new(5)),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(web::resource("/optimize").route(web::post().to(optimize_weapon_handler)))
            .service(web::resource("/weapons").route(web::get().to(get_weapons)))
            .service(web::resource("/server-ready").route(web::get().to(server_ready_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use mongodb::bson::doc;
use mongodb::{Client, options::ClientOptions, options::ServerApi, options::ServerApiVersion};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use dotenv::dotenv;
use std::env;
use log::{error, info};
use futures::TryStreamExt;
use serde_json::json;

pub mod weapons;
pub mod calculate;
pub mod modules;

use crate::weapons::Weapon;
use crate::calculate::OptimizationResult;

#[derive(Deserialize, Debug)]
struct OptimizationRequest {
    weapon: String,
    weak_point_hit_chance: f64,
    valby: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct WeaponResultDocument {
    weapon: String,
    weak_point_hit_chance: f64,
    valby: bool,
    #[serde(flatten)]
    result: OptimizationResult,
}

struct AppState {
    db: mongodb::Database,
    weapon_results: Arc<RwLock<HashMap<String, WeaponResultDocument>>>,
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Weapon Optimizer API is running")
}

async fn optimize_weapon_handler(
    req: web::Json<OptimizationRequest>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let cache_key = format!("{}_{}_{}", req.weapon, req.weak_point_hit_chance, req.valby);

    let weapon_results = data.weapon_results.read().await;
    if let Some(doc) = weapon_results.get(&cache_key) {
        return HttpResponse::Ok().json(&doc.result);
    }

    HttpResponse::NotFound().finish()
}

async fn get_weapons() -> HttpResponse {
    let weapons: Vec<String> = Weapon::all().iter().map(|w| w.to_string()).collect();
    info!("Returning {} weapons", weapons.len());
    HttpResponse::Ok().json(weapons)
}

async fn get_weapon_data(data: web::Data<AppState>) -> HttpResponse {
    let weapon_results = data.weapon_results.read().await;
    let weapon_data: HashMap<String, HashMap<String, OptimizationResult>> = weapon_results
        .iter()
        .map(|(key, doc)| {
            let parts: Vec<&str> = key.split('_').collect();
            let weapon = parts[0].to_string();
            let hit_chance = parts[1].to_string();
            let valby = parts[2] == "true";
            (weapon, hit_chance, valby, doc.result.clone())
        })
        .fold(HashMap::new(), |mut acc, (weapon, hit_chance, valby, result)| {
            acc.entry(weapon)
                .or_insert_with(|| HashMap::new())
                .insert(format!("{}_{}", hit_chance, if valby { "valby" } else { "noValby" }), result);
            acc
        });

    HttpResponse::Ok().json(weapon_data)
}

async fn refresh_weapon_results(data: web::Data<AppState>) -> HttpResponse {
    match load_all_weapon_results(&data.db).await {
        Ok(new_results) => {
            let mut weapon_results = data.weapon_results.write().await;
            *weapon_results = new_results;
            HttpResponse::Ok().json(json!({"status": "Weapon results refreshed successfully"}))
        },
        Err(e) => {
            error!("Failed to refresh weapon results: {}", e);
            HttpResponse::InternalServerError().json(json!({"status": "Failed to refresh weapon results"}))
        }
    }
}

async fn load_all_weapon_results(db: &mongodb::Database) -> Result<HashMap<String, WeaponResultDocument>, mongodb::error::Error> {
    let collection = db.collection::<WeaponResultDocument>("weapon_results");
    let mut cursor = collection.find(None, None).await?;

    let mut results = HashMap::new();
    while let Some(doc) = cursor.try_next().await? {
        let key = format!("{}_{}_{}", doc.weapon, doc.weak_point_hit_chance, doc.valby);
        results.insert(key, doc);
    }

    Ok(results)
}

async fn create_mongo_client() -> mongodb::error::Result<Client> {
    dotenv().ok();
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    println!("Connecting to MongoDB...");
    
    let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
    
    // Set the server API version to 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    
    // Increase timeout and add retries
    client_options.connect_timeout = Some(std::time::Duration::from_secs(30));
    client_options.retry_writes = Some(true);
    client_options.retry_reads = Some(true);

    let client = Client::with_options(client_options)?;
    
    // Test the connection
    client.database("admin").run_command(doc! {"ping": 1}, None).await?;
    
    println!("Connected to MongoDB successfully!");
    Ok(client)
}

#[derive(Deserialize)]
struct ClearCacheQuery {
    target: Option<String>,
}

async fn clear_cache_and_fetch(
    data: web::Data<AppState>,
    query: web::Query<ClearCacheQuery>,
) -> HttpResponse {
    let mut weapon_results = data.weapon_results.write().await;
    let db = &data.db;
    
    match query.target.as_deref() {
        Some("all") => {
            weapon_results.clear();
            // Fetch all results
            if let Err(e) = fetch_all_results(db, &mut weapon_results).await {
                return HttpResponse::InternalServerError().json(json!({"status": format!("Failed to fetch new data: {}", e)}));
            }
            HttpResponse::Ok().json(json!({"status": "All cache cleared and new data fetched successfully"}))
        },
        Some(weapon) => {
            let keys_to_remove: Vec<String> = weapon_results.keys()
                .filter(|k| k.starts_with(weapon))
                .cloned()
                .collect();
            
            for key in &keys_to_remove {
                weapon_results.remove(key);
            }

            // Fetch results for the specific weapon
            if let Err(e) = fetch_weapon_results(db, weapon, &mut weapon_results).await {
                return HttpResponse::InternalServerError().json(json!({"status": format!("Failed to fetch new data for weapon {}: {}", weapon, e)}));
            }
            HttpResponse::Ok().json(json!({"status": format!("Cache cleared and new data fetched for weapon: {}", weapon)}))
        },
        None => HttpResponse::BadRequest().json(json!({"status": "No target specified for cache clearing"}))
    }
}

async fn fetch_all_results(db: &mongodb::Database, weapon_results: &mut HashMap<String, WeaponResultDocument>) -> Result<(), mongodb::error::Error> {
    let collection = db.collection::<WeaponResultDocument>("weapon_results");
    let mut cursor = collection.find(None, None).await?;

    while let Some(doc) = cursor.try_next().await? {
        let key = format!("{}_{:.2}_{}", doc.weapon, doc.weak_point_hit_chance, doc.valby);
        weapon_results.insert(key, doc);
    }

    Ok(())
}

async fn fetch_weapon_results(db: &mongodb::Database, weapon: &str, weapon_results: &mut HashMap<String, WeaponResultDocument>) -> Result<(), mongodb::error::Error> {
    let collection = db.collection::<WeaponResultDocument>("weapon_results");
    let filter = doc! { "weapon": weapon };
    let mut cursor = collection.find(filter, None).await?;

    while let Some(doc) = cursor.try_next().await? {
        let key = format!("{}_{:.2}_{}", doc.weapon, doc.weak_point_hit_chance, doc.valby);
        weapon_results.insert(key, doc);
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let mongo_client = match create_mongo_client().await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create MongoDB client: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };

    let db = mongo_client.database("weapon_optimizer");
    
    info!("Loading all weapon results from database...");
    let weapon_results = match load_all_weapon_results(&db).await {
        Ok(results) => Arc::new(RwLock::new(results)),
        Err(e) => {
            error!("Failed to load weapon results: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };
    info!("Weapon results loaded successfully");

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    println!("Starting server at: {}", address);

    HttpServer::new(move || {
            App::new()
                .wrap(
                    Cors::default()
                        .allowed_origin("https://tfd-weapon.onrender.com")
                        .allowed_methods(vec!["GET", "POST"])
                        .allowed_headers(vec!["Content-Type"])
                        .max_age(3600)
                )
                .app_data(web::Data::new(AppState { 
                    db: db.clone(), 
                    weapon_results: weapon_results.clone() 
                }))
                .route("/", web::get().to(index))
                .route("/weapons", web::get().to(get_weapons))
                .route("/optimize", web::post().to(optimize_weapon_handler))
                .route("/weapon-data", web::get().to(get_weapon_data))
                .route("/refresh-results", web::post().to(refresh_weapon_results))
                .route("/clear-cache-and-fetch", web::post().to(clear_cache_and_fetch))
    })
    .bind(address)?
    .run()
    .await
}
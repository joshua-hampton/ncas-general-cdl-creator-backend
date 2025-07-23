use axum::extract::Query;
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, Level};
mod ncas_netcdf;
use tower_http::cors::CorsLayer;

// Simple handler that returns a String
async fn status() -> String {
    "Server is up and running!".to_string()
}

// Nested JSON structure for the json_response endpoint
#[derive(Serialize, Deserialize)]
struct InnerItem {
    value: i32,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct OuterItem {
    name: String,
    details: InnerItem,
    count: u64,
}

// Handler that returns a JSON response
async fn json_response() -> Json<OuterItem> {
    let data = OuterItem {
        name: "Example Object".to_string(),
        details: InnerItem {
            value: 42,
            description: "This is a nested detail".to_string(),
        },
        count: 123,
    };
    Json(data)
}

async fn simple_json(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let default_name = "Default Name".to_string();
    let name = params.get("name").unwrap_or(&default_name).to_string();
    let data = json!({
        "name": name,
        "details": {
            "value": 42,
            "description": "This is a nested detail"
        },
        "count": 123
    });
    Json(data)
}

async fn get_ncas_netcdf_cdl(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    info!("get_ncas_netcdf_cdl called with params: {:?}", params);
    let tag = params
        .get("ncas_general_version")
        .unwrap_or(&"v2.1.0".to_string())
        .to_string();
    let deployment = params
        .get("deployment_mode")
        .unwrap_or(&"land".to_string())
        .to_string();
    let include_requirement_info = params
        .get("include_requirement_info")
        .is_some_and(|v| v == "true");

    let instrument_name: String = match params.get("instrument") {
        Some(instrument) if instrument.is_empty() => {
            warn!("Instrument parameter is empty");
            return Json(json!({"error": "Instrument parameter is required"}));
        }
        Some(instrument) => instrument.to_string(),
        None => {
            warn!("Instrument parameter is missing");
            return Json(json!({"error": "Instrument parameter is required"}));
        }
    };

    let data_product: String = match params.get("data_product") {
        Some(data_product) if data_product.is_empty() => {
            warn!("Data product parameter is empty");
            return Json(json!({"error": "Data product parameter is required"}));
        }
        Some(data_product) => data_product.to_string(),
        None => {
            warn!("Data product parameter is missing");
            return Json(json!({"error": "Data product parameter is required"}));
        }
    };

    let start_date: String = match params.get("start_date") {
        Some(start_date) if start_date.is_empty() => {
            warn!("Start date parameter is empty");
            return Json(json!({"error": "Start date parameter is required"}));
        }
        Some(start_date) => start_date.to_string(),
        None => {
            warn!("Start date parameter is missing");
            return Json(json!({"error": "Start date parameter is required"}));
        }
    };

    match ncas_netcdf::main(
        instrument_name,
        data_product,
        deployment,
        start_date,
        tag,
        include_requirement_info,
    )
    .await
    {
        Ok(cdl) => {
            let response = json!({
                "filename": cdl.filename,
                "cdl": cdl.cdl,
            });
            Json(response)
        }
        Err(e) => {
            warn!("Error fetching NCAS NetCDF data: {}", e);
            Json(json!({"error": e.to_string()}))
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize the logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let origins = [
        "http://localhost:5173".parse().unwrap(),
        "http://localhost".parse().unwrap(),
    ];
    let cors = CorsLayer::new().allow_origin(origins);

    // Define the routes
    let app = Router::new()
        .route("/status", get(status))
        .route("/json_response", get(json_response))
        .route("/simple_json", get(simple_json))
        .route("/create-cdl", get(get_ncas_netcdf_cdl))
        .layer(cors);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

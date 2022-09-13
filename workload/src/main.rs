use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use color_eyre::eyre::Result;
use serde::Serialize;
use tracing::{debug, info, instrument, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let filter = EnvFilter::from_default_env();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let app = Router::new()
        // `GET /` just prints "hello, world!"
        .route("/", get(root))
        .route("/healthz", get(health))
        .route("/regression", get(health))
        .route("/bindings", get(bindings));

    let interface = &"0.0.0.0:8080".parse()?;
    info!("Listening on {}", interface);
    axum::Server::bind(interface)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[instrument]
async fn root() -> &'static str {
    "Hello, World!\n"
}

#[instrument]
async fn health() -> StatusCode {
    let result = get_bindings().await;
    result.map(|_| StatusCode::OK)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[instrument]
async fn bindings() -> Result<impl IntoResponse, impl IntoResponse> {
    debug!("Retrieving service bindings");
    let result = get_bindings().await;
    result
        .map(|r| {
            Json(r)
        })
        .map_err(|e| {
            warn!("Unable to retrieve projected service bindings!");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

static SERVICE_BINDING_ROOT: &str = "SERVICE_BINDING_ROOT";

async fn get_bindings() -> Result<Vec<Binding>> {
    let binding_root = std::env::var(SERVICE_BINDING_ROOT).unwrap_or_else(|_| "/bindings".to_string());
    info!(binding_root, "reading from {}", binding_root);
    let mut dir_entries = tokio::fs::read_dir(binding_root).await?;

    let mut binding_entries = vec![];
    while let Some(entry) = dir_entries.next_entry().await? {
        if entry.metadata().await?.is_dir() {
            binding_entries.push(entry)
        }
    }

    let mut result = vec![];
    for entry in binding_entries {
        let mut binding_information = HashMap::new();
        let mut binding_dir = tokio::fs::read_dir(entry.path()).await?;

        while let Some(entry) = binding_dir.next_entry().await? {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            debug!("reading {}", file_name);
            if let Ok(data) = tokio::fs::read_to_string(entry.path()).await {
                binding_information.insert(file_name, data);
            }
        }

        result.push(Binding {
            name: entry.file_name().to_string_lossy().into_owned(),
            binding_info: binding_information,
        });
    }

    Ok(result)
}

#[derive(Serialize)]
struct Binding {
    name: String,
    binding_info: HashMap<String, String>,
}

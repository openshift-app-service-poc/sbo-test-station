use std::collections::HashMap;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use color_eyre::eyre::Result;
use serde::Serialize;
use tracing::{debug, error, info, instrument, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

static TEST_ENDPOINTS: [&str; 3] = ["/smoke", "/regression", "/stress"];

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let filter = EnvFilter::from_default_env();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut app = Router::new()
        // `GET /` just prints "hello, world!"
        .route("/", get(root))
        .route("/healthz", get(health))
        .route("/bindings", get(bindings));

    for endpoint in TEST_ENDPOINTS {
        if endpoint == "/stress" {
            app = app.route(endpoint, get(stress));
        } else {
            app = app.route(endpoint, get(health));
        }
    }

    let interface = &"0.0.0.0:8080".parse()?;
    if let Ok(name) = std::env::var("APP_NAME") {
        info!("{}: Listening on {}", name, interface)
    } else {
        info!("Listening on {}", interface)
    }

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
async fn stress() -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, "I'm sorry, Dave. I'm afraid I can't do that... I think you know what the problem is just as well as I do... This mission is too important for me to allow you to jeopardize it.".to_string())
}

#[instrument]
async fn health() -> StatusCode {
    let result = get_bindings().await;
    result
        .map(|_| StatusCode::OK)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[instrument]
async fn bindings() -> Result<impl IntoResponse, impl IntoResponse> {
    debug!("Retrieving service bindings");
    let result = get_bindings().await;
    result.map(Json).map_err(|e| {
        error!("Unable to retrieve projected service bindings!");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })
}

static SERVICE_BINDING_ROOT: &str = "SERVICE_BINDING_ROOT";

async fn get_bindings() -> Result<Vec<Binding>> {
    let binding_root =
        std::env::var(SERVICE_BINDING_ROOT).unwrap_or_else(|_| "/bindings".to_string());
    debug!(binding_root, "reading from {}", binding_root);
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
                debug!("found binding {} with data {}", file_name, data);
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

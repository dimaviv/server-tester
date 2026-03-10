mod config;
mod embedded;
mod management;
mod persistence;
mod state;
mod virtual_server;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::state::{AppState, SharedState, VirtualServerEntry};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("server_tester=info".parse().unwrap()),
        )
        .init();

    let args = config::Args::parse();

    // Load persisted server configs
    let saved_configs = persistence::load_state(&args.data_file);
    tracing::info!("Loaded {} server configs from {}", saved_configs.len(), args.data_file);

    // Build initial state and spawn listeners for saved servers
    let mut servers = HashMap::new();
    for (id, config) in saved_configs {
        let handle = match virtual_server::spawn_virtual_server(&config).await {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to start server '{}' on port {}: {}", config.name, config.port, e);
                None
            }
        };
        servers.insert(id, VirtualServerEntry { config, handle });
    }

    let state: SharedState = Arc::new(RwLock::new(AppState {
        servers,
        data_file: args.data_file,
    }));

    let app = management::management_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], args.management_port));
    let listener = TcpListener::bind(addr).await.unwrap();

    tracing::info!("Management API + Web UI listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

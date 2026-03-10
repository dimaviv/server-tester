use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::persistence;
use crate::state::*;
use crate::virtual_server;

pub async fn list_servers(State(state): State<SharedState>) -> Json<Vec<ServerResponse>> {
    let app = state.read().await;
    let servers: Vec<ServerResponse> = app.servers.values().map(|e| (&e.config).into()).collect();
    Json(servers)
}

pub async fn get_server(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ServerResponse>, StatusCode> {
    let app = state.read().await;
    let entry = app.servers.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json((&entry.config).into()))
}

pub async fn create_server(
    State(state): State<SharedState>,
    Json(req): Json<CreateServerRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate port range
    if req.port < 1024 {
        return Err((StatusCode::BAD_REQUEST, "Port must be >= 1024".into()));
    }

    // Validate HTTP status code
    let http_code = req.http_status_code.unwrap_or(200);
    if axum::http::StatusCode::from_u16(http_code).is_err() {
        return Err((StatusCode::BAD_REQUEST, format!("Invalid HTTP status code: {}", http_code)));
    }

    // Cap response delay to 5 minutes
    let delay = req.response_delay_ms.unwrap_or(0).min(300_000);

    // Hold write lock for the entire create to prevent TOCTOU race on port
    let mut app = state.write().await;

    // Check port uniqueness (including management port collision)
    for entry in app.servers.values() {
        if entry.config.port == req.port {
            return Err((
                StatusCode::CONFLICT,
                format!("Port {} is already in use by server '{}'", req.port, entry.config.name),
            ));
        }
    }

    let now = Utc::now();
    let id = Uuid::new_v4();
    let config = VirtualServerConfig {
        id,
        name: req.name,
        port: req.port,
        status: ServerMode::Up,
        http_status_code: http_code,
        html_title: req.html_title.unwrap_or_else(|| format!("Server {}", id)),
        response_body: req.response_body,
        custom_headers: req.custom_headers.unwrap_or_default(),
        response_delay_ms: delay,
        created_at: now,
        updated_at: now,
    };

    // Spawn the virtual server listener
    let handle = virtual_server::spawn_virtual_server(&config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response: ServerResponse = (&config).into();
    app.servers.insert(id, VirtualServerEntry { config, handle });
    persist_state(&app);

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_server(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateServerRequest>,
) -> Result<Json<ServerResponse>, (StatusCode, String)> {
    // Validate HTTP status code if provided
    if let Some(code) = req.http_status_code {
        if axum::http::StatusCode::from_u16(code).is_err() {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid HTTP status code: {}", code)));
        }
    }

    let mut app = state.write().await;
    let entry = app.servers.get_mut(&id).ok_or((StatusCode::NOT_FOUND, "Server not found".into()))?;

    // Update config fields
    if let Some(name) = req.name {
        entry.config.name = name;
    }
    if let Some(code) = req.http_status_code {
        entry.config.http_status_code = code;
    }
    if let Some(title) = req.html_title {
        entry.config.html_title = title;
    }
    if req.response_body.is_some() {
        entry.config.response_body = req.response_body;
    }
    if let Some(headers) = req.custom_headers {
        entry.config.custom_headers = headers;
    }
    if let Some(delay) = req.response_delay_ms {
        entry.config.response_delay_ms = delay.min(300_000);
    }
    entry.config.updated_at = Utc::now();

    // Restart the listener with new config
    if let Some(handle) = entry.handle.take() {
        virtual_server::stop_virtual_server(handle).await;
    }
    let new_handle = virtual_server::spawn_virtual_server(&entry.config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    entry.handle = new_handle;

    let response: ServerResponse = (&entry.config).into();
    persist_state(&app);
    Ok(Json(response))
}

pub async fn delete_server(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut app = state.write().await;
    let entry = app.servers.remove(&id).ok_or(StatusCode::NOT_FOUND)?;

    if let Some(handle) = entry.handle {
        virtual_server::stop_virtual_server(handle).await;
    }

    persist_state(&app);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_mode(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<SetModeRequest>,
) -> Result<Json<ServerResponse>, StatusCode> {
    change_mode(state, id, req.mode).await
}

pub async fn set_up(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ServerResponse>, StatusCode> {
    change_mode(state, id, ServerMode::Up).await
}

pub async fn set_down(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ServerResponse>, StatusCode> {
    change_mode(state, id, ServerMode::DownConnectionRefused).await
}

async fn change_mode(
    state: SharedState,
    id: Uuid,
    new_mode: ServerMode,
) -> Result<Json<ServerResponse>, StatusCode> {
    let mut app = state.write().await;
    let entry = app.servers.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;

    // Stop old listener
    if let Some(handle) = entry.handle.take() {
        virtual_server::stop_virtual_server(handle).await;
    }

    // Update mode
    entry.config.status = new_mode;
    entry.config.updated_at = Utc::now();

    // Spawn new listener
    let new_handle = virtual_server::spawn_virtual_server(&entry.config)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    entry.handle = new_handle;

    let response: ServerResponse = (&entry.config).into();
    persist_state(&app);
    Ok(Json(response))
}

fn persist_state(app: &AppState) {
    let configs: HashMap<Uuid, VirtualServerConfig> = app
        .servers
        .iter()
        .map(|(id, entry)| (*id, entry.config.clone()))
        .collect();
    if let Err(e) = persistence::save_state(&app.data_file, &configs) {
        tracing::error!("Failed to persist state: {}", e);
    }
}

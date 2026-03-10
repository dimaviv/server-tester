use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub servers: HashMap<Uuid, VirtualServerEntry>,
    pub data_file: String,
}

pub struct VirtualServerEntry {
    pub config: VirtualServerConfig,
    pub handle: Option<VirtualServerHandle>,
}

pub struct VirtualServerHandle {
    pub cancel_token: CancellationToken,
    pub join_handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServerConfig {
    pub id: Uuid,
    pub name: String,
    pub port: u16,
    pub status: ServerMode,
    pub http_status_code: u16,
    pub html_title: String,
    pub response_body: Option<String>,
    pub custom_headers: HashMap<String, String>,
    pub response_delay_ms: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerMode {
    Up,
    DownConnectionRefused,
    #[serde(rename = "down_503")]
    Down503,
    DownTimeout,
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub port: u16,
    pub http_status_code: Option<u16>,
    pub html_title: Option<String>,
    pub response_body: Option<String>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub response_delay_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub http_status_code: Option<u16>,
    pub html_title: Option<String>,
    pub response_body: Option<String>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub response_delay_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SetModeRequest {
    pub mode: ServerMode,
}

#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub name: String,
    pub port: u16,
    pub status: ServerMode,
    pub http_status_code: u16,
    pub html_title: String,
    pub response_body: Option<String>,
    pub custom_headers: HashMap<String, String>,
    pub response_delay_ms: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&VirtualServerConfig> for ServerResponse {
    fn from(c: &VirtualServerConfig) -> Self {
        Self {
            id: c.id,
            name: c.name.clone(),
            port: c.port,
            status: c.status,
            http_status_code: c.http_status_code,
            html_title: c.html_title.clone(),
            response_body: c.response_body.clone(),
            custom_headers: c.custom_headers.clone(),
            response_delay_ms: c.response_delay_ms,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

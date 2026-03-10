use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;
use crate::state::VirtualServerConfig;

pub fn save_state(path: &str, servers: &HashMap<Uuid, VirtualServerConfig>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(servers).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_state(path: &str) -> HashMap<Uuid, VirtualServerConfig> {
    if !Path::new(path).exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse {}: {}, starting fresh", path, e);
            HashMap::new()
        }),
        Err(e) => {
            tracing::warn!("Failed to read {}: {}, starting fresh", path, e);
            HashMap::new()
        }
    }
}

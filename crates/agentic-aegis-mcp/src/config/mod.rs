pub struct ServerConfig {
    pub mode: String,
    pub port: u16,
    pub data_dir: Option<String>,
    pub server_name: String,
    pub autosave: bool,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: "stdio".to_string(),
            port: 3011,
            data_dir: None,
            server_name: "agentic-aegis".to_string(),
            autosave: true,
            log_level: "info".to_string(),
        }
    }
}

pub fn load_config() -> ServerConfig {
    let mode = std::env::var("AGENTIC_AEGIS_MODE").unwrap_or_else(|_| "stdio".to_string());
    let port = std::env::var("AGENTIC_AEGIS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3011);
    let data_dir = std::env::var("AGENTIC_AEGIS_DATA_DIR").ok();
    let autosave = std::env::var("AGENTIC_AEGIS_AUTOSAVE")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true);
    let log_level = std::env::var("AGENTIC_AEGIS_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    ServerConfig {
        mode,
        port,
        data_dir,
        server_name: "agentic-aegis".to_string(),
        autosave,
        log_level,
    }
}

pub fn resolve_data_path(explicit: Option<&str>) -> std::path::PathBuf {
    if let Some(path) = explicit {
        return std::path::PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("AGENTIC_AEGIS_DATA_DIR") {
        return std::path::PathBuf::from(path);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".agentic-aegis")
}

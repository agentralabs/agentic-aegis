const MAX_CONTENT_LENGTH_BYTES: usize = 8 * 1024 * 1024; // 8 MiB

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use agentic_aegis_mcp::config::load_config;
use agentic_aegis_mcp::protocol::ProtocolHandler;
use agentic_aegis_mcp::session::McpSessionManager;

#[tokio::main]
async fn main() {
    let config = load_config();

    // Init tracing to stderr
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level)),
        )
        .init();

    tracing::info!("agentic-aegis-mcp starting in {} mode", config.mode);

    let session = Arc::new(Mutex::new(McpSessionManager::new()));
    let handler = ProtocolHandler::new(session);

    match config.mode.as_str() {
        "stdio" => run_stdio(handler).await,
        other => {
            tracing::error!("unsupported mode: {}", other);
            std::process::exit(1);
        }
    }
}

async fn run_stdio(handler: ProtocolHandler) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("failed to read stdin: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip content-length: headers but enforce frame size limit
        if trimmed.to_lowercase().starts_with("content-length:") {
            if let Some(len_str) = trimmed.split(':').nth(1) {
                if let Ok(len) = len_str.trim().parse::<usize>() {
                    if len > MAX_CONTENT_LENGTH_BYTES {
                        tracing::warn!("frame too large: {} > {}", len, MAX_CONTENT_LENGTH_BYTES);
                    }
                }
            }
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("invalid JSON: {}", e);
                continue;
            }
        };

        let response = handler.handle_request(request).await;

        if response.is_null() {
            continue;
        }

        let response_str = match serde_json::to_string(&response) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("failed to serialize response: {}", e);
                continue;
            }
        };

        if writeln!(stdout, "{}", response_str).is_err() {
            break;
        }
        if stdout.flush().is_err() {
            break;
        }
    }

    tracing::info!("agentic-aegis-mcp shutting down");
}

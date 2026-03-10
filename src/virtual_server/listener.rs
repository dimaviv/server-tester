use std::net::SocketAddr;
use axum::Router;
use tokio::net::{TcpListener, TcpSocket};
use tokio_util::sync::CancellationToken;
use crate::state::{ServerMode, VirtualServerConfig, VirtualServerHandle};
use crate::virtual_server::handlers;

/// Spawn the appropriate listener for the given server config.
/// Returns None if mode is DownConnectionRefused (nothing to spawn).
pub async fn spawn_virtual_server(
    config: &VirtualServerConfig,
) -> Result<Option<VirtualServerHandle>, String> {
    match config.status {
        ServerMode::DownConnectionRefused => Ok(None),
        ServerMode::Up => spawn_http_server(config, false).await.map(Some),
        ServerMode::Down503 => spawn_http_server(config, true).await.map(Some),
        ServerMode::DownTimeout => spawn_timeout_server(config.port).await.map(Some),
    }
}

/// Stop a running virtual server by cancelling its token and awaiting its task.
pub async fn stop_virtual_server(handle: VirtualServerHandle) {
    handle.cancel_token.cancel();
    let _ = handle.join_handle.await;
}

/// Bind a TCP listener with SO_REUSEADDR to avoid port conflicts after quick restarts.
async fn bind_reuse(port: u16) -> Result<TcpListener, String> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket = TcpSocket::new_v4().map_err(|e| format!("Failed to create socket: {}", e))?;
    socket
        .set_reuseaddr(true)
        .map_err(|e| format!("Failed to set SO_REUSEADDR: {}", e))?;
    socket
        .bind(addr)
        .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;
    socket
        .listen(1024)
        .map_err(|e| format!("Failed to listen on port {}: {}", port, e))
}

async fn spawn_http_server(
    config: &VirtualServerConfig,
    is_503: bool,
) -> Result<VirtualServerHandle, String> {
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();

    let listener = bind_reuse(config.port).await?;

    let app = build_virtual_server_app(config, is_503);
    let port = config.port;

    let join_handle = tokio::spawn(async move {
        tracing::info!("Virtual server started on port {}", port);
        axum::serve(listener, app)
            .with_graceful_shutdown(token_clone.cancelled_owned())
            .await
            .ok();
        tracing::info!("Virtual server on port {} stopped", port);
    });

    Ok(VirtualServerHandle {
        cancel_token,
        join_handle,
    })
}

async fn spawn_timeout_server(port: u16) -> Result<VirtualServerHandle, String> {
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();

    let listener = bind_reuse(port).await?;

    let join_handle = tokio::spawn(async move {
        tracing::info!("Timeout server started on port {} (accepts but never responds)", port);
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    if let Ok((_socket, addr)) = accept_result {
                        tracing::debug!("Timeout server on port {}: accepted connection from {}, holding open", port, addr);
                        let inner_token = token_clone.clone();
                        tokio::spawn(async move {
                            inner_token.cancelled().await;
                            // _socket dropped here, closing connection
                        });
                    }
                }
                _ = token_clone.cancelled() => {
                    tracing::info!("Timeout server on port {} stopped", port);
                    break;
                }
            }
        }
    });

    Ok(VirtualServerHandle {
        cancel_token,
        join_handle,
    })
}

fn build_virtual_server_app(config: &VirtualServerConfig, is_503: bool) -> Router {
    let http_status = config.http_status_code;
    let html_title = config.html_title.clone();
    let custom_headers = config.custom_headers.clone();
    let delay_ms = config.response_delay_ms;
    let response_body = config.response_body.clone();

    if is_503 {
        let title = html_title.clone();
        Router::new().fallback(move || {
            let title = title.clone();
            async move { handlers::down_503_handler(title).await }
        })
    } else {
        Router::new().fallback(move || {
            let title = html_title.clone();
            let headers = custom_headers.clone();
            let body = response_body.clone();
            async move {
                handlers::up_handler(http_status, title, body, headers, delay_ms).await
            }
        })
    }
}

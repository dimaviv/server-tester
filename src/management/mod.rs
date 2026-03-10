pub mod api;
pub mod ui;

use axum::routing::{get, post};
use axum::Router;
use crate::state::SharedState;

pub fn management_router(state: SharedState) -> Router {
    Router::new()
        // Web UI
        .route("/", get(ui::index_html))
        .route("/style.css", get(ui::style_css))
        .route("/app.js", get(ui::app_js))
        // API
        .route("/api/servers", get(api::list_servers).post(api::create_server))
        .route(
            "/api/servers/{id}",
            get(api::get_server)
                .put(api::update_server)
                .delete(api::delete_server),
        )
        .route("/api/servers/{id}/mode", post(api::set_mode))
        .route("/api/servers/{id}/up", post(api::set_up))
        .route("/api/servers/{id}/down", post(api::set_down))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

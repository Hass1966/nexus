pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod websocket;

use axum::Router;

pub fn build_router(state: state::AppState) -> Router {
    routes::create_router(state)
}

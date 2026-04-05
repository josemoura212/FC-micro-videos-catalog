pub mod state;

use axum::Router;
use state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new().with_state(state)
}

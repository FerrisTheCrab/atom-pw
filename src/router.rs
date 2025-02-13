use axum::routing::post;

use crate::instance::PwInstance;

pub struct InternalRouter;
pub struct Router;

impl Router {
    pub fn get(instance: PwInstance) -> axum::Router {
        axum::Router::new()
            .route("/create", post(Router::create))
            .route("/set", post(Router::set))
            .route("/remove", post(Router::remove))
            .route("/check", post(Router::check))
            .with_state(instance)
    }
}

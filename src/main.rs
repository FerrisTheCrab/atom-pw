#[cfg(feature = "core")]
use std::path::PathBuf;

#[cfg(feature = "core")]
use atom_pw::{PwInstance, Router};

#[cfg(feature = "core")]
#[tokio::main]
async fn main() {
    let path = PathBuf::from(std::env::var("CONFIG").expect("env CONFIG not set"));
    let instance = PwInstance::load(&path);
    let port = instance.config.port;
    let app = axum::Router::new().nest("/api/pw/v1", Router::get(instance));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}",))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(not(feature = "core"))]
fn main() {
    println!("Core is disabled")
}

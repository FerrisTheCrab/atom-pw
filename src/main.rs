use atom_pw::{MasterConfig, Router};

#[tokio::main]
async fn main() {
    std::env::var("CONFIG").expect("env CONFIG not set");
    let app = axum::Router::new().nest("/api/pw/v1", Router::get());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", MasterConfig::get().port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

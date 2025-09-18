use axum::{Router, routing::get};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // set up logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server running on http://{addr}");

    axum::serve::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello from Chad Log API!"
}

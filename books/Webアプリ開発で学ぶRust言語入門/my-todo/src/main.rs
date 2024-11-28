use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", routing::get(root));

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

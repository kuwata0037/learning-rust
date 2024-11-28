use std::net::{Ipv4Addr, SocketAddr};

use axum::{response::IntoResponse, routing, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", routing::get(root))
        .route("/users", routing::post(create_user));

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, Serialize)]
struct User {
    id: u64,
    user_name: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    user_name: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1337,
        user_name: payload.user_name,
    };

    (StatusCode::CREATED, Json(user))
}

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{http::StatusCode, response::IntoResponse, routing, Extension, Json, Router};
use handlers::create_todo;
use repositories::{TodoRepository, TodoRepositoryForMemory};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod repositories;

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

    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", routing::get(root))
        .route("/todos", routing::post(create_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, extract::Request};
    use tower::ServiceExt;

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;

    #[tokio::test]
    async fn should_return_hello_world() -> Result<(), BoxError> {
        let repository = TodoRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty())?;

        let res = create_app(repository).oneshot(req).await?;

        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
        let body = String::from_utf8(bytes.to_vec())?;
        assert_eq!(body, "Hello, world!");

        Ok(())
    }
}

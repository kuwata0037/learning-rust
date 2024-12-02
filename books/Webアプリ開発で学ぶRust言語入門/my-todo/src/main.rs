use std::net::{Ipv4Addr, SocketAddr};

use axum::{http::StatusCode, response::IntoResponse, routing, Json, Router};
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

    let app = create_app();

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn create_app() -> Router {
    Router::new()
        .route("/", routing::get(root))
        .route("/users", routing::post(create_user))
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use std::usize;

    use axum::{
        body::Body,
        extract::Request,
        http::{header, method},
    };
    use tower::ServiceExt;

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;

    #[tokio::test]
    async fn should_return_hello_world() -> Result<(), BoxError> {
        let req = Request::builder().uri("/").body(Body::empty())?;

        let res = create_app().oneshot(req).await?;

        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
        let body = String::from_utf8(bytes.to_vec())?;
        assert_eq!(body, "Hello, world!");

        Ok(())
    }

    #[tokio::test]
    async fn should_return_user_data() -> Result<(), BoxError> {
        let req = Request::builder()
            .uri("/users")
            .method(method::Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(r#"{ "user_name": "田中 太郎" }"#))?;

        let res = create_app().oneshot(req).await?;

        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
        let body = String::from_utf8(bytes.to_vec())?;
        let user: User = serde_json::from_str(&body)?;
        assert_eq!(
            user,
            User {
                id: 1337,
                user_name: "田中 太郎".to_string(),
            }
        );

        Ok(())
    }
}

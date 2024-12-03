use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
};

use axum::{http::StatusCode, response::IntoResponse, routing, Extension, Json, Router};
use serde::{Deserialize, Serialize};
use thiserror::Error;
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
        .route("/users", routing::post(create_user))
        .route("/todos", routing::post(create_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn create_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = repository.create(payload);

    (StatusCode::CREATED, Json(todo))
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

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("Not Found, id is {0}")]
    NotFound(i32),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

impl Todo {
    fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CreateTodo {
    text: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

trait TodoRepository: Clone + Send + Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

type TodoData = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoData>>,
}

impl TodoRepositoryForMemory {
    fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        todo!()
    }

    fn find(&self, id: i32) -> Option<Todo> {
        todo!()
    }

    fn all(&self) -> Vec<Todo> {
        todo!()
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        todo!()
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        todo!()
    }
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
        let repository = TodoRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty())?;

        let res = create_app(repository).oneshot(req).await?;

        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
        let body = String::from_utf8(bytes.to_vec())?;
        assert_eq!(body, "Hello, world!");

        Ok(())
    }

    #[tokio::test]
    async fn should_return_user_data() -> Result<(), BoxError> {
        let repository = TodoRepositoryForMemory::new();
        let req = Request::builder()
            .uri("/users")
            .method(method::Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(r#"{ "user_name": "田中 太郎" }"#))?;

        let res = create_app(repository).oneshot(req).await?;

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

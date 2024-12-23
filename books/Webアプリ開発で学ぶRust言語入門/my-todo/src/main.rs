use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing, Extension, Router};
use handlers::{all_todos, create_todo, delete_todo, find_todo, update_todo};
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
        .route(
            "/todos",
            routing::get(all_todos::<T>).post(create_todo::<T>),
        )
        .route(
            "/todos/:id",
            routing::get(find_todo::<T>)
                .patch(update_todo::<T>)
                .delete(delete_todo::<T>),
        )
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{self, Body},
        extract::Request,
        http::{header, method, Method, StatusCode},
        response::Response,
    };
    use repositories::{CreateTodo, Todo};
    use tower::ServiceExt;

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;

    fn build_request_with_empty(path: &str, method: Method) -> Request {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    fn build_request_with_json(path: &str, method: Method, json_body: String) -> Request {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    async fn response_to_todo(response: Response) -> Todo {
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        serde_json::from_str(&body).expect(&format!("cannot convert Todo instance. body: {body}"))
    }

    #[tokio::test]
    async fn should_return_hello_world() -> Result<(), BoxError> {
        let repository = TodoRepositoryForMemory::new();
        let req = build_request_with_empty("/", method::Method::GET);

        let res = create_app(repository).oneshot(req).await?;

        let bytes = body::to_bytes(res.into_body(), usize::MAX).await?;
        let body = String::from_utf8(bytes.to_vec())?;
        assert_eq!(body, "Hello, world!");

        Ok(())
    }

    #[tokio::test]
    async fn should_created_todo() {
        let repository = TodoRepositoryForMemory::new();
        let request = build_request_with_json(
            "/todos",
            method::Method::POST,
            r#"{ "text": "should_return_created_todo" }"#.to_string(),
        );

        let response = create_app(repository).oneshot(request).await.unwrap();

        let todo = response_to_todo(response).await;
        let expected = Todo::new(1, "should_return_created_todo".to_string());
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_find_todo() {
        let repository = TodoRepositoryForMemory::new();

        repository.create(CreateTodo {
            text: "should_find_todo".to_string(),
        });

        let request = build_request_with_empty("/todos/1", method::Method::GET);
        let response = create_app(repository).oneshot(request).await.unwrap();
        let todo = response_to_todo(response).await;
        let expected = Todo::new(1, "should_find_todo".to_string());
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let repository = TodoRepositoryForMemory::new();
        repository.create(CreateTodo {
            text: "should_get_all_todos".to_string(),
        });

        let request = build_request_with_empty("/todos", method::Method::GET);

        let response = create_app(repository).oneshot(request).await.unwrap();

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let todos: Vec<Todo> = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Todo instance. body: {body}"));
        let expected = Todo::new(1, "should_get_all_todos".to_string());
        assert_eq!(vec![expected], todos);
    }

    #[tokio::test]
    async fn should_update_todo() {
        let repository = TodoRepositoryForMemory::new();
        repository.create(CreateTodo {
            text: "before_update_todo".to_string(),
        });

        let request = build_request_with_json(
            "/todos/1",
            method::Method::PATCH,
            r#"{ "text": "should_update_todo", "completed": false }"#.to_string(),
        );

        let response = create_app(repository).oneshot(request).await.unwrap();

        let todo = response_to_todo(response).await;
        let expected = Todo::new(1, "should_update_todo".to_string());
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_delete_todo() {
        let repository = TodoRepositoryForMemory::new();
        repository.create(CreateTodo {
            text: "should_delete_todo".to_string(),
        });
        let request = build_request_with_empty("/todos/1", method::Method::DELETE);

        let response = create_app(repository).oneshot(request).await.unwrap();

        assert_eq!(StatusCode::NO_CONTENT, response.status());
    }
}

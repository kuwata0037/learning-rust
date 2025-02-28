use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use thiserror::Error;
use validator::Validate;

pub(crate) mod database;
#[cfg(test)]
pub(crate) mod memory;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not Found, id is {0}")]
    NotFound(i32),
    #[error("Unexpected Error: [{0}]")]
    Unexpected(String),
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct CreateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    pub text: Option<String>,
    pub completed: Option<bool>,
}

#[async_trait]
pub trait TodoRepository: Clone + Send + Sync + 'static {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo>;
    async fn find(&self, id: i32) -> anyhow::Result<Todo>;
    async fn all(&self) -> anyhow::Result<Vec<Todo>>;
    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

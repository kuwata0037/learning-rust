use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Context;
use async_trait::async_trait;

use super::{CreateTodo, RepositoryError, Todo, TodoRepository, UpdateTodo};

type TodoData = HashMap<i32, Todo>;

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoData>>,
}

impl TodoRepositoryForMemory {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<TodoData> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<TodoData> {
        self.store.read().unwrap()
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryForMemory {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let todo = Todo::new(id, payload.text);
        store.insert(id, todo.clone());

        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let store = self.read_store_ref();
        let todo = store
            .get(&id)
            .cloned()
            .ok_or(RepositoryError::NotFound(id))?;
        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let store = self.read_store_ref();
        Ok(store.values().cloned().collect())
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();
        let todo = store.get(&id).context(RepositoryError::NotFound(id))?;

        let text = payload.text.unwrap_or_else(|| todo.text.clone());
        let completed = payload.completed.unwrap_or(todo.completed);
        let todo = Todo {
            id,
            text,
            completed,
        };

        store.insert(id, todo.clone());
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn todo_crud_scenario() {
        let sut = TodoRepositoryForMemory::new();

        let text = "todo text".to_string();
        let expected = Todo::new(1, text.clone());

        // create
        let todo = sut
            .create(CreateTodo { text })
            .await
            .expect("failed create todo");
        assert_eq!(expected, todo);

        // find
        let todo = sut.find(1).await.unwrap();
        assert_eq!(expected, todo);

        // all
        let todos = sut.all().await.expect("failed get all todos");
        assert_eq!(vec![expected.clone()], todos);

        // update
        let text = "update text".to_string();
        let todo = sut
            .update(
                1,
                UpdateTodo {
                    text: Some(text.clone()),
                    completed: Some(true),
                },
            )
            .await
            .expect("failed update todo");
        assert_eq!(
            Todo {
                id: 1,
                text: "update text".to_string(),
                completed: true
            },
            todo
        );

        // delete
        let result = sut.delete(1).await;
        assert!(result.is_ok());
    }
}

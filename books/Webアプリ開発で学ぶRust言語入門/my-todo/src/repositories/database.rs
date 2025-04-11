use async_trait::async_trait;
use sqlx::PgPool;

use super::{CreateTodo, RepositoryError, Todo, TodoRepository, UpdateTodo};

#[derive(Debug, Clone)]
pub struct TodoRepositoryForDb {
    pool: PgPool,
}

impl TodoRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryForDb {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
                INSERT INTO todos (text, completed)
                VALUES ($1, false)
                RETURNING *;
            "#,
        )
        .bind(payload.text.clone())
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
                SELECT * FROM todos WHERE id=$1;
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let todos = sqlx::query_as(
            r#"
                SELECT * FROM todos ORDER BY id DESC;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(todos)
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let old_todo = self.find(id).await?;
        let todo = sqlx::query_as::<_, Todo>(
            r#"
                UPDATE todos SET text=$1, completed=$2
                WHERE id=$3
                RETURNING *;
            "#,
        )
        .bind(payload.text.unwrap_or(old_todo.text))
        .bind(payload.completed.unwrap_or(old_todo.completed))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
                DELETE FROM todos WHERE id=$1;
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use super::*;

    #[tokio::test]
    #[ignore = "run only when database is available"]
    async fn crud_scenario() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
        let pool = PgPool::connect(&database_url)
            .await
            .expect(&format!("fail connect database, url is [{database_url}]"));

        let repository = TodoRepositoryForDb::new(pool.clone());
        let todo_text = "[crud_scenario] text]";

        // create
        let created = repository
            .create(CreateTodo {
                text: todo_text.to_string(),
            })
            .await
            .expect("[create] returned Err");
        assert_eq!(created.text, todo_text);
        assert!(!created.completed);

        // find
        let todo = repository
            .find(created.id)
            .await
            .expect("[find] returned Err");
        assert_eq!(created, todo);

        // all
        let todos = repository.all().await.expect("[all] returned Err");
        let todo = todos.first().unwrap();
        assert_eq!(created, *todo);

        // update
        let updated_text = "[crud_scenario] updated text";
        let todo = repository
            .update(
                todo.id,
                UpdateTodo {
                    text: Some(updated_text.to_string()),
                    completed: Some(true),
                },
            )
            .await
            .expect("[update] returned Err");
        assert_eq!(created.id, todo.id);
        assert_eq!(todo.text, updated_text);
        assert_eq!(todo.completed, true);

        // delete
        repository
            .delete(todo.id)
            .await
            .expect("[delete] returned Err");
        let res = repository.find(created.id).await; // expect not found err
        assert!(res.is_err());
    }
}

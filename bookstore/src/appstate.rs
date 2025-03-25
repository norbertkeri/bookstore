use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::handlers::BookRegistration;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn book_exists(&self, name: &str) -> Result<bool, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let book: Option<Book> =
            sqlx::query_as("SELECT id, name, description FROM book WHERE name = $1")
                .bind(name)
                .fetch_optional(&mut *conn)
                .await?;

        Ok(book.is_some())
    }

    pub async fn register_book(&self, book: &BookRegistration) -> color_eyre::Result<Book> {
        let mut conn = self.pool.acquire().await?;
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO book (id, name, description) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(&book.name)
            .bind(&book.description)
            .execute(&mut *conn)
            .await?;

        // safety: we know the book is going to exist, we just inserted it
        Ok(self.get_book_by_id(id).await?.unwrap())
    }

    pub async fn get_book_by_id(&self, id: Uuid) -> color_eyre::Result<Option<Book>> {
        let mut conn = self.pool.acquire().await?;
        let book: Option<Book> =
            sqlx::query_as("SELECT id, name, description FROM book WHERE id = $1")
                .bind(id)
                .fetch_optional(&mut *conn)
                .await?;

        Ok(book)
    }

    pub async fn list_books(&self) -> color_eyre::Result<Vec<Book>> {
        let mut conn = self.pool.acquire().await?;
        let books: Vec<Book> = sqlx::query_as("SELECT id, name, description FROM book")
            .fetch_all(&mut *conn)
            .await?;

        Ok(books)
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Book {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

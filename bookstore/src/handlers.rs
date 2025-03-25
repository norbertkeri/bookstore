use crate::{appstate::AppState, util::AxumHandlerError};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct BookRegistration {
    pub name: String,
    pub description: String,
}

pub async fn register_new_book(
    State(state): State<AppState>,
    body: Json<BookRegistration>,
) -> Result<Response, AxumHandlerError> {
    if state.book_exists(&body.name).await? {
        warn!(
            "Tried registering a book that already exists: {}",
            body.name
        );
        return Ok((StatusCode::CONFLICT, "Book already exists").into_response());
    }
    let book = state.register_book(&body).await?;
    Ok(Json(book).into_response())
}

pub async fn show_book(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Response, AxumHandlerError> {
    let book = state
        .get_book_by_id(id)
        .await?
        .ok_or(AxumHandlerError::NotFound {
            msg: format!("Cannot find book with id {id}").into(),
        })?;

    info!(id = %book.id, name = %book.name, "Showing book");
    Ok(Json(book).into_response())
}

pub async fn list_books(State(state): State<AppState>) -> Result<Response, AxumHandlerError> {
    let books = state.list_books().await?;
    Ok(Json(books).into_response())
}

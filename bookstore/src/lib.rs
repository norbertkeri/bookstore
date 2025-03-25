pub mod appstate;
pub mod bookstore;
pub mod handlers;
pub mod util;

use appstate::AppState;
use axum::{
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing,
};
use handlers::{list_books, register_new_book, show_book};
use rand::Rng;
use sqlx::PgConnection;
use tracing::{Instrument, info, info_span};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/book", routing::get(list_books))
        .route("/book/{book_id}", routing::get(show_book))
        .route("/book", routing::post(register_new_book))
        .layer(middleware::from_fn(tracing_mw))
        .with_state(app_state)
}

async fn tracing_mw(req: Request, next: Next) -> Response {
    let request_id: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(4)
        .map(char::from)
        .collect();
    let method = req.method();
    let path = req.uri().path();

    let span = info_span!("request", request_id);
    info!(parent: &span, %method, path,  "Incoming request");
    let mut res = next.run(req).instrument(span.clone()).await;
    res.headers_mut()
        // safety: the request id is always a valid header value, because it only contains alphanumeric characters
        .insert("x-request-id", request_id.try_into().unwrap());

    let status = res.status();
    info!(parent: &span, status = status.as_u16(), "Response sent");
    res
}

pub async fn run_migrations(conn: &mut PgConnection) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(conn).await
}

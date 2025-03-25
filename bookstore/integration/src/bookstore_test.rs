use axum::http::{Request, StatusCode};
use bookstore::{
    appstate::{AppState, Book},
    create_router,
    handlers::BookRegistration,
};
use tower::ServiceExt;

use crate::testharness::{IntegrationTestCase, TestHarness, TestReturn};

pub fn test_book_registering(harness: TestHarness) -> TestReturn {
    Box::pin(async move {
        let state = AppState::new(harness.connection);
        let app = create_router(state);
        let body = serde_json::to_string(&BookRegistration {
            name: String::from("Ship of Theseus"),
            description: String::from(
                "Elaborate book about two people tracking down a mysterious author",
            ),
        })?;
        let request = Request::post("/book")
            .header("content-type", "application/json")
            .body(body)?;
        let response = app.oneshot(request).await?;
        let (parts, body) = response.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await?;
        assert_eq!(parts.status, StatusCode::OK);
        let converted = serde_json::from_slice::<Book>(&bytes)?;
        assert_eq!(converted.name, "Ship of Theseus");
        Ok(())
    })
}

pub fn test_registering_conflict(harness: TestHarness) -> TestReturn {
    Box::pin(async move {
        let state = AppState::new(harness.connection);
        let app = create_router(state);
        let body = serde_json::to_string(&BookRegistration {
            name: String::from("Ship of Theseus"),
            description: String::from(
                "Elaborate book about two people tracking down a mysterious author",
            ),
        })?;
        let request = Request::post("/book")
            .header("content-type", "application/json")
            .body(body)?;
        let _ = app.clone().oneshot(request.clone()).await?;
        let response = app.oneshot(request).await?;
        let (parts, _body) = response.into_parts();
        assert_eq!(parts.status, StatusCode::CONFLICT);
        Ok(())
    })
}

inventory::submit!(IntegrationTestCase {
    name: "book_registering_success",
    fun: test_book_registering,
});

inventory::submit!(IntegrationTestCase {
    name: "book_registering_conflict",
    fun: test_registering_conflict,
});

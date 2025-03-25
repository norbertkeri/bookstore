use std::borrow::Cow;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AxumHandlerError {
    NotFound {
        msg: Cow<'static, str>,
    },
    BadRequest {
        msg: Cow<'static, str>,
    },
    Internal {
        msg: String,
        error: color_eyre::Report,
    },
}

impl IntoResponse for AxumHandlerError {
    fn into_response(self) -> Response {
        match self {
            AxumHandlerError::NotFound { msg } => {
                let e = (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": msg })),
                );
                e.into_response()
            }
            AxumHandlerError::Internal { msg, error: _ } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {msg}")})),
            )
                .into_response(),
            AxumHandlerError::BadRequest { msg } => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response(),
        }
    }
}

impl From<sqlx::Error> for AxumHandlerError {
    #[track_caller]
    fn from(e: sqlx::Error) -> Self {
        Self::Internal {
            msg: "Error while querying database".into(),
            error: e.into(),
        }
    }
}

impl From<color_eyre::Report> for AxumHandlerError {
    #[track_caller]
    fn from(e: color_eyre::Report) -> Self {
        Self::Internal {
            msg: "Unhandled internal error".into(),
            error: e,
        }
    }
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use app::error::AppError;
use app::RepositoryError;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self { Self(e) }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            AppError::Repository(RepositoryError::NotFound) => {
                tracing::warn!(error = %self.0, "not found");
                (StatusCode::NOT_FOUND, self.0.to_string())
            }
            AppError::InvalidName(_) | AppError::InvalidTitle(_) => {
                tracing::warn!(error = %self.0, "validation failed");
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
            AppError::Story(_)
            | AppError::Character(_)
            | AppError::Quest(_)
            | AppError::Location(_)
            | AppError::Session(_)
            | AppError::Encounter(_) => {
                tracing::warn!(error = %self.0, "domain rule violated");
                (StatusCode::UNPROCESSABLE_ENTITY, self.0.to_string())
            }
            _ => {
                tracing::error!(error = %self.0, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

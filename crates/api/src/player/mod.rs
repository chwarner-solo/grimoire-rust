
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use domain::character::CharacterId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/characters/:id",          get(get_character))
        .route("/characters/:id/name",     patch(rename))
        .route("/characters/:id/backstory", patch(update_backstory))
}

#[derive(Deserialize)]
struct NameBody { name: String }

#[derive(Deserialize)]
struct BackstoryBody { backstory: String }

async fn get_character(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::character::Character>, ApiError> {
    let character = s.characters.get(CharacterId::from(id)).await?;
    if character.metadata().owner() != user_id {
        return Err(ApiError(app::error::AppError::Repository(
            app::RepositoryError::NotFound,
        )));
    }
    Ok(Json(character))
}

async fn rename(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(b): Json<NameBody>,
) -> Result<StatusCode, ApiError> {
    let character_id = CharacterId::from(id);
    let character = s.characters.get(character_id).await?;
    if character.metadata().owner() != user_id {
        return Err(ApiError(app::error::AppError::Repository(
            app::RepositoryError::NotFound,
        )));
    }
    s.characters.rename(character_id, &b.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_backstory(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(b): Json<BackstoryBody>,
) -> Result<StatusCode, ApiError> {
    let character_id = CharacterId::from(id);
    let character = s.characters.get(character_id).await?;
    if character.metadata().owner() != user_id {
        return Err(ApiError(app::error::AppError::Repository(
            app::RepositoryError::NotFound,
        )));
    }
    s.characters.update_backstory(character_id, &b.backstory).await?;
    Ok(StatusCode::NO_CONTENT)
}

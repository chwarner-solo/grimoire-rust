
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::character::{CharacterId, CharacterKind};
use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories/:story_id/characters",    post(create))
        .route("/characters/:id",                  get(get_character))
        .route("/characters/:id/name",             patch(rename))
        .route("/characters/:id/backstory",        patch(update_backstory))
        .route("/characters/:id/depart",           post(depart))
        .route("/characters/:id/kill",             post(kill))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody { name: String, backstory: String, kind: CharacterKind }

#[derive(Deserialize)]
struct NameBody { name: String }

#[derive(Deserialize)]
struct BackstoryBody { backstory: String }

#[derive(Deserialize)]
struct WithSession { session_id: Option<Uuid> }

async fn create(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(story_id): Path<Uuid>,
    Json(b): Json<CreateBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.characters
        .create(StoryId::from(story_id), user_id, &b.name, &b.backstory, b.kind)
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_character(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::character::Character>, ApiError> {
    Ok(Json(s.characters.get(CharacterId::from(id)).await?))
}

async fn rename(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<NameBody>,
) -> Result<StatusCode, ApiError> {
    s.characters.rename(CharacterId::from(id), &b.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_backstory(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<BackstoryBody>,
) -> Result<StatusCode, ApiError> {
    s.characters.update_backstory(CharacterId::from(id), &b.backstory).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn depart(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.characters
        .depart(CharacterId::from(id), b.session_id.map(Into::into))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn kill(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.characters
        .kill(CharacterId::from(id), b.session_id.map(Into::into))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

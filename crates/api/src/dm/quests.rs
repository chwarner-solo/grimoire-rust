
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::quest::{QuestId, QuestKind};
use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories/:story_id/quests",   post(create))
        .route("/quests/:id",                 get(get_quest))
        .route("/quests/:id/name",            patch(rename))
        .route("/quests/:id/description",     patch(update_description))
        .route("/quests/:id/complete",        post(complete))
        .route("/quests/:id/fail",            post(fail))
        .route("/quests/:id/abandon",         post(abandon))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    description: String,
    kind: QuestKind,
    character_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct NameBody { name: String }

#[derive(Deserialize)]
struct DescriptionBody { description: String }

#[derive(Deserialize)]
struct WithSession { session_id: Option<Uuid> }

async fn create(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(story_id): Path<Uuid>,
    Json(b): Json<CreateBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.quests
        .create(
            StoryId::from(story_id),
            user_id,
            &b.name,
            &b.description,
            b.kind,
            b.character_id.map(Into::into),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_quest(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::quest::Quest>, ApiError> {
    Ok(Json(s.quests.get(QuestId::from(id)).await?))
}

async fn rename(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<NameBody>,
) -> Result<StatusCode, ApiError> {
    s.quests.rename(QuestId::from(id), &b.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_description(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<DescriptionBody>,
) -> Result<StatusCode, ApiError> {
    s.quests.update_description(QuestId::from(id), &b.description).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn complete(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.quests.complete(QuestId::from(id), b.session_id.map(Into::into)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn fail(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.quests.fail(QuestId::from(id), b.session_id.map(Into::into)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn abandon(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.quests.abandon(QuestId::from(id), b.session_id.map(Into::into)).await?;
    Ok(StatusCode::NO_CONTENT)
}

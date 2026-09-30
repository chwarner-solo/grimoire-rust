
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories",            post(create))
        .route("/stories/:id",        get(get_story))
        .route("/stories/:id/title",  patch(update_title))
        .route("/stories/:id/prose",  patch(update_prose))
        .route("/stories/:id/close",  post(close))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody { title: String, prose: String }

#[derive(Deserialize)]
struct TitleBody { title: String }

#[derive(Deserialize)]
struct ProseBody { prose: String }

async fn create(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(b): Json<CreateBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.stories.create(user_id, &b.title, &b.prose).await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_story(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::story::Story>, ApiError> {
    Ok(Json(s.stories.get(StoryId::from(id)).await?))
}

async fn update_title(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<TitleBody>,
) -> Result<StatusCode, ApiError> {
    s.stories.update_title(StoryId::from(id), &b.title).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_prose(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<ProseBody>,
) -> Result<StatusCode, ApiError> {
    s.stories.update_prose(StoryId::from(id), &b.prose).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn close(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    s.stories.close(StoryId::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

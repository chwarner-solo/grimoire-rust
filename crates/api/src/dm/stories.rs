
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::story::{Story, StoryId, StoryStatus};

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories",            get(list).post(create))
        .route("/stories/:id",        get(get_story))
        .route("/stories/:id/title",  patch(update_title))
        .route("/stories/:id/prose",  patch(update_prose))
        .route("/stories/:id/close",  post(close))
}

/// Flat API response — stable contract independent of domain struct layout.
#[derive(Serialize)]
struct StoryResponse {
    id: Uuid,
    owner: Uuid,
    title: String,
    prose: String,
    status: &'static str,
}

impl From<Story> for StoryResponse {
    fn from(s: Story) -> Self {
        Self {
            id: s.id().into(),
            owner: s.metadata().owner().into(),
            title: s.title().as_str().to_owned(),
            prose: s.prose().as_str().to_owned(),
            status: match s.status() {
                StoryStatus::Active   => "Active",
                StoryStatus::Inactive => "Inactive",
            },
        }
    }
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody { title: String, prose: String }

#[derive(Deserialize)]
struct TitleBody { title: String }

#[derive(Deserialize)]
struct ProseBody { prose: String }

async fn list(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Json<Vec<StoryResponse>>, ApiError> {
    let stories = s.stories.list_by_owner(user_id).await?;
    Ok(Json(stories.into_iter().map(StoryResponse::from).collect()))
}

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
) -> Result<Json<StoryResponse>, ApiError> {
    Ok(Json(s.stories.get(StoryId::from(id)).await?.into()))
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

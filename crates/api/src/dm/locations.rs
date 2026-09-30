
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::location::LocationId;
use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories/:story_id/locations",  post(create))
        .route("/locations/:id",                get(get_location))
        .route("/locations/:id/name",           patch(rename))
        .route("/locations/:id/description",    patch(update_description))
        .route("/locations/:id/visit",          post(visit))
        .route("/locations/:id/destroy",        post(destroy))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody { name: String, description: String }

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
    let id = s.locations
        .create(StoryId::from(story_id), user_id, &b.name, &b.description)
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_location(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::location::Location>, ApiError> {
    Ok(Json(s.locations.get(LocationId::from(id)).await?))
}

async fn rename(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<NameBody>,
) -> Result<StatusCode, ApiError> {
    s.locations.rename(LocationId::from(id), &b.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_description(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<DescriptionBody>,
) -> Result<StatusCode, ApiError> {
    s.locations.update_description(LocationId::from(id), &b.description).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn visit(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.locations.visit(LocationId::from(id), b.session_id.map(Into::into)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn destroy(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<WithSession>,
) -> Result<StatusCode, ApiError> {
    s.locations.destroy(LocationId::from(id), b.session_id.map(Into::into)).await?;
    Ok(StatusCode::NO_CONTENT)
}

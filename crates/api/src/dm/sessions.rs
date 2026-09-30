
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::session::SessionId;
use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stories/:story_id/sessions",  post(create))
        .route("/sessions/:id",                get(get_session))
        .route("/sessions/:id/open",           post(open))
        .route("/sessions/:id/notes",          patch(update_notes))
        .route("/sessions/:id/summary",        patch(summarize))
        .route("/sessions/:id/close",          post(close))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct CreateBody { number: u32, date: String, notes: String }

#[derive(Deserialize)]
struct NotesBody { notes: String }

#[derive(Deserialize)]
struct SummaryBody { summary: String }

async fn create(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(story_id): Path<Uuid>,
    Json(b): Json<CreateBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.sessions
        .create(StoryId::from(story_id), user_id, b.number, b.date, &b.notes)
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_session(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::session::Session>, ApiError> {
    Ok(Json(s.sessions.get(SessionId::from(id)).await?))
}

async fn open(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    s.sessions.open(SessionId::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_notes(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<NotesBody>,
) -> Result<StatusCode, ApiError> {
    s.sessions.update_notes(SessionId::from(id), &b.notes).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn summarize(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<SummaryBody>,
) -> Result<StatusCode, ApiError> {
    s.sessions.summarize(SessionId::from(id), &b.summary).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn close(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    s.sessions.close(SessionId::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

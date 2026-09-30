
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::encounter::{EncounterId, EncounterKind};
use domain::session::SessionId;
use domain::story::StoryId;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/sessions/:session_id/encounters/plan",      post(plan))
        .route("/sessions/:session_id/encounters/improvise", post(improvise))
        .route("/encounters/:id",                            get(get_encounter))
        .route("/encounters/:id/activate",                   post(activate))
        .route("/encounters/:id/resolve",                    post(resolve))
        .route("/encounters/:id/abandon",                    post(abandon))
        .route("/encounters/:id/dead-end",                   post(dead_end))
}

#[derive(Serialize)]
struct Created { id: Uuid }

#[derive(Deserialize)]
struct PlanBody { story_id: Uuid, kind: EncounterKind, description: String }

#[derive(Deserialize)]
struct ResolveBody { outcome: String }

#[derive(Deserialize)]
struct DeadEndBody { reason: String }

async fn plan(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(b): Json<PlanBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.encounters
        .plan(SessionId::from(session_id), StoryId::from(b.story_id), user_id, b.kind, &b.description)
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn improvise(
    State(s): State<SharedState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(b): Json<PlanBody>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = s.encounters
        .improvise(SessionId::from(session_id), StoryId::from(b.story_id), user_id, b.kind, &b.description)
        .await?;
    Ok((StatusCode::CREATED, Json(Created { id: id.into() })))
}

async fn get_encounter(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<domain::encounter::Encounter>, ApiError> {
    Ok(Json(s.encounters.get(EncounterId::from(id)).await?))
}

async fn activate(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    s.encounters.activate(EncounterId::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn resolve(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<ResolveBody>,
) -> Result<StatusCode, ApiError> {
    s.encounters.resolve(EncounterId::from(id), &b.outcome).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn abandon(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    s.encounters.abandon(EncounterId::from(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn dead_end(
    State(s): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(b): Json<DeadEndBody>,
) -> Result<StatusCode, ApiError> {
    s.encounters.dead_end(EncounterId::from(id), &b.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

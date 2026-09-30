use std::path::PathBuf;
use std::sync::Arc;

use axum::middleware;
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use adapters::FsAggregateRepository;
use app::character::CharacterService;
use app::encounter::EncounterService;
use app::location::LocationService;
use app::quest::QuestService;
use app::session::SessionService;
use app::story::StoryService;
use domain::character::Character;
use domain::encounter::Encounter;
use domain::location::Location;
use domain::quest::Quest;
use domain::session::Session;
use domain::story::Story;

pub mod auth;
pub mod dm;
pub mod error;
pub mod player;

pub use auth::{AuthConfig, AuthState};

// --- Concrete service types ---

type Stories   = StoryService<FsAggregateRepository<Story>>;
type Characters = CharacterService<FsAggregateRepository<Character>>;
type Quests    = QuestService<FsAggregateRepository<Quest>>;
type Locations = LocationService<FsAggregateRepository<Location>>;
type Sessions  = SessionService<FsAggregateRepository<Session>>;
type Encounters = EncounterService<FsAggregateRepository<Encounter>>;

// --- Application state ---

pub struct AppState {
    pub auth:       Arc<AuthState>,
    pub stories:    Stories,
    pub characters: Characters,
    pub quests:     Quests,
    pub locations:  Locations,
    pub sessions:   Sessions,
    pub encounters: Encounters,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(auth_config: AuthConfig, data_dir: impl Into<PathBuf>) -> Self {
        let base = data_dir.into();
        Self {
            auth:       Arc::new(AuthState::new(auth_config)),
            stories:    StoryService::new(FsAggregateRepository::new(base.join("stories"))),
            characters: CharacterService::new(FsAggregateRepository::new(base.join("characters"))),
            quests:     QuestService::new(FsAggregateRepository::new(base.join("quests"))),
            locations:  LocationService::new(FsAggregateRepository::new(base.join("locations"))),
            sessions:   SessionService::new(FsAggregateRepository::new(base.join("sessions"))),
            encounters: EncounterService::new(FsAggregateRepository::new(base.join("encounters"))),
        }
    }
}

// --- Router ---

pub fn build_router(state: AppState) -> Router {
    let auth_state = Arc::clone(&state.auth);
    let shared = Arc::new(state);

    Router::new()
        .route("/health", get(health))
        .nest("/dm", dm::router())
        .nest("/player", player::router())
        .layer(middleware::from_fn_with_state(
            auth_state,
            auth::auth_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(shared)
}

async fn health() -> &'static str {
    "ok"
}

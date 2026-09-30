use axum::Router;

use crate::SharedState;

pub mod characters;
pub mod encounters;
pub mod locations;
pub mod quests;
pub mod sessions;
pub mod stories;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(stories::router())
        .merge(characters::router())
        .merge(quests::router())
        .merge(locations::router())
        .merge(sessions::router())
        .merge(encounters::router())
}

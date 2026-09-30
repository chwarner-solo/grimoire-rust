use crate::{EventStoreError, RepositoryError, SnapshotError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Store(#[from] EventStoreError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    InvalidName(#[from] domain::shared::NameError),
    #[error(transparent)]
    InvalidTitle(#[from] domain::story::TitleError),
    #[error(transparent)]
    Story(#[from] domain::story::StoryError),
    #[error(transparent)]
    Character(#[from] domain::character::CharacterError),
    #[error(transparent)]
    Quest(#[from] domain::quest::QuestError),
    #[error(transparent)]
    Location(#[from] domain::location::LocationError),
    #[error(transparent)]
    Session(#[from] domain::session::SessionError),
    #[error(transparent)]
    Encounter(#[from] domain::encounter::EncounterError),
}

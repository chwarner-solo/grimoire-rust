use serde::{Deserialize, Serialize};

use crate::session::SessionId;
use crate::shared::{Metadata, Name, NameError, Prose, UserId};
use crate::story::StoryId;
use crate::traits::{Aggregate, Apply};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum CharacterError {
    #[error(transparent)]
    Name(#[from] NameError),
    #[error("character is not active")]
    NotActive,
    #[error("character is already deceased")]
    AlreadyDeceased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(Uuid);

impl CharacterId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for CharacterId {
    fn default() -> Self { Self(Uuid::nil()) }
}

impl From<CharacterId> for Uuid {
    fn from(id: CharacterId) -> Self { id.0 }
}

impl From<Uuid> for CharacterId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl std::fmt::Display for CharacterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterKind { PlayerCharacter, NonPlayer }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterStatus { Active, Departed, Deceased }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    id: CharacterId,
    story_id: StoryId,
    metadata: Metadata,
    name: Name,
    backstory: Prose,
    kind: CharacterKind,
    status: CharacterStatus,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            id: CharacterId::default(),
            story_id: StoryId::default(),
            metadata: Metadata::default(),
            name: Name::default(),
            backstory: Prose::default(),
            kind: CharacterKind::NonPlayer,
            status: CharacterStatus::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterEvent {
    Created { id: CharacterId, story_id: StoryId, owner: UserId, name: Name, backstory: Prose, kind: CharacterKind },
    Departed { session_id: Option<SessionId> },
    Deceased { session_id: Option<SessionId> },
    Renamed { name: Name },
    BackstoryUpdated { backstory: Prose },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterCommand {
    Create { story_id: StoryId, owner: UserId, name: Name, backstory: Prose, kind: CharacterKind },
    Depart { session_id: Option<SessionId> },
    Kill { session_id: Option<SessionId> },
    Rename { name: Name },
    UpdateBackstory { backstory: Prose },
}

impl Apply<CharacterEvent> for Character {
    fn apply(self, event: CharacterEvent) -> Self {
        match event {
            CharacterEvent::Created { id, story_id, owner, name, backstory, kind } => Self {
                id,
                story_id,
                metadata: Metadata::new(owner),
                name,
                backstory,
                kind,
                status: CharacterStatus::Active,
            },
            CharacterEvent::Departed { .. } => Self { status: CharacterStatus::Departed, ..self },
            CharacterEvent::Deceased { .. } => Self { status: CharacterStatus::Deceased, ..self },
            CharacterEvent::Renamed { name } => Self { name, ..self },
            CharacterEvent::BackstoryUpdated { backstory } => Self { backstory, ..self },
        }
    }
}

impl Aggregate for Character {
    type Id = CharacterId;
    type Command = CharacterCommand;
    type Event = CharacterEvent;
    type Error = CharacterError;

    fn id(&self) -> CharacterId { self.id }

    fn handle(&self, command: CharacterCommand) -> Result<Vec<CharacterEvent>, CharacterError> {
        match command {
            CharacterCommand::Create { story_id, owner, name, backstory, kind } => {
                Ok(vec![CharacterEvent::Created { id: CharacterId::new(), story_id, owner, name, backstory, kind }])
            }
            CharacterCommand::Depart { session_id } => match self.status {
                CharacterStatus::Active => Ok(vec![CharacterEvent::Departed { session_id }]),
                _ => Err(CharacterError::NotActive),
            },
            CharacterCommand::Kill { session_id } => match self.status {
                CharacterStatus::Deceased => Err(CharacterError::AlreadyDeceased),
                _ => Ok(vec![CharacterEvent::Deceased { session_id }]),
            },
            CharacterCommand::Rename { name } => Ok(vec![CharacterEvent::Renamed { name }]),
            CharacterCommand::UpdateBackstory { backstory } => Ok(vec![CharacterEvent::BackstoryUpdated { backstory }]),
        }
    }
}

impl Character {
    pub fn id(&self) -> CharacterId { self.id }
    pub fn story_id(&self) -> StoryId { self.story_id }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn name(&self) -> &Name { &self.name }
    pub fn backstory(&self) -> &Prose { &self.backstory }
    pub fn kind(&self) -> &CharacterKind { &self.kind }
    pub fn status(&self) -> &CharacterStatus { &self.status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pc(name: &str) -> Character {
        let events = Character::default().handle(CharacterCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            name: Name::parse(name).unwrap(),
            backstory: Prose::default(),
            kind: CharacterKind::PlayerCharacter,
        }).unwrap();
        events.into_iter().fold(Character::default(), Character::apply)
    }

    #[test]
    fn a_character_belongs_to_a_story() {
        let story_id = StoryId::new();
        let events = Character::default().handle(CharacterCommand::Create {
            story_id,
            owner: UserId::new(),
            name: Name::parse("Kira").unwrap(),
            backstory: Prose::default(),
            kind: CharacterKind::PlayerCharacter,
        }).unwrap();
        let kira = events.into_iter().fold(Character::default(), Character::apply);

        assert_eq!(kira.story_id(), story_id);
    }

    #[test]
    fn a_new_character_is_active() {
        assert_eq!(pc("Kira").status(), &CharacterStatus::Active);
    }

    #[test]
    fn a_character_requires_a_name() {
        assert_eq!(Name::parse("  "), Err(NameError::Empty));
    }

    #[test]
    fn a_departed_character_remains_part_of_the_story() {
        let kira = pc("Kira");
        let story_id = kira.story_id();
        let events = kira.handle(CharacterCommand::Depart { session_id: None }).unwrap();
        let kira = events.into_iter().fold(kira, Character::apply);

        assert_eq!(kira.status(), &CharacterStatus::Departed);
        assert_eq!(kira.kind(), &CharacterKind::PlayerCharacter);
        assert_eq!(kira.story_id(), story_id);
    }

    #[test]
    fn a_departed_character_cannot_depart_again() {
        let kira = pc("Kira");
        let events = kira.handle(CharacterCommand::Depart { session_id: None }).unwrap();
        let kira = events.into_iter().fold(kira, Character::apply);

        assert_eq!(kira.handle(CharacterCommand::Depart { session_id: None }), Err(CharacterError::NotActive));
    }

    #[test]
    fn a_deceased_character_cannot_be_killed_again() {
        let kira = pc("Kira");
        let events = kira.handle(CharacterCommand::Kill { session_id: None }).unwrap();
        let kira = events.into_iter().fold(kira, Character::apply);

        assert_eq!(kira.handle(CharacterCommand::Kill { session_id: None }), Err(CharacterError::AlreadyDeceased));
    }

    #[test]
    fn departing_a_character_records_the_session() {
        let session_id = SessionId::new();
        let kira = pc("Kira");
        let events = kira.handle(CharacterCommand::Depart { session_id: Some(session_id) }).unwrap();

        assert_eq!(events[0], CharacterEvent::Departed { session_id: Some(session_id) });
    }
}

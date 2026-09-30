use serde::{Deserialize, Serialize};

use crate::character::CharacterId;
use crate::session::SessionId;
use crate::shared::{Metadata, Name, NameError, Prose, UserId};
use crate::story::StoryId;
use crate::traits::{Aggregate, Apply};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestKind { Main, Side, Personal, Crafting }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum QuestError {
    #[error(transparent)]
    Name(#[from] NameError),
    #[error("quest is not active")]
    NotActive,
    #[error("quest is already complete")]
    AlreadyComplete,
    #[error("a personal quest requires a character")]
    MissingCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestId(Uuid);

impl QuestId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for QuestId {
    fn default() -> Self { Self(Uuid::nil()) }
}

impl From<QuestId> for Uuid {
    fn from(id: QuestId) -> Self { id.0 }
}

impl From<Uuid> for QuestId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl std::fmt::Display for QuestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus { Active, Completed, Failed, Abandoned }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quest {
    id: QuestId,
    story_id: StoryId,
    metadata: Metadata,
    name: Name,
    description: Prose,
    kind: QuestKind,
    character_id: Option<CharacterId>,
    status: QuestStatus,
}

impl Default for Quest {
    fn default() -> Self {
        Self {
            id: QuestId::default(),
            story_id: StoryId::default(),
            metadata: Metadata::default(),
            name: Name::default(),
            description: Prose::default(),
            kind: QuestKind::Side,
            character_id: None,
            status: QuestStatus::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestEvent {
    Created { id: QuestId, story_id: StoryId, owner: UserId, name: Name, description: Prose, kind: QuestKind, character_id: Option<CharacterId> },
    Completed { session_id: Option<SessionId> },
    Failed { session_id: Option<SessionId> },
    Abandoned { session_id: Option<SessionId> },
    Renamed { name: Name },
    DescriptionUpdated { description: Prose },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestCommand {
    Create { story_id: StoryId, owner: UserId, name: Name, description: Prose, kind: QuestKind, character_id: Option<CharacterId> },
    Complete { session_id: Option<SessionId> },
    Fail { session_id: Option<SessionId> },
    Abandon { session_id: Option<SessionId> },
    Rename { name: Name },
    UpdateDescription { description: Prose },
}

impl Apply<QuestEvent> for Quest {
    fn apply(self, event: QuestEvent) -> Self {
        match event {
            QuestEvent::Created { id, story_id, owner, name, description, kind, character_id } => Self {
                id,
                story_id,
                metadata: Metadata::new(owner),
                name,
                description,
                kind,
                character_id,
                status: QuestStatus::Active,
            },
            QuestEvent::Completed { .. } => Self { status: QuestStatus::Completed, ..self },
            QuestEvent::Failed { .. } => Self { status: QuestStatus::Failed, ..self },
            QuestEvent::Abandoned { .. } => Self { status: QuestStatus::Abandoned, ..self },
            QuestEvent::Renamed { name } => Self { name, ..self },
            QuestEvent::DescriptionUpdated { description } => Self { description, ..self },
        }
    }
}

impl Aggregate for Quest {
    type Id = QuestId;
    type Command = QuestCommand;
    type Event = QuestEvent;
    type Error = QuestError;

    fn id(&self) -> QuestId { self.id }

    fn handle(&self, command: QuestCommand) -> Result<Vec<QuestEvent>, QuestError> {
        match command {
            QuestCommand::Create { story_id, owner, name, description, kind, character_id } => {
                if kind == QuestKind::Personal && character_id.is_none() {
                    return Err(QuestError::MissingCharacter);
                }
                Ok(vec![QuestEvent::Created { id: QuestId::new(), story_id, owner, name, description, kind, character_id }])
            }
            QuestCommand::Complete { session_id } => match self.status {
                QuestStatus::Active => Ok(vec![QuestEvent::Completed { session_id }]),
                QuestStatus::Completed => Err(QuestError::AlreadyComplete),
                _ => Err(QuestError::NotActive),
            },
            QuestCommand::Fail { session_id } => match self.status {
                QuestStatus::Active => Ok(vec![QuestEvent::Failed { session_id }]),
                _ => Err(QuestError::NotActive),
            },
            QuestCommand::Abandon { session_id } => match self.status {
                QuestStatus::Active => Ok(vec![QuestEvent::Abandoned { session_id }]),
                _ => Err(QuestError::NotActive),
            },
            QuestCommand::Rename { name } => Ok(vec![QuestEvent::Renamed { name }]),
            QuestCommand::UpdateDescription { description } => Ok(vec![QuestEvent::DescriptionUpdated { description }]),
        }
    }
}

impl Quest {
    pub fn id(&self) -> QuestId { self.id }
    pub fn story_id(&self) -> StoryId { self.story_id }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn name(&self) -> &Name { &self.name }
    pub fn description(&self) -> &Prose { &self.description }
    pub fn kind(&self) -> &QuestKind { &self.kind }
    pub fn character_id(&self) -> Option<CharacterId> { self.character_id }
    pub fn status(&self) -> &QuestStatus { &self.status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(name: &str, kind: QuestKind) -> Quest {
        let events = Quest::default().handle(QuestCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            name: Name::parse(name).unwrap(),
            description: Prose::default(),
            kind,
            character_id: None,
        }).unwrap();
        events.into_iter().fold(Quest::default(), Quest::apply)
    }

    #[test]
    fn a_quest_belongs_to_a_story() {
        let story_id = StoryId::new();
        let events = Quest::default().handle(QuestCommand::Create {
            story_id,
            owner: UserId::new(),
            name: Name::parse("The Lost Artifact").unwrap(),
            description: Prose::default(),
            kind: QuestKind::Main,
            character_id: None,
        }).unwrap();
        let q = events.into_iter().fold(Quest::default(), Quest::apply);

        assert_eq!(q.story_id(), story_id);
    }

    #[test]
    fn a_new_quest_is_active() {
        assert_eq!(quest("The Lost Artifact", QuestKind::Main).status(), &QuestStatus::Active);
    }

    #[test]
    fn a_quest_requires_a_name() {
        assert_eq!(Name::parse("  "), Err(NameError::Empty));
    }

    #[test]
    fn a_personal_quest_requires_a_character() {
        let result = Quest::default().handle(QuestCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            name: Name::parse("Kira's Redemption").unwrap(),
            description: Prose::default(),
            kind: QuestKind::Personal,
            character_id: None,
        });
        assert_eq!(result, Err(QuestError::MissingCharacter));
    }

    #[test]
    fn a_personal_quest_records_its_character() {
        let character_id = CharacterId::new();
        let events = Quest::default().handle(QuestCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            name: Name::parse("Kira's Redemption").unwrap(),
            description: Prose::default(),
            kind: QuestKind::Personal,
            character_id: Some(character_id),
        }).unwrap();
        let q = events.into_iter().fold(Quest::default(), Quest::apply);

        assert_eq!(q.kind(), &QuestKind::Personal);
        assert_eq!(q.character_id(), Some(character_id));
    }

    #[test]
    fn an_active_quest_can_be_completed() {
        let q = quest("The Lost Artifact", QuestKind::Main);
        let events = q.handle(QuestCommand::Complete { session_id: None }).unwrap();
        let q = events.into_iter().fold(q, Quest::apply);

        assert_eq!(q.status(), &QuestStatus::Completed);
    }

    #[test]
    fn a_completed_quest_cannot_be_completed_again() {
        let q = quest("The Lost Artifact", QuestKind::Main);
        let events = q.handle(QuestCommand::Complete { session_id: None }).unwrap();
        let q = events.into_iter().fold(q, Quest::apply);

        assert_eq!(q.handle(QuestCommand::Complete { session_id: None }), Err(QuestError::AlreadyComplete));
    }

    #[test]
    fn an_active_quest_can_be_abandoned() {
        let q = quest("The Lost Artifact", QuestKind::Side);
        let events = q.handle(QuestCommand::Abandon { session_id: None }).unwrap();
        let q = events.into_iter().fold(q, Quest::apply);

        assert_eq!(q.status(), &QuestStatus::Abandoned);
    }

    #[test]
    fn completing_a_quest_records_the_session() {
        let session_id = SessionId::new();
        let q = quest("The Lost Artifact", QuestKind::Main);
        let events = q.handle(QuestCommand::Complete { session_id: Some(session_id) }).unwrap();

        assert_eq!(events[0], QuestEvent::Completed { session_id: Some(session_id) });
    }
}

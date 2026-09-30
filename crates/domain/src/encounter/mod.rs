use serde::{Deserialize, Serialize};

use crate::shared::{Metadata, Prose, UserId};
use crate::story::StoryId;
use crate::session::SessionId;
use crate::traits::{Aggregate, Apply};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncounterId(Uuid);

impl EncounterId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for EncounterId {
    fn default() -> Self { Self(Uuid::nil()) }
}

impl From<EncounterId> for Uuid {
    fn from(id: EncounterId) -> Self { id.0 }
}

impl From<Uuid> for EncounterId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl std::fmt::Display for EncounterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterKind { Combat, Diplomacy, Puzzle, Story, Random }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterStatus { Planned, Active, Resolved, Abandoned, DeadEnded }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum EncounterError {
    #[error("encounter is not planned")]
    NotPlanned,
    #[error("encounter is not active")]
    NotActive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    id: EncounterId,
    session_id: SessionId,
    story_id: StoryId,
    metadata: Metadata,
    kind: EncounterKind,
    description: Prose,
    outcome: Option<Prose>,
    reason: Option<Prose>,
    status: EncounterStatus,
}

impl Default for Encounter {
    fn default() -> Self {
        Self {
            id: EncounterId::default(),
            session_id: SessionId::default(),
            story_id: StoryId::default(),
            metadata: Metadata::default(),
            kind: EncounterKind::Story,
            description: Prose::default(),
            outcome: None,
            reason: None,
            status: EncounterStatus::Planned,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterEvent {
    Planned { id: EncounterId, session_id: SessionId, story_id: StoryId, owner: UserId, kind: EncounterKind, description: Prose },
    Improvised { id: EncounterId, session_id: SessionId, story_id: StoryId, owner: UserId, kind: EncounterKind, description: Prose },
    Activated,
    Resolved { outcome: Prose },
    Abandoned,
    DeadEnded { reason: Prose },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterCommand {
    Plan { session_id: SessionId, story_id: StoryId, owner: UserId, kind: EncounterKind, description: Prose },
    Improvise { session_id: SessionId, story_id: StoryId, owner: UserId, kind: EncounterKind, description: Prose },
    Activate,
    Resolve { outcome: Prose },
    Abandon,
    DeadEnd { reason: Prose },
}

impl Apply<EncounterEvent> for Encounter {
    fn apply(self, event: EncounterEvent) -> Self {
        match event {
            EncounterEvent::Planned { id, session_id, story_id, owner, kind, description } => Self {
                id,
                session_id,
                story_id,
                metadata: Metadata::new(owner),
                kind,
                description,
                outcome: None,
                reason: None,
                status: EncounterStatus::Planned,
            },
            EncounterEvent::Improvised { id, session_id, story_id, owner, kind, description } => Self {
                id,
                session_id,
                story_id,
                metadata: Metadata::new(owner),
                kind,
                description,
                outcome: None,
                reason: None,
                status: EncounterStatus::Active,
            },
            EncounterEvent::Activated => Self { status: EncounterStatus::Active, ..self },
            EncounterEvent::Resolved { outcome } => Self { status: EncounterStatus::Resolved, outcome: Some(outcome), ..self },
            EncounterEvent::Abandoned => Self { status: EncounterStatus::Abandoned, ..self },
            EncounterEvent::DeadEnded { reason } => Self { status: EncounterStatus::DeadEnded, reason: Some(reason), ..self },
        }
    }
}

impl Aggregate for Encounter {
    type Id = EncounterId;
    type Command = EncounterCommand;
    type Event = EncounterEvent;
    type Error = EncounterError;

    fn id(&self) -> EncounterId { self.id }

    fn handle(&self, command: EncounterCommand) -> Result<Vec<EncounterEvent>, EncounterError> {
        match command {
            EncounterCommand::Plan { session_id, story_id, owner, kind, description } => {
                Ok(vec![EncounterEvent::Planned { id: EncounterId::new(), session_id, story_id, owner, kind, description }])
            }
            EncounterCommand::Improvise { session_id, story_id, owner, kind, description } => {
                Ok(vec![EncounterEvent::Improvised { id: EncounterId::new(), session_id, story_id, owner, kind, description }])
            }
            EncounterCommand::Activate => match self.status {
                EncounterStatus::Planned => Ok(vec![EncounterEvent::Activated]),
                _ => Err(EncounterError::NotPlanned),
            },
            EncounterCommand::Resolve { outcome } => match self.status {
                EncounterStatus::Active => Ok(vec![EncounterEvent::Resolved { outcome }]),
                _ => Err(EncounterError::NotActive),
            },
            EncounterCommand::Abandon => match self.status {
                EncounterStatus::Planned => Ok(vec![EncounterEvent::Abandoned]),
                _ => Err(EncounterError::NotPlanned),
            },
            EncounterCommand::DeadEnd { reason } => match self.status {
                EncounterStatus::Planned => Ok(vec![EncounterEvent::DeadEnded { reason }]),
                _ => Err(EncounterError::NotPlanned),
            },
        }
    }
}

impl Encounter {
    pub fn id(&self) -> EncounterId { self.id }
    pub fn session_id(&self) -> SessionId { self.session_id }
    pub fn story_id(&self) -> StoryId { self.story_id }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn kind(&self) -> &EncounterKind { &self.kind }
    pub fn description(&self) -> &Prose { &self.description }
    pub fn outcome(&self) -> Option<&Prose> { self.outcome.as_ref() }
    pub fn reason(&self) -> Option<&Prose> { self.reason.as_ref() }
    pub fn status(&self) -> &EncounterStatus { &self.status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planned(kind: EncounterKind) -> Encounter {
        let events = Encounter::default().handle(EncounterCommand::Plan {
            session_id: SessionId::new(),
            story_id: StoryId::new(),
            owner: UserId::new(),
            kind,
            description: Prose::new("An encounter."),
        }).unwrap();
        events.into_iter().fold(Encounter::default(), Encounter::apply)
    }

    #[test]
    fn a_planned_encounter_starts_planned() {
        assert_eq!(planned(EncounterKind::Combat).status(), &EncounterStatus::Planned);
    }

    #[test]
    fn an_improvised_encounter_starts_active() {
        let events = Encounter::default().handle(EncounterCommand::Improvise {
            session_id: SessionId::new(),
            story_id: StoryId::new(),
            owner: UserId::new(),
            kind: EncounterKind::Random,
            description: Prose::new("A wolf pack appears."),
        }).unwrap();
        let e = events.into_iter().fold(Encounter::default(), Encounter::apply);

        assert_eq!(e.status(), &EncounterStatus::Active);
    }

    #[test]
    fn a_planned_encounter_can_be_activated() {
        let e = planned(EncounterKind::Diplomacy);
        let events = e.handle(EncounterCommand::Activate).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.status(), &EncounterStatus::Active);
    }

    #[test]
    fn an_active_encounter_can_be_resolved_with_an_outcome() {
        let e = planned(EncounterKind::Combat);
        let events = e.handle(EncounterCommand::Activate).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        let outcome = Prose::new("The party won. Barely.");
        let events = e.handle(EncounterCommand::Resolve { outcome: outcome.clone() }).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.status(), &EncounterStatus::Resolved);
        assert_eq!(e.outcome(), Some(&outcome));
    }

    #[test]
    fn a_planned_encounter_can_be_abandoned() {
        let e = planned(EncounterKind::Puzzle);
        let events = e.handle(EncounterCommand::Abandon).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.status(), &EncounterStatus::Abandoned);
    }

    #[test]
    fn a_planned_encounter_can_be_dead_ended_with_a_reason() {
        let e = planned(EncounterKind::Diplomacy);
        let reason = Prose::new("Lord Vasek was killed last session.");
        let events = e.handle(EncounterCommand::DeadEnd { reason: reason.clone() }).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.status(), &EncounterStatus::DeadEnded);
        assert_eq!(e.reason(), Some(&reason));
    }

    #[test]
    fn a_non_planned_encounter_cannot_be_activated() {
        let e = planned(EncounterKind::Combat);
        let events = e.handle(EncounterCommand::Activate).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.handle(EncounterCommand::Activate), Err(EncounterError::NotPlanned));
    }

    #[test]
    fn a_non_active_encounter_cannot_be_resolved() {
        assert_eq!(
            planned(EncounterKind::Story).handle(EncounterCommand::Resolve {
                outcome: Prose::new("outcome")
            }),
            Err(EncounterError::NotActive)
        );
    }

    #[test]
    fn an_active_encounter_cannot_be_abandoned() {
        let e = planned(EncounterKind::Combat);
        let events = e.handle(EncounterCommand::Activate).unwrap();
        let e = events.into_iter().fold(e, Encounter::apply);

        assert_eq!(e.handle(EncounterCommand::Abandon), Err(EncounterError::NotPlanned));
    }
}

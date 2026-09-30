use serde::{Deserialize, Serialize};

use crate::session::SessionId;
use crate::shared::{Metadata, Name, NameError, Prose, UserId};
use crate::story::StoryId;
use crate::traits::{Aggregate, Apply};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum LocationError {
    #[error(transparent)]
    Name(#[from] NameError),
    #[error("location is already destroyed")]
    AlreadyDestroyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(Uuid);

impl LocationId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for LocationId {
    fn default() -> Self { Self(Uuid::nil()) }
}

impl From<LocationId> for Uuid {
    fn from(id: LocationId) -> Self { id.0 }
}

impl From<Uuid> for LocationId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl std::fmt::Display for LocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationStatus { Known, Visited, Destroyed }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    id: LocationId,
    story_id: StoryId,
    metadata: Metadata,
    name: Name,
    description: Prose,
    status: LocationStatus,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            id: LocationId::default(),
            story_id: StoryId::default(),
            metadata: Metadata::default(),
            name: Name::default(),
            description: Prose::default(),
            status: LocationStatus::Known,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationEvent {
    Created { id: LocationId, story_id: StoryId, owner: UserId, name: Name, description: Prose },
    Visited { session_id: Option<SessionId> },
    Destroyed { session_id: Option<SessionId> },
    Renamed { name: Name },
    DescriptionUpdated { description: Prose },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationCommand {
    Create { story_id: StoryId, owner: UserId, name: Name, description: Prose },
    Visit { session_id: Option<SessionId> },
    Destroy { session_id: Option<SessionId> },
    Rename { name: Name },
    UpdateDescription { description: Prose },
}

impl Apply<LocationEvent> for Location {
    fn apply(self, event: LocationEvent) -> Self {
        match event {
            LocationEvent::Created { id, story_id, owner, name, description } => Self {
                id,
                story_id,
                metadata: Metadata::new(owner),
                name,
                description,
                status: LocationStatus::Known,
            },
            LocationEvent::Visited { .. } => Self { status: LocationStatus::Visited, ..self },
            LocationEvent::Destroyed { .. } => Self { status: LocationStatus::Destroyed, ..self },
            LocationEvent::Renamed { name } => Self { name, ..self },
            LocationEvent::DescriptionUpdated { description } => Self { description, ..self },
        }
    }
}

impl Aggregate for Location {
    type Id = LocationId;
    type Command = LocationCommand;
    type Event = LocationEvent;
    type Error = LocationError;

    fn id(&self) -> LocationId { self.id }

    fn handle(&self, command: LocationCommand) -> Result<Vec<LocationEvent>, LocationError> {
        match command {
            LocationCommand::Create { story_id, owner, name, description } => {
                Ok(vec![LocationEvent::Created { id: LocationId::new(), story_id, owner, name, description }])
            }
            LocationCommand::Visit { session_id } => match self.status {
                LocationStatus::Destroyed => Err(LocationError::AlreadyDestroyed),
                _ => Ok(vec![LocationEvent::Visited { session_id }]),
            },
            LocationCommand::Destroy { session_id } => match self.status {
                LocationStatus::Destroyed => Err(LocationError::AlreadyDestroyed),
                _ => Ok(vec![LocationEvent::Destroyed { session_id }]),
            },
            LocationCommand::Rename { name } => Ok(vec![LocationEvent::Renamed { name }]),
            LocationCommand::UpdateDescription { description } => Ok(vec![LocationEvent::DescriptionUpdated { description }]),
        }
    }
}

impl Location {
    pub fn id(&self) -> LocationId { self.id }
    pub fn story_id(&self) -> StoryId { self.story_id }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn name(&self) -> &Name { &self.name }
    pub fn description(&self) -> &Prose { &self.description }
    pub fn status(&self) -> &LocationStatus { &self.status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(name: &str) -> Location {
        let events = Location::default().handle(LocationCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            name: Name::parse(name).unwrap(),
            description: Prose::default(),
        }).unwrap();
        events.into_iter().fold(Location::default(), Location::apply)
    }

    #[test]
    fn a_location_belongs_to_a_story() {
        let story_id = StoryId::new();
        let events = Location::default().handle(LocationCommand::Create {
            story_id,
            owner: UserId::new(),
            name: Name::parse("The Dark Forest").unwrap(),
            description: Prose::default(),
        }).unwrap();
        let loc = events.into_iter().fold(Location::default(), Location::apply);

        assert_eq!(loc.story_id(), story_id);
    }

    #[test]
    fn a_new_location_is_known() {
        assert_eq!(location("The Dark Forest").status(), &LocationStatus::Known);
    }

    #[test]
    fn a_location_requires_a_name() {
        assert_eq!(Name::parse("  "), Err(NameError::Empty));
    }

    #[test]
    fn a_known_location_can_be_visited() {
        let loc = location("The Dark Forest");
        let events = loc.handle(LocationCommand::Visit { session_id: None }).unwrap();
        let loc = events.into_iter().fold(loc, Location::apply);

        assert_eq!(loc.status(), &LocationStatus::Visited);
    }

    #[test]
    fn a_destroyed_location_cannot_be_visited() {
        let loc = location("The Dark Forest");
        let events = loc.handle(LocationCommand::Destroy { session_id: None }).unwrap();
        let loc = events.into_iter().fold(loc, Location::apply);

        assert_eq!(loc.handle(LocationCommand::Visit { session_id: None }), Err(LocationError::AlreadyDestroyed));
    }

    #[test]
    fn a_destroyed_location_cannot_be_destroyed_again() {
        let loc = location("The Dark Forest");
        let events = loc.handle(LocationCommand::Destroy { session_id: None }).unwrap();
        let loc = events.into_iter().fold(loc, Location::apply);

        assert_eq!(loc.handle(LocationCommand::Destroy { session_id: None }), Err(LocationError::AlreadyDestroyed));
    }

    #[test]
    fn visiting_a_location_records_the_session() {
        let session_id = SessionId::new();
        let loc = location("The Dark Forest");
        let events = loc.handle(LocationCommand::Visit { session_id: Some(session_id) }).unwrap();

        assert_eq!(events[0], LocationEvent::Visited { session_id: Some(session_id) });
    }
}

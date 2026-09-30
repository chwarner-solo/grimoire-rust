use domain::location::{Location, LocationCommand, LocationEvent, LocationId};
use domain::session::SessionId;
use domain::shared::{Name, Prose, UserId};
use domain::story::StoryId;
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct LocationService<R> {
    repository: R,
}

impl<R: AggregateRepository<Location>> LocationService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[tracing::instrument(skip(self), fields(owner = %owner, story_id = %story_id))]
    pub async fn create(
        &self,
        story_id: StoryId,
        owner: UserId,
        name: &str,
        description: &str,
    ) -> Result<LocationId, AppError> {
        let command = LocationCommand::Create {
            story_id,
            owner,
            name: Name::parse(name)?,
            description: Prose::new(description),
        };
        let events = Location::default().handle(command)?;
        let id = { match events.first() {
            Some(LocationEvent::Created { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(location_id = %id, "location created");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: LocationId) -> Result<Location, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn rename(&self, id: LocationId, name: &str) -> Result<(), AppError> {
        let location = self.repository.load(id).await?;
        let events = location.handle(LocationCommand::Rename { name: Name::parse(name)? })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_description(&self, id: LocationId, description: &str) -> Result<(), AppError> {
        let location = self.repository.load(id).await?;
        let events = location.handle(LocationCommand::UpdateDescription { description: Prose::new(description) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn visit(&self, id: LocationId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let location = self.repository.load(id).await?;
        let events = location.handle(LocationCommand::Visit { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(location_id = %id, "location visited");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn destroy(&self, id: LocationId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let location = self.repository.load(id).await?;
        let events = location.handle(LocationCommand::Destroy { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(location_id = %id, "location destroyed");
        Ok(())
    }
}

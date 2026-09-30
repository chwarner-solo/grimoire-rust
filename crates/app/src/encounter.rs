use domain::encounter::{Encounter, EncounterCommand, EncounterEvent, EncounterId, EncounterKind};
use domain::session::SessionId;
use domain::shared::{Prose, UserId};
use domain::story::StoryId;
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct EncounterService<R> {
    repository: R,
}

impl<R: AggregateRepository<Encounter>> EncounterService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[tracing::instrument(skip(self), fields(owner = %owner, session_id = %session_id))]
    pub async fn plan(
        &self,
        session_id: SessionId,
        story_id: StoryId,
        owner: UserId,
        kind: EncounterKind,
        description: &str,
    ) -> Result<EncounterId, AppError> {
        let command = EncounterCommand::Plan {
            session_id,
            story_id,
            owner,
            kind,
            description: Prose::new(description),
        };
        let events = Encounter::default().handle(command)?;
        let id = { match events.first() {
            Some(EncounterEvent::Planned { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter planned");
        Ok(id)
    }

    #[tracing::instrument(skip(self), fields(owner = %owner, session_id = %session_id))]
    pub async fn improvise(
        &self,
        session_id: SessionId,
        story_id: StoryId,
        owner: UserId,
        kind: EncounterKind,
        description: &str,
    ) -> Result<EncounterId, AppError> {
        let command = EncounterCommand::Improvise {
            session_id,
            story_id,
            owner,
            kind,
            description: Prose::new(description),
        };
        let events = Encounter::default().handle(command)?;
        let id = { match events.first() {
            Some(EncounterEvent::Improvised { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter improvised");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: EncounterId) -> Result<Encounter, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn activate(&self, id: EncounterId) -> Result<(), AppError> {
        let encounter = self.repository.load(id).await?;
        let events = encounter.handle(EncounterCommand::Activate)?;
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter activated");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn resolve(&self, id: EncounterId, outcome: &str) -> Result<(), AppError> {
        let encounter = self.repository.load(id).await?;
        let events = encounter.handle(EncounterCommand::Resolve { outcome: Prose::new(outcome) })?;
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter resolved");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn abandon(&self, id: EncounterId) -> Result<(), AppError> {
        let encounter = self.repository.load(id).await?;
        let events = encounter.handle(EncounterCommand::Abandon)?;
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter abandoned");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn dead_end(&self, id: EncounterId, reason: &str) -> Result<(), AppError> {
        let encounter = self.repository.load(id).await?;
        let events = encounter.handle(EncounterCommand::DeadEnd { reason: Prose::new(reason) })?;
        self.repository.save(id, events).await?;
        tracing::info!(encounter_id = %id, "encounter dead-ended");
        Ok(())
    }
}

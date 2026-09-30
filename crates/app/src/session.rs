use domain::session::{Session, SessionCommand, SessionEvent, SessionId};
use domain::shared::{Prose, UserId};
use domain::story::StoryId;
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct SessionService<R> {
    repository: R,
}

impl<R: AggregateRepository<Session>> SessionService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[tracing::instrument(skip(self), fields(owner = %owner, story_id = %story_id))]
    pub async fn create(
        &self,
        story_id: StoryId,
        owner: UserId,
        number: u32,
        date: String,
        notes: &str,
    ) -> Result<SessionId, AppError> {
        let command = SessionCommand::Create {
            story_id,
            owner,
            number,
            date,
            notes: Prose::new(notes),
        };
        let events = Session::default().handle(command)?;
        let id = { match events.first() {
            Some(SessionEvent::Created { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(session_id = %id, number, "session created");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: SessionId) -> Result<Session, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn open(&self, id: SessionId) -> Result<(), AppError> {
        let session = self.repository.load(id).await?;
        let events = session.handle(SessionCommand::Open)?;
        self.repository.save(id, events).await?;
        tracing::info!(session_id = %id, "session opened");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_notes(&self, id: SessionId, notes: &str) -> Result<(), AppError> {
        let session = self.repository.load(id).await?;
        let events = session.handle(SessionCommand::UpdateNotes { notes: Prose::new(notes) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn summarize(&self, id: SessionId, summary: &str) -> Result<(), AppError> {
        let session = self.repository.load(id).await?;
        let events = session.handle(SessionCommand::Summarize { summary: Prose::new(summary) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn close(&self, id: SessionId) -> Result<(), AppError> {
        let session = self.repository.load(id).await?;
        let events = session.handle(SessionCommand::Close)?;
        self.repository.save(id, events).await?;
        tracing::info!(session_id = %id, "session closed");
        Ok(())
    }
}

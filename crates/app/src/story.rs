use domain::shared::{Prose, UserId};
use domain::story::{Story, StoryCommand, StoryEvent, StoryId, Title};
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct StoryService<R> {
    repository: R,
}

impl<R: AggregateRepository<Story>> StoryService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[tracing::instrument(skip(self), fields(owner = %owner))]
    pub async fn create(&self, owner: UserId, title: &str, prose: &str) -> Result<StoryId, AppError> {
        let command = StoryCommand::Create {
            owner,
            title: Title::parse(title)?,
            prose: Prose::new(prose),
        };
        let events = Story::default().handle(command)?;
        let id = { match events.first() {
            Some(StoryEvent::Created { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(story_id = %id, "story created");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: StoryId) -> Result<Story, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_title(&self, id: StoryId, title: &str) -> Result<(), AppError> {
        let story = self.repository.load(id).await?;
        let events = story.handle(StoryCommand::UpdateTitle { title: Title::parse(title)? })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_prose(&self, id: StoryId, prose: &str) -> Result<(), AppError> {
        let story = self.repository.load(id).await?;
        let events = story.handle(StoryCommand::UpdateProse { prose: Prose::new(prose) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn close(&self, id: StoryId) -> Result<(), AppError> {
        let story = self.repository.load(id).await?;
        let events = story.handle(StoryCommand::Close)?;
        self.repository.save(id, events).await?;
        tracing::info!(story_id = %id, "story closed");
        Ok(())
    }
}

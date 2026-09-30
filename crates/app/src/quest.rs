use domain::character::CharacterId;
use domain::quest::{Quest, QuestCommand, QuestEvent, QuestId, QuestKind};
use domain::session::SessionId;
use domain::shared::{Name, Prose, UserId};
use domain::story::StoryId;
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct QuestService<R> {
    repository: R,
}

impl<R: AggregateRepository<Quest>> QuestService<R> {
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
        kind: QuestKind,
        character_id: Option<CharacterId>,
    ) -> Result<QuestId, AppError> {
        let command = QuestCommand::Create {
            story_id,
            owner,
            name: Name::parse(name)?,
            description: Prose::new(description),
            kind,
            character_id,
        };
        let events = Quest::default().handle(command)?;
        let id = { match events.first() {
            Some(QuestEvent::Created { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(quest_id = %id, "quest created");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: QuestId) -> Result<Quest, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn rename(&self, id: QuestId, name: &str) -> Result<(), AppError> {
        let quest = self.repository.load(id).await?;
        let events = quest.handle(QuestCommand::Rename { name: Name::parse(name)? })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_description(&self, id: QuestId, description: &str) -> Result<(), AppError> {
        let quest = self.repository.load(id).await?;
        let events = quest.handle(QuestCommand::UpdateDescription { description: Prose::new(description) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn complete(&self, id: QuestId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let quest = self.repository.load(id).await?;
        let events = quest.handle(QuestCommand::Complete { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(quest_id = %id, "quest completed");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn fail(&self, id: QuestId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let quest = self.repository.load(id).await?;
        let events = quest.handle(QuestCommand::Fail { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(quest_id = %id, "quest failed");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn abandon(&self, id: QuestId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let quest = self.repository.load(id).await?;
        let events = quest.handle(QuestCommand::Abandon { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(quest_id = %id, "quest abandoned");
        Ok(())
    }
}

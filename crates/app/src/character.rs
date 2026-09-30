use domain::character::{Character, CharacterCommand, CharacterEvent, CharacterId, CharacterKind};
use domain::session::SessionId;
use domain::shared::{Name, Prose, UserId};
use domain::story::StoryId;
use domain::traits::Aggregate;

use crate::{AggregateRepository, AppError};

pub struct CharacterService<R> {
    repository: R,
}

impl<R: AggregateRepository<Character>> CharacterService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[tracing::instrument(skip(self), fields(owner = %owner, story_id = %story_id))]
    pub async fn create(
        &self,
        story_id: StoryId,
        owner: UserId,
        name: &str,
        backstory: &str,
        kind: CharacterKind,
    ) -> Result<CharacterId, AppError> {
        let command = CharacterCommand::Create {
            story_id,
            owner,
            name: Name::parse(name)?,
            backstory: Prose::new(backstory),
            kind,
        };
        let events = Character::default().handle(command)?;
        let id = { match events.first() {
            Some(CharacterEvent::Created { id, .. }) => *id,
            _ => unreachable!(),
        }};
        self.repository.save(id, events).await?;
        tracing::info!(character_id = %id, "character created");
        Ok(id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, id: CharacterId) -> Result<Character, AppError> {
        Ok(self.repository.load(id).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn rename(&self, id: CharacterId, name: &str) -> Result<(), AppError> {
        let character = self.repository.load(id).await?;
        let events = character.handle(CharacterCommand::Rename { name: Name::parse(name)? })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_backstory(&self, id: CharacterId, backstory: &str) -> Result<(), AppError> {
        let character = self.repository.load(id).await?;
        let events = character.handle(CharacterCommand::UpdateBackstory { backstory: Prose::new(backstory) })?;
        self.repository.save(id, events).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn depart(&self, id: CharacterId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let character = self.repository.load(id).await?;
        let events = character.handle(CharacterCommand::Depart { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(character_id = %id, "character departed");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn kill(&self, id: CharacterId, session_id: Option<SessionId>) -> Result<(), AppError> {
        let character = self.repository.load(id).await?;
        let events = character.handle(CharacterCommand::Kill { session_id })?;
        self.repository.save(id, events).await?;
        tracing::info!(character_id = %id, "character killed");
        Ok(())
    }
}

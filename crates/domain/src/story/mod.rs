use crate::shared::{Metadata, Prose, UserId};
use crate::traits::{Aggregate, Apply};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Story { id: StoryId, metadata: Metadata, title: Title, prose: Prose, status: StoryStatus }

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct StoryId(uuid::Uuid);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(String);

impl StoryId {
    pub fn new() -> Self { Self(uuid::Uuid::now_v7()) }
}

impl Default for StoryId {
    fn default() -> Self { Self(uuid::Uuid::nil()) }
}

impl Default for Title {
    fn default() -> Self { Self(String::new()) }
}


#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TitleError {
    #[error("title must not be empty")]
    Empty,
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoryError {
    #[error(transparent)]
    Title(#[from] TitleError),
    #[error("invalid story")]
    InvalidCommand,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryEvent {
    Created { id: StoryId, user_id: UserId, title: Title, prose: Prose },
    TitleUpdated { title: Title },
    ProseUpdated { prose: Prose },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryCommand {
    Create { owner: UserId, title: Title, prose: Prose },
    UpdateTitle { title: Title},
    UpdateProse { prose: Prose },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryStatus {
    Active,
    Inactive,
}

impl Story {
    pub fn create(owner: UserId, title: &str, prose: &str) -> Result<Story, StoryError> {
        Ok(Self {
            id: StoryId::new(),
            title: Title::parse(title)?,
            metadata: Metadata::new(owner),
            prose: Prose::new(prose),
            status: StoryStatus::Active,
        })
    }
    pub fn metadata(&self) -> &Metadata{ &self.metadata }
    pub fn prose(&self) -> &Prose { &self.prose }
    pub fn id(&self) -> &StoryId { &self.id }
    pub fn title(&self) -> &Title { &self.title }
    pub fn status(&self) -> &StoryStatus { &self.status }
}

impl Apply<StoryEvent> for Story {
    fn apply(self, event: StoryEvent) -> Self {
        match event {
            StoryEvent::Created { id, user_id, title, prose } => {
                Self { id, title, metadata: Metadata::new(user_id), prose, status: StoryStatus::Active }
            }
            StoryEvent::TitleUpdated { title } => Self { title, ..self },
            StoryEvent::ProseUpdated { prose } => Self { prose, ..self },
            StoryEvent::Closed => Self { status: StoryStatus::Inactive, ..self },
        }
    }
}

impl Aggregate for Story {
    type Command = StoryCommand;
    type Event = StoryEvent;
    type Error = StoryError;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            StoryCommand::Create { owner, title, prose } => {
                Ok(vec![StoryEvent::Created { id: StoryId::new(), user_id: owner, title, prose }])
            },
            StoryCommand::UpdateTitle { title } => {
                Ok(vec![StoryEvent::TitleUpdated { title }])
            }
            StoryCommand::UpdateProse { prose } => {
                Ok(vec![StoryEvent::ProseUpdated { prose }])
            }
            StoryCommand::Close => {
                Ok(vec![StoryEvent::Closed])
            }
        }
    }


}

impl Default for Story {
    fn default() -> Self {
        Self {
            id: StoryId::default(),
            metadata: Metadata::default(),
            title: Title::default(),
            prose: Prose::default(),
            status: StoryStatus::Inactive,
        }
    }
}

impl Title {
    pub fn parse(raw: &str) -> Result<Self, TitleError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(TitleError::Empty);
        }
        Ok(Self(trimmed.to_owned()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_story_has_a_title_and_prose() {
        let story = Story::create(UserId::new(), "The Long Road", "It began with ash.").unwrap();

        assert_eq!(story.title().as_str(),"The Long Road");
        assert_eq!(story.prose().as_str(), "It began with ash.");
    }

    #[test]
    fn a_story_requires_a_title() {
        let result = Story::create(UserId::new(), "    ", "Some prose.");

        assert_eq!(result, Err(StoryError::Title(TitleError::Empty)));
    }

    #[test]
    fn a_new_story_is_owned_by_its_creator() {
        let owner = UserId::new();
        let story = Story::create(owner, "The Long Road", "It began with ash.").unwrap();

        assert_eq!(story.metadata().owner(), owner);
    }

    #[test]
    fn each_story_gets_a_distinct_identity() {
        let owner = UserId::new();
        let a = Story::create(owner, "A", "").unwrap();
        let b = Story::create(owner, "B", "").unwrap();

        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn a_title_is_trimmed() {
        let title = Title::parse("   The Long Road   ").unwrap();
        assert_eq!(title.as_str(), "The Long Road");
    }

    #[test]
    fn a_blank_title_is_rejected() {
        assert_eq!(Title::parse("   "), Err(TitleError::Empty));
    }
}
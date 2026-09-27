use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
    pub fn from_uuid(id: Uuid) -> Self { Self(id) }
}

impl Default for UserId {
    fn default() -> Self { Self(Uuid::nil()) }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Metadata { owner: UserId }

impl Metadata {
    pub fn new(owner: UserId) -> Self { Self { owner } }
    pub fn owner(&self) -> UserId { self.owner }
}

impl Default for Metadata {
    fn default() -> Self { Self { owner: UserId::default() } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prose(String);

impl Prose {
    pub(crate) fn new(prose: &str) -> Self {
        Prose(prose.to_owned())
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl Default for Prose {
    fn default() -> Self { Self(String::new()) }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    #[error("name must not be empty")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() { return Err(NameError::Empty) }
        Ok(Self(trimmed.to_owned()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl Default for Name {
    fn default() -> Self { Self(String::new()) }
}
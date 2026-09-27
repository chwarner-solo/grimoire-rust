use crate::shared::{Metadata, Prose, UserId};
use crate::story::StoryId;
use crate::traits::{Aggregate, Apply};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self { Self(Uuid::now_v7()) }
}

impl Default for SessionId {
    fn default() -> Self { Self(Uuid::nil()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus { Planned, Active, Closed }

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("session is not planned")]
    NotPlanned,
    #[error("session is not active")]
    NotActive,
    #[error("session is already closed")]
    AlreadyClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    id: SessionId,
    story_id: StoryId,
    metadata: Metadata,
    number: u32,
    date: String,
    notes: Prose,
    summary: Prose,
    status: SessionStatus,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: SessionId::default(),
            story_id: StoryId::default(),
            metadata: Metadata::default(),
            number: 0,
            date: String::new(),
            notes: Prose::default(),
            summary: Prose::default(),
            status: SessionStatus::Planned,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Created { id: SessionId, story_id: StoryId, owner: UserId, number: u32, date: String, notes: Prose },
    Opened,
    NotesUpdated { notes: Prose },
    Summarized { summary: Prose },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    Create { story_id: StoryId, owner: UserId, number: u32, date: String, notes: Prose },
    Open,
    UpdateNotes { notes: Prose },
    Summarize { summary: Prose },
    Close,
}

impl Apply<SessionEvent> for Session {
    fn apply(self, event: SessionEvent) -> Self {
        match event {
            SessionEvent::Created { id, story_id, owner, number, date, notes } => Self {
                id,
                story_id,
                metadata: Metadata::new(owner),
                number,
                date,
                notes,
                summary: Prose::default(),
                status: SessionStatus::Planned,
            },
            SessionEvent::Opened => Self { status: SessionStatus::Active, ..self },
            SessionEvent::NotesUpdated { notes } => Self { notes, ..self },
            SessionEvent::Summarized { summary } => Self { summary, ..self },
            SessionEvent::Closed => Self { status: SessionStatus::Closed, ..self },
        }
    }
}

impl Aggregate for Session {
    type Command = SessionCommand;
    type Event = SessionEvent;
    type Error = SessionError;

    fn handle(&self, command: SessionCommand) -> Result<Vec<SessionEvent>, SessionError> {
        match command {
            SessionCommand::Create { story_id, owner, number, date, notes } => {
                Ok(vec![SessionEvent::Created { id: SessionId::new(), story_id, owner, number, date, notes }])
            }
            SessionCommand::Open => match self.status {
                SessionStatus::Planned => Ok(vec![SessionEvent::Opened]),
                _ => Err(SessionError::NotPlanned),
            },
            SessionCommand::UpdateNotes { notes } => match self.status {
                SessionStatus::Closed => Err(SessionError::AlreadyClosed),
                _ => Ok(vec![SessionEvent::NotesUpdated { notes }]),
            },
            SessionCommand::Summarize { summary } => Ok(vec![SessionEvent::Summarized { summary }]),
            SessionCommand::Close => match self.status {
                SessionStatus::Active => Ok(vec![SessionEvent::Closed]),
                _ => Err(SessionError::NotActive),
            },
        }
    }
}

impl Session {
    pub fn id(&self) -> SessionId { self.id }
    pub fn story_id(&self) -> StoryId { self.story_id }
    pub fn metadata(&self) -> &Metadata { &self.metadata }
    pub fn number(&self) -> u32 { self.number }
    pub fn date(&self) -> &str { &self.date }
    pub fn notes(&self) -> &Prose { &self.notes }
    pub fn summary(&self) -> &Prose { &self.summary }
    pub fn status(&self) -> &SessionStatus { &self.status }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(number: u32) -> Session {
        let events = Session::default().handle(SessionCommand::Create {
            story_id: StoryId::new(),
            owner: UserId::new(),
            number,
            date: "2026-09-27".to_string(),
            notes: Prose::default(),
        }).unwrap();
        events.into_iter().fold(Session::default(), Session::apply)
    }

    #[test]
    fn a_new_session_is_planned() {
        assert_eq!(session(1).status(), &SessionStatus::Planned);
    }

    #[test]
    fn a_planned_session_can_be_opened() {
        let s = session(1);
        let events = s.handle(SessionCommand::Open).unwrap();
        let s = events.into_iter().fold(s, Session::apply);

        assert_eq!(s.status(), &SessionStatus::Active);
    }

    #[test]
    fn an_active_session_can_be_closed() {
        let s = session(1);
        let events = s.handle(SessionCommand::Open).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        let events = s.handle(SessionCommand::Close).unwrap();
        let s = events.into_iter().fold(s, Session::apply);

        assert_eq!(s.status(), &SessionStatus::Closed);
    }

    #[test]
    fn a_planned_session_cannot_be_closed_directly() {
        assert_eq!(session(1).handle(SessionCommand::Close), Err(SessionError::NotActive));
    }

    #[test]
    fn an_already_open_session_cannot_be_opened_again() {
        let s = session(1);
        let events = s.handle(SessionCommand::Open).unwrap();
        let s = events.into_iter().fold(s, Session::apply);

        assert_eq!(s.handle(SessionCommand::Open), Err(SessionError::NotPlanned));
    }

    #[test]
    fn notes_cannot_be_updated_on_a_closed_session() {
        let s = session(1);
        let events = s.handle(SessionCommand::Open).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        let events = s.handle(SessionCommand::Close).unwrap();
        let s = events.into_iter().fold(s, Session::apply);

        assert_eq!(
            s.handle(SessionCommand::UpdateNotes { notes: Prose::default() }),
            Err(SessionError::AlreadyClosed)
        );
    }

    #[test]
    fn a_summary_can_be_written_at_any_status() {
        let summary = Prose::new("The party survived.");

        // planned
        let s = session(1);
        let events = s.handle(SessionCommand::Summarize { summary: summary.clone() }).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        assert_eq!(s.summary(), &summary);

        // closed
        let s = session(2);
        let events = s.handle(SessionCommand::Open).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        let events = s.handle(SessionCommand::Close).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        let events = s.handle(SessionCommand::Summarize { summary: summary.clone() }).unwrap();
        let s = events.into_iter().fold(s, Session::apply);
        assert_eq!(s.summary(), &summary);
    }

    #[test]
    fn a_session_records_its_number_and_date() {
        let s = session(3);
        assert_eq!(s.number(), 3);
        assert_eq!(s.date(), "2026-09-27");
    }
}

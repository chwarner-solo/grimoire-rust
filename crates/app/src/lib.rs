pub mod character;
pub mod encounter;
pub mod error;
pub mod location;
pub mod quest;
pub mod session;
pub mod story;

pub use error::AppError;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::traits::Aggregate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Domain event vocabulary ---

pub use domain::character::CharacterEvent;
pub use domain::encounter::EncounterEvent;
pub use domain::location::LocationEvent;
pub use domain::quest::QuestEvent;
pub use domain::session::SessionEvent;
pub use domain::story::StoryEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    Story(StoryEvent),
    Character(CharacterEvent),
    Quest(QuestEvent),
    Location(LocationEvent),
    Session(SessionEvent),
    Encounter(EncounterEvent),
}

// --- Event envelope ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
    pub aggregate_id: Uuid,
    pub sequence: u64,
    pub occurred_at: DateTime<Utc>,
    pub payload: E,
}

// --- Error types ---

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("aggregate not found")]
    NotFound,
    #[error(transparent)]
    Store(#[from] EventStoreError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("store failure: {0}")]
    Failure(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("snapshot failure: {0}")]
    Failure(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("publish failure: {0}")]
    Failure(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("projection failure: {0}")]
    Failure(String),
}

// --- Port traits ---

/// The only interface use cases interact with.
/// Hides the decorator chain (RAM → snapshot → event store) and the publish pipeline.
#[async_trait]
pub trait AggregateRepository<A>: Send + Sync
where
    A: Aggregate + Send + Sync + 'static,
    A::Event: Send + Sync,
{
    async fn load(&self, id: A::Id) -> Result<A, RepositoryError>;
    async fn save(&self, id: A::Id, events: Vec<A::Event>) -> Result<(), RepositoryError>;
    async fn list_all(&self) -> Result<Vec<A>, RepositoryError>;
}

/// Append-only event log. Returns the last sequence number after appending.
/// Adapters: filesystem, GCS, Kafka.
#[async_trait]
pub trait EventStore<E: Send + Sync + 'static>: Send + Sync {
    async fn append(&self, aggregate_id: Uuid, events: &[E]) -> Result<u64, EventStoreError>;
    async fn load(&self, aggregate_id: Uuid) -> Result<Vec<EventEnvelope<E>>, EventStoreError>;
    async fn load_from(&self, aggregate_id: Uuid, after_sequence: u64) -> Result<Vec<EventEnvelope<E>>, EventStoreError>;
}

/// Persisted aggregate state snapshot for fast cold-start hydration.
/// Returns the aggregate and the event sequence it was snapshotted at.
/// Adapters: MongoDB, Firestore, DynamoDB.
#[async_trait]
pub trait SnapshotStore<A>: Send + Sync
where
    A: Aggregate + Send + Sync + 'static,
    A::Event: Send + Sync,
{
    async fn load(&self, id: A::Id) -> Result<Option<(A, u64)>, SnapshotError>;
    async fn save(&self, id: A::Id, state: &A, sequence: u64) -> Result<(), SnapshotError>;
}

/// Publishes events after a successful store append. Fan-out to all registered projections.
/// Adapters: in-process, GCP Pub/Sub, Kafka.
#[async_trait]
pub trait EventBus<E: Send + Sync + 'static>: Send + Sync {
    async fn publish(&self, envelope: &EventEnvelope<E>) -> Result<(), BusError>;
}

/// Downstream read model updated from the event bus.
/// Adapters: Neo4j (search/graph), BigQuery/Athena (analytics/AI).
#[async_trait]
pub trait EventProjection<E: Send + Sync + 'static>: Send + Sync {
    async fn project(&self, envelope: &EventEnvelope<E>) -> Result<(), ProjectionError>;
}

use std::marker::PhantomData;
use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

use app::{EventEnvelope, EventStore, EventStoreError};

#[derive(Clone)]
pub struct FsEventStore<E> {
    dir: PathBuf,
    _marker: PhantomData<E>,
}

impl<E> FsEventStore<E> {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), _marker: PhantomData }
    }

    fn path(&self, id: Uuid) -> PathBuf {
        self.dir.join(format!("{id}.jsonl"))
    }

    async fn event_count(&self, id: Uuid) -> Result<u64, EventStoreError> {
        let path = self.path(id);
        if !path.exists() {
            return Ok(0);
        }
        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| EventStoreError::Failure(e.to_string()))?;
        Ok(content.lines().filter(|l| !l.trim().is_empty()).count() as u64)
    }
}

#[async_trait]
impl<E> EventStore<E> for FsEventStore<E>
where
    E: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    async fn append(&self, aggregate_id: Uuid, events: &[E]) -> Result<u64, EventStoreError> {
        if events.is_empty() {
            return self.event_count(aggregate_id).await;
        }
        tokio::fs::create_dir_all(&self.dir).await
            .map_err(|e| EventStoreError::Failure(e.to_string()))?;

        let mut seq = self.event_count(aggregate_id).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true).append(true).open(self.path(aggregate_id)).await
            .map_err(|e| EventStoreError::Failure(e.to_string()))?;

        for event in events {
            seq += 1;
            let envelope = EventEnvelope {
                aggregate_id,
                sequence: seq,
                occurred_at: Utc::now(),
                payload: event.clone(),
            };
            let mut line = serde_json::to_string(&envelope)
                .map_err(|e| EventStoreError::Failure(e.to_string()))?;
            line.push('\n');
            file.write_all(line.as_bytes()).await
                .map_err(|e| EventStoreError::Failure(e.to_string()))?;
        }
        Ok(seq)
    }

    async fn load(&self, aggregate_id: Uuid) -> Result<Vec<EventEnvelope<E>>, EventStoreError> {
        self.load_from(aggregate_id, 0).await
    }

    async fn load_from(
        &self,
        aggregate_id: Uuid,
        after_sequence: u64,
    ) -> Result<Vec<EventEnvelope<E>>, EventStoreError> {
        let path = self.path(aggregate_id);
        if !path.exists() {
            return Ok(vec![]);
        }
        let file = tokio::fs::File::open(&path).await
            .map_err(|e| EventStoreError::Failure(e.to_string()))?;
        let mut lines = BufReader::new(file).lines();
        let mut result = vec![];

        while let Some(line) = lines.next_line().await
            .map_err(|e| EventStoreError::Failure(e.to_string()))?
        {
            if line.trim().is_empty() { continue; }
            let envelope: EventEnvelope<E> = serde_json::from_str(&line)
                .map_err(|e| EventStoreError::Failure(e.to_string()))?;
            if envelope.sequence > after_sequence {
                result.push(envelope);
            }
        }
        Ok(result)
    }
}

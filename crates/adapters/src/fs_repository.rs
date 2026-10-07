use std::path::PathBuf;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use app::{AggregateRepository, RepositoryError, SnapshotStore, EventStore};
use domain::traits::Aggregate;

use crate::fs_event_store::FsEventStore;
use crate::fs_snapshot::FsSnapshotStore;
use crate::ram_cache::RamCache;

#[derive(Clone)]
pub struct FsAggregateRepository<A>
where
    A: Aggregate + Clone,
{
    snapshots: FsSnapshotStore<A>,
    events: FsEventStore<A::Event>,
    cache: RamCache<A>,
}

impl<A> FsAggregateRepository<A>
where
    A: Aggregate + Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
    A::Id: Into<Uuid> + From<Uuid> + std::fmt::Display + Send + Sync,
    A::Event: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base = base_dir.into();
        Self {
            snapshots: FsSnapshotStore::new(base.join("snapshots")),
            events: FsEventStore::new(base.join("events")),
            cache: RamCache::new(),
        }
    }

    async fn hydrate(&self, id: A::Id) -> Result<A, RepositoryError> {
        let uuid: Uuid = id.into();

        let (mut state, snapshot_seq) = self.snapshots.load(id).await
            .map_err(RepositoryError::Snapshot)?
            .unwrap_or_else(|| (A::default(), 0));

        let envelopes = self.events.load_from(uuid, snapshot_seq).await
            .map_err(RepositoryError::Store)?;

        if snapshot_seq == 0 && envelopes.is_empty() {
            return Err(RepositoryError::NotFound);
        }

        for env in envelopes {
            state = state.apply(env.payload);
        }

        Ok(state)
    }
}

#[async_trait]
impl<A> AggregateRepository<A> for FsAggregateRepository<A>
where
    A: Aggregate + Clone + Default + Serialize + DeserializeOwned + Send + Sync + 'static,
    A::Id: Into<Uuid> + From<Uuid> + std::fmt::Display + Send + Sync,
    A::Event: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    async fn load(&self, id: A::Id) -> Result<A, RepositoryError> {
        if let Some(cached) = self.cache.get(&id).await {
            return Ok(cached);
        }
        let state = self.hydrate(id).await?;
        self.cache.insert(id, state.clone()).await;
        Ok(state)
    }

    async fn save(&self, id: A::Id, events: Vec<A::Event>) -> Result<(), RepositoryError> {
        let uuid: Uuid = id.into();

        // Fold events onto current state (cloning each event, keeping the vec owned)
        let current = self.cache.get(&id).await.unwrap_or_default();
        let new_state = events.iter().cloned().fold(current, |s, e| s.apply(e));

        // Persist events; snapshot store gets the new sequence for future cold-starts
        let new_seq = self.events.append(uuid, &events).await
            .map_err(RepositoryError::Store)?;

        self.snapshots.save(id, &new_state, new_seq).await
            .map_err(RepositoryError::Snapshot)?;

        self.cache.insert(id, new_state).await;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<A>, RepositoryError> {
        let uuids = self.snapshots.list_ids().await
            .map_err(RepositoryError::Snapshot)?;

        let mut results = Vec::with_capacity(uuids.len());
        for uuid in uuids {
            let id = A::Id::from(uuid);
            match self.load(id).await {
                Ok(a) => results.push(a),
                Err(RepositoryError::NotFound) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }
}

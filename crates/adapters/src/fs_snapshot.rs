use std::marker::PhantomData;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use app::{SnapshotError, SnapshotStore};
use domain::traits::Aggregate;

#[derive(Clone)]
pub struct FsSnapshotStore<A> {
    dir: PathBuf,
    _marker: PhantomData<A>,
}

impl<A: Aggregate> FsSnapshotStore<A>
where
    A::Id: std::fmt::Display,
{
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), _marker: PhantomData }
    }

    fn path(&self, id: A::Id) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }
}

#[derive(Serialize, Deserialize)]
struct SnapshotFile<A> {
    state: A,
    sequence: u64,
}

#[async_trait]
impl<A> SnapshotStore<A> for FsSnapshotStore<A>
where
    A: Aggregate + Serialize + DeserializeOwned + Send + Sync + 'static,
    A::Event: Send + Sync,
    A::Id: std::fmt::Display + Send + Sync,
{
    async fn load(&self, id: A::Id) -> Result<Option<(A, u64)>, SnapshotError> {
        let path = self.path(id);
        if !path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| SnapshotError::Failure(e.to_string()))?;
        let snap: SnapshotFile<A> = serde_json::from_str(&content)
            .map_err(|e| SnapshotError::Failure(e.to_string()))?;
        Ok(Some((snap.state, snap.sequence)))
    }

    async fn save(&self, id: A::Id, state: &A, sequence: u64) -> Result<(), SnapshotError> {
        tokio::fs::create_dir_all(&self.dir).await
            .map_err(|e| SnapshotError::Failure(e.to_string()))?;
        let content = serde_json::to_string_pretty(&SnapshotFile { state, sequence })
            .map_err(|e| SnapshotError::Failure(e.to_string()))?;
        tokio::fs::write(self.path(id), content).await
            .map_err(|e| SnapshotError::Failure(e.to_string()))?;
        Ok(())
    }
}

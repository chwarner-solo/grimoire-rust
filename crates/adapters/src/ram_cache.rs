use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use domain::traits::Aggregate;
use tokio::sync::RwLock;

pub struct RamCache<A: Aggregate> {
    inner: Arc<RwLock<HashMap<A::Id, (A, Instant)>>>,
}

impl<A: Aggregate + Clone> RamCache<A> {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, id: &A::Id) -> Option<A> {
        self.inner.read().await.get(id).map(|(a, _)| a.clone())
    }

    pub async fn insert(&self, id: A::Id, state: A) {
        self.inner.write().await.insert(id, (state, Instant::now()));
    }
}

impl<A: Aggregate + Clone> Clone for RamCache<A> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

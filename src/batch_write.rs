use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

#[derive(Debug)]
pub struct BatchWrite<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    inner: BTreeMap<K, V>,
    deleted: BTreeSet<K>,
}

impl<K, V> BatchWrite<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            deleted: Default::default(),
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.deleted.remove(&key);
        self.inner.insert(key, value);
    }

    pub fn delete(&mut self, key: K) {
        self.inner.remove(&key);
        self.deleted.insert(key);
    }
}

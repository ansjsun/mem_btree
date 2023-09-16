use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Action<V> {
    Put(V),
    Delete,
}
impl<V> Action<V> {
    pub(crate) fn value(self) -> V {
        match self {
            Self::Put(v) => v,
            Self::Delete => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct BatchWrite<K, V> {
    inner: BTreeMap<K, Action<V>>,
}

impl<K, V> BatchWrite<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.inner.insert(key, Action::Put(value));
    }

    pub fn delete(&mut self, key: K) {
        self.inner.insert(key, Action::Delete);
    }

    pub fn to_map(self) -> BTreeMap<K, Action<V>> {
        self.inner
    }
}

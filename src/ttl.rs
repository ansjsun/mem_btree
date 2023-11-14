use std::sync::atomic::AtomicI64;

use crate::{BTree, BatchWrite, Item, Iterator};

#[derive(Debug)]
pub struct TTL {
    live: AtomicI64,
    idle: AtomicI64,
}

impl TTL {
    pub fn new(live: i64, idle: i64) -> Self {
        Self {
            live: AtomicI64::new(live),
            idle: AtomicI64::new(idle),
        }
    }

    pub fn live(&self) -> i64 {
        self.live.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn idle(&self) -> i64 {
        self.idle.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_live(&self, live: i64) {
        self.live.store(live, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_idle(&self, idle: i64) {
        self.idle.store(idle, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_expir(&self, now: i64) -> bool {
        now > self.live() || now > self.idle()
    }
}

pub struct TTLBTree<K, V> {
    inner: BTree<K, (V, TTL)>,
    time_to_live: Option<i64>,
    time_to_idle: Option<i64>,
    max_capacity: Option<usize>,
    now: Option<i64>,
}

impl<K, V> TTLBTree<K, V>
where
    K: Ord + std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn new(m: usize) -> Self {
        Self {
            inner: BTree::new(m),
            time_to_live: None,
            time_to_idle: None,
            max_capacity: None,
            now: None,
        }
    }

    pub fn set_time_to_live(&mut self, ttl: i64) {
        self.time_to_live = Some(ttl);
    }

    pub fn set_time_to_idle(&mut self, ttl: i64) {
        self.time_to_idle = Some(ttl);
    }

    pub fn set_max_capacity(&mut self, capacity: usize) {
        self.max_capacity = Some(capacity);
    }

    fn mk_ttl(&self) -> TTL {
        let now = self.now();
        let live = self.time_to_live.map(|ttl| now + ttl).unwrap_or(i64::MAX);
        let idle = self.time_to_idle.map(|ttl| now + ttl).unwrap_or(i64::MAX);
        TTL::new(live, idle)
    }

    fn now(&self) -> i64 {
        self.now.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64
        })
    }
}

impl<K, V> TTLBTree<K, V>
where
    K: Ord + std::fmt::Debug,
    V: std::fmt::Debug,
{
    pub fn put(&mut self, key: K, value: V) -> Option<Item<K, (V, TTL)>> {
        self.inner.put(key, (value, self.mk_ttl()))
    }

    pub fn remove(&mut self, key: &K) -> Option<Item<K, (V, TTL)>> {
        self.inner.remove(key).map(|v| v)
    }

    pub fn write(&mut self, batch_write: BatchWrite<K, (V, TTL)>) {
        self.inner.write(batch_write)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some((value, ttl)) = self.inner.get(key) {
            let now = self.now();

            if ttl.is_expir(now) {
                return None;
            }

            if let Some(v) = self.time_to_idle {
                ttl.set_idle(now + v);
            }

            return Some(value);
        }
        None
    }

    pub fn get_with_expir(&self, key: &K) -> Option<(&V, bool)> {
        if let Some((value, ttl)) = self.inner.get(key) {
            return Some((value, ttl.is_expir(self.now())));
        }
        None
    }

    pub fn iter(&self) -> Iterator<K, (V, TTL)> {
        self.inner.iter()
    }
}

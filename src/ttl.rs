use std::sync::atomic::AtomicI64;

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

use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct InternalEntry<V> {
    pub value: V,
    pub expiration: Instant,
}

impl<V> InternalEntry<V> {
    pub fn new(value: V, expiration: Instant) -> Self {
        InternalEntry { value, expiration }
    }

    pub fn ttl(value: V, ttl: Duration) -> Self {
        Self::new(value, Instant::now() + ttl)
    }

    #[inline]
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expiration
    }

    pub fn get(&self) -> Option<&V> {
        if self.is_expired() {
            None
        } else {
            Some(&self.value)
        }
    }
}

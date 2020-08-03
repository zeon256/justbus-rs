use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct InternalEntry<V: Debug> {
    value: V,
    expiration: Instant,
}

impl<V: Debug> InternalEntry<V> {
    pub fn new(value: V, expiration: Instant) -> Self {
        InternalEntry { value, expiration }
    }

    fn is_expired(&self) -> bool {
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

#[derive(Debug)]
pub struct CacheVec {
    vec: Vec<InternalEntry<String>>,
    ttl: Duration,
}

impl CacheVec {
    pub fn with_ttl(ttl: Duration) -> Self {
        // pre fill the vector
        let vec = vec![InternalEntry::new(String::new(), Instant::now()); 99189];
        CacheVec { vec, ttl }
    }

    pub fn get(&self, idx: u32) -> Option<&String> {
        self.vec.get(idx as usize).and_then(|v| v.get())
    }

    pub fn insert(&mut self, idx: u32, value: &str) {
        let data = self.vec.get_mut(idx as usize);

        if let Some(v) = data {
            v.value.clear();
            v.value.push_str(value);
            v.expiration = Instant::now() + self.ttl;
        }
    }
}

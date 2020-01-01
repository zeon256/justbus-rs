use cht::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct InternalEntry<V> {
    value: V,
    expiration: Instant,
}

impl<V> InternalEntry<V> {
    pub fn new(value: V, expiration: Instant) -> Self {
        InternalEntry { value, expiration }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expiration
    }

    pub fn get(self) -> Option<V> {
        if self.is_expired() {
            None
        } else {
            Some(self.value)
        }
    }
}

pub struct Cache<K: Hash + Eq, V: Clone> {
    map: HashMap<K, InternalEntry<V>>,
    ttl: Duration,
}
impl<K: Hash + Eq, V: Clone> Cache<K, V> {
    pub fn with_ttl(ttl: Duration) -> Self {
        Cache {
            map: HashMap::<K, InternalEntry<V>>::new(),
            ttl,
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.map
            .insert(key, InternalEntry::new(value, Instant::now() + self.ttl))
            .map(|f| f.value)
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.map.get(&key).map(|f| f.get().unwrap())
    }
}

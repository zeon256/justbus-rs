use dashmap::{mapref::one::Ref, DashMap};
use justbus_utils::InternalEntry;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct Cache<K: Hash + Eq, V: Debug> {
    map: DashMap<K, InternalEntry<V>>,
    ttl: Duration,
}

impl<K: Hash + Eq, V: Debug> Cache<K, V> {
    pub fn with_ttl(ttl: Duration) -> Self {
        Cache {
            map: DashMap::<K, InternalEntry<V>>::new(),
            ttl,
        }
    }

    pub fn with_ttl_and_size(ttl: Duration, capacity: usize) -> Self {
        Cache {
            map: DashMap::<K, InternalEntry<V>>::with_capacity(capacity),
            ttl,
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let to_insert = InternalEntry::new(value, Instant::now() + self.ttl);
        self.map.insert(key, to_insert).map(|v| v.value)
    }

    pub fn get(&self, key: &K) -> Option<Ref<K, InternalEntry<V>>> {
        self.map
            .get(&key)
            .and_then(|f| if !f.is_expired() { Some(f) } else { None })
    }
}

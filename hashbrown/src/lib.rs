use std::hash::Hash;
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[cfg(test)]
mod test {
    use crate::Cache;
    use std::thread;
    use std::time::Duration;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use crossbeam::thread::scope;

    const DURATION: Duration = Duration::from_secs(1);

    /// Simple test to check if values
    /// get replaced and whether the returned value is not expired
    #[test]
    fn hm_test_single_threaded() {
        let mut hm = Cache::with_ttl(DURATION.clone());

        // insert an entry that will expire in 1s
        hm.insert(32, "hello_32");
        thread::sleep(DURATION.clone());
        assert_eq!(hm.get(32), None);
        println!("{:?}", hm.get(32));

        // check if value with same key is replaced
        hm.insert(32, "hello_32_replaced");
        thread::sleep(Duration::from_millis(10));
        assert_eq!(hm.get(32), Some(&"hello_32_replaced"));
        println!("{:?}", hm.get(32));
    }
}

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

    pub fn get(&self) -> Option<&V> {
        if self.is_expired() {
            None
        } else {
            Some(&self.value)
        }
    }
}

pub struct Cache<K: Hash + Eq, V> {
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

    pub fn with_ttl_and_size(ttl: Duration, capacity: usize) -> Self {
        Cache {
            map: HashMap::<K, InternalEntry<V>>::with_capacity(capacity),
            ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, InternalEntry::new(value, Instant::now() + self.ttl))
            .map(|v| v.value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.map
            .get(&key).and_then(|f| f.get())
    }
}

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

#[cfg(test)]
mod test {
    use crate::Cache;
    use std::sync::RwLock as StdRwLock;
    use std::thread;
    use std::time::Duration;

    const DURATION: Duration = Duration::from_secs(1);

    /// Simple test to check if values
    /// get replaced and whether the returned value is not expired
    #[test]
    fn hm_test_single_threaded() {
        let hm = Cache::with_ttl(DURATION.clone());

        // insert an entry that will expire in 1s
        hm.insert(32, "hello_32");
        thread::sleep(DURATION.clone());
        let g = hm.get(&32);

        dbg!(&g);
        if let Some(_) = g {
            panic!("Values dont match!");
        }

        // check if value with same key is replaced
        hm.insert(32, "hello_32_replaced");
        thread::sleep(Duration::from_millis(10));
        let g = hm.get(&32);

        dbg!(&g);
        if let None = g {
            panic!("Values dont match!");
        }
    }

    #[test]
    fn hm_test_multi_threaded_std() {
        let hm = StdRwLock::new(Cache::with_ttl(DURATION.clone()));

        {
            let hm_w = hm.write().unwrap();
            // insert an entry that will expire in 1s
            hm_w.insert(32, "hello_32");
            thread::sleep(DURATION.clone());
        }

        {
            let hm_r = hm.read().unwrap();
            let g = hm_r.get(&32);

            dbg!(&g);
            if let Some(_) = g {
                panic!("Values dont match!");
            }
        }

        {
            let hm_w = hm.write().unwrap();
            // check if value with same key is replaced
            hm_w.insert(32, "hello_32_replaced");
        }

        let hm_r = hm.read().unwrap();
        let g = hm_r.get(&32);

        dbg!(&g);

        if let None = g {
            panic!("Values dont match!");
        }
    }
}

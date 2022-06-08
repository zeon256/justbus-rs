use ahash::RandomState;
use dashmap::{mapref::one::Ref, DashMap};
use justbus_utils::InternalEntry;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};
use std::time::{Duration, Instant};

/// Cache that will expire based on the ttl provided
/// 
/// DOES NOT NEED TO BE WRAPPED IN RwLock to be used in multithreaded code
/// 
/// Defaults to AHash for Hasher
pub struct Cache<K: Hash + Eq, V: Debug, S = RandomState> {
    map: DashMap<K, InternalEntry<V>, S>,
    ttl: Duration,
}

impl<K, V, S> Cache<K, V, S>
where
    K: Hash + Eq,
    V: Debug,
    S: BuildHasher + Clone,
{
    pub fn with_ttl_sz_and_hasher(ttl: Duration, capacity: usize, hasher: S) -> Self {
        Cache {
            map: DashMap::with_capacity_and_hasher(capacity, hasher),
            ttl,
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let to_insert = InternalEntry::new(value, Instant::now() + self.ttl);
        self.map.insert(key, to_insert).map(|v| v.value)
    }

    pub fn get(&self, key: &K) -> Option<Ref<K, InternalEntry<V>, S>> {
        self.map
            .get(&key)
            .and_then(|f| if !f.is_expired() { Some(f) } else { None })
    }
}

impl<K, V> Cache<K, V, RandomState>
where
    K: Hash + Eq,
    V: Debug,
{
    pub fn with_ttl(ttl: Duration) -> Self {
        Cache {
            map: DashMap::<K, InternalEntry<V>, RandomState>::default(),
            ttl,
        }
    }

    pub fn with_ttl_and_size(ttl: Duration, capacity: usize) -> Self {
        Cache {
            map: DashMap::<K, InternalEntry<V>, _>::with_capacity_and_hasher(
                capacity,
                RandomState::default(),
            ),
            ttl,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Cache;
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
    fn hm_test_multi_threaded() {
        let hm = Cache::with_ttl(DURATION.clone());

        {
            // insert an entry that will expire in 1s
            hm.insert(32, "hello_32");
            thread::sleep(DURATION.clone());
        }

        {
            let g = hm.get(&32);

            dbg!(&g);
            if let Some(_) = g {
                panic!("Values dont match!");
            }
        }

        {
            // check if value with same key is replaced
            hm.insert(32, "hello_32_replaced");
        }

        let g = hm.get(&32);

        dbg!(&g);

        if let None = g {
            panic!("Values dont match!");
        }
    }

    #[test]
    fn construct_std_hasher() {
        use std::collections::hash_map::RandomState as StdRandomState;
        let s = StdRandomState::new();
        let _ = Cache::<u32, &str, _>::with_ttl_sz_and_hasher(Duration::from_secs(1), 500, s);
    }
}

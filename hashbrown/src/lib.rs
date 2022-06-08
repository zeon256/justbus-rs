use ahash::RandomState;
use justbus_utils::InternalEntry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};
use std::time::{Duration, Instant};

/// Cache that will expire based on the ttl provided
/// 
/// Can be wrapped in `Arc<RwLock<Cache<_, _>>>` to be used on multithreaded env
/// 
/// Defaults to AHash for Hasher
pub struct Cache<K: Hash + Eq, V: Debug, S = RandomState> {
    map: HashMap<K, InternalEntry<V>, S>,
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
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
            ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let to_insert = InternalEntry::new(value, Instant::now() + self.ttl);
        self.map.insert(key, to_insert).map(|v| v.value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.map.get(&key).and_then(|f| f.get())
    }
}

impl<K, V> Cache<K, V, RandomState>
where
    K: Hash + Eq,
    V: Debug,
{
    pub fn with_ttl(ttl: Duration) -> Self {
        Cache {
            map: HashMap::<K, InternalEntry<V>, _>::default(),
            ttl,
        }
    }

    pub fn with_ttl_and_size(ttl: Duration, capacity: usize) -> Self {
        let map = HashMap::<K, InternalEntry<V>, _>::with_capacity_and_hasher(
            capacity,
            RandomState::default(),
        );

        Cache { map, ttl }
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

    #[test]
    fn hm_test_multi_threaded_std() {
        let hm = StdRwLock::new(Cache::with_ttl(DURATION.clone()));

        {
            let mut hm_w = hm.write().unwrap();
            // insert an entry that will expire in 1s
            hm_w.insert(32, "hello_32");
            thread::sleep(DURATION.clone());
        }

        {
            let hm_r = hm.read().unwrap();

            assert_eq!(hm_r.get(32), None);
            println!("{:?}", hm_r.get(32));
        }

        {
            let mut hm_w = hm.write().unwrap();
            // check if value with same key is replaced
            hm_w.insert(32, "hello_32_replaced");
        }

        let hm_r = hm.read().unwrap();
        assert_eq!(hm_r.get(32), Some(&"hello_32_replaced"));
        println!("{:?}", hm_r.get(32));
    }

    #[test]
    fn construct_std_hasher() {
        use std::collections::hash_map::RandomState as StdRandomState;
        let s = StdRandomState::new();
        let _ = Cache::<u32, &str, _>::with_ttl_sz_and_hasher(Duration::from_secs(1), 500, s);
    }
}

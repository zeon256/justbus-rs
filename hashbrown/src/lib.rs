use justbus_utils::InternalEntry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

#[cfg(test)]
mod test {
    use crate::Cache;
    use crossbeam::thread::scope;
    use std::sync::{Arc, RwLock as StdRwLock};
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
}

pub struct Cache<K: Hash + Eq, V: Debug> {
    map: HashMap<K, InternalEntry<V>>,
    ttl: Duration,
}

impl<K: Hash + Eq, V: Debug> Cache<K, V> {
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
        let to_insert = InternalEntry::new(value, Instant::now() + self.ttl);
        self.map.insert(key, to_insert).map(|v| v.value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.map.get(&key).and_then(|f| f.get())
    }
}

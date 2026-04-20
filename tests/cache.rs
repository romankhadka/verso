use verso::reader::cache::PageCache;

#[test]
fn caches_and_evicts_by_lru() {
    let mut cache = PageCache::new(2);
    cache.put(1, 68, "dark", vec![]);
    cache.put(2, 68, "dark", vec![]);
    assert!(cache.get(1, 68, "dark").is_some());
    cache.put(3, 68, "dark", vec![]);
    // 2 was least-recently-used after we got(1) → evicted.
    assert!(cache.get(2, 68, "dark").is_none());
    assert!(cache.get(1, 68, "dark").is_some());
    assert!(cache.get(3, 68, "dark").is_some());
}

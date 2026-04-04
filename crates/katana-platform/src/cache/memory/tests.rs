use super::*;

#[test]
fn test_in_memory_cache_service() {
    let cache = InMemoryCacheService::default();

    assert_eq!(cache.get_memory("test"), None);
    cache.set_memory("test", "val1".to_string());
    assert_eq!(cache.get_memory("test"), Some("val1".to_string()));
    cache.set_memory("test", "val2".to_string());
    assert_eq!(cache.get_memory("test"), Some("val2".to_string()));

    assert_eq!(cache.get_persistent("pkey"), None);
    cache.set_persistent("pkey", "pval1".to_string()).unwrap();
    assert_eq!(cache.get_persistent("pkey"), Some("pval1".to_string()));
    cache.set_persistent("pkey", "pval2".to_string()).unwrap();
    assert_eq!(cache.get_persistent("pkey"), Some("pval2".to_string()));
}

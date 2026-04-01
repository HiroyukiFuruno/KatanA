#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;
    use tempfile::TempDir;

    #[test]
    fn benchmark_set_persistent_large_data() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("cache.json");
        let cache = DefaultCacheService::new(path.clone());

        // Simulate 50 large diagram caches, e.g. 500KB each
        let large_value = "x".repeat(500 * 1024);
        for i in 0..50 {
            cache.set_persistent(&format!("diagram_{}", i), large_value.clone()).unwrap();
        }

        // Now measure adding one more small workspace_tabs value
        let start = Instant::now();
        cache.set_persistent("workspace_tabs:test", "small_value".to_string()).unwrap();
        let elapsed = start.elapsed();
        println!("Time to set one small value with 50 large diagrams in cache: {:?}", elapsed);

        // Measure read of the small workspace tab value
        let start_read = Instant::now();
        cache.get_persistent("workspace_tabs:test");
        let elapsed_read = start_read.elapsed();
        println!("Time to read small value (linear search): {:?}", elapsed_read);
        
        // Let's break down save_persistent
        let start_serialize = Instant::now();
        let data = read_guard(&cache.persistent);
        let json = serde_json::to_string_pretty(&*data).unwrap();
        let elapsed_serialize = start_serialize.elapsed();
        println!("Time to serialize all: {:?}", elapsed_serialize);

        let start_write = Instant::now();
        std::fs::write(&cache.persistent_path, json).unwrap();
        let elapsed_write = start_write.elapsed();
        println!("Time to write to disk: {:?}", elapsed_write);
    }
}

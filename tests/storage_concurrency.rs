use dinah::storage::{DiskStore, Storage};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn independent_disk_stores_can_be_used_on_separate_threads() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let paths: Vec<_> = (0..4)
        .map(|i| {
            std::env::temp_dir().join(format!(
                "dinah-storage-thread-{}-{nonce}-{i}.journal",
                std::process::id()
            ))
        })
        .collect();
    let handles: Vec<_> = paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut store = DiskStore::open(&path).unwrap();
                let key = format!("key-{i}");
                let value = vec![i as u8; 1024];
                store.put(key.as_bytes(), &value).unwrap();
                assert_eq!(store.get(key.as_bytes()), Some(value));
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    for path in paths {
        fs::remove_file(path).unwrap();
    }
}

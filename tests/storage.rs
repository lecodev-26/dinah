use dinah::storage::{DiskStore, MemoryStore, Storage};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "dinah-storage-{}-{nonce}.journal",
        std::process::id()
    ))
}

#[test]
fn memory_store_put_get_replace_and_delete() {
    let mut store = MemoryStore::new();
    assert!(store.is_empty());
    store.put(b"key", b"one").unwrap();
    assert_eq!(store.get(b"key"), Some(b"one".to_vec()));
    store.put(b"key", b"two").unwrap();
    assert_eq!(store.get(b"key"), Some(b"two".to_vec()));
    assert!(store.delete(b"key").unwrap());
    assert!(!store.delete(b"key").unwrap());
    assert_eq!(store.get(b"key"), None);
}

#[test]
fn disk_store_persists_values_and_replays_updates_and_deletes() {
    let path = temp_path();
    {
        let mut store = DiskStore::open(&path).unwrap();
        store.put(b"alpha", b"first").unwrap();
        store.put(b"alpha", b"second").unwrap();
        store.put(b"beta", b"value").unwrap();
        assert!(store.delete(b"beta").unwrap());
    }
    {
        let store = DiskStore::open(&path).unwrap();
        assert_eq!(store.get(b"alpha"), Some(b"second".to_vec()));
        assert_eq!(store.get(b"beta"), None);
        assert_eq!(store.len(), 1);
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn disk_store_ignores_a_torn_final_record_and_recovers_prior_records() {
    let path = temp_path();
    {
        let mut store = DiskStore::open(&path).unwrap();
        store.put(b"kept", b"complete").unwrap();
    }
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"DNST\x01\x01\x05");
    fs::write(&path, bytes).unwrap();
    let store = DiskStore::open(&path).unwrap();
    assert_eq!(store.get(b"kept"), Some(b"complete".to_vec()));
    fs::remove_file(path).unwrap();
}

#[test]
fn disk_store_rejects_malformed_complete_records() {
    let path = temp_path();
    fs::write(&path, b"NOPE\x01\x01\0\0\0\0\0\0\0\0").unwrap();
    assert!(DiskStore::open(&path).is_err());
    fs::remove_file(path).unwrap();
}

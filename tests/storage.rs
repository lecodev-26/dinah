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

#[test]
fn disk_store_detects_payload_corruption_with_checksum() {
    let path = temp_path();
    {
        let mut store = DiskStore::open(&path).unwrap();
        store.put(b"key", b"value").unwrap();
    }
    let mut bytes = fs::read(&path).unwrap();
    // v2 record: 14-byte header, then key and value, then CRC-32.
    bytes[15] ^= 0x20;
    fs::write(&path, bytes).unwrap();
    assert!(DiskStore::open(&path)
        .unwrap_err()
        .to_string()
        .contains("checksum"));
    fs::remove_file(path).unwrap();
}

#[test]
fn disk_store_recovers_a_torn_checksum_suffix_and_truncates_it() {
    let path = temp_path();
    {
        let mut store = DiskStore::open(&path).unwrap();
        store.put(b"safe", b"record").unwrap();
    }
    let original_len = fs::metadata(&path).unwrap().len();
    let mut bytes = fs::read(&path).unwrap();
    bytes.truncate(bytes.len() - 2);
    fs::write(&path, bytes).unwrap();
    {
        let mut store = DiskStore::open(&path).unwrap();
        assert_eq!(
            store.get(b"safe"),
            None,
            "record with a torn checksum is not applied"
        );
        store.put(b"new", b"complete").unwrap();
    }
    assert!(fs::metadata(&path).unwrap().len() > original_len - 2);
    let store = DiskStore::open(&path).unwrap();
    assert_eq!(store.get(b"new"), Some(b"complete".to_vec()));
    fs::remove_file(path).unwrap();
}

#[test]
fn disk_store_reads_legacy_v1_records() {
    let path = temp_path();
    // Historical v1: magic, version/op, two little-endian lengths, key, value; no checksum.
    let mut record = b"DNST\x01\x01".to_vec();
    record.extend_from_slice(&1_u32.to_le_bytes());
    record.extend_from_slice(&3_u32.to_le_bytes());
    record.extend_from_slice(b"kold");
    fs::write(&path, record).unwrap();
    let mut store = DiskStore::open(&path).unwrap();
    assert_eq!(store.get(b"k"), Some(b"old".to_vec()));
    store.put(b"k", b"new").unwrap();
    drop(store);
    let store = DiskStore::open(&path).unwrap();
    assert_eq!(store.get(b"k"), Some(b"new".to_vec()));
    fs::remove_file(path).unwrap();
}

#[test]
fn memory_store_iterates_in_key_order_without_changing_values() {
    let mut store = MemoryStore::new();
    store.put(b"z", b"last").unwrap();
    store.put(b"a", b"first").unwrap();
    store.put(b"m", b"middle").unwrap();
    let rows: Vec<_> = store.iter().collect();
    assert_eq!(
        rows,
        vec![
            (b"a".as_slice(), b"first".as_slice()),
            (b"m", b"middle"),
            (b"z", b"last")
        ]
    );
}

#[test]
fn disk_store_rejects_oversized_declared_field_lengths() {
    let path = temp_path();
    // A complete header whose key length exceeds the 16 MiB field limit.
    let mut header = b"DNST".to_vec();
    header.extend_from_slice(&[2, 1]);
    header.extend_from_slice(&(16_u32 * 1024 * 1024 + 1).to_le_bytes());
    header.extend_from_slice(&0_u32.to_le_bytes());
    fs::write(&path, header).unwrap();
    let error = DiskStore::open(&path).unwrap_err();
    assert!(error.to_string().contains("field length exceeds limit"));
    fs::remove_file(path).unwrap();
}

#[test]
fn disk_store_rejects_unknown_version_and_operation() {
    for (version, operation, expected) in [
        (99_u8, 1_u8, "unsupported version"),
        (2_u8, 99_u8, "unknown operation"),
    ] {
        let path = temp_path();
        let mut header = b"DNST".to_vec();
        header.extend_from_slice(&[version, operation]);
        header.extend_from_slice(&0_u32.to_le_bytes());
        header.extend_from_slice(&0_u32.to_le_bytes());
        fs::write(&path, header).unwrap();
        let error = DiskStore::open(&path).unwrap_err();
        assert!(error.to_string().contains(expected));
        fs::remove_file(path).unwrap();
    }
}

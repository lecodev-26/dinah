//! Repeated cross-module checks for stable error and ordering contracts.
use dinah::native::{ffi, Hal, HostPlatform, Platform};
use dinah::storage::{MemoryStore, Storage};
use std::time::{Duration, Instant};

#[test]
fn native_scalar_abi_contract_is_stable_and_self_consistent() {
    assert_eq!(ffi::dinah_abi_version(), 1);
    assert_eq!(ffi::dinah_abi_features() & 1, 1);
    assert!(ffi::dinah_abi_platform_family() <= 2);
    let host = HostPlatform::new();
    assert!(!host.operating_system().is_empty());
    assert!(!host.architecture().is_empty());
    assert!(host.process_id() > 0);
    assert!(
        host.elapsed_since(Instant::now() - Duration::from_millis(2)) >= Duration::from_millis(2)
    );
}

#[test]
fn memory_store_order_and_overwrite_are_repeatable() {
    for _ in 0..32 {
        let mut store = MemoryStore::new();
        store.put(b"z", b"old").unwrap();
        store.put(b"a", b"first").unwrap();
        store.put(b"z", b"new").unwrap();
        let rows: Vec<_> = store.iter().collect();
        assert_eq!(
            rows,
            vec![(b"a".as_slice(), b"first".as_slice()), (b"z", b"new")]
        );
    }
}

#[test]
fn independent_store_instances_survive_parallel_repeated_updates() {
    let root = std::env::temp_dir().join(format!("dinah-regression-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let workers: Vec<_> = (0..6)
        .map(|worker| {
            let path = root.join(format!("store-{worker}.journal"));
            std::thread::spawn(move || {
                use dinah::storage::DiskStore;
                let mut store = DiskStore::open(&path).unwrap();
                for iteration in 0..128_u32 {
                    let key = format!("key-{}", iteration % 8);
                    let value = format!("worker-{worker}-iteration-{iteration}");
                    store.put(key.as_bytes(), value.as_bytes()).unwrap();
                    assert_eq!(store.get(key.as_bytes()), Some(value.into_bytes()));
                }
                drop(store);
                let store = DiskStore::open(&path).unwrap();
                assert_eq!(store.len(), 8);
                for entry in 0..8 {
                    assert!(store.get(format!("key-{entry}").as_bytes()).is_some());
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker completed");
    }
    std::fs::remove_dir_all(root).unwrap();
}

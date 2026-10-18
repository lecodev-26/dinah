use dinah::runtime::Scheduler;
use dinah::storage::runtime::{StorageCommand, StorageWorkQueue};
use dinah::storage::{MemoryStore, Storage};
use std::num::NonZeroUsize;

#[test]
fn storage_work_queue_dispatches_mutations_in_fifo_order() {
    let scheduler = Scheduler::new();
    let mut queue = StorageWorkQueue::new(NonZeroUsize::new(2).unwrap());
    let mut store = MemoryStore::new();
    queue
        .enqueue(StorageCommand::Put {
            key: b"k".to_vec(),
            value: b"one".to_vec(),
        })
        .unwrap();
    queue
        .enqueue(StorageCommand::Put {
            key: b"k".to_vec(),
            value: b"two".to_vec(),
        })
        .unwrap();
    assert_eq!(store.get(b"k"), None, "enqueue does not execute mutations");
    assert!(queue
        .dispatch_next(&scheduler, &mut store)
        .unwrap()
        .unwrap()
        .is_ok());
    assert_eq!(store.get(b"k"), Some(b"one".to_vec()));
    assert!(queue
        .dispatch_next(&scheduler, &mut store)
        .unwrap()
        .unwrap()
        .is_ok());
    assert_eq!(store.get(b"k"), Some(b"two".to_vec()));
    assert!(queue.is_empty());
}

#[test]
fn storage_work_queue_returns_command_when_full() {
    let mut queue = StorageWorkQueue::new(NonZeroUsize::new(1).unwrap());
    queue
        .enqueue(StorageCommand::Delete {
            key: b"first".to_vec(),
        })
        .unwrap();
    let rejected = StorageCommand::Put {
        key: b"second".to_vec(),
        value: vec![],
    };
    assert!(
        matches!(queue.enqueue(rejected), Err(StorageCommand::Put { key, .. }) if key == b"second")
    );
}

//! Minimal in-memory and journal-backed Storage examples.

use dinah::storage::{DiskStore, MemoryStore, Storage};
use std::error::Error;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn Error>> {
    let mut memory = MemoryStore::new();
    memory.put(b"greeting", b"hello from memory")?;
    memory.put(b"version", b"1")?;
    println!("memory_keys={}", memory.len());
    for (key, value) in memory.iter() {
        println!(
            "memory {:?}={:?}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }

    // Use a unique temporary journal so repeated runs do not share state.
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!(
        "dinah-storage-{}-{nonce}.journal",
        std::process::id()
    ));
    let result = (|| -> Result<(), Box<dyn Error>> {
        {
            let mut disk = DiskStore::open(&path)?;
            disk.put(b"session", b"saved locally")?;
            disk.put(b"counter", b"7")?;
            println!("disk_keys_before_reopen={}", disk.len());
        }

        // Reopening replays complete journal records into the in-memory index.
        let mut recovered = DiskStore::open(&path)?;
        let session = recovered
            .get(b"session")
            .ok_or("session was not recovered")?;
        println!("recovered_session={}", String::from_utf8_lossy(&session));
        println!("recovered_keys={}", recovered.len());
        assert!(recovered.delete(b"counter")?);
        assert!(recovered.get(b"counter").is_none());
        Ok(())
    })();
    let cleanup = fs::remove_file(&path);
    result?;
    cleanup?;
    Ok(())
}

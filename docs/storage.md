# DINAH Storage — implementation guide (points 140–156)

Storage starts with a key/value API for opaque bytes, without serializing Rust structs or coupling the contract to a domain. The public module `dinah::storage` contains `Storage`, `MemoryStore`, `DiskStore` and `StorageError`. It is an incremental foundation, not a production database.

## Common API — point 141

`Storage` exposes `get(&self, key) -> Option<Vec<u8>>`, `put(&mut self, key, value)` and `delete(&mut self, key)`. Returned values are copies owned by the caller; no references that depend on the store's lifetime are returned. `put` and the concrete implementations validate the current 16 MiB limit per key and value. I/O and format errors are represented with `StorageError`.

The API is synchronous and requires `&mut self` for mutations. It does not promise concurrent access, multi-key transactions, global cross-process ordering, secondary indexes, compaction, encryption or access control. Applications are responsible for validating and versioning their payloads.

## Memory — point 142

`MemoryStore` uses `BTreeMap<Vec<u8>, Vec<u8>>`: lexicographically ordered binary keys, explicit replacement when `put` repeats, cloned reads, and deletion that reports whether the key existed. `iter()` exposes borrowed key/value views in deterministic order, without cloning each entry. Its contents live only as long as the instance and are not shared across processes. `len` and `is_empty` are observational.

## Disk and format — points 143–144

`DiskStore::open(path)` creates or opens a local journal file and rebuilds the map by applying its complete records in order. The common header is 14 bytes: magic `DNST`, one-byte format version, one-byte opcode, `u32` little-endian key length and `u32` little-endian value length; then the key and value bytes are stored. New records use the v2 format and include a 4-byte IEEE CRC-32 at the end. Historical v1 records, without a checksum, remain readable. Opcode 1 is PUT; opcode 2 is DELETE with an empty value. Lengths are validated before processing the record and every field is capped at 16 MiB.

This format is internal and versioned, not a portable encoding of Rust objects. The CRC-32 detects accidental corruption of complete v2 records; it is not cryptographic, does not authenticate the file and does not protect against deliberate modification. A bad checksum in a complete record produces an error, it is not silently skipped. An incomplete final record, including a truncated checksum, is treated as an interrupted write: previous records are preserved and the suffix is truncated before new writes are accepted. Prior v1 records are read without a checksum for compatibility.

## Persistence and journal — points 145–146

Every effective `put` or deletion appends a record and calls `flush`; later updates replace the key during replay. `delete` of an absent key does not append a record. The journal is append-only and not yet compacted. `flush` only flushes user-space buffers: `sync_all` is not called and no durability is promised against power loss, kernel failure or device cache. Atomicity against every failure type is not guaranteed, and multiple instances open on the same file are not coordinated.

## Integrity and compatibility — point 148

New records are v2 and store an IEEE CRC-32 over the version, opcode, lengths and payload. A bad checksum in a complete record returns `StorageError::InvalidFormat`; replay does not apply that record. The declared length is validated before computing ranges, and an incomplete record (including a partial checksum) is treated as a truncated suffix. Checksum-less v1 readers are preserved so journals created by the first implementation are not invalidated. CRC-32 detects accidental damage, but it is not a MAC, signature or defense against an attacker.

## Indexes and structures — points 149–150

The current decision is a single primary index `BTreeMap<Vec<u8>, Vec<u8>>` for O(log n) reads, stable ordering and O(n) ordered scans. `DiskStore` rebuilds the index on open and mutations update the index after the append and `flush` succeed. `iter()` allows scanning by reference without cloning values. No secondary index is added yet: without application prefix/field queries or measurements justifying the cost, duplicating keys would increase memory and coherence complexity without proven benefit. There is no journal compaction; the index is in memory and must be rebuilt on every open.

## Runtime integration — point 151

`storage::runtime::StorageWorkQueue` accepts its own `Put`/`Delete` commands and has a fixed non-zero capacity. Enqueuing does not touch the store. `dispatch_next` removes one FIFO command and applies it synchronously via `Scheduler` on the calling thread; if the queue is full, it returns the rejected command so the caller can keep or retry it. Storage errors are preserved as task outcomes. It creates no threads, does not automatically serialize reads, does not share the store and provides no transactions or background execution.

## Recovery — point 147

On open, the store walks the records from the beginning and applies PUT/DELETE in sequence. It rejects a wrong magic, unknown version, invalid opcode, out-of-bounds fields and DELETE with payload. If the last record is incomplete, it recovers up to the last valid boundary and truncates the suffix. This initial recovery does not replace checksums, backups, migrations or exhaustive corruption testing.

## Runnable example

`examples/storage_basics.rs` demonstrates the common contract with `MemoryStore` and `DiskStore`: insertion and ordered traversal, journal reopening to recover values, and deleting a key. Run it from the repository root with `cargo run --example storage_basics`. The example creates a unique temporary name and removes the journal when finished. This example is demonstrative; it does not change the durability guarantees described above and does not turn the journal into a transactional database.

## Tests

`tests/storage.rs` covers in-memory operations, persistence/replay, truncated-suffix recovery, malformed-header rejection, checksum-detected corruption, checksum truncation and v1 record reads. `tests/storage_runtime.rs` checks FIFO dispatch and capacity rejection; `tests/storage_concurrency.rs` verifies parallel operations on independent journals. Concurrent access to the same file from several instances is neither tested nor promised.

## Next work

The `benches/storage.rs` suite measures batched MemoryStore operations and ordered scans; CI runs the suite on its five-platform matrix and archives the measurements per commit. Results are machine-specific and must not be compared across platforms as if they were equivalent. The implementation is not presented as a transactional database or a strong-persistence guarantee. Before using DiskStore for important data, keep backups and assume a power loss may lose recent writes. The journal format is internal; v1 reading is preserved, but there is no migration or compaction tool. Concurrency guarantees are limited to the mutual exclusion that Rust ownership imposes within one instance; opening the same path multiple times does not coordinate writers.

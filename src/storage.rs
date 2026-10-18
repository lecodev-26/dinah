//! Small, explicit key-value storage primitives.
//!
//! The API stores opaque byte values and does not deserialize application types.
//! `MemoryStore` is process-local; `DiskStore` appends a versioned journal and
//! replays complete records when opened. New records use CRC-32 integrity checks;
//! legacy v1 records remain readable. Concurrent access, compaction and strong
//! crash-durability guarantees are deliberately separate concerns.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"DNST";
const FORMAT_VERSION: u8 = 2;
const LEGACY_FORMAT_VERSION: u8 = 1;
const CHECKSUM_LEN: usize = 4;
const OP_PUT: u8 = 1;
const OP_DELETE: u8 = 2;
const HEADER_LEN: usize = 14;
const MAX_FIELD_LEN: usize = 16 * 1024 * 1024;

/// Errors produced by storage operations.
#[derive(Debug)]
pub enum StorageError {
    /// An operating-system file operation failed.
    Io(io::Error),
    /// A key or value exceeds the format's current field limit.
    FieldTooLarge,
    /// A complete record contains an unsupported or malformed encoding.
    InvalidFormat(&'static str),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "storage I/O failed: {error}"),
            Self::FieldTooLarge => write!(f, "storage key or value exceeds the field limit"),
            Self::InvalidFormat(reason) => write!(f, "invalid storage format: {reason}"),
        }
    }
}
impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Minimal opaque-byte key-value contract. Implementations own copied keys and values.
pub trait Storage {
    /// Returns a copy of the value associated with `key`, if present.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    /// Inserts or replaces a value.
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    /// Removes a key and reports whether it existed.
    fn delete(&mut self, key: &[u8]) -> Result<bool, StorageError>;
}

/// In-memory key-value storage with deterministic bytewise key ordering.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MemoryStore {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}
impl MemoryStore {
    /// Creates an empty store.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns the number of stored keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Returns whether the store contains no keys.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Iterates over borrowed key/value slices in deterministic bytewise key order.
    /// This avoids cloning values for scans; references cannot outlive this store borrow.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value.as_slice()))
    }
}
impl Storage for MemoryStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.entries.get(key).cloned()
    }
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        validate_field(key)?;
        validate_field(value)?;
        self.entries.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    fn delete(&mut self, key: &[u8]) -> Result<bool, StorageError> {
        Ok(self.entries.remove(key).is_some())
    }
}

/// Append-only local disk store. Opening it replays its journal into memory.
///
/// Each successful mutation appends a complete record and flushes userspace
/// buffers. This does not call `sync_all` and therefore does not promise survival
/// across power loss or storage-device write-cache loss. A torn final record is
/// ignored during recovery; malformed complete records are reported as errors.
#[derive(Debug)]
pub struct DiskStore {
    path: PathBuf,
    file: File,
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}
impl DiskStore {
    /// Opens or creates a journal at `path`, recovering complete records.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let (entries, valid_len) = replay(&bytes)?;
        if valid_len < bytes.len() {
            // Remove a torn suffix before any later append, so future records remain reachable.
            file.set_len(valid_len as u64)?;
        }
        file.seek(SeekFrom::End(0))?;
        Ok(Self {
            path,
            file,
            entries,
        })
    }
    /// Returns the journal path used by this store.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Returns the number of currently stored keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Returns whether the store contains no keys.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Iterates over borrowed key/value slices in deterministic bytewise key order.
    /// This scans the recovered in-memory index and does not read the journal again.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value.as_slice()))
    }
    fn append(&mut self, operation: u8, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        validate_field(key)?;
        validate_field(value)?;
        let key_len = u32::try_from(key.len()).map_err(|_| StorageError::FieldTooLarge)?;
        let value_len = u32::try_from(value.len()).map_err(|_| StorageError::FieldTooLarge)?;
        let mut record = Vec::with_capacity(HEADER_LEN + key.len() + value.len());
        record.extend_from_slice(MAGIC);
        record.push(FORMAT_VERSION);
        record.push(operation);
        record.extend_from_slice(&key_len.to_le_bytes());
        record.extend_from_slice(&value_len.to_le_bytes());
        record.extend_from_slice(key);
        record.extend_from_slice(value);
        let checksum = crc32(&record[4..]);
        record.extend_from_slice(&checksum.to_le_bytes());
        self.file.write_all(&record)?;
        self.file.flush()?;
        Ok(())
    }
}
impl Storage for DiskStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.entries.get(key).cloned()
    }
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.append(OP_PUT, key, value)?;
        self.entries.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    fn delete(&mut self, key: &[u8]) -> Result<bool, StorageError> {
        if !self.entries.contains_key(key) {
            return Ok(false);
        }
        self.append(OP_DELETE, key, &[])?;
        self.entries.remove(key);
        Ok(true)
    }
}

fn validate_field(field: &[u8]) -> Result<(), StorageError> {
    if field.len() > MAX_FIELD_LEN {
        Err(StorageError::FieldTooLarge)
    } else {
        Ok(())
    }
}

type Entries = BTreeMap<Vec<u8>, Vec<u8>>;

fn replay(bytes: &[u8]) -> Result<(Entries, usize), StorageError> {
    let mut entries = BTreeMap::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        // A short trailing header is a torn append, not a complete record.
        if bytes.len() - cursor < HEADER_LEN {
            break;
        }
        let header = &bytes[cursor..cursor + HEADER_LEN];
        if &header[..4] != MAGIC {
            return Err(StorageError::InvalidFormat("bad record magic"));
        }
        let version = header[4];
        if version != LEGACY_FORMAT_VERSION && version != FORMAT_VERSION {
            return Err(StorageError::InvalidFormat("unsupported version"));
        }
        let operation = header[5];
        let key_len = u32::from_le_bytes(header[6..10].try_into().expect("fixed header")) as usize;
        let value_len =
            u32::from_le_bytes(header[10..14].try_into().expect("fixed header")) as usize;
        if key_len > MAX_FIELD_LEN || value_len > MAX_FIELD_LEN {
            return Err(StorageError::InvalidFormat("field length exceeds limit"));
        }
        if operation != OP_PUT && operation != OP_DELETE {
            return Err(StorageError::InvalidFormat("unknown operation"));
        }
        if operation == OP_DELETE && value_len != 0 {
            return Err(StorageError::InvalidFormat(
                "delete record contains a value",
            ));
        }
        let payload_len = HEADER_LEN
            .checked_add(key_len)
            .and_then(|n| n.checked_add(value_len))
            .ok_or(StorageError::InvalidFormat("record length overflow"))?;
        let record_len = payload_len
            .checked_add(if version == FORMAT_VERSION {
                CHECKSUM_LEN
            } else {
                0
            })
            .ok_or(StorageError::InvalidFormat("record length overflow"))?;
        if bytes.len() - cursor < record_len {
            break;
        }
        if version == FORMAT_VERSION {
            let expected = u32::from_le_bytes(
                bytes[cursor + payload_len..cursor + record_len]
                    .try_into()
                    .expect("fixed checksum"),
            );
            let actual = crc32(&bytes[cursor + 4..cursor + payload_len]);
            if actual != expected {
                return Err(StorageError::InvalidFormat("record checksum mismatch"));
            }
        }
        let key_start = cursor + HEADER_LEN;
        let value_start = key_start + key_len;
        let key = bytes[key_start..value_start].to_vec();
        let value = bytes[value_start..cursor + payload_len].to_vec();
        match operation {
            OP_PUT => {
                entries.insert(key, value);
            }
            OP_DELETE => {
                entries.remove(&key);
            }
            _ => unreachable!(),
        }
        cursor += record_len;
    }
    Ok((entries, cursor))
}

/// IEEE CRC-32. Detects accidental corruption; it is not a cryptographic MAC.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Explicit Runtime bridge for queued storage mutations.
pub mod runtime;

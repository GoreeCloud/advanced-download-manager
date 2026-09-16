#![forbid(unsafe_code)]

use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use goreecloud_download_core::{
    FailureKind, InputError, JobId, JobState, RemoteValidators, SensitiveUrl, WaitReason,
};
use goreecloud_download_state::{
    CURRENT_SCHEMA_VERSION, CommitBatchV1, JobCheckpointFieldsV1, JobCheckpointV1, JobMutationV1,
    StateError,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};

const APPLICATION_ID: i64 = 0x4743_444d; // ASCII "GCDM"
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PATH_BYTES: usize = 64 * 1024;
const MAX_VALIDATOR_BYTES: usize = 16 * 1024;

const SCHEMA_V1: &str = r#"
CREATE TABLE store_metadata (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    generation BLOB NOT NULL CHECK (length(generation) = 8)
);

INSERT INTO store_metadata (singleton, schema_version, generation)
VALUES (1, 1, X'0000000000000000');

CREATE TABLE jobs (
    job_id TEXT PRIMARY KEY NOT NULL,
    source_url TEXT NOT NULL,
    path_encoding TEXT NOT NULL,
    final_path BLOB NOT NULL,
    staging_path BLOB NOT NULL,
    state_code TEXT NOT NULL,
    state_detail TEXT,
    downloaded_bytes BLOB NOT NULL CHECK (length(downloaded_bytes) = 8),
    expected_bytes BLOB CHECK (expected_bytes IS NULL OR length(expected_bytes) = 8),
    etag TEXT,
    last_modified TEXT
) WITHOUT ROWID;
"#;

const UPSERT_JOB_V1: &str = r#"
INSERT INTO jobs (
    job_id,
    source_url,
    path_encoding,
    final_path,
    staging_path,
    state_code,
    state_detail,
    downloaded_bytes,
    expected_bytes,
    etag,
    last_modified
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT(job_id) DO UPDATE SET
    source_url = excluded.source_url,
    path_encoding = excluded.path_encoding,
    final_path = excluded.final_path,
    staging_path = excluded.staging_path,
    state_code = excluded.state_code,
    state_detail = excluded.state_detail,
    downloaded_bytes = excluded.downloaded_bytes,
    expected_bytes = excluded.expected_bytes,
    etag = excluded.etag,
    last_modified = excluded.last_modified;
"#;

#[derive(Debug)]
pub struct SqliteStore {
    connection: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path).map_err(StoreError::Database)?;
        configure_connection(&connection)?;
        prepare_store(&mut connection)?;
        validate_schema(&connection)?;
        Ok(Self { connection })
    }

    pub fn generation(&self) -> Result<u64, StoreError> {
        read_generation(&self.connection)
    }

    pub fn sqlite_version(&self) -> Result<String, StoreError> {
        self.connection
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .map_err(StoreError::Database)
    }

    pub fn load_all(&self) -> Result<Vec<JobCheckpointV1>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                r#"
                SELECT
                    job_id,
                    source_url,
                    path_encoding,
                    final_path,
                    staging_path,
                    state_code,
                    state_detail,
                    downloaded_bytes,
                    expected_bytes,
                    etag,
                    last_modified
                FROM jobs
                ORDER BY job_id
                "#,
            )
            .map_err(StoreError::Database)?;

        let mut rows = statement.query([]).map_err(StoreError::Database)?;
        let mut checkpoints = Vec::new();

        while let Some(row) = rows.next().map_err(StoreError::Database)? {
            let id_text: String = row.get(0).map_err(StoreError::Database)?;
            let source_text: String = row.get(1).map_err(StoreError::Database)?;
            let path_encoding: String = row.get(2).map_err(StoreError::Database)?;
            let final_path_bytes: Vec<u8> = row.get(3).map_err(StoreError::Database)?;
            let staging_path_bytes: Vec<u8> = row.get(4).map_err(StoreError::Database)?;
            let state_code: String = row.get(5).map_err(StoreError::Database)?;
            let state_detail: Option<String> = row.get(6).map_err(StoreError::Database)?;
            let downloaded_bytes_blob: Vec<u8> = row.get(7).map_err(StoreError::Database)?;
            let expected_bytes_blob: Option<Vec<u8>> = row.get(8).map_err(StoreError::Database)?;
            let etag: Option<String> = row.get(9).map_err(StoreError::Database)?;
            let last_modified: Option<String> = row.get(10).map_err(StoreError::Database)?;

            validate_optional_text(etag.as_deref())?;
            validate_optional_text(last_modified.as_deref())?;

            let fields = JobCheckpointFieldsV1 {
                id: JobId::parse(id_text).map_err(StoreError::CoreInput)?,
                source: SensitiveUrl::parse(source_text).map_err(StoreError::CoreInput)?,
                final_path: decode_path(&path_encoding, &final_path_bytes)?,
                staging_path: decode_path(&path_encoding, &staging_path_bytes)?,
                state: decode_state(&state_code, state_detail.as_deref())?,
                downloaded_bytes: decode_u64(&downloaded_bytes_blob)?,
                expected_bytes: expected_bytes_blob
                    .as_deref()
                    .map(decode_u64)
                    .transpose()?,
                validators: RemoteValidators {
                    etag,
                    last_modified,
                },
            };

            checkpoints.push(JobCheckpointV1::from_fields(fields).map_err(StoreError::State)?);
        }

        Ok(checkpoints)
    }

    pub fn load(&self, id: &JobId) -> Result<Option<JobCheckpointV1>, StoreError> {
        let checkpoints = self.load_all()?;
        Ok(checkpoints
            .into_iter()
            .find(|checkpoint| checkpoint.id() == id))
    }

    pub fn apply_batch(&mut self, batch: &CommitBatchV1) -> Result<(), StoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Database)?;

        let actual_generation = read_generation(&transaction)?;
        if actual_generation != batch.expected_generation() {
            return Err(StoreError::GenerationConflict {
                expected: batch.expected_generation(),
                actual: actual_generation,
            });
        }

        for mutation in batch.mutations() {
            match mutation {
                JobMutationV1::Upsert(checkpoint) => upsert_checkpoint(&transaction, checkpoint)?,
                JobMutationV1::Remove(id) => {
                    transaction
                        .execute("DELETE FROM jobs WHERE job_id = ?1", params![id.as_str()])
                        .map_err(StoreError::Database)?;
                }
            }
        }

        let changed = transaction
            .execute(
                "UPDATE store_metadata SET generation = ?1 WHERE singleton = 1",
                params![encode_u64(batch.next_generation())],
            )
            .map_err(StoreError::Database)?;
        if changed != 1 {
            return Err(StoreError::MetadataInvariant);
        }

        transaction.commit().map_err(StoreError::Database)
    }

    pub fn quick_check(&self) -> Result<(), StoreError> {
        let result: String = self
            .connection
            .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
            .map_err(StoreError::Database)?;
        if result == "ok" {
            Ok(())
        } else {
            Err(StoreError::IntegrityCheckFailed)
        }
    }
}

fn configure_connection(connection: &Connection) -> Result<(), StoreError> {
    connection
        .busy_timeout(BUSY_TIMEOUT)
        .map_err(StoreError::Database)?;
    connection
        .pragma_update(None, "journal_mode", "DELETE")
        .map_err(StoreError::Database)?;
    connection
        .pragma_update(None, "synchronous", "EXTRA")
        .map_err(StoreError::Database)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(StoreError::Database)?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(StoreError::Database)?;
    Ok(())
}

fn prepare_store(connection: &mut Connection) -> Result<(), StoreError> {
    let application_id: i64 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(StoreError::Database)?;
    let table_count = user_table_count(connection)?;

    match application_id {
        0 if table_count == 0 => {
            connection
                .pragma_update(None, "application_id", APPLICATION_ID)
                .map_err(StoreError::Database)?;
            initialize_schema(connection)
        }
        0 => Err(StoreError::UnrecognizedStore),
        APPLICATION_ID if table_count == 0 => initialize_schema(connection),
        APPLICATION_ID => Ok(()),
        found => Err(StoreError::WrongApplicationId { found }),
    }
}

fn initialize_schema(connection: &mut Connection) -> Result<(), StoreError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::Database)?;
    transaction
        .execute_batch(SCHEMA_V1)
        .map_err(StoreError::Database)?;
    transaction.commit().map_err(StoreError::Database)
}

fn validate_schema(connection: &Connection) -> Result<(), StoreError> {
    if !table_exists(connection, "store_metadata")? || !table_exists(connection, "jobs")? {
        return Err(StoreError::MissingSchema);
    }

    let schema_version: i64 = connection
        .query_row(
            "SELECT schema_version FROM store_metadata WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Database)?
        .ok_or(StoreError::MetadataInvariant)?;

    let schema_version = u32::try_from(schema_version).map_err(|_| StoreError::InvalidSchemaVersion)?;
    if schema_version == 0 {
        return Err(StoreError::InvalidSchemaVersion);
    }
    if schema_version < CURRENT_SCHEMA_VERSION {
        return Err(StoreError::MigrationRequired {
            found: schema_version,
            current: CURRENT_SCHEMA_VERSION,
        });
    }
    if schema_version > CURRENT_SCHEMA_VERSION {
        return Err(StoreError::UnsupportedFutureSchema {
            found: schema_version,
            current: CURRENT_SCHEMA_VERSION,
        });
    }

    read_generation(connection)?;
    Ok(())
}

fn user_table_count(connection: &Connection) -> Result<u64, StoreError> {
    connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Database)
}

fn table_exists(connection: &Connection, name: &str) -> Result<bool, StoreError> {
    let count: u64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |row| row.get(0),
        )
        .map_err(StoreError::Database)?;
    Ok(count == 1)
}

fn read_generation(connection: &Connection) -> Result<u64, StoreError> {
    let blob: Vec<u8> = connection
        .query_row(
            "SELECT generation FROM store_metadata WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Database)?
        .ok_or(StoreError::MetadataInvariant)?;
    decode_u64(&blob)
}

fn upsert_checkpoint(
    transaction: &Transaction<'_>,
    checkpoint: &JobCheckpointV1,
) -> Result<(), StoreError> {
    validate_optional_text(checkpoint.validators().etag.as_deref())?;
    validate_optional_text(checkpoint.validators().last_modified.as_deref())?;

    let (final_encoding, final_path) = encode_path(checkpoint.paths().final_path())?;
    let (staging_encoding, staging_path) = encode_path(checkpoint.paths().staging_path())?;
    if final_encoding != staging_encoding {
        return Err(StoreError::PathEncodingMismatch);
    }

    let (state_code, state_detail) = encode_state(checkpoint.state());
    let expected_bytes = checkpoint.expected_bytes().map(encode_u64);

    transaction
        .execute(
            UPSERT_JOB_V1,
            params![
                checkpoint.id().as_str(),
                checkpoint.source().expose(),
                final_encoding,
                final_path,
                staging_path,
                state_code,
                state_detail,
                encode_u64(checkpoint.downloaded_bytes()),
                expected_bytes,
                checkpoint.validators().etag.as_deref(),
                checkpoint.validators().last_modified.as_deref(),
            ],
        )
        .map_err(StoreError::Database)?;
    Ok(())
}

fn encode_u64(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn decode_u64(bytes: &[u8]) -> Result<u64, StoreError> {
    let array: [u8; 8] = bytes
        .try_into()
        .map_err(|_| StoreError::InvalidUnsignedInteger)?;
    Ok(u64::from_be_bytes(array))
}

fn validate_optional_text(value: Option<&str>) -> Result<(), StoreError> {
    if value.is_some_and(|value| value.len() > MAX_VALIDATOR_BYTES) {
        return Err(StoreError::ValidatorTooLong);
    }
    Ok(())
}

fn encode_state(state: JobState) -> (&'static str, Option<&'static str>) {
    match state {
        JobState::Queued => ("queued", None),
        JobState::Downloading => ("downloading", None),
        JobState::Paused => ("paused", None),
        JobState::Waiting(reason) => ("waiting", Some(encode_wait_reason(reason))),
        JobState::Verifying => ("verifying", None),
        JobState::Processing => ("processing", None),
        JobState::Completed => ("completed", None),
        JobState::Failed(kind) => ("failed", Some(encode_failure_kind(kind))),
        JobState::Cancelled => ("cancelled", None),
    }
}

fn decode_state(code: &str, detail: Option<&str>) -> Result<JobState, StoreError> {
    match (code, detail) {
        ("queued", None) => Ok(JobState::Queued),
        ("downloading", None) => Ok(JobState::Downloading),
        ("paused", None) => Ok(JobState::Paused),
        ("waiting", Some(reason)) => Ok(JobState::Waiting(decode_wait_reason(reason)?)),
        ("verifying", None) => Ok(JobState::Verifying),
        ("processing", None) => Ok(JobState::Processing),
        ("completed", None) => Ok(JobState::Completed),
        ("failed", Some(kind)) => Ok(JobState::Failed(decode_failure_kind(kind)?)),
        ("cancelled", None) => Ok(JobState::Cancelled),
        _ => Err(StoreError::InvalidJobState),
    }
}

fn encode_wait_reason(reason: WaitReason) -> &'static str {
    match reason {
        WaitReason::Network => "network",
        WaitReason::Wifi => "wifi",
        WaitReason::Vpn => "vpn",
        WaitReason::Power => "power",
        WaitReason::Storage => "storage",
        WaitReason::Schedule => "schedule",
    }
}

fn decode_wait_reason(value: &str) -> Result<WaitReason, StoreError> {
    match value {
        "network" => Ok(WaitReason::Network),
        "wifi" => Ok(WaitReason::Wifi),
        "vpn" => Ok(WaitReason::Vpn),
        "power" => Ok(WaitReason::Power),
        "storage" => Ok(WaitReason::Storage),
        "schedule" => Ok(WaitReason::Schedule),
        _ => Err(StoreError::InvalidJobState),
    }
}

fn encode_failure_kind(kind: FailureKind) -> &'static str {
    match kind {
        FailureKind::Network => "network",
        FailureKind::RemoteServer => "remote-server",
        FailureKind::Authentication => "authentication",
        FailureKind::Storage => "storage",
        FailureKind::Integrity => "integrity",
        FailureKind::Policy => "policy",
        FailureKind::Internal => "internal",
    }
}

fn decode_failure_kind(value: &str) -> Result<FailureKind, StoreError> {
    match value {
        "network" => Ok(FailureKind::Network),
        "remote-server" => Ok(FailureKind::RemoteServer),
        "authentication" => Ok(FailureKind::Authentication),
        "storage" => Ok(FailureKind::Storage),
        "integrity" => Ok(FailureKind::Integrity),
        "policy" => Ok(FailureKind::Policy),
        "internal" => Ok(FailureKind::Internal),
        _ => Err(StoreError::InvalidJobState),
    }
}

#[cfg(unix)]
fn encode_path(path: &Path) -> Result<(&'static str, Vec<u8>), StoreError> {
    let bytes = path.as_os_str().as_bytes();
    if bytes.len() > MAX_PATH_BYTES {
        return Err(StoreError::PathTooLong);
    }
    Ok(("unix-bytes-v1", bytes.to_vec()))
}

#[cfg(unix)]
fn decode_path(encoding: &str, bytes: &[u8]) -> Result<PathBuf, StoreError> {
    if encoding != "unix-bytes-v1" {
        return Err(StoreError::UnsupportedPathEncoding);
    }
    if bytes.len() > MAX_PATH_BYTES {
        return Err(StoreError::PathTooLong);
    }
    Ok(PathBuf::from(OsString::from_vec(bytes.to_vec())))
}

#[cfg(windows)]
fn encode_path(path: &Path) -> Result<(&'static str, Vec<u8>), StoreError> {
    let units: Vec<u16> = path.as_os_str().encode_wide().collect();
    if units.len().saturating_mul(2) > MAX_PATH_BYTES {
        return Err(StoreError::PathTooLong);
    }
    let mut bytes = Vec::with_capacity(units.len() * 2);
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    Ok(("windows-utf16le-v1", bytes))
}

#[cfg(windows)]
fn decode_path(encoding: &str, bytes: &[u8]) -> Result<PathBuf, StoreError> {
    if encoding != "windows-utf16le-v1" {
        return Err(StoreError::UnsupportedPathEncoding);
    }
    if bytes.len() > MAX_PATH_BYTES {
        return Err(StoreError::PathTooLong);
    }
    if !bytes.len().is_multiple_of(2) {
        return Err(StoreError::InvalidPathEncoding);
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    Ok(PathBuf::from(OsString::from_wide(&units)))
}

#[derive(Debug)]
pub enum StoreError {
    Database(rusqlite::Error),
    CoreInput(InputError),
    State(StateError),
    WrongApplicationId { found: i64 },
    UnrecognizedStore,
    MissingSchema,
    MetadataInvariant,
    InvalidSchemaVersion,
    MigrationRequired { found: u32, current: u32 },
    UnsupportedFutureSchema { found: u32, current: u32 },
    GenerationConflict { expected: u64, actual: u64 },
    InvalidUnsignedInteger,
    InvalidJobState,
    ValidatorTooLong,
    PathTooLong,
    UnsupportedPathEncoding,
    InvalidPathEncoding,
    PathEncodingMismatch,
    IntegrityCheckFailed,
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite operation failed: {error}"),
            Self::CoreInput(error) => write!(formatter, "stored core value is invalid: {error}"),
            Self::State(error) => write!(formatter, "stored checkpoint violates state contract: {error}"),
            Self::WrongApplicationId { found } => {
                write!(formatter, "database application id {found} does not belong to this store")
            }
            Self::UnrecognizedStore => {
                formatter.write_str("database contains unrecognized tables and is not a GoreeCloud download store")
            }
            Self::MissingSchema => formatter.write_str("required durable-store schema is incomplete"),
            Self::MetadataInvariant => formatter.write_str("durable-store metadata invariant failed"),
            Self::InvalidSchemaVersion => formatter.write_str("durable-store schema version is invalid"),
            Self::MigrationRequired { found, current } => write!(
                formatter,
                "durable-store schema {found} requires migration to {current}"
            ),
            Self::UnsupportedFutureSchema { found, current } => write!(
                formatter,
                "durable-store schema {found} is newer than supported schema {current}"
            ),
            Self::GenerationConflict { expected, actual } => write!(
                formatter,
                "durable-store generation conflict: expected {expected}, found {actual}"
            ),
            Self::InvalidUnsignedInteger => {
                formatter.write_str("stored unsigned integer encoding is invalid")
            }
            Self::InvalidJobState => formatter.write_str("stored job-state encoding is invalid"),
            Self::ValidatorTooLong => formatter.write_str("stored remote validator exceeds the supported length"),
            Self::PathTooLong => formatter.write_str("stored path exceeds the supported length"),
            Self::UnsupportedPathEncoding => {
                formatter.write_str("stored path encoding is not supported on this platform")
            }
            Self::InvalidPathEncoding => formatter.write_str("stored path encoding is malformed"),
            Self::PathEncodingMismatch => {
                formatter.write_str("final and staging paths use different platform encodings")
            }
            Self::IntegrityCheckFailed => formatter.write_str("SQLite quick-check reported database corruption"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::CoreInput(error) => Some(error),
            Self::State(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goreecloud_download_core::DownloadJob;
    use goreecloud_download_state::PartialArtifactPaths;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

    static NEXT_TEST_DB: AtomicU64 = AtomicU64::new(1);

    struct TestDatabase {
        path: PathBuf,
    }

    impl TestDatabase {
        fn new() -> Self {
            let serial = NEXT_TEST_DB.fetch_add(1, AtomicOrdering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "gcdm-store-test-{}-{serial}.sqlite3",
                std::process::id()
            ));
            remove_if_exists(&path);
            remove_if_exists(&PathBuf::from(format!("{}-journal", path.display())));
            Self { path }
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            remove_if_exists(&self.path);
            remove_if_exists(&PathBuf::from(format!("{}-journal", self.path.display())));
        }
    }

    fn remove_if_exists(path: &Path) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("failed to remove test database artifact: {error}"),
        }
    }

    fn active_job(id: &str, downloaded_bytes: u64) -> DownloadJob {
        let mut job = DownloadJob::new(
            JobId::parse(id).unwrap(),
            SensitiveUrl::parse("https://example.invalid/archive.iso?token=secret").unwrap(),
            PathBuf::from(format!("downloads/{id}.iso")),
            None,
            RemoteValidators {
                etag: Some("\"v1\"".into()),
                last_modified: None,
            },
        )
        .unwrap();
        job.start().unwrap();
        job.record_progress(downloaded_bytes).unwrap();
        job.pause().unwrap();
        job
    }

    #[test]
    fn new_store_uses_verified_sqlite_and_durable_profile() {
        let database = TestDatabase::new();
        let store = SqliteStore::open(&database.path).unwrap();

        assert_eq!(store.sqlite_version().unwrap(), "3.53.2");
        assert_eq!(store.generation().unwrap(), 0);

        let journal_mode: String = store
            .connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        let synchronous: i64 = store
            .connection
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .unwrap();
        let application_id: i64 = store
            .connection
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .unwrap();

        assert_eq!(journal_mode, "delete");
        assert_eq!(synchronous, 3);
        assert_eq!(application_id, APPLICATION_ID);
        store.quick_check().unwrap();
    }

    #[test]
    fn checkpoint_round_trips_after_reopen_without_exposing_url_in_debug() {
        let database = TestDatabase::new();
        let checkpoint = JobCheckpointV1::capture(&active_job("job-001", 4096)).unwrap();

        {
            let mut store = SqliteStore::open(&database.path).unwrap();
            let batch = CommitBatchV1::new(0, vec![JobMutationV1::Upsert(checkpoint.clone())]).unwrap();
            store.apply_batch(&batch).unwrap();
            assert_eq!(store.generation().unwrap(), 1);
        }

        let store = SqliteStore::open(&database.path).unwrap();
        let loaded = store.load(checkpoint.id()).unwrap().unwrap();
        assert_eq!(loaded, checkpoint);
        let debug = format!("{loaded:?}");
        assert!(debug.contains("SensitiveUrl([REDACTED])"));
        assert!(!debug.contains("token=secret"));
    }

    #[test]
    fn stale_generation_is_rejected_without_overwriting_newer_state() {
        let database = TestDatabase::new();
        let first = JobCheckpointV1::capture(&active_job("job-001", 10)).unwrap();
        let second = JobCheckpointV1::capture(&active_job("job-002", 20)).unwrap();
        let mut store = SqliteStore::open(&database.path).unwrap();

        store
            .apply_batch(&CommitBatchV1::new(0, vec![JobMutationV1::Upsert(first.clone())]).unwrap())
            .unwrap();

        let stale = CommitBatchV1::new(0, vec![JobMutationV1::Upsert(second)]).unwrap();
        assert!(matches!(
            store.apply_batch(&stale),
            Err(StoreError::GenerationConflict {
                expected: 0,
                actual: 1
            })
        ));
        assert_eq!(store.generation().unwrap(), 1);
        assert_eq!(store.load_all().unwrap(), vec![first]);
    }

    #[test]
    fn remove_and_upsert_commit_atomically_under_one_generation() {
        let database = TestDatabase::new();
        let first = JobCheckpointV1::capture(&active_job("job-001", 10)).unwrap();
        let second = JobCheckpointV1::capture(&active_job("job-002", 20)).unwrap();
        let mut store = SqliteStore::open(&database.path).unwrap();

        store
            .apply_batch(&CommitBatchV1::new(0, vec![JobMutationV1::Upsert(first.clone())]).unwrap())
            .unwrap();
        store
            .apply_batch(
                &CommitBatchV1::new(
                    1,
                    vec![
                        JobMutationV1::Remove(first.id().clone()),
                        JobMutationV1::Upsert(second.clone()),
                    ],
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(store.generation().unwrap(), 2);
        assert_eq!(store.load_all().unwrap(), vec![second]);
    }

    #[test]
    fn unsigned_counters_round_trip_above_sqlite_signed_integer_range() {
        let database = TestDatabase::new();
        let id = JobId::parse("huge-job").unwrap();
        let paths = PartialArtifactPaths::for_job(PathBuf::from("downloads/huge.bin"), &id).unwrap();
        let checkpoint = JobCheckpointV1::from_fields(JobCheckpointFieldsV1 {
            id,
            source: SensitiveUrl::parse("https://example.invalid/huge.bin").unwrap(),
            final_path: paths.final_path().clone(),
            staging_path: paths.staging_path().clone(),
            state: JobState::Paused,
            downloaded_bytes: u64::MAX,
            expected_bytes: None,
            validators: RemoteValidators::default(),
        })
        .unwrap();

        let mut store = SqliteStore::open(&database.path).unwrap();
        store
            .apply_batch(&CommitBatchV1::new(0, vec![JobMutationV1::Upsert(checkpoint.clone())]).unwrap())
            .unwrap();
        assert_eq!(store.load_all().unwrap(), vec![checkpoint]);
    }

    #[test]
    fn waiting_and_failure_state_details_round_trip() {
        let database = TestDatabase::new();
        let mut waiting = active_job("waiting-job", 5);
        waiting.resume().unwrap();
        waiting.wait(WaitReason::Vpn).unwrap();

        let mut failed = active_job("failed-job", 7);
        failed.resume().unwrap();
        failed.fail(FailureKind::Storage).unwrap();

        let waiting = JobCheckpointV1::capture(&waiting).unwrap();
        let failed = JobCheckpointV1::capture(&failed).unwrap();
        let mut store = SqliteStore::open(&database.path).unwrap();
        store
            .apply_batch(
                &CommitBatchV1::new(
                    0,
                    vec![
                        JobMutationV1::Upsert(waiting.clone()),
                        JobMutationV1::Upsert(failed.clone()),
                    ],
                )
                .unwrap(),
            )
            .unwrap();

        let loaded = store.load_all().unwrap();
        assert!(loaded.contains(&waiting));
        assert!(loaded.contains(&failed));
    }

    #[test]
    fn unsupported_future_schema_fails_closed() {
        let database = TestDatabase::new();
        {
            let store = SqliteStore::open(&database.path).unwrap();
            store
                .connection
                .execute(
                    "UPDATE store_metadata SET schema_version = ?1 WHERE singleton = 1",
                    params![i64::from(CURRENT_SCHEMA_VERSION) + 1],
                )
                .unwrap();
        }

        assert!(matches!(
            SqliteStore::open(&database.path),
            Err(StoreError::UnsupportedFutureSchema { .. })
        ));
    }

    #[test]
    fn unrelated_sqlite_database_is_not_reinterpreted_as_download_state() {
        let database = TestDatabase::new();
        {
            let connection = Connection::open(&database.path).unwrap();
            connection
                .execute_batch("CREATE TABLE unrelated (id INTEGER PRIMARY KEY);")
                .unwrap();
        }

        assert!(matches!(
            SqliteStore::open(&database.path),
            Err(StoreError::UnrecognizedStore)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn unix_non_utf8_paths_round_trip_losslessly() {
        let database = TestDatabase::new();
        let id = JobId::parse("non-utf8").unwrap();
        let destination = PathBuf::from(OsString::from_vec(vec![b'd', b'l', b'/', 0xff, b'.', b'b', b'i', b'n']));
        let paths = PartialArtifactPaths::for_job(destination, &id).unwrap();
        let checkpoint = JobCheckpointV1::from_fields(JobCheckpointFieldsV1 {
            id,
            source: SensitiveUrl::parse("https://example.invalid/non-utf8").unwrap(),
            final_path: paths.final_path().clone(),
            staging_path: paths.staging_path().clone(),
            state: JobState::Paused,
            downloaded_bytes: 0,
            expected_bytes: None,
            validators: RemoteValidators::default(),
        })
        .unwrap();

        let mut store = SqliteStore::open(&database.path).unwrap();
        store
            .apply_batch(&CommitBatchV1::new(0, vec![JobMutationV1::Upsert(checkpoint.clone())]).unwrap())
            .unwrap();
        assert_eq!(store.load_all().unwrap(), vec![checkpoint]);
    }
}

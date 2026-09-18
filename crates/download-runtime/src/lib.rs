#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use goreecloud_download_core::{DownloadJob, JobId, JobState, RemoteValidators, SensitiveUrl};
use goreecloud_download_fs::{DurableStagingFile, StagingOpenDisposition};
use goreecloud_download_http::{
    ByteContentRange, HttpRequestPlan, HttpResponseDecision, HttpResponseMetadata,
    evaluate_response, parse_content_range,
};
use goreecloud_download_state::{
    CommitBatchV1, FinalArtifactDecision, JobCheckpointFieldsV1, JobCheckpointV1, JobMutationV1,
    StateError, evaluate_final_artifact,
};
use goreecloud_download_store_sqlite::{SqliteStore, StoreError};
use reqwest::blocking::{Client, Response};
use reqwest::header::{
    ACCEPT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, ETAG, IF_RANGE, LAST_MODIFIED, RANGE,
};
use reqwest::redirect::Policy;

pub const DEFAULT_CHECKPOINT_CHUNK_BYTES: usize = 1024 * 1024;
pub const MAX_CHECKPOINT_CHUNK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportErrorKind {
    Timeout,
    Connect,
    Request,
    Body,
    Protocol,
}

impl fmt::Display for TransportErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Timeout => "network operation timed out",
            Self::Connect => "network connection failed",
            Self::Request => "HTTP request failed",
            Self::Body => "HTTP response body failed",
            Self::Protocol => "HTTP response metadata is invalid",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    UnexpectedStatus(u16),
    UnexpectedContentRange,
    ResumeRangeMissingTotal,
    ResumeRangeNotToEnd,
    ResumeContentLengthMismatch,
    ExpectedLengthChanged,
    RemoteValidatorChanged,
    BodyExceededExpectedLength,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedStatus(status) => {
                write!(formatter, "HTTP response status {status} is not accepted")
            }
            Self::UnexpectedContentRange => {
                formatter.write_str("HTTP response contains an unexpected Content-Range")
            }
            Self::ResumeRangeMissingTotal => {
                formatter.write_str("resume response does not identify the complete object length")
            }
            Self::ResumeRangeNotToEnd => {
                formatter.write_str("resume response does not cover the remainder of the object")
            }
            Self::ResumeContentLengthMismatch => {
                formatter.write_str("resume response Content-Length does not match Content-Range")
            }
            Self::ExpectedLengthChanged => {
                formatter.write_str("remote object length changed during resume")
            }
            Self::RemoteValidatorChanged => {
                formatter.write_str("remote validator changed during resume")
            }
            Self::BodyExceededExpectedLength => {
                formatter.write_str("response body exceeded the expected object length")
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    JobNotFound,
    JobAlreadyExists,
    Suspended,
    TerminalState,
    InvalidCheckpointChunkSize,
    ConcurrentSnapshot,
    State(StateError),
    Store(StoreError),
    Storage,
    Transport(TransportErrorKind),
    Protocol(ProtocolError),
    IncompleteBody { expected: u64, actual: u64 },
    FinalArtifactMissing,
    FinalArtifactSizeMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobNotFound => formatter.write_str("download job does not exist"),
            Self::JobAlreadyExists => formatter.write_str("download job already exists"),
            Self::Suspended => formatter.write_str("download job is paused or waiting"),
            Self::TerminalState => formatter.write_str("download job is in a terminal state"),
            Self::InvalidCheckpointChunkSize => {
                formatter.write_str("checkpoint chunk size is outside the supported bounds")
            }
            Self::ConcurrentSnapshot => {
                formatter.write_str("could not obtain a stable durable-state snapshot")
            }
            Self::State(error) => write!(formatter, "download state is invalid: {error}"),
            Self::Store(error) => write!(formatter, "durable job store failed: {error}"),
            Self::Storage => formatter.write_str("download staging storage operation failed"),
            Self::Transport(kind) => write!(formatter, "{kind}"),
            Self::Protocol(error) => write!(formatter, "{error}"),
            Self::IncompleteBody { expected, actual } => write!(
                formatter,
                "response ended before the expected object length: expected {expected}, got {actual}"
            ),
            Self::FinalArtifactMissing => {
                formatter.write_str("completed download artifact is missing")
            }
            Self::FinalArtifactSizeMismatch { expected, actual } => write!(
                formatter,
                "completed download artifact length mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}

impl Error for RuntimeError {}

impl From<StateError> for RuntimeError {
    fn from(value: StateError) -> Self {
        Self::State(value)
    }
}

impl From<StoreError> for RuntimeError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

pub struct TransportResponse {
    pub status_code: u16,
    pub content_range: Option<ByteContentRange>,
    pub content_length: Option<u64>,
    pub validators: RemoteValidators,
    pub body: Box<dyn Read + Send>,
}

pub trait HttpTransport {
    fn get(
        &self,
        source: &SensitiveUrl,
        plan: &HttpRequestPlan,
    ) -> Result<TransportResponse, TransportErrorKind>;
}

#[derive(Debug)]
pub struct ReqwestTransport {
    client: Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, TransportErrorKind> {
        let client = Client::builder()
            .redirect(Policy::none())
            .no_proxy()
            .tls_backend_rustls()
            .user_agent(concat!(
                "GoreeCloud-Advanced-Download-Manager/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(classify_reqwest_error)?;
        Ok(Self { client })
    }

    fn response(&self, response: Response) -> Result<TransportResponse, TransportErrorKind> {
        let status_code = response.status().as_u16();
        let headers = response.headers();
        let content_range = match header_text(headers.get(CONTENT_RANGE))? {
            Some(value) => {
                Some(parse_content_range(&value).map_err(|_| TransportErrorKind::Protocol)?)
            }
            None => None,
        };
        let content_length = match header_text(headers.get(CONTENT_LENGTH))? {
            Some(value) => Some(
                value
                    .parse::<u64>()
                    .map_err(|_| TransportErrorKind::Protocol)?,
            ),
            None => None,
        };
        let validators = RemoteValidators {
            etag: header_text(headers.get(ETAG))?,
            last_modified: header_text(headers.get(LAST_MODIFIED))?,
        };

        Ok(TransportResponse {
            status_code,
            content_range,
            content_length,
            validators,
            body: Box::new(response),
        })
    }
}

impl HttpTransport for ReqwestTransport {
    fn get(
        &self,
        source: &SensitiveUrl,
        plan: &HttpRequestPlan,
    ) -> Result<TransportResponse, TransportErrorKind> {
        let mut request = self
            .client
            .get(source.expose())
            .header(ACCEPT_ENCODING, "identity");

        if let Some(range) = plan.range_header_value() {
            request = request.header(RANGE, range);
        }
        if let Some(if_range) = plan.if_range_header_value() {
            request = request.header(IF_RANGE, if_range);
        }

        let response = request.send().map_err(classify_reqwest_error)?;
        self.response(response)
    }
}

fn classify_reqwest_error(error: reqwest::Error) -> TransportErrorKind {
    if error.is_timeout() {
        TransportErrorKind::Timeout
    } else if error.is_connect() {
        TransportErrorKind::Connect
    } else if error.is_body() {
        TransportErrorKind::Body
    } else {
        TransportErrorKind::Request
    }
}

fn header_text(
    value: Option<&reqwest::header::HeaderValue>,
) -> Result<Option<String>, TransportErrorKind> {
    value
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| TransportErrorKind::Protocol)
        })
        .transpose()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadOutcome {
    pub final_path: PathBuf,
    pub durable_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct SingleStreamRuntime {
    checkpoint_chunk_bytes: usize,
}

impl Default for SingleStreamRuntime {
    fn default() -> Self {
        Self {
            checkpoint_chunk_bytes: DEFAULT_CHECKPOINT_CHUNK_BYTES,
        }
    }
}

impl SingleStreamRuntime {
    pub fn new(checkpoint_chunk_bytes: usize) -> Result<Self, RuntimeError> {
        if checkpoint_chunk_bytes == 0 || checkpoint_chunk_bytes > MAX_CHECKPOINT_CHUNK_BYTES {
            return Err(RuntimeError::InvalidCheckpointChunkSize);
        }
        Ok(Self {
            checkpoint_chunk_bytes,
        })
    }

    pub fn enqueue(&self, store: &mut SqliteStore, job: &DownloadJob) -> Result<(), RuntimeError> {
        let (generation, existing) = load_consistent(store, job.id())?;
        if existing.is_some() {
            return Err(RuntimeError::JobAlreadyExists);
        }

        let checkpoint = JobCheckpointV1::capture(job)?;
        let mut generation = generation;
        commit_checkpoint(store, &mut generation, &checkpoint)
    }

    pub fn execute<T: HttpTransport>(
        &self,
        store: &mut SqliteStore,
        id: &JobId,
        transport: &T,
    ) -> Result<DownloadOutcome, RuntimeError> {
        let (generation, checkpoint) = load_consistent(store, id)?;
        let checkpoint = checkpoint.ok_or(RuntimeError::JobNotFound)?;
        self.execute_loaded(store, generation, checkpoint, transport)
    }

    pub fn resume<T: HttpTransport>(
        &self,
        store: &mut SqliteStore,
        id: &JobId,
        transport: &T,
    ) -> Result<DownloadOutcome, RuntimeError> {
        let (mut generation, checkpoint) = load_consistent(store, id)?;
        let mut checkpoint = checkpoint.ok_or(RuntimeError::JobNotFound)?;

        if matches!(checkpoint.state(), JobState::Paused | JobState::Waiting(_)) {
            checkpoint = rebuild_checkpoint(
                &checkpoint,
                JobState::Downloading,
                checkpoint.downloaded_bytes(),
                checkpoint.expected_bytes(),
                checkpoint.validators().clone(),
            )?;
            commit_checkpoint(store, &mut generation, &checkpoint)?;
        }

        self.execute_loaded(store, generation, checkpoint, transport)
    }

    fn execute_loaded<T: HttpTransport>(
        &self,
        store: &mut SqliteStore,
        mut generation: u64,
        mut checkpoint: JobCheckpointV1,
        transport: &T,
    ) -> Result<DownloadOutcome, RuntimeError> {
        match checkpoint.state() {
            JobState::Completed => return verify_completed(&checkpoint),
            JobState::Failed(_) | JobState::Cancelled => return Err(RuntimeError::TerminalState),
            JobState::Paused | JobState::Waiting(_) => return Err(RuntimeError::Suspended),
            JobState::Verifying | JobState::Processing => {
                if let Some(outcome) =
                    recover_post_transfer(store, &mut generation, &mut checkpoint)?
                {
                    return Ok(outcome);
                }
            }
            JobState::Queued | JobState::Downloading => {}
        }

        self.transfer(store, &mut generation, checkpoint, transport)
    }

    fn transfer<T: HttpTransport>(
        &self,
        store: &mut SqliteStore,
        generation: &mut u64,
        mut checkpoint: JobCheckpointV1,
        transport: &T,
    ) -> Result<DownloadOutcome, RuntimeError> {
        let (mut staging, disposition) =
            DurableStagingFile::open(checkpoint.paths().clone(), checkpoint.downloaded_bytes())
                .map_err(|_| RuntimeError::Storage)?;

        if matches!(
            disposition,
            StagingOpenDisposition::RestartedFromBeginning { .. }
        ) {
            checkpoint = rebuild_checkpoint(
                &checkpoint,
                JobState::Downloading,
                0,
                checkpoint.expected_bytes(),
                checkpoint.validators().clone(),
            )?;
            commit_checkpoint(store, generation, &checkpoint)?;
        } else if checkpoint.state() == JobState::Queued {
            checkpoint = rebuild_checkpoint(
                &checkpoint,
                JobState::Downloading,
                checkpoint.downloaded_bytes(),
                checkpoint.expected_bytes(),
                checkpoint.validators().clone(),
            )?;
            commit_checkpoint(store, generation, &checkpoint)?;
        }

        let mut plan = goreecloud_download_http::plan_request(
            checkpoint.downloaded_bytes(),
            checkpoint.validators(),
        );

        if matches!(plan, HttpRequestPlan::RestartFull { .. }) {
            checkpoint = reset_for_full_restart(store, generation, &mut staging, &checkpoint)?;
            plan = HttpRequestPlan::Full;
        }

        let mut response = transport
            .get(checkpoint.source(), &plan)
            .map_err(RuntimeError::Transport)?;
        let mut decision = evaluate_response(
            &plan,
            HttpResponseMetadata {
                status_code: response.status_code,
                content_range: response.content_range,
            },
        );

        if decision == HttpResponseDecision::RetryFromBeginning {
            checkpoint = reset_for_full_restart(store, generation, &mut staging, &checkpoint)?;
            plan = HttpRequestPlan::Full;
            response = transport
                .get(checkpoint.source(), &plan)
                .map_err(RuntimeError::Transport)?;
            decision = evaluate_response(
                &plan,
                HttpResponseMetadata {
                    status_code: response.status_code,
                    content_range: response.content_range,
                },
            );
        }

        if decision == HttpResponseDecision::Reject
            || decision == HttpResponseDecision::RetryFromBeginning
        {
            return Err(RuntimeError::Protocol(ProtocolError::UnexpectedStatus(
                response.status_code,
            )));
        }

        let (expected_bytes, validators) =
            validate_response_metadata(&checkpoint, decision, &response)?;

        if decision == HttpResponseDecision::AcceptFullBody {
            if response.content_range.is_some() {
                return Err(RuntimeError::Protocol(
                    ProtocolError::UnexpectedContentRange,
                ));
            }
            if checkpoint.downloaded_bytes() != 0 {
                checkpoint = reset_for_full_restart(store, generation, &mut staging, &checkpoint)?;
            }
        }

        let metadata_checkpoint = rebuild_checkpoint(
            &checkpoint,
            JobState::Downloading,
            checkpoint.downloaded_bytes(),
            expected_bytes,
            validators.clone(),
        )?;
        if metadata_checkpoint != checkpoint {
            checkpoint = metadata_checkpoint;
            commit_checkpoint(store, generation, &checkpoint)?;
        }

        let mut buffer = vec![0_u8; self.checkpoint_chunk_bytes];
        loop {
            let read = read_checkpoint_chunk(&mut *response.body, &mut buffer)?;
            if read == 0 {
                break;
            }

            let write = staging
                .append_bytes(&buffer[..read])
                .map_err(|_| RuntimeError::Storage)?;

            if expected_bytes.is_some_and(|expected| write.durable_bytes > expected) {
                return Err(RuntimeError::Protocol(
                    ProtocolError::BodyExceededExpectedLength,
                ));
            }

            checkpoint = rebuild_checkpoint(
                &checkpoint,
                JobState::Downloading,
                write.durable_bytes,
                expected_bytes,
                validators.clone(),
            )?;
            commit_checkpoint(store, generation, &checkpoint)?;
        }

        let durable_bytes = staging.durable_len().map_err(|_| RuntimeError::Storage)?;
        if let Some(expected) = expected_bytes
            && durable_bytes != expected
        {
            return Err(RuntimeError::IncompleteBody {
                expected,
                actual: durable_bytes,
            });
        }

        checkpoint = rebuild_checkpoint(
            &checkpoint,
            JobState::Verifying,
            durable_bytes,
            expected_bytes,
            validators,
        )?;
        commit_checkpoint(store, generation, &checkpoint)?;

        let final_path = staging.promote().map_err(|_| RuntimeError::Storage)?;

        checkpoint = rebuild_checkpoint(
            &checkpoint,
            JobState::Completed,
            durable_bytes,
            expected_bytes,
            checkpoint.validators().clone(),
        )?;
        commit_checkpoint(store, generation, &checkpoint)?;

        Ok(DownloadOutcome {
            final_path,
            durable_bytes,
        })
    }
}

fn read_checkpoint_chunk(body: &mut dyn Read, buffer: &mut [u8]) -> Result<usize, RuntimeError> {
    let mut filled = 0;

    while filled < buffer.len() {
        match body.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(RuntimeError::Transport(TransportErrorKind::Body)),
        }
    }

    Ok(filled)
}

fn validate_response_metadata(
    checkpoint: &JobCheckpointV1,
    decision: HttpResponseDecision,
    response: &TransportResponse,
) -> Result<(Option<u64>, RemoteValidators), RuntimeError> {
    match decision {
        HttpResponseDecision::AcceptFullBody => {
            Ok((response.content_length, response.validators.clone()))
        }
        HttpResponseDecision::AppendResumeBody => {
            let range = response.content_range.ok_or(RuntimeError::Protocol(
                ProtocolError::ResumeRangeMissingTotal,
            ))?;
            let complete_length = range.complete_length.ok_or(RuntimeError::Protocol(
                ProtocolError::ResumeRangeMissingTotal,
            ))?;

            if range.end.checked_add(1) != Some(complete_length) {
                return Err(RuntimeError::Protocol(ProtocolError::ResumeRangeNotToEnd));
            }

            if let Some(content_length) = response.content_length {
                let expected_range_length = range
                    .end
                    .checked_sub(range.start)
                    .and_then(|length| length.checked_add(1))
                    .ok_or(RuntimeError::Protocol(
                        ProtocolError::ResumeContentLengthMismatch,
                    ))?;
                if content_length != expected_range_length {
                    return Err(RuntimeError::Protocol(
                        ProtocolError::ResumeContentLengthMismatch,
                    ));
                }
            }

            if checkpoint
                .expected_bytes()
                .is_some_and(|expected| expected != complete_length)
            {
                return Err(RuntimeError::Protocol(ProtocolError::ExpectedLengthChanged));
            }

            let validators =
                merge_resume_validators(checkpoint.validators(), &response.validators)?;
            Ok((Some(complete_length), validators))
        }
        HttpResponseDecision::RetryFromBeginning | HttpResponseDecision::Reject => Err(
            RuntimeError::Protocol(ProtocolError::UnexpectedStatus(response.status_code)),
        ),
    }
}

fn merge_resume_validators(
    persisted: &RemoteValidators,
    response: &RemoteValidators,
) -> Result<RemoteValidators, RuntimeError> {
    if persisted
        .etag
        .as_ref()
        .zip(response.etag.as_ref())
        .is_some_and(|(persisted, response)| persisted != response)
        || persisted
            .last_modified
            .as_ref()
            .zip(response.last_modified.as_ref())
            .is_some_and(|(persisted, response)| persisted != response)
    {
        return Err(RuntimeError::Protocol(
            ProtocolError::RemoteValidatorChanged,
        ));
    }

    Ok(RemoteValidators {
        etag: response.etag.clone().or_else(|| persisted.etag.clone()),
        last_modified: response
            .last_modified
            .clone()
            .or_else(|| persisted.last_modified.clone()),
    })
}

fn reset_for_full_restart(
    store: &mut SqliteStore,
    generation: &mut u64,
    staging: &mut DurableStagingFile,
    checkpoint: &JobCheckpointV1,
) -> Result<JobCheckpointV1, RuntimeError> {
    staging.truncate_to(0).map_err(|_| RuntimeError::Storage)?;

    let checkpoint = rebuild_checkpoint(
        checkpoint,
        JobState::Downloading,
        0,
        None,
        RemoteValidators::default(),
    )?;
    commit_checkpoint(store, generation, &checkpoint)?;
    Ok(checkpoint)
}

fn recover_post_transfer(
    store: &mut SqliteStore,
    generation: &mut u64,
    checkpoint: &mut JobCheckpointV1,
) -> Result<Option<DownloadOutcome>, RuntimeError> {
    let final_path = checkpoint.paths().final_path().clone();
    if final_path.exists() {
        let metadata = fs::metadata(&final_path).map_err(|_| RuntimeError::Storage)?;
        if !metadata.is_file() {
            return Err(RuntimeError::FinalArtifactMissing);
        }

        match evaluate_final_artifact(checkpoint.expected_bytes(), Some(metadata.len())) {
            FinalArtifactDecision::AcceptCompleted => {
                *checkpoint = rebuild_checkpoint(
                    checkpoint,
                    JobState::Completed,
                    checkpoint.downloaded_bytes(),
                    checkpoint.expected_bytes(),
                    checkpoint.validators().clone(),
                )?;
                commit_checkpoint(store, generation, checkpoint)?;
                return Ok(Some(DownloadOutcome {
                    final_path: final_path.clone(),
                    durable_bytes: metadata.len(),
                }));
            }
            FinalArtifactDecision::Missing => return Err(RuntimeError::FinalArtifactMissing),
            FinalArtifactDecision::SizeMismatch { expected, actual } => {
                return Err(RuntimeError::FinalArtifactSizeMismatch { expected, actual });
            }
        }
    }

    let (staging, disposition) =
        DurableStagingFile::open(checkpoint.paths().clone(), checkpoint.downloaded_bytes())
            .map_err(|_| RuntimeError::Storage)?;

    if matches!(
        disposition,
        StagingOpenDisposition::RestartedFromBeginning { .. }
    ) {
        *checkpoint = rebuild_checkpoint(
            checkpoint,
            JobState::Downloading,
            0,
            checkpoint.expected_bytes(),
            checkpoint.validators().clone(),
        )?;
        commit_checkpoint(store, generation, checkpoint)?;
        return Ok(None);
    }

    let durable_bytes = staging.durable_len().map_err(|_| RuntimeError::Storage)?;
    if let Some(expected) = checkpoint.expected_bytes()
        && durable_bytes != expected
    {
        return Err(RuntimeError::IncompleteBody {
            expected,
            actual: durable_bytes,
        });
    }

    let final_path = staging.promote().map_err(|_| RuntimeError::Storage)?;
    *checkpoint = rebuild_checkpoint(
        checkpoint,
        JobState::Completed,
        durable_bytes,
        checkpoint.expected_bytes(),
        checkpoint.validators().clone(),
    )?;
    commit_checkpoint(store, generation, checkpoint)?;

    Ok(Some(DownloadOutcome {
        final_path,
        durable_bytes,
    }))
}

fn verify_completed(checkpoint: &JobCheckpointV1) -> Result<DownloadOutcome, RuntimeError> {
    let metadata = fs::metadata(checkpoint.paths().final_path())
        .map_err(|_| RuntimeError::FinalArtifactMissing)?;
    if !metadata.is_file() {
        return Err(RuntimeError::FinalArtifactMissing);
    }

    match evaluate_final_artifact(checkpoint.expected_bytes(), Some(metadata.len())) {
        FinalArtifactDecision::AcceptCompleted => Ok(DownloadOutcome {
            final_path: checkpoint.paths().final_path().clone(),
            durable_bytes: metadata.len(),
        }),
        FinalArtifactDecision::Missing => Err(RuntimeError::FinalArtifactMissing),
        FinalArtifactDecision::SizeMismatch { expected, actual } => {
            Err(RuntimeError::FinalArtifactSizeMismatch { expected, actual })
        }
    }
}

fn rebuild_checkpoint(
    checkpoint: &JobCheckpointV1,
    state: JobState,
    downloaded_bytes: u64,
    expected_bytes: Option<u64>,
    validators: RemoteValidators,
) -> Result<JobCheckpointV1, RuntimeError> {
    JobCheckpointV1::from_fields(JobCheckpointFieldsV1 {
        id: checkpoint.id().clone(),
        source: checkpoint.source().clone(),
        final_path: checkpoint.paths().final_path().clone(),
        staging_path: checkpoint.paths().staging_path().clone(),
        state,
        downloaded_bytes,
        expected_bytes,
        validators,
    })
    .map_err(RuntimeError::State)
}

fn commit_checkpoint(
    store: &mut SqliteStore,
    generation: &mut u64,
    checkpoint: &JobCheckpointV1,
) -> Result<(), RuntimeError> {
    let batch = CommitBatchV1::new(*generation, vec![JobMutationV1::Upsert(checkpoint.clone())])?;
    let next_generation = batch.next_generation();
    store.apply_batch(&batch)?;
    *generation = next_generation;
    Ok(())
}

fn load_consistent(
    store: &SqliteStore,
    id: &JobId,
) -> Result<(u64, Option<JobCheckpointV1>), RuntimeError> {
    for _ in 0..4 {
        let before = store.generation()?;
        let checkpoint = store.load(id)?;
        let after = store.generation()?;
        if before == after {
            return Ok((after, checkpoint));
        }
    }

    Err(RuntimeError::ConcurrentSnapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use goreecloud_download_core::SensitiveUrl;
    use goreecloud_download_state::{CommitBatchV1, JobCheckpointV1, JobMutationV1};
    use std::collections::VecDeque;
    use std::io::{Cursor, Write};
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "goreecloud-download-runtime-{label}-{}-{sequence}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create test root");
            Self(path)
        }

        fn store(&self) -> SqliteStore {
            SqliteStore::open(self.0.join("state.sqlite")).expect("open SQLite store")
        }

        fn destination(&self) -> PathBuf {
            self.0.join("downloads/file.bin")
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn serve_http_once(response: Vec<u8>) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback HTTP server");
        let address = listener.local_addr().expect("resolve loopback address");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept loopback request");
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .expect("set loopback read timeout");

            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).expect("read loopback request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                assert!(
                    request.len() <= 32 * 1024,
                    "HTTP request headers are too large"
                );
            }

            stream
                .write_all(&response)
                .expect("write loopback response");
            stream.flush().expect("flush loopback response");
            String::from_utf8(request).expect("loopback request must be UTF-8 headers")
        });

        (
            format!("http://{address}/file.bin?token=synthetic-test"),
            handle,
        )
    }

    enum ScriptedBody {
        Bytes(Vec<u8>),
        FailAfter { bytes: Vec<u8>, fail_after: usize },
    }

    struct ScriptedResponse {
        status_code: u16,
        content_range: Option<ByteContentRange>,
        content_length: Option<u64>,
        validators: RemoteValidators,
        body: ScriptedBody,
    }

    struct ScriptedTransport {
        responses: Mutex<VecDeque<ScriptedResponse>>,
        plans: Mutex<Vec<HttpRequestPlan>>,
    }

    impl ScriptedTransport {
        fn new(responses: Vec<ScriptedResponse>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                plans: Mutex::new(Vec::new()),
            }
        }

        fn plans(&self) -> Vec<HttpRequestPlan> {
            self.plans.lock().unwrap().clone()
        }
    }

    impl HttpTransport for ScriptedTransport {
        fn get(
            &self,
            _source: &SensitiveUrl,
            plan: &HttpRequestPlan,
        ) -> Result<TransportResponse, TransportErrorKind> {
            self.plans.lock().unwrap().push(plan.clone());
            let response = self
                .responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("scripted response");

            let body: Box<dyn Read + Send> = match response.body {
                ScriptedBody::Bytes(bytes) => Box::new(Cursor::new(bytes)),
                ScriptedBody::FailAfter { bytes, fail_after } => {
                    Box::new(FailAfterReader::new(bytes, fail_after))
                }
            };

            Ok(TransportResponse {
                status_code: response.status_code,
                content_range: response.content_range,
                content_length: response.content_length,
                validators: response.validators,
                body,
            })
        }
    }

    struct FailAfterReader {
        bytes: Vec<u8>,
        position: usize,
        fail_after: usize,
    }

    impl FailAfterReader {
        fn new(bytes: Vec<u8>, fail_after: usize) -> Self {
            Self {
                bytes,
                position: 0,
                fail_after,
            }
        }
    }

    impl Read for FailAfterReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.position >= self.fail_after {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "scripted body failure",
                ));
            }
            if self.position >= self.bytes.len() {
                return Ok(0);
            }

            let end = self
                .bytes
                .len()
                .min(self.fail_after)
                .min(self.position.saturating_add(buffer.len()));
            let length = end.saturating_sub(self.position);
            buffer[..length].copy_from_slice(&self.bytes[self.position..end]);
            self.position = end;
            Ok(length)
        }
    }

    fn id() -> JobId {
        JobId::parse("job-001").unwrap()
    }

    fn source() -> SensitiveUrl {
        SensitiveUrl::parse("https://example.invalid/file.bin?token=secret").unwrap()
    }

    fn validators(etag: Option<&str>) -> RemoteValidators {
        RemoteValidators {
            etag: etag.map(str::to_owned),
            last_modified: None,
        }
    }

    fn checkpoint(
        root: &TestRoot,
        state: JobState,
        downloaded_bytes: u64,
        expected_bytes: Option<u64>,
        validators: RemoteValidators,
    ) -> JobCheckpointV1 {
        let job = DownloadJob::new(
            id(),
            source(),
            root.destination(),
            expected_bytes,
            validators,
        )
        .unwrap();

        let mut checkpoint = JobCheckpointV1::capture(&job).unwrap();
        checkpoint = rebuild_checkpoint(
            &checkpoint,
            state,
            downloaded_bytes,
            expected_bytes,
            checkpoint.validators().clone(),
        )
        .unwrap();
        checkpoint
    }

    fn seed(store: &mut SqliteStore, checkpoint: &JobCheckpointV1) {
        let generation = store.generation().unwrap();
        let batch = CommitBatchV1::new(generation, vec![JobMutationV1::Upsert(checkpoint.clone())])
            .unwrap();
        store.apply_batch(&batch).unwrap();
    }

    fn write_staging(checkpoint: &JobCheckpointV1, bytes: &[u8]) {
        let path = checkpoint.paths().staging_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn reqwest_transport_executes_real_loopback_full_download() {
        let root = TestRoot::new("real-http-full");
        let mut store = root.store();
        let (source_url, server) = serve_http_once(
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nETag: \"v1\"\r\nConnection: close\r\n\r\nabcdef"
                .to_vec(),
        );
        let job = DownloadJob::new(
            id(),
            SensitiveUrl::parse(source_url).unwrap(),
            root.destination(),
            None,
            RemoteValidators::default(),
        )
        .unwrap();

        let runtime = SingleStreamRuntime::new(3).unwrap();
        runtime.enqueue(&mut store, &job).unwrap();
        let transport = ReqwestTransport::new().unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();

        assert_eq!(fs::read(outcome.final_path).unwrap(), b"abcdef");
        let request = server
            .join()
            .expect("loopback server thread")
            .to_ascii_lowercase();
        assert!(request.starts_with("get /file.bin?token=synthetic-test http/1.1\r\n"));
        assert!(request.contains("\r\naccept-encoding: identity\r\n"));
        assert!(!request.contains("\r\nrange:"));
        assert!(!request.contains("\r\nif-range:"));
    }

    #[test]
    fn reqwest_transport_sends_validator_bound_loopback_resume_headers() {
        let root = TestRoot::new("real-http-resume");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(6),
            validators(Some("\"v1\"")),
        );
        write_staging(&checkpoint, b"abc");
        seed(&mut store, &checkpoint);

        let (source_url, server) = serve_http_once(
            b"HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\nContent-Range: bytes 3-5/6\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ndef"
                .to_vec(),
        );
        let checkpoint = rebuild_checkpoint(
            &checkpoint,
            JobState::Downloading,
            3,
            Some(6),
            validators(Some("\"v1\"")),
        )
        .unwrap();
        let source_checkpoint = JobCheckpointV1::from_fields(JobCheckpointFieldsV1 {
            id: checkpoint.id().clone(),
            source: SensitiveUrl::parse(source_url).unwrap(),
            final_path: checkpoint.paths().final_path().clone(),
            staging_path: checkpoint.paths().staging_path().clone(),
            state: checkpoint.state(),
            downloaded_bytes: checkpoint.downloaded_bytes(),
            expected_bytes: checkpoint.expected_bytes(),
            validators: checkpoint.validators().clone(),
        })
        .unwrap();

        let generation = store.generation().unwrap();
        let batch =
            CommitBatchV1::new(generation, vec![JobMutationV1::Upsert(source_checkpoint)]).unwrap();
        store.apply_batch(&batch).unwrap();

        let runtime = SingleStreamRuntime::new(2).unwrap();
        let transport = ReqwestTransport::new().unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();

        assert_eq!(fs::read(outcome.final_path).unwrap(), b"abcdef");
        let request = server.join().expect("loopback server thread").to_ascii_lowercase();
        assert!(request.starts_with("get /file.bin?token=synthetic-test http/1.1\r\n"));
        assert!(request.contains("\r\nrange: bytes=3-\r\n"));
        assert!(request.contains("\r\nif-range: \"v1\"\r\n"));
        assert!(request.contains("\r\naccept-encoding: identity\r\n"));
    }

    #[test]
    fn fresh_full_download_checkpoints_then_promotes() {
        let root = TestRoot::new("fresh");
        let mut store = root.store();
        let job = DownloadJob::new(
            id(),
            source(),
            root.destination(),
            None,
            RemoteValidators::default(),
        )
        .unwrap();
        let runtime = SingleStreamRuntime::new(3).unwrap();
        runtime.enqueue(&mut store, &job).unwrap();

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 200,
            content_range: None,
            content_length: Some(6),
            validators: validators(Some("\"v1\"")),
            body: ScriptedBody::Bytes(b"abcdef".to_vec()),
        }]);

        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();
        assert_eq!(outcome.durable_bytes, 6);
        assert_eq!(fs::read(&outcome.final_path).unwrap(), b"abcdef");
        assert!(
            !root
                .destination()
                .with_file_name("file.bin.gcdm-part.job-001")
                .exists()
        );

        let stored = store.load(&id()).unwrap().unwrap();
        assert_eq!(stored.state(), JobState::Completed);
        assert_eq!(stored.downloaded_bytes(), 6);
        assert_eq!(stored.expected_bytes(), Some(6));
        assert_eq!(stored.validators().etag.as_deref(), Some("\"v1\""));
        assert_eq!(transport.plans(), vec![HttpRequestPlan::Full]);
    }

    #[test]
    fn validator_safe_resume_appends_only_the_missing_suffix() {
        let root = TestRoot::new("resume");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(6),
            validators(Some("\"v1\"")),
        );
        write_staging(&checkpoint, b"abc");
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 3,
                end: 5,
                complete_length: Some(6),
            }),
            content_length: Some(3),
            validators: validators(Some("\"v1\"")),
            body: ScriptedBody::Bytes(b"def".to_vec()),
        }]);

        let runtime = SingleStreamRuntime::new(2).unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();
        assert_eq!(fs::read(outcome.final_path).unwrap(), b"abcdef");
        assert_eq!(
            transport.plans(),
            vec![HttpRequestPlan::Resume {
                start_at: 3,
                if_range: goreecloud_download_http::IfRangeValidator::StrongEtag("\"v1\"".into()),
            }]
        );
    }

    #[test]
    fn missing_safe_validator_resets_file_and_checkpoint_before_full_request() {
        let root = TestRoot::new("restart-full");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(6),
            RemoteValidators::default(),
        );
        write_staging(&checkpoint, b"old");
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 200,
            content_range: None,
            content_length: Some(6),
            validators: validators(Some("\"v2\"")),
            body: ScriptedBody::Bytes(b"new123".to_vec()),
        }]);

        let runtime = SingleStreamRuntime::new(2).unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();
        assert_eq!(fs::read(outcome.final_path).unwrap(), b"new123");
        assert_eq!(transport.plans(), vec![HttpRequestPlan::Full]);
    }

    #[test]
    fn precondition_failure_retries_once_from_zero() {
        let root = TestRoot::new("precondition");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(6),
            validators(Some("\"v1\"")),
        );
        write_staging(&checkpoint, b"abc");
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(vec![
            ScriptedResponse {
                status_code: 412,
                content_range: None,
                content_length: None,
                validators: RemoteValidators::default(),
                body: ScriptedBody::Bytes(Vec::new()),
            },
            ScriptedResponse {
                status_code: 200,
                content_range: None,
                content_length: Some(6),
                validators: validators(Some("\"v2\"")),
                body: ScriptedBody::Bytes(b"new123".to_vec()),
            },
        ]);

        let runtime = SingleStreamRuntime::new(3).unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();
        assert_eq!(fs::read(outcome.final_path).unwrap(), b"new123");
        assert_eq!(transport.plans().len(), 2);
        assert!(matches!(
            &transport.plans()[0],
            HttpRequestPlan::Resume { start_at: 3, .. }
        ));
        assert_eq!(transport.plans()[1], HttpRequestPlan::Full);
    }

    #[test]
    fn body_failure_never_advances_checkpoint_beyond_durable_bytes() {
        let root = TestRoot::new("body-failure");
        let mut store = root.store();
        let job = DownloadJob::new(
            id(),
            source(),
            root.destination(),
            None,
            RemoteValidators::default(),
        )
        .unwrap();
        let runtime = SingleStreamRuntime::new(3).unwrap();
        runtime.enqueue(&mut store, &job).unwrap();

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 200,
            content_range: None,
            content_length: Some(6),
            validators: validators(Some("\"v1\"")),
            body: ScriptedBody::FailAfter {
                bytes: b"abcdef".to_vec(),
                fail_after: 3,
            },
        }]);

        assert!(matches!(
            runtime.execute(&mut store, &id(), &transport),
            Err(RuntimeError::Transport(TransportErrorKind::Body))
        ));

        let stored = store.load(&id()).unwrap().unwrap();
        assert_eq!(stored.downloaded_bytes(), 3);
        assert_eq!(
            fs::metadata(stored.paths().staging_path()).unwrap().len(),
            3
        );
        assert_eq!(stored.state(), JobState::Downloading);
    }

    #[test]
    fn crash_after_promotion_is_recovered_from_verifying_checkpoint() {
        let root = TestRoot::new("promotion-recovery");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Verifying,
            6,
            Some(6),
            validators(Some("\"v1\"")),
        );
        fs::create_dir_all(root.destination().parent().unwrap()).unwrap();
        fs::write(root.destination(), b"abcdef").unwrap();
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(Vec::new());
        let runtime = SingleStreamRuntime::new(3).unwrap();
        let outcome = runtime.execute(&mut store, &id(), &transport).unwrap();
        assert_eq!(outcome.durable_bytes, 6);
        assert!(transport.plans().is_empty());

        let stored = store.load(&id()).unwrap().unwrap();
        assert_eq!(stored.state(), JobState::Completed);
    }

    #[test]
    fn changed_validator_on_resume_is_rejected_without_appending() {
        let root = TestRoot::new("validator-change");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(6),
            validators(Some("\"v1\"")),
        );
        write_staging(&checkpoint, b"abc");
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 3,
                end: 5,
                complete_length: Some(6),
            }),
            content_length: Some(3),
            validators: validators(Some("\"v2\"")),
            body: ScriptedBody::Bytes(b"def".to_vec()),
        }]);

        let runtime = SingleStreamRuntime::new(3).unwrap();
        assert!(matches!(
            runtime.execute(&mut store, &id(), &transport),
            Err(RuntimeError::Protocol(
                ProtocolError::RemoteValidatorChanged
            ))
        ));
        assert_eq!(fs::read(checkpoint.paths().staging_path()).unwrap(), b"abc");
        assert_eq!(store.load(&id()).unwrap().unwrap().downloaded_bytes(), 3);
    }

    #[test]
    fn resume_response_must_cover_the_remainder_of_the_object() {
        let root = TestRoot::new("partial-range");
        let mut store = root.store();
        let checkpoint = checkpoint(
            &root,
            JobState::Downloading,
            3,
            Some(9),
            validators(Some("\"v1\"")),
        );
        write_staging(&checkpoint, b"abc");
        seed(&mut store, &checkpoint);

        let transport = ScriptedTransport::new(vec![ScriptedResponse {
            status_code: 206,
            content_range: Some(ByteContentRange {
                start: 3,
                end: 5,
                complete_length: Some(9),
            }),
            content_length: Some(3),
            validators: validators(Some("\"v1\"")),
            body: ScriptedBody::Bytes(b"def".to_vec()),
        }]);

        let runtime = SingleStreamRuntime::new(3).unwrap();
        assert!(matches!(
            runtime.execute(&mut store, &id(), &transport),
            Err(RuntimeError::Protocol(ProtocolError::ResumeRangeNotToEnd))
        ));
    }
}

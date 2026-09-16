#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use goreecloud_download_core::{
    DownloadJob, FailureKind, JobId, JobState, RemoteValidators, SensitiveUrl,
};
use goreecloud_download_http::{HttpRequestPlan, plan_request};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    pub const CURRENT: Self = Self(CURRENT_SCHEMA_VERSION);

    pub fn new(value: u32) -> Result<Self, StateError> {
        if value == 0 {
            return Err(StateError::InvalidSchemaVersion);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u32 {
        self.0
    }

    pub fn compatibility(self) -> SchemaCompatibility {
        match self.0.cmp(&CURRENT_SCHEMA_VERSION) {
            Ordering::Equal => SchemaCompatibility::Current,
            Ordering::Less => SchemaCompatibility::UpgradeRequired {
                from: self.0,
                to: CURRENT_SCHEMA_VERSION,
            },
            Ordering::Greater => SchemaCompatibility::UnsupportedFuture {
                found: self.0,
                current: CURRENT_SCHEMA_VERSION,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaCompatibility {
    Current,
    UpgradeRequired { from: u32, to: u32 },
    UnsupportedFuture { found: u32, current: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialArtifactPaths {
    final_path: PathBuf,
    staging_path: PathBuf,
}

impl PartialArtifactPaths {
    pub fn for_job(final_path: PathBuf, job_id: &JobId) -> Result<Self, StateError> {
        if final_path.as_os_str().is_empty() {
            return Err(StateError::EmptyDestination);
        }

        let file_name = final_path
            .file_name()
            .ok_or(StateError::DestinationMissingFileName)?;
        let mut staging_name: OsString = file_name.to_os_string();
        staging_name.push(".gcdm-part.");
        staging_name.push(job_id.as_str());
        let staging_path = final_path.with_file_name(staging_name);

        Ok(Self {
            final_path,
            staging_path,
        })
    }

    pub fn final_path(&self) -> &PathBuf {
        &self.final_path
    }

    pub fn staging_path(&self) -> &PathBuf {
        &self.staging_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobCheckpointFieldsV1 {
    pub id: JobId,
    pub source: SensitiveUrl,
    pub final_path: PathBuf,
    pub staging_path: PathBuf,
    pub state: JobState,
    pub downloaded_bytes: u64,
    pub expected_bytes: Option<u64>,
    pub validators: RemoteValidators,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobCheckpointV1 {
    id: JobId,
    source: SensitiveUrl,
    paths: PartialArtifactPaths,
    state: JobState,
    downloaded_bytes: u64,
    expected_bytes: Option<u64>,
    validators: RemoteValidators,
}

impl JobCheckpointV1 {
    pub fn capture(job: &DownloadJob) -> Result<Self, StateError> {
        let paths = PartialArtifactPaths::for_job(job.destination().clone(), job.id())?;
        let fields = JobCheckpointFieldsV1 {
            id: job.id().clone(),
            source: job.source().clone(),
            final_path: paths.final_path().clone(),
            staging_path: paths.staging_path().clone(),
            state: job.state(),
            downloaded_bytes: job.downloaded_bytes(),
            expected_bytes: job.expected_bytes(),
            validators: job.validators().clone(),
        };
        Self::from_fields(fields)
    }

    pub fn from_fields(fields: JobCheckpointFieldsV1) -> Result<Self, StateError> {
        if fields.final_path.as_os_str().is_empty() {
            return Err(StateError::EmptyDestination);
        }

        if fields
            .expected_bytes
            .is_some_and(|expected| fields.downloaded_bytes > expected)
        {
            return Err(StateError::ProgressExceedsExpected);
        }

        if matches!(
            fields.state,
            JobState::Verifying | JobState::Processing | JobState::Completed
        ) && fields
            .expected_bytes
            .is_some_and(|expected| fields.downloaded_bytes != expected)
        {
            return Err(StateError::IncompleteAdvancedState);
        }

        let expected_paths = PartialArtifactPaths::for_job(fields.final_path.clone(), &fields.id)?;
        if fields.staging_path != *expected_paths.staging_path() {
            return Err(StateError::UnexpectedStagingPath);
        }

        Ok(Self {
            id: fields.id,
            source: fields.source,
            paths: expected_paths,
            state: fields.state,
            downloaded_bytes: fields.downloaded_bytes,
            expected_bytes: fields.expected_bytes,
            validators: fields.validators,
        })
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn source(&self) -> &SensitiveUrl {
        &self.source
    }

    pub fn paths(&self) -> &PartialArtifactPaths {
        &self.paths
    }

    pub fn state(&self) -> JobState {
        self.state
    }

    pub fn downloaded_bytes(&self) -> u64 {
        self.downloaded_bytes
    }

    pub fn expected_bytes(&self) -> Option<u64> {
        self.expected_bytes
    }

    pub fn validators(&self) -> &RemoteValidators {
        &self.validators
    }

    pub fn recovery_disposition(&self) -> RecoveryDisposition {
        match self.state {
            JobState::Queued
            | JobState::Downloading
            | JobState::Paused
            | JobState::Waiting(_) => {
                RecoveryDisposition::Transfer(plan_request(self.downloaded_bytes, &self.validators))
            }
            JobState::Verifying => RecoveryDisposition::ReverifyStaging,
            JobState::Processing => RecoveryDisposition::ReprocessStaging,
            JobState::Completed => RecoveryDisposition::VerifyCompletedArtifact,
            JobState::Failed(kind) => RecoveryDisposition::KeepFailed(kind),
            JobState::Cancelled => RecoveryDisposition::KeepCancelled,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryDisposition {
    Transfer(HttpRequestPlan),
    ReverifyStaging,
    ReprocessStaging,
    VerifyCompletedArtifact,
    KeepFailed(FailureKind),
    KeepCancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartialLengthDecision {
    Consistent,
    TruncateUncommittedTail { from: u64, to: u64 },
    RestartRequired {
        checkpoint_bytes: u64,
        actual_bytes: u64,
    },
}

pub fn reconcile_partial_length(
    checkpoint_bytes: u64,
    actual_bytes: u64,
) -> PartialLengthDecision {
    match actual_bytes.cmp(&checkpoint_bytes) {
        Ordering::Equal => PartialLengthDecision::Consistent,
        Ordering::Greater => PartialLengthDecision::TruncateUncommittedTail {
            from: actual_bytes,
            to: checkpoint_bytes,
        },
        Ordering::Less => PartialLengthDecision::RestartRequired {
            checkpoint_bytes,
            actual_bytes,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalArtifactDecision {
    AcceptCompleted,
    Missing,
    SizeMismatch { expected: u64, actual: u64 },
}

pub fn evaluate_final_artifact(
    expected_bytes: Option<u64>,
    actual_bytes: Option<u64>,
) -> FinalArtifactDecision {
    let Some(actual) = actual_bytes else {
        return FinalArtifactDecision::Missing;
    };

    match expected_bytes {
        Some(expected) if expected != actual => {
            FinalArtifactDecision::SizeMismatch { expected, actual }
        }
        Some(_) | None => FinalArtifactDecision::AcceptCompleted,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobMutationV1 {
    Upsert(JobCheckpointV1),
    Remove(JobId),
}

impl JobMutationV1 {
    pub fn job_id(&self) -> &JobId {
        match self {
            Self::Upsert(checkpoint) => checkpoint.id(),
            Self::Remove(id) => id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitBatchV1 {
    expected_generation: u64,
    next_generation: u64,
    mutations: Vec<JobMutationV1>,
}

impl CommitBatchV1 {
    pub fn new(
        expected_generation: u64,
        mutations: Vec<JobMutationV1>,
    ) -> Result<Self, StateError> {
        if mutations.is_empty() {
            return Err(StateError::EmptyMutationBatch);
        }

        for (index, mutation) in mutations.iter().enumerate() {
            if mutations[..index]
                .iter()
                .any(|existing| existing.job_id() == mutation.job_id())
            {
                return Err(StateError::DuplicateMutation);
            }
        }

        let next_generation = expected_generation
            .checked_add(1)
            .ok_or(StateError::GenerationOverflow)?;

        Ok(Self {
            expected_generation,
            next_generation,
            mutations,
        })
    }

    pub fn expected_generation(&self) -> u64 {
        self.expected_generation
    }

    pub fn next_generation(&self) -> u64 {
        self.next_generation
    }

    pub fn mutations(&self) -> &[JobMutationV1] {
        &self.mutations
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    InvalidSchemaVersion,
    EmptyDestination,
    DestinationMissingFileName,
    UnexpectedStagingPath,
    ProgressExceedsExpected,
    IncompleteAdvancedState,
    EmptyMutationBatch,
    DuplicateMutation,
    GenerationOverflow,
}

impl fmt::Display for StateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSchemaVersion => "schema version must be greater than zero",
            Self::EmptyDestination => "destination path must not be empty",
            Self::DestinationMissingFileName => "destination path must include a file name",
            Self::UnexpectedStagingPath => {
                "staging path does not match the deterministic per-job partial-file path"
            }
            Self::ProgressExceedsExpected => {
                "checkpoint progress exceeds the expected transfer length"
            }
            Self::IncompleteAdvancedState => {
                "verifying, processing, or completed state requires complete known-length progress"
            }
            Self::EmptyMutationBatch => "a durable-state commit batch must contain a mutation",
            Self::DuplicateMutation => {
                "a durable-state commit batch cannot mutate the same job more than once"
            }
            Self::GenerationOverflow => "durable-state generation counter overflowed",
        })
    }
}

impl Error for StateError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn job(expected_bytes: Option<u64>, validators: RemoteValidators) -> DownloadJob {
        DownloadJob::new(
            JobId::parse("job-001").expect("valid job id"),
            SensitiveUrl::parse("https://example.invalid/file.iso?token=secret")
                .expect("valid URL"),
            PathBuf::from("downloads/file.iso"),
            expected_bytes,
            validators,
        )
        .expect("valid job")
    }

    fn validators(etag: Option<&str>, last_modified: Option<&str>) -> RemoteValidators {
        RemoteValidators {
            etag: etag.map(str::to_owned),
            last_modified: last_modified.map(str::to_owned),
        }
    }

    #[test]
    fn schema_versions_fail_closed_for_future_state() {
        assert_eq!(
            SchemaVersion::CURRENT.compatibility(),
            SchemaCompatibility::Current
        );
        assert_eq!(
            SchemaVersion::new(2).unwrap().compatibility(),
            SchemaCompatibility::UnsupportedFuture {
                found: 2,
                current: 1,
            }
        );
    }

    #[test]
    fn deterministic_staging_path_is_distinct_from_final_path() {
        let id = JobId::parse("job-001").unwrap();
        let paths =
            PartialArtifactPaths::for_job(PathBuf::from("downloads/file.iso"), &id).unwrap();
        assert_eq!(paths.final_path(), &PathBuf::from("downloads/file.iso"));
        assert_eq!(
            paths.staging_path(),
            &PathBuf::from("downloads/file.iso.gcdm-part.job-001")
        );
        assert_ne!(paths.final_path(), paths.staging_path());
    }

    #[test]
    fn checkpoint_debug_output_keeps_sensitive_url_redacted() {
        let checkpoint = JobCheckpointV1::capture(&job(None, RemoteValidators::default())).unwrap();
        let debug = format!("{checkpoint:?}");
        assert!(debug.contains("SensitiveUrl([REDACTED])"));
        assert!(!debug.contains("token=secret"));
    }

    #[test]
    fn active_recovery_uses_validator_bound_resume_plan() {
        let mut job = job(None, validators(Some("\"v1\""), None));
        job.start().unwrap();
        job.record_progress(4096).unwrap();
        let checkpoint = JobCheckpointV1::capture(&job).unwrap();
        assert_eq!(
            checkpoint.recovery_disposition(),
            RecoveryDisposition::Transfer(HttpRequestPlan::Resume {
                start_at: 4096,
                if_range: goreecloud_download_http::IfRangeValidator::StrongEtag(
                    "\"v1\"".into()
                ),
            })
        );
    }

    #[test]
    fn active_recovery_restarts_when_only_weak_etag_exists() {
        let mut job = job(None, validators(Some("W/\"weak\""), None));
        job.start().unwrap();
        job.record_progress(512).unwrap();
        let checkpoint = JobCheckpointV1::capture(&job).unwrap();
        assert_eq!(
            checkpoint.recovery_disposition(),
            RecoveryDisposition::Transfer(HttpRequestPlan::RestartFull {
                discarded_partial_bytes: 512,
            })
        );
    }

    #[test]
    fn verifying_state_reenters_verification_after_restart() {
        let mut job = job(Some(10), RemoteValidators::default());
        job.start().unwrap();
        job.record_progress(10).unwrap();
        job.begin_verification().unwrap();
        let checkpoint = JobCheckpointV1::capture(&job).unwrap();
        assert_eq!(
            checkpoint.recovery_disposition(),
            RecoveryDisposition::ReverifyStaging
        );
    }

    #[test]
    fn uncommitted_partial_tail_is_truncated_to_checkpoint() {
        assert_eq!(
            reconcile_partial_length(4096, 4608),
            PartialLengthDecision::TruncateUncommittedTail {
                from: 4608,
                to: 4096,
            }
        );
    }

    #[test]
    fn checkpoint_ahead_of_partial_file_requires_restart() {
        assert_eq!(
            reconcile_partial_length(4096, 2048),
            PartialLengthDecision::RestartRequired {
                checkpoint_bytes: 4096,
                actual_bytes: 2048,
            }
        );
    }

    #[test]
    fn completed_artifact_requires_presence_and_known_length_match() {
        assert_eq!(
            evaluate_final_artifact(Some(10), Some(10)),
            FinalArtifactDecision::AcceptCompleted
        );
        assert_eq!(
            evaluate_final_artifact(Some(10), None),
            FinalArtifactDecision::Missing
        );
        assert_eq!(
            evaluate_final_artifact(Some(10), Some(9)),
            FinalArtifactDecision::SizeMismatch {
                expected: 10,
                actual: 9,
            }
        );
    }

    #[test]
    fn commit_batch_advances_generation_and_rejects_duplicate_jobs() {
        let checkpoint = JobCheckpointV1::capture(&job(None, RemoteValidators::default())).unwrap();
        let batch = CommitBatchV1::new(7, vec![JobMutationV1::Upsert(checkpoint.clone())]).unwrap();
        assert_eq!(batch.expected_generation(), 7);
        assert_eq!(batch.next_generation(), 8);

        assert_eq!(
            CommitBatchV1::new(
                7,
                vec![
                    JobMutationV1::Upsert(checkpoint.clone()),
                    JobMutationV1::Remove(checkpoint.id().clone()),
                ],
            ),
            Err(StateError::DuplicateMutation)
        );
        assert_eq!(
            CommitBatchV1::new(7, Vec::new()),
            Err(StateError::EmptyMutationBatch)
        );
    }

    #[test]
    fn restored_fields_must_use_the_deterministic_staging_path() {
        let fields = JobCheckpointFieldsV1 {
            id: JobId::parse("job-001").unwrap(),
            source: SensitiveUrl::parse("https://example.invalid/file").unwrap(),
            final_path: PathBuf::from("downloads/file.iso"),
            staging_path: PathBuf::from("downloads/arbitrary.partial"),
            state: JobState::Paused,
            downloaded_bytes: 1,
            expected_bytes: None,
            validators: RemoteValidators::default(),
        };
        assert_eq!(
            JobCheckpointV1::from_fields(fields),
            Err(StateError::UnexpectedStagingPath)
        );
    }
}

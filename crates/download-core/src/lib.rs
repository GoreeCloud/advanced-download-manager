#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, PartialEq, Eq)]
pub struct SensitiveUrl(String);

impl SensitiveUrl {
    pub fn parse(value: impl Into<String>) -> Result<Self, InputError> {
        let value = value.into();
        let lower = value.to_ascii_lowercase();
        if !(lower.starts_with("https://") || lower.starts_with("http://")) {
            return Err(InputError::UnsupportedUrlScheme);
        }
        if value.len() > 16 * 1024 {
            return Err(InputError::UrlTooLong);
        }
        Ok(Self(value))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SensitiveUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveUrl([REDACTED])")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JobId(String);

impl JobId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InputError> {
        let value = value.into();
        if value.is_empty() || value.len() > 128 {
            return Err(InputError::InvalidJobId);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(InputError::InvalidJobId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitReason {
    Network,
    Wifi,
    Vpn,
    Power,
    Storage,
    Schedule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    Network,
    RemoteServer,
    Authentication,
    Storage,
    Integrity,
    Policy,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Downloading,
    Paused,
    Waiting(WaitReason),
    Verifying,
    Processing,
    Completed,
    Failed(FailureKind),
    Cancelled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteValidators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl RemoteValidators {
    pub fn has_any(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }

    fn matches(&self, remote: &Self) -> bool {
        let etag_matches = self
            .etag
            .as_ref()
            .is_none_or(|etag| remote.etag.as_ref() == Some(etag));
        let last_modified_matches = self
            .last_modified
            .as_ref()
            .is_none_or(|value| remote.last_modified.as_ref() == Some(value));
        etag_matches && last_modified_matches
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeDecision {
    ResumeAllowed,
    RestartRequired,
    ValidatorUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadJob {
    id: JobId,
    source: SensitiveUrl,
    destination: PathBuf,
    state: JobState,
    downloaded_bytes: u64,
    expected_bytes: Option<u64>,
    validators: RemoteValidators,
}

impl DownloadJob {
    pub fn new(
        id: JobId,
        source: SensitiveUrl,
        destination: PathBuf,
        expected_bytes: Option<u64>,
        validators: RemoteValidators,
    ) -> Result<Self, InputError> {
        if destination.as_os_str().is_empty() {
            return Err(InputError::EmptyDestination);
        }
        Ok(Self {
            id,
            source,
            destination,
            state: JobState::Queued,
            downloaded_bytes: 0,
            expected_bytes,
            validators,
        })
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }

    pub fn source(&self) -> &SensitiveUrl {
        &self.source
    }

    pub fn destination(&self) -> &PathBuf {
        &self.destination
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

    pub fn start(&mut self) -> Result<(), TransitionError> {
        self.require_state(JobState::Queued)?;
        self.state = JobState::Downloading;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), TransitionError> {
        self.require_state(JobState::Downloading)?;
        self.state = JobState::Paused;
        Ok(())
    }

    pub fn wait(&mut self, reason: WaitReason) -> Result<(), TransitionError> {
        if !matches!(self.state, JobState::Queued | JobState::Downloading) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "wait",
            });
        }
        self.state = JobState::Waiting(reason);
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), TransitionError> {
        if !matches!(self.state, JobState::Paused | JobState::Waiting(_)) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "resume",
            });
        }
        self.state = JobState::Downloading;
        Ok(())
    }

    pub fn retry(&mut self) -> Result<(), TransitionError> {
        if !matches!(self.state, JobState::Failed(_)) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "retry",
            });
        }
        self.state = JobState::Queued;
        Ok(())
    }

    pub fn record_progress(&mut self, total_downloaded: u64) -> Result<(), TransitionError> {
        self.require_state(JobState::Downloading)?;
        if total_downloaded < self.downloaded_bytes {
            return Err(TransitionError::ProgressRegressed);
        }
        if self
            .expected_bytes
            .is_some_and(|expected| total_downloaded > expected)
        {
            return Err(TransitionError::ProgressExceedsExpected);
        }
        self.downloaded_bytes = total_downloaded;
        Ok(())
    }

    pub fn begin_verification(&mut self) -> Result<(), TransitionError> {
        self.require_state(JobState::Downloading)?;
        if self
            .expected_bytes
            .is_some_and(|expected| self.downloaded_bytes != expected)
        {
            return Err(TransitionError::TransferIncomplete);
        }
        self.state = JobState::Verifying;
        Ok(())
    }

    pub fn begin_processing(&mut self) -> Result<(), TransitionError> {
        self.require_state(JobState::Verifying)?;
        self.state = JobState::Processing;
        Ok(())
    }

    pub fn complete(&mut self, final_file_promoted: bool) -> Result<(), TransitionError> {
        if !matches!(self.state, JobState::Verifying | JobState::Processing) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "complete",
            });
        }
        if !final_file_promoted {
            return Err(TransitionError::FinalFileNotPromoted);
        }
        self.state = JobState::Completed;
        Ok(())
    }

    pub fn fail(&mut self, kind: FailureKind) -> Result<(), TransitionError> {
        if matches!(self.state, JobState::Completed | JobState::Cancelled) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "fail",
            });
        }
        self.state = JobState::Failed(kind);
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), TransitionError> {
        if matches!(self.state, JobState::Completed | JobState::Cancelled) {
            return Err(TransitionError::InvalidState {
                from: self.state,
                action: "cancel",
            });
        }
        self.state = JobState::Cancelled;
        Ok(())
    }

    pub fn evaluate_resume(&self, remote: &RemoteValidators) -> ResumeDecision {
        if !self.validators.has_any() {
            return ResumeDecision::ValidatorUnavailable;
        }
        if self.validators.matches(remote) {
            ResumeDecision::ResumeAllowed
        } else {
            ResumeDecision::RestartRequired
        }
    }

    fn require_state(&self, expected: JobState) -> Result<(), TransitionError> {
        if self.state == expected {
            Ok(())
        } else {
            Err(TransitionError::InvalidState {
                from: self.state,
                action: "state transition",
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    InvalidJobId,
    UnsupportedUrlScheme,
    UrlTooLong,
    EmptyDestination,
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidJobId => "job id is invalid",
            Self::UnsupportedUrlScheme => "only HTTP and HTTPS source URLs are accepted",
            Self::UrlTooLong => "source URL exceeds the supported length",
            Self::EmptyDestination => "destination path must not be empty",
        })
    }
}

impl Error for InputError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionError {
    InvalidState {
        from: JobState,
        action: &'static str,
    },
    ProgressRegressed,
    ProgressExceedsExpected,
    TransferIncomplete,
    FinalFileNotPromoted,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState { from, action } => {
                write!(formatter, "cannot {action} while job is in state {from:?}")
            }
            Self::ProgressRegressed => formatter.write_str("download progress cannot regress"),
            Self::ProgressExceedsExpected => {
                formatter.write_str("download progress exceeds expected file size")
            }
            Self::TransferIncomplete => {
                formatter.write_str("transfer is incomplete and cannot enter verification")
            }
            Self::FinalFileNotPromoted => formatter
                .write_str("a job cannot complete before the final file is safely promoted"),
        }
    }
}

impl Error for TransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn sensitive_url_debug_output_is_redacted() {
        let url = SensitiveUrl::parse("https://example.invalid/file?token=secret").unwrap();
        assert_eq!(format!("{url:?}"), "SensitiveUrl([REDACTED])");
        assert!(url.expose().contains("token=secret"));
    }

    #[test]
    fn only_http_and_https_sources_are_accepted() {
        assert!(SensitiveUrl::parse("HTTPS://example.invalid/file").is_ok());
        assert_eq!(
            SensitiveUrl::parse("ftp://example.invalid/file"),
            Err(InputError::UnsupportedUrlScheme)
        );
    }

    #[test]
    fn happy_path_requires_final_file_promotion() {
        let mut job = job(Some(10), RemoteValidators::default());
        job.start().unwrap();
        job.record_progress(10).unwrap();
        job.begin_verification().unwrap();
        assert_eq!(
            job.complete(false),
            Err(TransitionError::FinalFileNotPromoted)
        );
        assert_eq!(job.state(), JobState::Verifying);
        job.complete(true).unwrap();
        assert_eq!(job.state(), JobState::Completed);
    }

    #[test]
    fn incomplete_known_length_transfer_cannot_verify() {
        let mut job = job(Some(10), RemoteValidators::default());
        job.start().unwrap();
        job.record_progress(9).unwrap();
        assert_eq!(
            job.begin_verification(),
            Err(TransitionError::TransferIncomplete)
        );
        assert_eq!(job.state(), JobState::Downloading);
    }

    #[test]
    fn progress_cannot_regress_or_exceed_expected_size() {
        let mut job = job(Some(10), RemoteValidators::default());
        job.start().unwrap();
        job.record_progress(5).unwrap();
        assert_eq!(
            job.record_progress(4),
            Err(TransitionError::ProgressRegressed)
        );
        assert_eq!(
            job.record_progress(11),
            Err(TransitionError::ProgressExceedsExpected)
        );
    }

    #[test]
    fn changed_etag_requires_restart() {
        let job = job(
            None,
            RemoteValidators {
                etag: Some("old-etag".into()),
                last_modified: None,
            },
        );
        let remote = RemoteValidators {
            etag: Some("new-etag".into()),
            last_modified: None,
        };
        assert_eq!(
            job.evaluate_resume(&remote),
            ResumeDecision::RestartRequired
        );
    }

    #[test]
    fn matching_validator_allows_resume() {
        let validators = RemoteValidators {
            etag: Some("same-etag".into()),
            last_modified: None,
        };
        let job = job(None, validators.clone());
        assert_eq!(
            job.evaluate_resume(&validators),
            ResumeDecision::ResumeAllowed
        );
    }

    #[test]
    fn resume_without_validator_is_not_silently_approved() {
        let job = job(None, RemoteValidators::default());
        assert_eq!(
            job.evaluate_resume(&RemoteValidators::default()),
            ResumeDecision::ValidatorUnavailable
        );
    }

    #[test]
    fn waiting_job_can_resume() {
        let mut job = job(None, RemoteValidators::default());
        job.wait(WaitReason::Wifi).unwrap();
        assert_eq!(job.state(), JobState::Waiting(WaitReason::Wifi));
        job.resume().unwrap();
        assert_eq!(job.state(), JobState::Downloading);
    }
}

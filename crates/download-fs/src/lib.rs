#![forbid(unsafe_code)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use goreecloud_download_state::{
    PartialArtifactPaths, PartialLengthDecision, reconcile_partial_length,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagingOpenDisposition {
    Fresh,
    ResumedConsistent {
        durable_bytes: u64,
    },
    TruncatedUncommittedTail {
        from: u64,
        to: u64,
    },
    RestartedFromBeginning {
        checkpoint_bytes: u64,
        actual_bytes: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurableWrite {
    pub appended_bytes: u64,
    pub durable_bytes: u64,
}

#[derive(Debug)]
pub struct DurableStagingFile {
    paths: PartialArtifactPaths,
    file: File,
}

impl DurableStagingFile {
    pub fn open(
        paths: PartialArtifactPaths,
        checkpoint_bytes: u64,
    ) -> io::Result<(Self, StagingOpenDisposition)> {
        if paths.final_path().exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "final artifact already exists; recovery must reconcile it before transfer resumes",
            ));
        }

        let parent = paths.staging_path().parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "staging path has no parent")
        })?;
        fs::create_dir_all(parent)?;

        let staging_existed = paths.staging_path().exists();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(paths.staging_path())?;
        let actual_bytes = file.metadata()?.len();

        let disposition = match reconcile_partial_length(checkpoint_bytes, actual_bytes) {
            PartialLengthDecision::Consistent => {
                if staging_existed || checkpoint_bytes > 0 {
                    StagingOpenDisposition::ResumedConsistent {
                        durable_bytes: checkpoint_bytes,
                    }
                } else {
                    StagingOpenDisposition::Fresh
                }
            }
            PartialLengthDecision::TruncateUncommittedTail { from, to } => {
                file.set_len(to)?;
                file.sync_data()?;
                StagingOpenDisposition::TruncatedUncommittedTail { from, to }
            }
            PartialLengthDecision::RestartRequired {
                checkpoint_bytes,
                actual_bytes,
            } => {
                file.set_len(0)?;
                file.sync_data()?;
                StagingOpenDisposition::RestartedFromBeginning {
                    checkpoint_bytes,
                    actual_bytes,
                }
            }
        };

        file.seek(SeekFrom::End(0))?;
        Ok((Self { paths, file }, disposition))
    }

    pub fn paths(&self) -> &PartialArtifactPaths {
        &self.paths
    }

    pub fn durable_len(&self) -> io::Result<u64> {
        self.file.metadata().map(|metadata| metadata.len())
    }

    pub fn append_from<R: Read>(&mut self, reader: &mut R) -> io::Result<DurableWrite> {
        self.file.seek(SeekFrom::End(0))?;
        let appended_bytes = io::copy(reader, &mut self.file)?;
        self.file.flush()?;
        self.file.sync_data()?;
        let durable_bytes = self.durable_len()?;
        Ok(DurableWrite {
            appended_bytes,
            durable_bytes,
        })
    }

    pub fn truncate_to(&mut self, durable_bytes: u64) -> io::Result<()> {
        let current = self.durable_len()?;
        if durable_bytes > current {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot extend a partial artifact while reconciling durable progress",
            ));
        }
        self.file.set_len(durable_bytes)?;
        self.file.sync_data()?;
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }

    pub fn promote(mut self) -> io::Result<PathBuf> {
        self.file.flush()?;
        self.file.sync_all()?;

        let final_path = self.paths.final_path().clone();
        let staging_path = self.paths.staging_path().clone();
        if final_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "refusing to replace an existing final artifact",
            ));
        }

        drop(self.file);
        fs::rename(&staging_path, &final_path)?;
        sync_parent_directory(&final_path)?;
        Ok(final_path)
    }
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "final path has no parent"))?;
    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    // std does not expose a portable directory fsync primitive on every target.
    // The final file itself is synced before rename; platform-specific service layers
    // can add a stronger directory-durability primitive where their APIs permit it.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use goreecloud_download_core::JobId;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "goreecloud-download-fs-{label}-{}-{sequence}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create test root");
            Self(path)
        }

        fn paths(&self) -> PartialArtifactPaths {
            let id = JobId::parse("job-001").expect("valid job id");
            PartialArtifactPaths::for_job(self.0.join("nested/file.bin"), &id)
                .expect("valid artifact paths")
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn append_is_synced_before_durable_length_is_reported() {
        let root = TestRoot::new("append");
        let paths = root.paths();
        let (mut staging, disposition) =
            DurableStagingFile::open(paths.clone(), 0).expect("open staging file");
        assert_eq!(disposition, StagingOpenDisposition::Fresh);

        let write = staging
            .append_from(&mut Cursor::new(b"abcdef"))
            .expect("append durable bytes");
        assert_eq!(write.appended_bytes, 6);
        assert_eq!(write.durable_bytes, 6);
        assert_eq!(fs::read(paths.staging_path()).unwrap(), b"abcdef");
        assert!(!paths.final_path().exists());
    }

    #[test]
    fn uncommitted_tail_is_truncated_to_checkpoint_before_resume() {
        let root = TestRoot::new("tail");
        let paths = root.paths();
        fs::create_dir_all(paths.staging_path().parent().unwrap()).unwrap();
        fs::write(paths.staging_path(), b"abcdefgh").unwrap();

        let (staging, disposition) =
            DurableStagingFile::open(paths.clone(), 5).expect("reconcile staging file");
        assert_eq!(
            disposition,
            StagingOpenDisposition::TruncatedUncommittedTail { from: 8, to: 5 }
        );
        assert_eq!(staging.durable_len().unwrap(), 5);
        assert_eq!(fs::read(paths.staging_path()).unwrap(), b"abcde");
    }

    #[test]
    fn checkpoint_ahead_of_partial_restarts_from_zero() {
        let root = TestRoot::new("restart");
        let paths = root.paths();
        fs::create_dir_all(paths.staging_path().parent().unwrap()).unwrap();
        fs::write(paths.staging_path(), b"abc").unwrap();

        let (staging, disposition) =
            DurableStagingFile::open(paths.clone(), 5).expect("reconcile short partial");
        assert_eq!(
            disposition,
            StagingOpenDisposition::RestartedFromBeginning {
                checkpoint_bytes: 5,
                actual_bytes: 3,
            }
        );
        assert_eq!(staging.durable_len().unwrap(), 0);
        assert!(fs::read(paths.staging_path()).unwrap().is_empty());
    }

    #[test]
    fn final_promotion_moves_synced_staging_file_without_leaving_partial() {
        let root = TestRoot::new("promote");
        let paths = root.paths();
        let (mut staging, _) = DurableStagingFile::open(paths.clone(), 0).unwrap();
        staging
            .append_from(&mut Cursor::new(b"finished"))
            .expect("write staging file");

        let promoted = staging.promote().expect("promote final file");
        assert_eq!(promoted, *paths.final_path());
        assert_eq!(fs::read(paths.final_path()).unwrap(), b"finished");
        assert!(!paths.staging_path().exists());
    }

    #[test]
    fn existing_final_artifact_is_never_silently_replaced() {
        let root = TestRoot::new("existing-final");
        let paths = root.paths();
        fs::create_dir_all(paths.final_path().parent().unwrap()).unwrap();
        fs::write(paths.final_path(), b"existing").unwrap();

        let error = DurableStagingFile::open(paths.clone(), 0).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(paths.final_path()).unwrap(), b"existing");
    }

    #[test]
    fn truncate_cannot_extend_partial_artifact() {
        let root = TestRoot::new("truncate");
        let paths = root.paths();
        let (mut staging, _) = DurableStagingFile::open(paths, 0).unwrap();
        staging.append_from(&mut Cursor::new(b"abc")).unwrap();

        let error = staging.truncate_to(4).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(staging.durable_len().unwrap(), 3);
    }
}

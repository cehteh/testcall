use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Trait for test directoy objects
pub trait TestDir {
    /// Returns the underlying Path of an TestDir implementation
    fn path(&self) -> &Path;
}

impl TestDir for Path {
    fn path(&self) -> &Path {
        self
    }
}

impl TestDir for PathBuf {
    fn path(&self) -> &Path {
        self.as_path()
    }
}

impl TestDir for TempDir {
    fn path(&self) -> &Path {
        self.path()
    }
}

/// Augment a TempDir with a custom callback function that can do additional cleanup work
/// (like unmounting filesystem etc.)
pub struct TempDirCleanup {
    dir: TempDir,
    cleanup_fn: fn(&TempDir),
}

impl Drop for TempDirCleanup {
    fn drop(&mut self) {
        (self.cleanup_fn)(&self.dir);
    }
}

impl TestDir for TempDirCleanup {
    fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl TempDirCleanup {
    /// creates a temporary directory with a cleanup function to be called at drop time.
    pub fn new(cleanup_fn: fn(&TempDir)) -> io::Result<Self> {
        Ok(TempDirCleanup {
            dir: TempDir::new()?,
            cleanup_fn,
        })
    }
}

#[cfg(test)]
#[cfg(unix)]
mod test {
    use crate::*;
    use std::path::PathBuf;

    #[test]
    fn dircleanup() {
        let cleaned_up = {
            let tmpdir =
                TempDirCleanup::new(|_| println!("TempDir cleaned up")).expect("TempDir created");
            println!("TempDir path: {:?}", tmpdir.path());
            PathBuf::from(tmpdir.path())
        };

        assert!(
            !std::path::Path::new(&cleaned_up).exists(),
            "TempDir got deleted"
        );
    }
}

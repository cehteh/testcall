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

// normalize paths in rust including components that do not exist yet
trait PathNormalize {
    fn normalize(&self) -> PathBuf;
}

impl PathNormalize for Path {
    fn normalize(&self) -> PathBuf {
        use std::path::Component::*;
        let mut normalized = PathBuf::new();
        for component in self.components() {
            match component {
                Prefix(_) | RootDir | Normal(_) => normalized.push(component),
                CurDir => (),
                ParentDir => {
                    if let Some(_) = normalized.parent() {
                        normalized.pop();
                    }
                }
            }
            normalized = normalized.canonicalize().unwrap_or(normalized);
        }
        normalized
    }
}

// concatenate & normalize path and assert that it doesn't escape the inital directory
fn assert_path<N>(testdir: &Path, subcomponents: &N) -> PathBuf
where
    N: AsRef<Path> + ?Sized,
{
    let testdir = testdir.canonicalize().expect("absolute existing path");
    let mut fullpath = PathBuf::from(&testdir);
    fullpath.push(subcomponents);
    let path = fullpath.normalize();
    assert!(path.starts_with(testdir), "escaped into parent dir");
    path
}

// + check that it already exists
fn path_exists<N>(testdir: &Path, subcomponents: &N) -> PathBuf
where
    N: AsRef<Path> + ?Sized,
{
    let path = assert_path(testdir, subcomponents);
    assert!(path.exists(), "path exists");
    path
}

// + check that it does not exist
fn path_available<N>(testdir: &Path, subcomponents: &N) -> PathBuf
where
    N: AsRef<Path> + ?Sized,
{
    let path = assert_path(testdir, subcomponents);
    assert!(!path.exists(), "path available");
    path
}


#[cfg(test)]
#[cfg(unix)]
mod test {
    use super::PathNormalize;
    use crate::*;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn dircleanup() {
        let cleaned_up = {
            let tmpdir =
                TempDirCleanup::new(|_| println!("TempDir cleaned up")).expect("TempDir created");
            println!("TempDir path: {:?}", tmpdir.path());
            PathBuf::from(tmpdir.path())
        };

        assert!(!Path::new(&cleaned_up).exists(), "TempDir got deleted");
    }

    #[test]
    fn create_file() {
        let tmpdir = TempDir::new().expect("TempDir created");
        println!("TempDir path: {:?}", tmpdir.path());
        tmpdir.create_file("path/to/testfile", "Hello File!".as_bytes());

        tmpdir.assert_exists("path/to/testfile");
    }

    #[test]
    fn path_normalize() {
        assert_eq!(Path::new("/foo/bar"), Path::new("/foo/bar").normalize());
        assert_eq!(Path::new("/foo"), Path::new("/foo/bar/..").normalize());
        assert_eq!(Path::new("/foo/bar"), Path::new("/foo/./bar/.").normalize());
        assert_ne!(Path::new("/foo/bar"), Path::new("/foo/bar/..").normalize());
        assert_eq!(Path::new("foo/bar"), Path::new("./foo/bar").normalize());
    }
}

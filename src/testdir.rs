use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Trait for test directoy objects
pub trait TestDir {
    /// Returns the underlying Path of an TestDir implementation
    fn path(&self) -> &Path;
}

/// Trait for test directoy objects
pub trait Fixtures: TestDir {
    /// Create a file with the given content in the test directory. Any leading directories
    /// are created automatically. The file itself must not already exist.
    #[track_caller]
    fn create_file<N>(&self, name: &N, content: &[u8]) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_available(self.path(), name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create directory");
        }

        fs::write(path, content).expect("create file");

        self
    }

    /// Create a directory within the test directory. Any leading directories
    /// are created automatically. The path must not exist already.
    #[track_caller]
    fn create_dir<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_available(self.path(), name);
        fs::create_dir_all(path).expect("create directory");
        self
    }

}

/// Assertions on content of a TestDir
pub trait Assertions: TestDir {
    /// Assert that at the given path exists
    #[track_caller]
    fn assert_exists<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        path_exists(self.path(), name);
        self
    }

    /// Assert that the given path does not exist
    #[track_caller]
    fn assert_available<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        path_available(self.path(), name);
        self
    }

    /// Assert that the given path is a directory
    #[track_caller]
    fn assert_is_dir<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert!(path.is_dir());
        self
    }

    /// Assert that the given path is a file
    #[track_caller]
    fn assert_is_file<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert!(path.is_file());
        self
    }

    /// Assert that the given path is a symlink
    #[track_caller]
    fn assert_is_symlink<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert!(path.symlink_metadata().unwrap().file_type().is_symlink());
        self
    }

    /// Assert that the given path resolves to a element of the given size
    #[track_caller]
    fn assert_size<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert_eq!(path.metadata().unwrap().len(), size);
        self
    }

    /// Assert that the given path resolves to a element of more than the given size
    #[track_caller]
    fn assert_size_greater<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert!(path.metadata().unwrap().len() > size);
        self
    }

    /// Assert that the given path resolves to a element of less than the given size
    #[track_caller]
    fn assert_size_smaller<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = path_exists(self.path(), name);
        assert!(path.metadata().unwrap().len() < size);
        self
    }
}

impl TestDir for Path {
    fn path(&self) -> &Path {
        self
    }
}

impl Fixtures for Path {}
impl Assertions for Path {}

impl TestDir for PathBuf {
    fn path(&self) -> &Path {
        self.as_path()
    }
}

impl Fixtures for PathBuf {}
impl Assertions for PathBuf {}

impl TestDir for TempDir {
    fn path(&self) -> &Path {
        self.path()
    }
}

impl Fixtures for TempDir {
    //TODO: implement rm
}
impl Assertions for TempDir {}

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

impl Fixtures for TempDirCleanup {
    //TODO: implement rm
}
impl Assertions for TempDirCleanup {}

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
    #[should_panic]
    fn create_file_fail() {
        let tmpdir = TempDir::new().expect("TempDir created");
        println!("TempDir path: {:?}", tmpdir.path());
        tmpdir.create_file("path/to/testfile", "Hello File!".as_bytes());
        tmpdir.assert_exists("path/to/wrongfile");
    }

    #[test]
    #[should_panic]
    fn create_file_again_fails() {
        let tmpdir = TempDir::new().expect("TempDir created");
        println!("TempDir path: {:?}", tmpdir.path());
        tmpdir.create_file("path/to/testfile", "Hello File!".as_bytes());
        tmpdir.create_file("path/to/testfile", "Hello File again!".as_bytes());
    }

    #[test]
    fn create_is_something() {
        let tmpdir = TempDir::new().expect("TempDir created");
        println!("TempDir path: {:?}", tmpdir.path());
        tmpdir.create_file("path/to/testfile", "Hello File!".as_bytes());
        tmpdir
            .assert_exists("path/to/testfile")
            .assert_is_file("path/to/testfile")
            .assert_is_dir("path/to");
    }

    #[test]
    fn create_dir() {
        let tmpdir = TempDir::new().expect("TempDir created");
        println!("TempDir path: {:?}", tmpdir.path());
        tmpdir.create_dir("path/to/test/dir");
        tmpdir.assert_is_dir("path/to/test/dir");
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

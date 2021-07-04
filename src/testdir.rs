use std::ffi::OsStr;
use std::fs;
use std::io;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use crate::CaptureKey;

/// Trait for test directoy objects
pub trait TestDir {
    /// Return the underlying Path of an TestDir implementation
    fn path(&self) -> &Path;

    /// Return a canonalized/normalized PathBuf to components within the testdir. Assert and
    /// panic when path escapes from the testdir. Handles non existing components.
    #[track_caller]
    fn sub_path(&self, subcomponents: &Path) -> PathBuf {
        let testdir = self.path();
        let mut fullpath = PathBuf::from(&testdir);
        fullpath.push(subcomponents);
        let path = fullpath.normalize();
        assert!(path.starts_with(testdir), "escaped from testdir");
        path
    }

    /// Return a canonalized/normalized PathBuf to components within the testdir. Assert and
    /// panic when path escapes from the testdir. Asserts that the given subpath exists.
    #[track_caller]
    fn sub_path_exists(&self, subcomponents: &Path) -> PathBuf {
        let path = self.sub_path(subcomponents);
        assert!(path.exists(), "path exists");
        path
    }

    /// Return a canonalized/normalized PathBuf to components within the testdir. Assert and
    /// panic when path escapes from the testdir. Asserts that the given subpath does not exist.
    #[track_caller]
    fn sub_path_available(&self, subcomponents: &Path) -> PathBuf {
        let path = self.sub_path(subcomponents);
        assert!(!path.exists(), "path does not exist");
        path
    }
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
        let path = self.sub_path_available(name.as_ref());

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
        let path = self.sub_path_available(name.as_ref());
        fs::create_dir_all(path).expect("create directory");
        self
    }

    /// Install something (from outside) into the testdir.
    /// * When 'from' is a directory then all its contents are recursively copied
    ///   * When 'to' does not exist then the last component of 'from' is created there,
    ///     any leading dirs are created
    ///   * When 'to' exists and is a directory then the contents of 'from/*' are copied
    ///   * When 'to' exists and is a file
    /// * When 'from' is a file
    ///   * When 'to' does not exist any leading dirs are created, with the last component
    ///     being its new filename, if 'to' is empty then use the original filename
    ///   * When 'to' exists and is a directory then 'from' is copied into that.
    ///   * When 'to' exists and is a file it is overwritten with 'from'.
    #[track_caller]
    fn install<N, M>(&self, from: &N, to: &M) -> &Self
    where
        N: AsRef<Path> + ?Sized,
        M: AsRef<Path> + ?Sized,
    {
        let from = from.as_ref();
        assert!(from.exists());

        self
    }

    #[track_caller]
    fn symlink<N, M>(&self, from: &N, to: &M) -> &Self
    where
        N: AsRef<Path> + ?Sized,
        M: AsRef<Path> + ?Sized,
    {
        let from = from.as_ref();
        assert!(from.exists());
        todo!();
        self
    }

    #[track_caller]
    fn hardlink<N, M>(&self, from: &N, to: &M) -> &Self
    where
        N: AsRef<Path> + ?Sized,
        M: AsRef<Path> + ?Sized,
    {
        let from = from.as_ref();
        assert!(from.exists());
        todo!();
        self
    }

    /// Delete an element from a testdir. Directories are deleted as well.  This trait
    /// functions defaults to unimplemented!() because it is deemed to be dangerous. Only the
    /// trait implementations which create an disposable directory implement it.
    #[track_caller]
    fn delete<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        unimplemented!()
    }
}

/// Assertions on content of a TestDir
pub trait DirAssertions: TestDir {
    /// Assert that at the given path exists
    #[track_caller]
    fn assert_exists<N>(&self, subpath: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path(subpath.as_ref());
        assert!(path.exists(), "path exists");
        self
    }

    /// Assert that the given path does not exist
    #[track_caller]
    fn assert_available<N>(&self, subpath: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path(subpath.as_ref());
        assert!(!path.exists(), "path does not exist");
        self
    }

    /// Assert that the given path is a directory
    #[track_caller]
    fn assert_is_dir<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert!(path.is_dir());
        self
    }

    /// Assert that the given path is a file
    #[track_caller]
    fn assert_is_file<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert!(path.is_file());
        self
    }

    /// Assert that the given path is a symlink
    #[track_caller]
    fn assert_is_symlink<N>(&self, name: &N) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert!(path.symlink_metadata().unwrap().file_type().is_symlink());
        self
    }

    /// Assert that the given path resolves to a element of the given size
    #[track_caller]
    fn assert_size<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert_eq!(path.metadata().unwrap().len(), size);
        self
    }

    /// Assert that the given path resolves to a element of more than the given size
    #[track_caller]
    fn assert_size_greater<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert!(path.metadata().unwrap().len() > size);
        self
    }

    /// Assert that the given path resolves to a element of less than the given size
    #[track_caller]
    fn assert_size_smaller<N>(&self, name: &N, size: u64) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        let path = self.sub_path_exists(name.as_ref());
        assert!(path.metadata().unwrap().len() < size);
        self
    }

    /// Assert that the two components contain exactly the same things (directories are
    /// recursed).
    #[track_caller]
    fn assert_equal<N, M>(&self, from: &N, to: &M) -> &Self
    where
        N: AsRef<Path> + ?Sized,
        M: AsRef<Path> + ?Sized,
    {
        todo!();
        self
    }

    /// Assert that the two components contain the same things (directories are
    /// recursed) for any existing component on either side.
    #[track_caller]
    fn assert_equal_exists<N, M>(&self, from: &N, to: &M) -> &Self
    where
        N: AsRef<Path> + ?Sized,
        M: AsRef<Path> + ?Sized,
    {
        todo!();
        self
    }

    /// Assert that a file content matches the given regex in utf8.
    #[track_caller]
    fn assert_utf8<N>(&self, name: &N, regex: &str) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        todo!();
        self
    }

    /// Assert that a file content matches the given regex in bytes.
    #[track_caller]
    fn assert_bytes<N>(&self, name: &N, regex: &str) -> &Self
    where
        N: AsRef<Path> + ?Sized,
    {
        todo!();
        self
    }

    /// Return all captures from a regex in utf8.
    #[track_caller]
    fn captures_utf8<N>(&self, name: &N, regex: &str) -> HashMap<CaptureKey, String>
    where
        N: AsRef<Path> + ?Sized,
    {
        todo!()
    }
}

impl TestDir for Path {
    fn path(&self) -> &Path {
        self
    }
}

impl Fixtures for Path {}
impl DirAssertions for Path {}

impl TestDir for PathBuf {
    fn path(&self) -> &Path {
        self.as_path()
    }
}

impl Fixtures for PathBuf {}
impl DirAssertions for PathBuf {}

impl TestDir for TempDir {
    fn path(&self) -> &Path {
        self.path()
    }
}

impl Fixtures for TempDir {
    //TODO: implement rm
}
impl DirAssertions for TempDir {}

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
impl DirAssertions for TempDirCleanup {}

impl TempDirCleanup {
    /// creates a temporary directory with a cleanup function to be called at drop time.
    //TODO: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
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

#[cfg(test)]
#[cfg(unix)]
mod test {
    use super::PathNormalize;
    use crate::*;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn path_normalize() {
        assert_eq!(Path::new("/foo/bar"), Path::new("/foo/bar").normalize());
        assert_eq!(Path::new("/foo"), Path::new("/foo/bar/..").normalize());
        assert_eq!(Path::new("/foo/bar"), Path::new("/foo/./bar/.").normalize());
        assert_ne!(Path::new("/foo/bar"), Path::new("/foo/bar/..").normalize());
        assert_eq!(Path::new("foo/bar"), Path::new("./foo/bar").normalize());
    }

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
    fn install_from_dir_to_none() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.install("src", "");
        tmpdir.assert_equal("src", "src");
    }

    #[test]
    fn install_from_dir_to_some() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.install("src", "into/this/dir");
        tmpdir.assert_equal("src", "into/this/dir/src");
    }

    #[test]
    fn install_from_dir_to_dir() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_dir("other");
        tmpdir.install("src", "other");
        tmpdir.assert_equal("src", "other");
    }

    #[test]
    #[should_panic]
    fn install_from_dir_to_file() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("src", "Hello File!".as_bytes());
        tmpdir.install("src", "src");
    }

    #[test]
    fn install_from_file_to_none() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.install("Cargo.toml", "");
        tmpdir.assert_equal("Cargo.toml", "Cargo.toml");
    }

    fn install_from_file_to_nodir() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.install("Cargo.toml", "test.toml");
        tmpdir.assert_equal("Cargo.toml", "test.toml");
    }

    #[test]
    fn install_from_file_to_some() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.install("Cargo.toml", "other/dir/Cargo.toml");
        tmpdir.assert_equal("Cargo.toml", "other/dir/Cargo.toml");
    }

    #[test]
    fn install_from_file_to_dir() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_dir("other");
        tmpdir.install("Cargo.toml", "other");
        tmpdir.assert_equal("Cargo.toml", "other/Cargo.toml");
    }

    #[test]
    fn install_from_file_to_file() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("Cargo.toml", "Hello File!".as_bytes());
        tmpdir.install("Cargo.toml", "Cargo.toml");
        tmpdir.assert_equal("Cargo.toml", "Cargo.toml");
    }

    #[test]
    fn hardlink() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        tmpdir.hardlink("testfile", "testfile");
        tmpdir.assert_equal("testfile", "testfile");
    }

    #[test]
    fn delete_in_tempdir() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        tmpdir.delete("testfile");
        tmpdir.assert_available("testfile");
    }

    #[test]
    #[should_panic]
    fn delete_in_path() {
        let underlay = TempDir::new().expect("TempDir created");
        let tmpdir = Path::new(underlay.path());
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        tmpdir.delete("testfile");
    }

    #[test]
    fn assert_utf8() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        tmpdir.assert_utf8("testfile", "Hello File!");
    }

    #[test]
    fn assert_bytes() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        tmpdir.assert_bytes("testfile", "Hello File!");
    }

    #[test]
    fn captures_utf8() {
        let tmpdir = TempDir::new().expect("TempDir created");
        tmpdir.create_file("testfile", "Hello File!".as_bytes());
        let captures = tmpdir.captures_utf8("testfile", "(?P<first>[^ ]*) (?P<second>[^ ]*)");

        use CaptureKey::*;

        assert_eq!(captures[&Index(0)], "Hello File!");
        assert_eq!(captures[&Index(1)], "Hello");
        assert_eq!(captures[&Index(2)], "File!");
        assert_eq!(captures[&Name(String::from("first"))], "Hello");
        assert_eq!(captures[&Name(String::from("second"))], "File!");
    }
}

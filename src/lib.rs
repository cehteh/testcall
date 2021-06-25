//! Companinon crate to 'BinTest', implements test facilities
//!
//! # Description
//!
//! A TestCall uses BinTest and std::process::Command to wrap process execution in a way that
//! is ergonomic to use for (repeated) testing. Few more test facilities are provided and will
//! grow in future.
//!
//! # Example
//!
//! ```rust
//! #[test]
//! fn myprogram_test() {
//!     let executables = BinTest::new();
//!     let mut myprogram = TestCall::new(&executables, "myprogram");
//!
//!     myprogram.current_dir(Box::new(TempDir::new().expect("created tempdir")));
//!     myprogram
//!         .call(["--help"])
//!         .assert_success();
//! }
//! ```
//!
//! # Future Plans
//!
//! New features will be added as needed, PR's are welcome. This is work in progress.
//!
//! Things to be done soon are:
//!  * Regex filters for the stdout/stderr
//!  * Populating TestDirs from template directories
//!  * Validating directory contents
//!

use regex::Regex;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

use bintest::BinTest;

/// exporting things for convinience
pub mod prelude {
    pub use crate::TestCall;
    pub use crate::TestDir;
    pub use crate::TestOutput;
}

/// A TestCall object binds a BinTest::Command to a single executable and environment and
/// provides functions to call this multiple times.
pub struct TestCall<'a> {
    executables: &'a BinTest,
    name: &'static str,
    dir: Option<Box<dyn TestDir>>,
    //PLANNED env: env_clear: env_remove...,
}

impl<'a> TestCall<'a> {
    /// Creates a new testcall object for the executable 'name'
    pub fn new(executables: &'a BinTest, name: &'static str) -> TestCall<'a> {
        TestCall {
            executables,
            name,
            dir: None,
        }
    }

    /// Sets the current dir in which the next call shall execute
    pub fn current_dir(&mut self, dir: Box<dyn TestDir>) -> &mut Self {
        self.dir = Some(dir);
        self
    }

    /// Calls the executable with the given arguments and expects successful exit.
    /// Returns a TestOutput object for further investigation.
    #[track_caller]
    pub fn call<I, S>(&self, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = self.executables.command(self.name);
        if let Some(dir) = &self.dir {
            command.current_dir(dir.path());
        }
        //PLANNED: env vars
        let output = command.args(args).output().expect("called command");
        output
    }
}

/// Augment std::process::Output with testing and assertions
pub trait TestOutput {
    /// Will panic when the program did not exited successful.
    #[track_caller]
    fn assert_success(&self) -> &Self;

    /// Expects that the program exited with a failure.
    #[track_caller]
    fn assert_failure(&self) -> &Self;

    /// Expects that the program exited with the provided code.
    #[track_caller]
    fn assert_exitcode(&self, code: i32) -> &Self;

    /// Applies a regex match check to stdout, will panic when the match failed.
    #[track_caller]
    fn assert_stdout(&self, regex: &str) -> &Self;

    /// Applies a regex match check to stderr, will panic when the match failed.
    #[track_caller]
    fn assert_stderr(&self, regex: &str) -> &Self;
}

impl TestOutput for Output {
    fn assert_success(&self) -> &Self {
        assert!(self.status.success(), "expected success at exit");
        self
    }

    fn assert_failure(&self) -> &Self {
        assert!(!self.status.success(), "expected failure at exit");
        self
    }

    fn assert_exitcode(&self, code: i32) -> &Self {
        assert_eq!(self.status.code(), Some(code), "unexpected exitcode");
        self
    }

    fn assert_stdout(&self, regex: &str) -> &Self {
        let re = Regex::new(regex).expect("compiled regex");
        let text = String::from_utf8_lossy(&self.stdout);
        assert!(
            re.is_match(&text),
            "stdout does not match:\n{}\nstdout was:\n{}",
            regex,
            text
        );
        self
    }

    fn assert_stderr(&self, regex: &str) -> &Self {
        let re = Regex::new(regex).expect("compiled regex");
        let text = String::from_utf8_lossy(&self.stderr);
        assert!(
            re.is_match(&text),
            "stderr does not match:\n{}\nstdout was:\n{}",
            regex,
            text
        );
        self
    }
}

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
/// (like unmountinf filesystem etc.)
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

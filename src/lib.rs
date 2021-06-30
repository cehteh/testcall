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
//!         .call(["--version"])
//!         .assert_success()
//!         .assert_stdout_utf8("myprogram 0.1.*");
//! }
//! ```
//!
//! # Future Plans
//!
//! New features will be added as needed, PR's are welcome. This is work in progress.
//!
//! Things to be done soon are:
//!  * Populating TestDirs from template directories
//!  * Validating directory contents
//!

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

enum ExeLocation<'a> {
    BinTest {
        executables: &'a BinTest,
        name: &'static str,
    },
    External(&'static Path),
}

/// A TestCall object binds a BinTest::Command to a single executable and environment and
/// provides functions to call this multiple times.
pub struct TestCall<'a> {
    executable: ExeLocation<'a>,
    dir: Option<Box<dyn TestDir>>,
    //PLANNED env: env_clear: env_remove...,
}

impl<'a> TestCall<'a> {
    /// Creates a new testcall object for 'name' from the current crates executables.
    pub fn new(executables: &'a BinTest, name: &'static str) -> TestCall<'a> {
        TestCall {
            executable: ExeLocation::BinTest { executables, name },
            dir: None,
        }
    }

    /// Creates a new testcall object for an external command given by path.
    pub fn external_command(path: &'static Path) -> TestCall<'a> {
        TestCall {
            executable: ExeLocation::External(path),
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
        let mut command = match self.executable {
            ExeLocation::BinTest { executables, name } => executables.command(name),
            ExeLocation::External(path) => Command::new(path),
        };
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
    /// This check matches utf8 text, stdout is lossy convered to utf8 first.
    #[track_caller]
    fn assert_stdout_utf8(&self, regex: &str) -> &Self;

    /// Applies a regex match check to stderr, will panic when the match failed.
    /// This check matches utf8 text, stdout is lossy convered to utf8 first.
    #[track_caller]
    fn assert_stderr_utf8 (&self, regex: &str) -> &Self;

    /// Applies a regex match check to stdout, will panic when the match failed.
    /// This check uses the 'bytes' module from the regex package and matches bytes.
    #[track_caller]
    fn assert_stdout_bytes(&self, regex: &str) -> &Self;

    /// Applies a regex match check to stderr, will panic when the match failed.
    /// This check uses the 'bytes' module from the regex package and matches bytes.
    #[track_caller]
    fn assert_stderr_bytes(&self, regex: &str) -> &Self;
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

    fn assert_stdout_utf8(&self, regex: &str) -> &Self {
        use regex::Regex;
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

    fn assert_stderr_utf8(&self, regex: &str) -> &Self {
        use regex::Regex;
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

    fn assert_stdout_bytes(&self, regex: &str) -> &Self {
        use regex::bytes::Regex;
        let re = Regex::new(regex).expect("compiled regex");
        assert!(
            re.is_match(&self.stdout),
            "stdout does not match:\n{}\nstdout was:\n{}",
            regex,
            String::from_utf8_lossy(&self.stderr),
        );
        self
    }

    fn assert_stderr_bytes(&self, regex: &str) -> &Self {
        use regex::bytes::Regex;
        let re = Regex::new(regex).expect("compiled regex");
        assert!(
            re.is_match(&self.stderr),
            "stdout does not match:\n{}\nstdout was:\n{}",
            regex,
            String::from_utf8_lossy(&self.stderr),
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

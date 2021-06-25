use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

use bintest::BinTest;

// A TestCall object binds a BinTest::Command to a single executable and environment and
// provides functions to call this multiple times
struct TestCall<'a> {
    executables: &'a BinTest,
    name: &'static str,
    dir: Option<Box<dyn TestDir>>,
    //PLANNED env: env_clear: env_remove...,
}

impl<'a> TestCall<'a> {
    // Creates a new testcall object for the executable 'name'
    pub fn new(executables: &'a BinTest, name: &'static str) -> TestCall<'a> {
        TestCall {
            executables,
            name,
            dir: None,
        }
    }

    // Sets the current dir in which the next call shall execute
    pub fn current_dir(&mut self, dir: Box<dyn TestDir>) {
        self.dir = Some(dir);
    }

    // Calls the executable with the given arguments and expects successful exit.
    // Returns an Output object for further investigation.
    #[track_caller]
    pub fn call<I, S>(&self, args: I) -> TestOutput
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
        TestOutput(output)
    }
}

// Wraps std::process::Output
struct TestOutput(Output);

//TODO: make this a trait
impl TestOutput {
    #[track_caller]
    pub fn assert_success(&self) -> &TestOutput {
        assert!(self.0.status.success(), "expected successful exit status");
        self
    }

    #[track_caller]
    pub fn assert_failure(&self) -> &TestOutput {
        assert!(self.0.status.success() == false, "expected failure at exit");
        self
    }
}

trait TestDir {
    fn path(&self) -> &Path;
}

impl TestDir for TempDir {
    fn path(&self) -> &Path {
        self.path()
    }
}

struct TempDirCleanup {
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
    pub fn new(cleanup_fn: fn(&TempDir)) -> io::Result<Self> {
        Ok(TempDirCleanup {
            dir: TempDir::new()?,
            cleanup_fn,
        })
    }
}

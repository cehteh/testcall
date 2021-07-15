use bintest::BinTest;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    dir: Option<PathBuf>, //TODO: should be a reference with lifetime to a TestPath
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
    pub fn current_dir(&mut self, dir: &Path) -> &mut Self {
        self.dir = Some(PathBuf::from(dir));
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
            command.current_dir(dir);
        }
        //PLANNED: env vars
        let output = command.args(args).output().expect("called command");
        output
    }
}

#[cfg(test)]
#[cfg(unix)]
mod test {
    use crate::*;
    use std::path::Path;

    #[test]
    fn echo() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call(["Hello World!"])
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }

    #[test]
    #[should_panic]
    fn echo_fail() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call(["No World!"])
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }
}

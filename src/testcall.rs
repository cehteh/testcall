use bintest::BinTest;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};
use testpath::TestPath;

enum ExeLocation<'a> {
    BinTest {
        executables: &'a BinTest,
        name: &'a str,
    },
    External(&'a Path),
}

/// A TestCall object binds a BinTest::Command to a single executable and environment and
/// provides functions to call this multiple times.
pub struct TestCall<'a> {
    executable: ExeLocation<'a>,
    dir: Option<&'a dyn TestPath>,
    //PLANNED env: env_clear: env_remove...,
}

impl<'a> TestCall<'a> {
    /// Creates a new testcall object for 'name' from the current crates executables.
    pub fn new(executables: &'a BinTest, name: &'a str) -> TestCall<'a> {
        TestCall {
            executable: ExeLocation::BinTest { executables, name },
            dir: None,
        }
    }

    /// Creates a new testcall object for an external command given by path.
    pub fn external_command(path: &'a Path) -> TestCall<'a> {
        TestCall {
            executable: ExeLocation::External(path),
            dir: None,
        }
    }

    /// Sets the current dir in which the next call shall execute
    pub fn current_dir(&mut self, dir: &'a dyn TestPath) -> &mut Self {
        self.dir = Some(dir);
        self
    }

    /// Calls the executable with the given arguments and expects successful exit.
    /// `args` can be `NO_ARGS` or something iterateable that yields the arguments.
    /// `envs` can be `NO_ENVS` or something iterateable that yields the key/value pairs.
    /// When any envs are given then the environment is cleared first.
    /// Returns a TestOutput object for further investigation.
    #[track_caller]
    pub fn call<IA, S, IE, K, V>(&self, args: IA, envs: IE) -> Output
    where
        IA: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        IE: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut command = match self.executable {
            ExeLocation::BinTest { executables, name } => executables.command(name),
            ExeLocation::External(path) => Command::new(path),
        };
        if let Some(dir) = &self.dir {
            command.current_dir(dir.path());
        }

        let mut envs = envs.into_iter().fuse().peekable();
        if envs.peek().is_some() {
            command.env_clear();
            command.envs(envs);
        }

        let output = command.args(args).output().expect("called command");
        output
    }
}

pub const NO_ARGS: [&OsStr; 0] = [];
pub const NO_ENVS: [(&OsStr, &OsStr); 0] = [];

#[cfg(test)]
#[cfg(unix)]
mod test {
    use crate::*;
    use std::path::Path;

    #[test]
    fn echo_no_args() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call(NO_ARGS, NO_ENVS)
            .assert_success()
            .assert_stdout_utf8("");
    }

    #[test]
    fn echo() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call(["Hello World!"], NO_ENVS)
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }

    #[test]
    #[should_panic]
    fn echo_fail() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call(["No World!"], NO_ENVS)
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }
}

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

    /// Calls the executable with the given arguments and environment.
    /// `args` can be `NO_ARGS` or something iterateable that yields the arguments.
    /// `envs` can be `NO_ENVS` or something iterateable that yields the key/value pairs.
    /// When any envs are given then the environment is cleared first.
    /// Returns a Output object for further investigation.
    #[track_caller]
    pub fn call_args_envs<IA, S, IE, K, V>(&self, args: IA, envs: IE) -> Output
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

    /// Calls the executable with the given arguments.
    /// `args` can be `NO_ARGS` or something iterateable that yields the arguments.
    /// Returns a Output object for further investigation.
    #[inline]
    #[track_caller]
    pub fn call_args<IA, S>(&self, args: IA) -> Output
    where
        IA: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.call_args_envs(args, NO_ENVS)
    }

    /// Calls the executable without arguments.
    /// `envs` can be `NO_ENVS` or something iterateable that yields the key/value pairs.
    /// When any envs are given then the environment is cleared first.
    /// Returns a Output object for further investigation.
    #[inline]
    #[track_caller]
    pub fn call_envs<IE, K, V>(&self, envs: IE) -> Output
    where
        IE: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.call_args_envs(NO_ARGS, envs)
    }

    /// Calls the executable without arguments.
    /// Returns a Output object for further investigation.
    #[inline]
    #[track_caller]
    pub fn call(&self) -> Output {
        self.call_args_envs(NO_ARGS, NO_ENVS)
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
            .call()
            .assert_success()
            .assert_stdout_utf8("");
    }

    #[test]
    fn echo() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call_args(["Hello World!"])
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }

    #[test]
    #[should_panic]
    fn echo_fail() {
        let testcall = TestCall::external_command(Path::new("echo"));

        testcall
            .call_args(["No World!"])
            .assert_success()
            .assert_stdout_utf8("Hello World!");
    }
}

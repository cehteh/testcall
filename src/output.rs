use std::process::Output;

use crate::Captured;

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
    fn assert_stderr_utf8(&self, regex: &str) -> &Self;

    /// Applies a regex match check to stdout, will panic when the match failed.
    /// This check uses the 'bytes' module from the regex package and matches bytes.
    #[track_caller]
    fn assert_stdout_bytes(&self, regex: &str) -> &Self;

    /// Applies a regex match check to stderr, will panic when the match failed.
    /// This check uses the 'bytes' module from the regex package and matches bytes.
    #[track_caller]
    fn assert_stderr_bytes(&self, regex: &str) -> &Self;

    /// Applies a regex on stdout, returns named captures as CaptureKey:String map.
    /// Matches utf8 text, input is lossy convered to utf8 first.
    fn stdout_captures_utf8(&self, regex: &str) -> Captured;

    /// Applies a regex on stderr, returns named captures as CaptureKey:String map.
    /// Matches utf8 text, input is lossy convered to utf8 first.
    fn stderr_captures_utf8(&self, regex: &str) -> Captured;
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

    //PLANNED: make a HashMap<String, Regex> to cache compiled regex

    fn assert_stdout_utf8(&self, regex: &str) -> &Self {
        let (ok, utf8) = crate::regex::regex_match_utf8(&self.stdout, regex);
        assert!(
            ok,
            "stdout does not match:\n{}\nstdout was:\n{}",
            regex, utf8
        );
        self
    }

    fn assert_stderr_utf8(&self, regex: &str) -> &Self {
        let (ok, utf8) = crate::regex::regex_match_utf8(&self.stderr, regex);
        assert!(
            ok,
            "stderr does not match:\n{}\nstderr was:\n{}",
            regex, utf8
        );
        self
    }

    fn assert_stdout_bytes(&self, regex: &str) -> &Self {
        let (ok, bytes) = crate::regex::regex_match_bytes(&self.stdout, regex);
        assert!(
            ok,
            "stdout does not match:\n{}\nstdout was:\n{}",
            regex, bytes
        );
        self
    }

    fn assert_stderr_bytes(&self, regex: &str) -> &Self {
        let (ok, bytes) = crate::regex::regex_match_bytes(&self.stderr, regex);
        assert!(
            ok,
            "stderr does not match:\n{}\nstderr was:\n{}",
            regex, bytes
        );
        self
    }

    fn stdout_captures_utf8(&self, regex: &str) -> Captured {
        crate::regex::captures_utf8(&self.stdout, regex)
    }

    fn stderr_captures_utf8(&self, regex: &str) -> Captured {
        crate::regex::captures_utf8(&self.stderr, regex)
    }
}

#[cfg(test)]
#[cfg(unix)]
mod test {
    use crate::*;
    use std::path::Path;

    #[test]
    fn captures() {
        let testcall = TestCall::external_command(Path::new("echo"));

        let captures = testcall
            .call(["Hello World!"], NO_ENVS)
            .stdout_captures_utf8("(?P<first>[^ ]*) (?P<second>[^ ]*)");

        assert_eq!(&captures[0], "Hello World!\n");
        assert_eq!(&captures[1], "Hello");
        assert_eq!(&captures[2], "World!\n");
        assert_eq!(&captures["first"], "Hello");
        assert_eq!(&captures["second"], "World!\n");
    }
}

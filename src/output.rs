use std::collections::HashMap;
use std::process::Output;

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
    fn stdout_captures_utf8(&self, regex: &str) -> HashMap<CaptureKey, String>;

    /// Applies a regex on stderr, returns named captures as CaptureKey:String map.
    /// Matches utf8 text, input is lossy convered to utf8 first.
    fn stderr_captures_utf8(&self, regex: &str) -> HashMap<CaptureKey, String>;
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

    fn stdout_captures_utf8(&self, regex: &str) -> HashMap<CaptureKey, String> {
        captures_utf8(&self.stdout, regex)
    }

    fn stderr_captures_utf8(&self, regex: &str) -> HashMap<CaptureKey, String> {
        captures_utf8(&self.stderr, regex)
    }
}

fn captures_utf8(input: &[u8], regex: &str) -> HashMap<CaptureKey, String> {
    let mut captures = HashMap::new();
    use regex::Regex;
    let re = Regex::new(regex).expect("compiled regex");
    let text = String::from_utf8_lossy(input);

    use CaptureKey::*;

    if let Some(c) = re.captures(&text) {
        for n in 0..c.len() {
            if let Some(m) = c.get(n) {
                captures.insert(Index(n), String::from(m.as_str()));
            }
        }

        for n in re.capture_names() {
            if let (Some(n), Some(m)) = (n, c.name(n.unwrap_or_default())) {
                captures.insert(Name(String::from(n)), String::from(m.as_str()));
            }
        }
    };

    captures
}

/// Captured keys which can be identified by numeric index or by name.
#[derive(Hash, PartialEq)]
pub enum CaptureKey {
    Index(usize),
    Name(String),
}

impl Eq for CaptureKey {}

#[cfg(test)]
#[cfg(unix)]
mod test {
    use crate::*;
    use std::path::Path;

    #[test]
    fn captures() {
        let testcall = TestCall::external_command(Path::new("echo"));

        let captures = testcall
            .call(["Hello World!"])
            .stdout_captures_utf8("(?P<first>[^ ]*) (?P<second>[^ ]*)");

        use CaptureKey::*;

        assert_eq!(captures[&Index(0)], "Hello World!\n");
        assert_eq!(captures[&Index(1)], "Hello");
        assert_eq!(captures[&Index(2)], "World!\n");
        assert_eq!(captures[&Name(String::from("first"))], "Hello");
        assert_eq!(captures[&Name(String::from("second"))], "World!\n");
    }
}

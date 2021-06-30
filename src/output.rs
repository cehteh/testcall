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

    // /// Applies a regex on stdout, returns named captures as name:value map.
    // /// Matches utf8 text, stdout is lossy convered to utf8 first.
    // fn stdout_captures_utf8(&self, regex: &str) -> HashMap<CaptureKey, String>;

    // Applies a regex on stderr, returns the regex captures.
    //   fn stderr_captures(&self, regex: &str) -> Option<Captures<'a>>;
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

    //WIP:
    // fn stdout_captures_utf8(
    //     &self,
    //     regex: &str,
    // ) -> HashMap<CaptureKey, String> {
    //     let mut captures = HashMap::new();
    //     use regex::Regex;
    //     let re = Regex::new(regex).expect("compiled regex");
    //     let text = String::from_utf8_lossy(&self.stdout);
    //     re.captures(text).for_each()
    //     self
    // }
}

//WIP:
// /// Captured keys which can be identified by numeric index or by name.
// #[derive(Hash, PartialEq)]
// enum CaptureKey {
//     Index(usize),
//     Name(String),
// }
//
// impl Eq for CaptureKey {}

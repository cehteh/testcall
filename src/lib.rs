//! Companinon crate to 'BinTest', implements test facilities
//!
//!
//! # Description
//!
//! A TestCall uses BinTest and std::process::Command to wrap process execution in a way that
//! is ergonomic to use for (repeated) testing. Few more test facilities are provided and will
//! grow in future.
//!
//!
//! # Initial Example
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
//!
//! # Panics vs. Results
//!
//! 'testcall' is made explicitly for writing tests. To ease this it prefers aborting by panic
//! over error handling. When anything goes wrong the test is aborted and the cause is
//! reported.
//!
//!
//! # Concepts and Facilities
//!
//! augmenting standard/existing things assert capture regex
//! ## Regular Expressions and Captures
//!
//!
//!
//!
//! ## TestCall
//!
//! Allows setting up and calling programs build by your project through the 'bintest' crate
//! or any other executable. Augments 'std::process::Command'. The result of running tests is
//! collected and returned in a 'std::process::Output'.
//!
//!
//! ## TestOutput
//!
//! A Trait that augments 'std::process::Output' with assertions and regex capturing functions
//! to validate the result of a test run. Note that 'std::process::Output' stores the results
//! of a call in memory. Thus testing should not generate excessive outputs (on
//! stdout/stderr).
//!
//!
//! # Future Plans
//!
//! New features will be added as needed, PR's are welcome. This is work in progress.
//!
//!
mod output;
pub mod regex;
mod testcall;

pub use crate::output::TestOutput;
pub use crate::regex::CaptureKey;
pub use crate::testcall::TestCall;

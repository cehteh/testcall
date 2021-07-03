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
//!
//! # Panics vs. Results
//!
//! 'testcall' is made explicitly for writing tests. To ease this it prefers aborting by panic
//! over error handling. When anything goes wrong the test is aborted and the cause is
//! reported.
//!
//!
//! # Future Plans
//!
//! New features will be added as needed, PR's are welcome. This is work in progress.
//!
//! Things to be done soon are:
//!  * Populating TestDirs from template directories
//!  * Validating directory contents
//!
mod output;
mod testcall;
mod testdir;
mod regex;

pub use crate::testcall::TestCall;
pub use crate::testdir::{TestDir, TempDirCleanup, Fixtures, DirAssertions};
pub use crate::output::TestOutput;
pub use crate::regex::CaptureKey;

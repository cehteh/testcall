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

mod output;
mod testcall;
mod testdir;

pub use crate::output::TestOutput;
pub use crate::testcall::TestCall;
pub use crate::testdir::{TestDir, TempDirCleanup};

Companinon crate to ‘BinTest’, implements test facilities

# Description

A TestCall uses BinTest and std::process::Command to wrap process execution in a way that is
ergonomic to use for (repeated) testing. Few more test facilities are provided and will grow
in future.

# Example

```rust
#[test]
fn myprogram_test() {
    let executables = BinTest::new();
    let mut myprogram = TestCall::new(&executables, "myprogram");

    myprogram.current_dir(Box::new(TempDir::new().expect("created tempdir")));
    myprogram
        .call(["--help"])
        .assert_success();
}
```

# Future Plans

New features will be added as needed, PR’s are welcome. This is work in progress.

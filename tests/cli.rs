use std::process::Command;

#[test]
fn runs_a_source_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_kivil"))
        .arg("examples/merhaba.kivil")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Merhaba Herdem\nPuanın: 15\n"
    );
}

#[test]
fn reports_usage_missing_files_and_runtime_errors() {
    for args in [vec![], vec!["one", "two"], vec!["missing-file.kivil"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_kivil"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
}

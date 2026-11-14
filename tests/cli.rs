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

fn run_source(source: &str, input: &str) -> std::process::Output {
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "kivil-{}-{}.kivil",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kivil"))
        .arg(&path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    std::fs::remove_file(path).unwrap();
    output
}

#[test]
fn runs_the_complete_language_with_piped_input() {
    let source = std::fs::read_to_string("examples/not_hesapla.kivil").unwrap();
    let output = run_source(&source, "Herdem\r\n60\n70\n80\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Adın: Notun: Notun: Notun: Herdem ortalaman: 70\nGeçtin\n"
    );
}

#[test]
fn errors_include_file_line_and_column_and_stop_execution() {
    for (source, location) in [
        ("öğrenci = 1\n  @", ":2:3:"),
        ("1 +", ":1:4:"),
        ("yaz(\"önce\")\n10 / 0\nyaz(\"sonra\")", ":2:4:"),
        ("islev hata() { dondur 1 / 0 }\nhata()", ":1:25:"),
        ("a = [1]\na[2]", ":2:2:"),
    ] {
        let output = run_source(source, "");
        assert!(!output.status.success(), "{source}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(location), "{error}");
        assert!(error.contains(".kivil:"), "{error}");
        assert!(!String::from_utf8(output.stdout).unwrap().contains("sonra"));
    }
}

#[test]
fn input_eof_and_failed_conversions_exit_with_errors() {
    for (source, input) in [("girdi()", ""), ("sayi(girdi())", "abc\n")] {
        let output = run_source(source, input);
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
}

use std::process::Command;

fn check_rungrep() -> Command {
    Command::new(env!("CARGO_BIN_EXE_check_rungrep"))
}

/// Running a nonexistent executable must exit with code 3 and print an exec(3) error.
#[test]
fn test_command_not_found() {
    let output = check_rungrep()
        .args(["command", "this_executable_does_not_exist_12345"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exec(3)"), "stdout was: {stdout}");
}

/// Invoking the plugin with no arguments must exit with code 3 and print a usage error.
#[test]
fn test_no_args_exits_3() {
    let output = check_rungrep().output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("☯️"),
        "expected usage error on stderr, got: {stderr}"
    );
}

/// `command sh -c 'exit 0'` must succeed with exit code 0.
#[test]
#[cfg(unix)]
fn test_command_success() {
    let output = check_rungrep()
        .args(["command", "sh", "-c", "exit 0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

/// `cd` to a nonexistent directory must exit with code 3 and print a chdir(2) error.
#[test]
#[cfg(unix)]
fn test_cd_not_found() {
    let output = check_rungrep()
        .args([
            "cd",
            "/this_directory_does_not_exist_12345",
            "command",
            "sh",
            "-c",
            "exit 0",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("chdir(2)"), "stdout was: {stdout}");
}

/// `exit` condition: exit code 0 is within default OK range → overall exit 0.
#[test]
#[cfg(unix)]
fn test_exit_condition_ok() {
    let output = check_rungrep()
        .args(["exit", "", "", "", "command", "sh", "-c", "exit 0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Command returned 0"),
        "stdout was: {stdout}"
    );
}

/// `exit` condition: exit code 1 triggers a critical when range is `0:0` → overall exit 2.
#[test]
#[cfg(unix)]
fn test_exit_condition_critical() {
    let output = check_rungrep()
        .args(["exit", "", "0:0", "", "command", "sh", "-c", "exit 1"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Command returned 1"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("🚨"), "stdout was: {stdout}");
}

/// `exit` condition: exit code 1 triggers a warning when warn range is `0:0` → overall exit 1.
#[test]
#[cfg(unix)]
fn test_exit_condition_warning() {
    let output = check_rungrep()
        .args(["exit", "0:0", "", "", "command", "sh", "-c", "exit 1"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("⚠️"), "stdout was: {stdout}");
}

/// `stdout literal` matching: pattern found exactly once within critical `1:1` range → exit 0.
#[test]
#[cfg(unix)]
fn test_stdout_literal_match_ok() {
    let output = check_rungrep()
        .args([
            "stdout",
            "literal",
            "hello",
            "",
            "1:1",
            "",
            "command",
            "sh",
            "-c",
            "echo hello",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Command's stdout matched"),
        "stdout was: {stdout}"
    );
}

/// `stdout literal` matching: pattern not found, but critical `1:1` requires exactly one → exit 2.
#[test]
#[cfg(unix)]
fn test_stdout_literal_no_match_critical() {
    let output = check_rungrep()
        .args([
            "stdout",
            "literal",
            "hello",
            "",
            "1:1",
            "",
            "command",
            "sh",
            "-c",
            "echo goodbye",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🚨"), "stdout was: {stdout}");
}

/// `stdout regex` matching: regex `hel+o` matches "hello" → exit 0.
#[test]
#[cfg(unix)]
fn test_stdout_regex_match_ok() {
    let output = check_rungrep()
        .args([
            "stdout",
            "regex",
            "hel+o",
            "",
            "1:1",
            "",
            "command",
            "sh",
            "-c",
            "echo hello",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

/// `stderr` matching: command writes to stderr and the plugin detects it.
#[test]
#[cfg(unix)]
fn test_stderr_literal_match() {
    let output = check_rungrep()
        .args([
            "stderr",
            "literal",
            "err_msg",
            "",
            "1:1",
            "",
            "command",
            "sh",
            "-c",
            "echo err_msg >&2",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Command's stderr matched"),
        "stdout was: {stdout}"
    );
}

/// `time` condition with a perfdata label produces a performance data line.
#[test]
#[cfg(unix)]
fn test_time_condition_with_label() {
    let output = check_rungrep()
        .args([
            "time",
            "",
            "",
            "run_seconds",
            "command",
            "sh",
            "-c",
            "exit 0",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("run_seconds"), "stdout was: {stdout}");
    assert!(stdout.contains('|'), "stdout was: {stdout}");
}

/// `CHECK_RUNGREP_STDIN` env var: the value is piped to the command's stdin.
#[test]
#[cfg(unix)]
fn test_stdin_env_var() {
    let output = check_rungrep()
        .env("CHECK_RUNGREP_STDIN", "hello")
        .args([
            "stdout", "literal", "hello", "", "1:1", "", "command", "cat",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

use std::ffi::OsString;
use std::io::Write;
use std::process::{exit, Command, Stdio};
use std::thread::spawn;
use std::time::{Duration, Instant};

pub(crate) fn run_cmd(
    exe: OsString,
    args: Vec<OsString>,
    input: Option<OsString>,
) -> (Vec<u8>, Vec<u8>, i32, Duration) {
    let mut cmd = Command::new(exe);

    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let start = Instant::now();

    match cmd.spawn() {
        Err(err) => {
            println!("☯️ exec(3): {}", err);
            exit(3);
        }
        Ok(mut child) => {
            match child.stdin.take() {
                None => {}
                Some(mut stdin) => {
                    spawn(move || {
                        match input {
                            None => {}
                            Some(data) => {
                                drop(stdin.write_all(data.into_encoded_bytes().as_slice()));
                            }
                        }

                        drop(stdin);
                    });
                }
            }

            match child.wait_with_output() {
                Err(err) => {
                    println!("☯️ waitpid(2): {}", err);
                    exit(3);
                }
                Ok(result) => {
                    let end = Instant::now();
                    match result.status.code() {
                        None => {
                            println!("☯️ waitpid(2): child was killed");
                            exit(3);
                        }
                        Some(code) => (result.stdout, result.stderr, code, end - start),
                    }
                }
            }
        }
    }
}

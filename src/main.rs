mod alerts;
mod arg_parser;
mod cli;
mod cnt_iter;
mod plugin;
mod process;
mod usage;

use crate::cli::{Condition, Matcher};
use crate::plugin::Check;
use alerts::{ExecTime, ExitCode, Matches};
use memchr::memmem::find_iter;
use plugin::{Perfdat, Perfdata, State};
use std::env::{args_os, set_current_dir, var_os};
use std::io;
use std::io::Write;
use std::process::exit;

fn main() {
    let (_, rtodo, consumed) = arg_parser::parse_cmdline(args_os());
    match rtodo {
        Err(err) => {
            usage::blame_user(err, consumed);
            exit(3);
        }
        Ok(todo) => {
            for cd in todo.cd {
                match set_current_dir(cd.as_os_str()) {
                    Err(err) => {
                        println!("☯️ chdir(2): {}", err);
                        exit(3);
                    }
                    Ok(_) => {}
                }
            }

            let (stdout, stderr, code, time) = process::run_cmd(
                todo.exe,
                todo.args,
                var_os("CHECK_RUNGREP_STDIN").filter(|s| !s.is_empty()),
            );

            let mut check = Check::new();
            for condition in todo.conditions {
                match condition {
                    Condition::ExecSeconds(thresholds) => {
                        check.add(
                            Box::new(ExecTime {
                                time,
                                thresholds: thresholds.thresholds.clone(),
                            }),
                            Perfdata {
                                value: time.as_secs_f64(),
                                uom: "s",
                                thresholds,
                                min: Some(0.0),
                                max: None,
                            },
                        );
                    }
                    Condition::ExitCode(thresholds) => {
                        check.add(
                            Box::new(ExitCode {
                                code,
                                thresholds: thresholds.thresholds.clone(),
                            }),
                            Perfdata {
                                value: code as f64,
                                uom: "",
                                thresholds,
                                min: None,
                                max: None,
                            },
                        );
                    }
                    Condition::Stdout(matcher, thresholds) => {
                        handle_matcher(&mut check, "stdout", &stdout, matcher, thresholds);
                    }
                    Condition::Stderr(matcher, thresholds) => {
                        handle_matcher(&mut check, "stderr", &stderr, matcher, thresholds);
                    }
                }
            }

            print!("{}", check);

            if check.state() != State::Ok {
                show_out("STDOUT", stdout);
                show_out("STDERR", stderr);
            }

            exit(check.state() as i32);
        }
    }
}

fn handle_matcher(
    check: &mut Check,
    source: &'static str,
    data: &Vec<u8>,
    matcher: Matcher,
    thresholds: Perfdat,
) {
    let times = match &matcher {
        Matcher::Literal(literal) => find_iter(data.as_slice(), literal).count(),
        Matcher::RegExp(regexp) => regexp.find_iter(data.as_slice()).count(),
    };

    check.add(
        Box::new(Matches {
            source,
            matcher,
            times,
            thresholds: thresholds.thresholds.clone(),
        }),
        Perfdata {
            value: times as f64,
            uom: "",
            thresholds,
            min: Some(0.0),
            max: None,
        },
    );
}

fn show_out(h1: &str, data: Vec<u8>) {
    println!();
    println!("{}", h1);
    println!("======");
    println!();

    match io::stdout().write(data.as_ref()) {
        Ok(_) => {}
        Err(err) => {
            panic!("failed printing to stdout: {err}");
        }
    }
}

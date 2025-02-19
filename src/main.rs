mod arg_parser;
mod cli;
mod cnt_iter;
mod usage;

use std::env::{args_os, set_current_dir};
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
                        eprintln!("chdir(2): {}", err);
                        exit(3);
                    }
                    Ok(_) => {}
                }
            }

            // TODO
        }
    }
}

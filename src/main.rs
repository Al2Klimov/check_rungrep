mod arg_parser;
mod cli;
mod cnt_iter;
mod usage;

use std::env::args_os;
use std::process::exit;

fn main() {
    let (_, rtodo, consumed) = arg_parser::parse_cmdline(args_os());
    match rtodo {
        Err(err) => {
            usage::blame_user(err, consumed);
            exit(3);
        }
        Ok(todo) => {
            // TODO
        }
    }
}

use crate::cli::{ExpectedArg, ParseArgsError};

pub(crate) fn blame_user(err: ParseArgsError, consumed: usize) {
    match err {
        ParseArgsError::UnexpectedEnd(ea) => {
            eprintln!(
                "Unexpected end of CLI arguments, expected {}.",
                expected(ea)
            );
        }
        ParseArgsError::EmptyString(ea) => {
            eprintln!(
                "Illegal empty string (CLI argument #{}), expected {}.",
                consumed,
                expected(ea)
            );
        }
        ParseArgsError::BadUnicode(ea, er) => {
            eprintln!(
                "Invalid UTF-8 (CLI argument #{}), expected {}. Error: {}",
                consumed,
                expected(ea),
                er
            );
        }
        ParseArgsError::UnknownParam => {
            eprintln!(
                "Unknown parameter (CLI argument #{}), expected {}.",
                consumed,
                expected(ExpectedArg::Command)
            );
        }
        ParseArgsError::BadThreshold(ea, er) => {
            eprintln!(
                "Invalid @start:end {} (CLI argument #{}): {}",
                expected(ea),
                consumed,
                er
            );
        }
        ParseArgsError::UnknownMatcher => {
            eprintln!(
                "Unknown kind of pattern (CLI argument #{}), expected {}.",
                consumed,
                expected(ExpectedArg::Matcher)
            );
        }
        ParseArgsError::BadRegex(er) => {
            eprintln!(
                "Invalid regular expression (CLI argument #{}): {}",
                consumed, er
            );
        }
    }
}

fn expected(ea: ExpectedArg) -> &'static str {
    match ea {
        ExpectedArg::Matcher => "\"literal\"/\"regex\"",
        ExpectedArg::Pattern => "search pattern",
        ExpectedArg::Warning => "warning threshold",
        ExpectedArg::Critical => "critical threshold",
        ExpectedArg::Label => "perfdata label",
        ExpectedArg::Command => "\"command\"/\"cd\"/\"time\"/\"exit\"/\"stdout\"/\"stderr\"",
        ExpectedArg::WorkDir => "working directory",
        ExpectedArg::Exe => "executable name",
    }
}

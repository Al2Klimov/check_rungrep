use crate::plugin::Perfdat;
use regex::bytes::Regex;
use std::ffi::OsString;
use std::str::Utf8Error;

pub(crate) struct Args {
    pub(crate) cd: Vec<OsString>,
    pub(crate) exe: OsString,
    pub(crate) args: Vec<OsString>,
    pub(crate) conditions: Vec<Condition>,
}

pub(crate) enum Condition {
    ExecSeconds(Perfdat),
    ExitCode(Perfdat),
    Stdout(Matcher, Perfdat),
    Stderr(Matcher, Perfdat),
}

pub(crate) enum Matcher {
    Literal(Vec<u8>),
    RegExp(Regex),
}

pub(crate) enum ParseArgsError {
    UnexpectedEnd(ExpectedArg),
    EmptyString(ExpectedArg),
    BadUnicode(ExpectedArg, Utf8Error),
    UnknownParam,
    BadThreshold(ExpectedArg, nagios_range::Error),
    UnknownMatcher,
    BadRegex(regex::Error),
}

#[derive(Clone)]
pub(crate) enum ExpectedArg {
    Matcher,
    Pattern,
    Warning,
    Critical,
    Label,
    Command,
    WorkDir,
    Exe,
}

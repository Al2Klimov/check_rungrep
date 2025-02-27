use crate::cli::{Args, Condition, ExpectedArg, Matcher, ParseArgsError};
use crate::cnt_iter::CounterIterator;
use crate::plugin::{Perfdat, Thresholds};
use nagios_range::NagiosRange;
use regex::bytes::Regex;
use std::ffi::OsString;

pub(crate) fn parse_cmdline<A>(
    mut args: A,
) -> (Option<OsString>, Result<Args, ParseArgsError>, usize)
where
    A: Iterator<Item = OsString>,
{
    let program = args.next();
    let mut ci = CounterIterator::new(args);

    (program, parse_args(&mut ci), ci.taken())
}

fn parse_args(args: &mut dyn Iterator<Item = OsString>) -> Result<Args, ParseArgsError> {
    let mut cd = Vec::new();
    let mut conditions = Vec::new();

    loop {
        match require_noempty_utf8(args.next(), ExpectedArg::Command)?.as_ref() {
            "time" => {
                conditions.push(Condition::ExecSeconds(parse_perfdata(args)?));
            }
            "exit" => {
                conditions.push(Condition::ExitCode(parse_perfdata(args)?));
            }
            "stdout" => {
                conditions.push(Condition::Stdout(
                    parse_matcher(args)?,
                    parse_perfdata(args)?,
                ));
            }
            "stderr" => {
                conditions.push(Condition::Stderr(
                    parse_matcher(args)?,
                    parse_perfdata(args)?,
                ));
            }
            "cd" => {
                cd.push(require_noempty(args.next(), ExpectedArg::WorkDir)?);
            }
            "command" => {
                return Ok(Args {
                    cd,
                    exe: require_noempty(args.next(), ExpectedArg::Exe)?,
                    args: args.collect(),
                    conditions,
                })
            }
            _ => return Err(ParseArgsError::UnknownParam),
        }
    }
}

fn parse_matcher(args: &mut dyn Iterator<Item = OsString>) -> Result<Matcher, ParseArgsError> {
    match require_noempty_utf8(args.next(), ExpectedArg::Matcher)?.as_ref() {
        "literal" => Ok(Matcher::Literal(
            require_noempty(args.next(), ExpectedArg::Pattern)?.into_encoded_bytes(),
        )),
        "regex" => Ok(Matcher::RegExp(
            Regex::new(require_noempty_utf8(args.next(), ExpectedArg::Pattern)?.as_ref())
                .map_err(|err| ParseArgsError::BadRegex(err))?,
        )),
        _ => Err(ParseArgsError::UnknownMatcher),
    }
}

fn parse_perfdata(args: &mut dyn Iterator<Item = OsString>) -> Result<Perfdat, ParseArgsError> {
    Ok(Perfdat {
        thresholds: Thresholds {
            warn: require_threshold(args.next(), ExpectedArg::Warning)?,
            crit: require_threshold(args.next(), ExpectedArg::Critical)?,
        },
        label: require_utf8(args.next(), ExpectedArg::Label)?,
    })
}

fn require_threshold(
    oarg: Option<OsString>,
    want: ExpectedArg,
) -> Result<Option<NagiosRange>, ParseArgsError> {
    let arg = require_utf8(oarg, want.clone())?;
    if arg.is_empty() {
        Ok(None)
    } else {
        match NagiosRange::from(arg.as_str()) {
            Err(err) => Err(ParseArgsError::BadThreshold(want, err)),
            Ok(nr) => Ok(Some(nr)),
        }
    }
}

fn require_noempty_utf8(
    oarg: Option<OsString>,
    want: ExpectedArg,
) -> Result<String, ParseArgsError> {
    String::from_utf8(require_noempty(oarg, want.clone())?.into_encoded_bytes())
        .map_err(|err| ParseArgsError::BadUnicode(want, err.utf8_error()))
}

fn require_utf8(oarg: Option<OsString>, want: ExpectedArg) -> Result<String, ParseArgsError> {
    String::from_utf8(require(oarg, want.clone())?.into_encoded_bytes())
        .map_err(|err| ParseArgsError::BadUnicode(want, err.utf8_error()))
}

fn require_noempty(oarg: Option<OsString>, want: ExpectedArg) -> Result<OsString, ParseArgsError> {
    let arg = require(oarg, want.clone())?;
    if arg.is_empty() {
        Err(ParseArgsError::EmptyString(want))
    } else {
        Ok(arg)
    }
}

fn require(oarg: Option<OsString>, want: ExpectedArg) -> Result<OsString, ParseArgsError> {
    oarg.ok_or(ParseArgsError::UnexpectedEnd(want))
}

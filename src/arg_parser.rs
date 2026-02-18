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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Condition, Matcher, ParseArgsError};

    fn args(v: Vec<&'static str>) -> impl Iterator<Item = OsString> {
        v.into_iter().map(OsString::from)
    }

    fn unwrap_args(result: Result<Args, ParseArgsError>) -> Args {
        match result {
            Ok(v) => v,
            Err(_) => panic!("unexpected parse error"),
        }
    }

    #[test]
    fn test_minimal_command() {
        let (prog, result, _) = parse_cmdline(args(vec!["prog", "command", "exe"]));
        assert_eq!(prog, Some(OsString::from("prog")));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.exe, OsString::from("exe"));
        assert!(parsed.args.is_empty());
        assert!(parsed.cd.is_empty());
        assert!(parsed.conditions.is_empty());
    }

    #[test]
    fn test_command_with_args() {
        let (_, result, _) = parse_cmdline(args(vec!["prog", "command", "exe", "arg1", "arg2"]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.exe, OsString::from("exe"));
        assert_eq!(
            parsed.args,
            vec![OsString::from("arg1"), OsString::from("arg2")]
        );
    }

    #[test]
    fn test_cd_option() {
        let (_, result, _) = parse_cmdline(args(vec!["prog", "cd", "/tmp", "command", "exe"]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.cd, vec![OsString::from("/tmp")]);
    }

    #[test]
    fn test_time_condition() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog", "time", "", "", "label", "command", "exe",
        ]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.conditions.len(), 1);
        match &parsed.conditions[0] {
            Condition::ExecSeconds(pd) => {
                assert!(pd.thresholds.warn.is_none());
                assert!(pd.thresholds.crit.is_none());
                assert_eq!(pd.label, "label");
            }
            _ => panic!("expected ExecSeconds"),
        }
    }

    #[test]
    fn test_exit_condition() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog",
            "exit",
            "",
            "",
            "exitlabel",
            "command",
            "exe",
        ]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.conditions.len(), 1);
        match &parsed.conditions[0] {
            Condition::ExitCode(pd) => {
                assert_eq!(pd.label, "exitlabel");
            }
            _ => panic!("expected ExitCode"),
        }
    }

    #[test]
    fn test_stdout_literal_condition() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog", "stdout", "literal", "pattern", "", "", "", "command", "exe",
        ]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.conditions.len(), 1);
        match &parsed.conditions[0] {
            Condition::Stdout(Matcher::Literal(lit), _) => {
                assert_eq!(lit, b"pattern");
            }
            _ => panic!("expected Stdout(Literal)"),
        }
    }

    #[test]
    fn test_stderr_regex_condition() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog", "stderr", "regex", "foo.*", "", "", "", "command", "exe",
        ]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.conditions.len(), 1);
        match &parsed.conditions[0] {
            Condition::Stderr(Matcher::RegExp(re), _) => {
                assert_eq!(re.as_str(), "foo.*");
            }
            _ => panic!("expected Stderr(RegExp)"),
        }
    }

    #[test]
    fn test_multiple_conditions() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog", "exit", "", "", "ec", "time", "", "", "t", "command", "exe",
        ]));
        let parsed = unwrap_args(result);
        assert_eq!(parsed.conditions.len(), 2);
    }

    #[test]
    fn test_error_unexpected_end_no_args() {
        let (_, result, consumed) = parse_cmdline(args(vec!["prog"]));
        assert!(matches!(
            result,
            Err(ParseArgsError::UnexpectedEnd(ExpectedArg::Command))
        ));
        assert_eq!(consumed, 0);
    }

    #[test]
    fn test_error_unknown_param() {
        let (_, result, _) = parse_cmdline(args(vec!["prog", "unknown"]));
        assert!(matches!(result, Err(ParseArgsError::UnknownParam)));
    }

    #[test]
    fn test_error_bad_regex() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog", "stdout", "regex", "[invalid", "", "", "", "command", "exe",
        ]));
        assert!(matches!(result, Err(ParseArgsError::BadRegex(_))));
    }

    #[test]
    fn test_error_bad_threshold() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog",
            "exit",
            "not_a_range",
            "",
            "",
            "command",
            "exe",
        ]));
        assert!(matches!(
            result,
            Err(ParseArgsError::BadThreshold(ExpectedArg::Warning, _))
        ));
    }

    #[test]
    fn test_error_unknown_matcher() {
        let (_, result, _) = parse_cmdline(args(vec![
            "prog",
            "stdout",
            "substring",
            "pattern",
            "",
            "",
            "",
            "command",
            "exe",
        ]));
        assert!(matches!(result, Err(ParseArgsError::UnknownMatcher)));
    }

    #[test]
    fn test_consumed_count_reflects_args_iterated() {
        // 3 items consumed after program name: "command", "exe", "arg1"
        let (_, _, consumed) = parse_cmdline(args(vec!["prog", "command", "exe", "arg1"]));
        assert_eq!(consumed, 3);
    }
}

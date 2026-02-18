use crate::cli::Matcher;
use crate::plugin::Thresholds;
use humantime::format_duration;
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub(crate) struct ExecTime {
    pub(crate) time: Duration,
    pub(crate) thresholds: Thresholds,
}

impl Display for ExecTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command ran for {} seconds ({}).{}",
            self.time.as_secs_f64(),
            format_duration(self.time),
            AlertThresholds {
                thresholds: self.thresholds.clone()
            }
        )
    }
}

pub(crate) struct ExitCode {
    pub(crate) code: i32,
    pub(crate) thresholds: Thresholds,
}

impl Display for ExitCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command returned {}.{}",
            self.code,
            AlertThresholds {
                thresholds: self.thresholds.clone()
            }
        )
    }
}

pub(crate) struct Matches {
    pub(crate) source: &'static str,
    pub(crate) matcher: Matcher,
    pub(crate) times: usize,
    pub(crate) thresholds: Thresholds,
}

impl Display for Matches {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command's {} matched the following pattern {} times.{}",
            self.source,
            self.times,
            AlertThresholds {
                thresholds: self.thresholds.clone()
            }
        )?;

        match &self.matcher {
            Matcher::Literal(literal) => {
                write!(f, " Literal string: {}", String::from_utf8_lossy(literal))
            }
            Matcher::RegExp(regexp) => write!(f, " Regular expression: {}", regexp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Thresholds;
    use regex::bytes::Regex;
    use std::time::Duration;

    fn no_thresholds() -> Thresholds {
        Thresholds { warn: None, crit: None }
    }

    #[test]
    fn test_exec_time_display_contains_seconds_and_description() {
        let et = ExecTime {
            time: Duration::from_secs(2),
            thresholds: no_thresholds(),
        };
        let s = et.to_string();
        assert!(s.contains("Command ran for"));
        assert!(s.contains("2"));
    }

    #[test]
    fn test_exit_code_display_contains_code() {
        let ec = ExitCode {
            code: 42,
            thresholds: no_thresholds(),
        };
        let s = ec.to_string();
        assert!(s.contains("Command returned"));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_exit_code_zero_display() {
        let ec = ExitCode {
            code: 0,
            thresholds: no_thresholds(),
        };
        let s = ec.to_string();
        assert!(s.contains("Command returned"));
        assert!(s.contains("0"));
    }

    #[test]
    fn test_matches_literal_display() {
        let m = Matches {
            source: "stdout",
            matcher: Matcher::Literal(b"hello".to_vec()),
            times: 3,
            thresholds: no_thresholds(),
        };
        let s = m.to_string();
        assert!(s.contains("stdout"));
        assert!(s.contains("3"));
        assert!(s.contains("hello"));
        assert!(s.contains("Literal string"));
    }

    #[test]
    fn test_matches_regex_display() {
        let m = Matches {
            source: "stderr",
            matcher: Matcher::RegExp(Regex::new("foo.*bar").unwrap()),
            times: 1,
            thresholds: no_thresholds(),
        };
        let s = m.to_string();
        assert!(s.contains("stderr"));
        assert!(s.contains("1"));
        assert!(s.contains("foo.*bar"));
        assert!(s.contains("Regular expression"));
    }

    #[test]
    fn test_matches_zero_times_display() {
        let m = Matches {
            source: "stdout",
            matcher: Matcher::Literal(b"needle".to_vec()),
            times: 0,
            thresholds: no_thresholds(),
        };
        let s = m.to_string();
        assert!(s.contains("0"));
        assert!(s.contains("needle"));
    }
}

struct AlertThresholds {
    thresholds: Thresholds,
}

impl Display for AlertThresholds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.thresholds.warn {
            None => {}
            Some(warn) => {
                write!(f, " Warning: {}.", warn)?;
            }
        }

        match self.thresholds.crit {
            None => {}
            Some(crit) => {
                write!(f, " Critical: {}.", crit)?;
            }
        }

        Ok(())
    }
}

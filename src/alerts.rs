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

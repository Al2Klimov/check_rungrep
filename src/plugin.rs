use nagios_range::NagiosRange;
use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

pub(crate) struct Check {
    state: State,
    perfdata: Vec<Perfdata>,
    alerts: BTreeMap<State, Vec<Box<dyn Display>>>,
}

#[derive(Ord, Eq, PartialOrd, PartialEq, Clone)]
pub(crate) enum State {
    Ok = 0,
    Warning = 1,
    Critical = 2,
}

pub(crate) struct Perfdata {
    pub(crate) value: f64,
    pub(crate) uom: &'static str,
    pub(crate) thresholds: Perfdat,
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
}

pub(crate) struct Perfdat {
    pub(crate) thresholds: Thresholds,
    pub(crate) label: String,
}

#[derive(Clone)]
pub(crate) struct Thresholds {
    pub(crate) warn: Option<NagiosRange>,
    pub(crate) crit: Option<NagiosRange>,
}

impl Check {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Ok,
            perfdata: Vec::new(),
            alerts: BTreeMap::new(),
        }
    }

    pub(crate) fn add(&mut self, alert: Box<dyn Display>, perfdata: Perfdata) {
        let my_state = if threshold_alert(perfdata.value, &perfdata.thresholds.thresholds.crit) {
            State::Critical
        } else if threshold_alert(perfdata.value, &perfdata.thresholds.thresholds.warn) {
            State::Warning
        } else {
            State::Ok
        };

        self.state = max(self.state.clone(), my_state.clone());
        self.alerts.entry(my_state).or_default().push(alert);

        if !perfdata.thresholds.label.is_empty() {
            self.perfdata.push(perfdata);
        }
    }

    pub(crate) fn state(&self) -> State {
        self.state.clone()
    }
}

fn threshold_alert(value: f64, threshold: &Option<NagiosRange>) -> bool {
    match threshold {
        None => false,
        Some(range) => range.check(value),
    }
}

impl Display for Check {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.alerts.get(&self.state) {
            None => {}
            Some(alerts) => {
                let li = match self.state {
                    State::Ok => "✅",
                    State::Warning => "⚠️",
                    State::Critical => "🚨",
                };

                for alert in alerts {
                    writeln!(f, "{} {}", li, alert)?;
                }
            }
        }

        if !self.perfdata.is_empty() {
            write!(f, " |")?;

            for perfdat in &self.perfdata {
                write!(
                    f,
                    " '{}'={}{};{};{};{};{}",
                    perfdat.thresholds.label.replace("'", "''"),
                    perfdat.value,
                    perfdat.uom,
                    OptionDisplay {
                        o: perfdat.thresholds.thresholds.warn
                    },
                    OptionDisplay {
                        o: perfdat.thresholds.thresholds.crit
                    },
                    OptionDisplay { o: perfdat.min },
                    OptionDisplay { o: perfdat.max }
                )?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nagios_range::NagiosRange;

    fn no_thresholds() -> Perfdat {
        Perfdat {
            thresholds: Thresholds {
                warn: None,
                crit: None,
            },
            label: String::new(),
        }
    }

    fn warn_threshold(range: &str) -> Perfdat {
        Perfdat {
            thresholds: Thresholds {
                warn: Some(NagiosRange::from(range).unwrap()),
                crit: None,
            },
            label: String::new(),
        }
    }

    fn crit_threshold(range: &str) -> Perfdat {
        Perfdat {
            thresholds: Thresholds {
                warn: None,
                crit: Some(NagiosRange::from(range).unwrap()),
            },
            label: String::new(),
        }
    }

    #[test]
    fn test_state_ordering() {
        assert!(State::Ok < State::Warning);
        assert!(State::Warning < State::Critical);
        assert!(State::Ok < State::Critical);
    }

    #[test]
    fn test_check_initial_state_is_ok() {
        let check = Check::new();
        assert!(check.state() == State::Ok);
    }

    #[test]
    fn test_check_add_no_thresholds_stays_ok() {
        let mut check = Check::new();
        check.add(Box::new("alert"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: no_thresholds(),
            min: None,
            max: None,
        });
        assert!(check.state() == State::Ok);
    }

    #[test]
    fn test_check_add_warn_threshold_triggers_warning() {
        let mut check = Check::new();
        // @0:10 alerts when value is inside [0, 10]; value 5.0 triggers
        check.add(Box::new("alert"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: warn_threshold("@0:10"),
            min: None,
            max: None,
        });
        assert!(check.state() == State::Warning);
    }

    #[test]
    fn test_check_add_crit_threshold_triggers_critical() {
        let mut check = Check::new();
        check.add(Box::new("alert"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: crit_threshold("@0:10"),
            min: None,
            max: None,
        });
        assert!(check.state() == State::Critical);
    }

    #[test]
    fn test_check_state_is_worst_of_all_added() {
        let mut check = Check::new();
        check.add(Box::new("warn alert"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: warn_threshold("@0:10"),
            min: None,
            max: None,
        });
        assert!(check.state() == State::Warning);

        check.add(Box::new("crit alert"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: crit_threshold("@0:10"),
            min: None,
            max: None,
        });
        assert!(check.state() == State::Critical);
    }

    #[test]
    fn test_check_display_ok_emoji() {
        let mut check = Check::new();
        check.add(Box::new("everything fine"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: no_thresholds(),
            min: None,
            max: None,
        });
        let s = check.to_string();
        assert!(s.contains("✅"));
        assert!(s.contains("everything fine"));
    }

    #[test]
    fn test_check_display_warning_emoji() {
        let mut check = Check::new();
        check.add(Box::new("something off"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: warn_threshold("@0:10"),
            min: None,
            max: None,
        });
        let s = check.to_string();
        assert!(s.contains("⚠️"));
        assert!(s.contains("something off"));
    }

    #[test]
    fn test_check_display_critical_emoji() {
        let mut check = Check::new();
        check.add(Box::new("bad things"), Perfdata {
            value: 5.0,
            uom: "",
            thresholds: crit_threshold("@0:10"),
            min: None,
            max: None,
        });
        let s = check.to_string();
        assert!(s.contains("🚨"));
        assert!(s.contains("bad things"));
    }

    #[test]
    fn test_check_display_perfdata_included_when_label_nonempty() {
        let mut check = Check::new();
        check.add(Box::new("alert"), Perfdata {
            value: 42.0,
            uom: "s",
            thresholds: Perfdat {
                thresholds: Thresholds { warn: None, crit: None },
                label: String::from("exec_time"),
            },
            min: Some(0.0),
            max: None,
        });
        let s = check.to_string();
        assert!(s.contains("exec_time"));
        assert!(s.contains("42"));
        assert!(s.contains("s"));
    }

    #[test]
    fn test_check_display_no_perfdata_when_label_empty() {
        let mut check = Check::new();
        check.add(Box::new("alert"), Perfdata {
            value: 42.0,
            uom: "s",
            thresholds: no_thresholds(),
            min: None,
            max: None,
        });
        let s = check.to_string();
        assert!(!s.contains('|'));
    }
}

struct OptionDisplay<T>
where
    T: Display,
{
    o: Option<T>,
}

impl<T> Display for OptionDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.o {
            None => Ok(()),
            Some(v) => v.fmt(f),
        }
    }
}

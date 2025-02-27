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

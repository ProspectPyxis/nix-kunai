use clap::ValueEnum;
use env_logger::fmt::style as anstyle;
use log::{Level, LevelFilter};
use std::io::Write;

#[derive(Clone, Copy, ValueEnum)]
pub enum LevelFilterArg {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LevelFilterArg> for LevelFilter {
    fn from(value: LevelFilterArg) -> Self {
        match value {
            LevelFilterArg::Off => LevelFilter::Off,
            LevelFilterArg::Trace => LevelFilter::Trace,
            LevelFilterArg::Debug => LevelFilter::Debug,
            LevelFilterArg::Info => LevelFilter::Info,
            LevelFilterArg::Warn => LevelFilter::Warn,
            LevelFilterArg::Error => LevelFilter::Error,
        }
    }
}

pub fn init_logger(level_filter: LevelFilter) {
    env_logger::builder()
        .filter_level(level_filter)
        .format(|buf, record| {
            let log_style = anstyle::Style::new().bold();
            let log_style = log_style.fg_color(Some(
                (match record.level() {
                    Level::Trace => anstyle::AnsiColor::Magenta,
                    Level::Debug => anstyle::AnsiColor::Blue,
                    Level::Info => anstyle::AnsiColor::Green,
                    Level::Warn => anstyle::AnsiColor::Yellow,
                    Level::Error => anstyle::AnsiColor::Red,
                })
                .into(),
            ));

            writeln!(
                buf,
                "{} {log_style}{:5}{log_style:#} {}",
                buf.timestamp_seconds(),
                record.level(),
                record.args()
            )
        })
        .init();
}

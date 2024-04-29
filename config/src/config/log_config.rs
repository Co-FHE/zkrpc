use std::{default, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogRotation {
    Minutely,
    Hourly,
    Daily,
    Never,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogConfig {
    pub log_dir: PathBuf,
    pub log_level: LogLevel,
    pub show_file_path: bool,
    pub show_line_number: bool,
    pub show_with_target: bool,
    pub write_to_file: bool,
    pub rotation: LogRotation,
}
impl Into<tracing_appender::rolling::Rotation> for LogRotation {
    fn into(self) -> tracing_appender::rolling::Rotation {
        match self {
            LogRotation::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
            LogRotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            LogRotation::Daily => tracing_appender::rolling::Rotation::DAILY,
            LogRotation::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}
// implement into tracing::LevelFilter for LogLevel
impl Into<tracing::Level> for LogLevel {
    fn into(self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
// implement into tracing::LevelFilter for LogLevel
impl Into<tracing::level_filters::LevelFilter> for LogLevel {
    fn into(self) -> tracing::level_filters::LevelFilter {
        match self {
            LogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
            LogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
            LogLevel::Info => tracing::level_filters::LevelFilter::INFO,
            LogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
            LogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
        }
    }
}
impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl default::Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            log_level: LogLevel::Trace,
            show_file_path: true,
            show_line_number: true,
            show_with_target: true,
            write_to_file: false,
            rotation: LogRotation::Daily,
        }
    }
}

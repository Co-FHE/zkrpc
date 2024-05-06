use std::{
    default,
    fmt::{Debug, Display},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum LogRotation {
    Minutely,
    Hourly,
    Daily,
    Never,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub enum LogFormat {
    OneLine,
    Pretty,
    TokioConsole,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct LogConfig {
    pub format: LogFormat,
    pub log_dir: PathBuf,
    pub log_level: LogLevel,
    pub show_source_location: bool,
    pub show_with_target: bool,
    pub show_thread_ids: bool,
    pub show_thread_names: bool,
    pub write_to_file: bool,
    pub rotation: LogRotation,
    pub show_span_duration: bool,
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
impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
// implement into tracing::LevelFilter for LogLevel
impl From<LogLevel> for tracing::level_filters::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
            LogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
            LogLevel::Info => tracing::level_filters::LevelFilter::INFO,
            LogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
            LogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
        }
    }
}
// implement into tracing::LevelFilter for LogLevel
impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => log::LevelFilter::Trace,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
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
impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl default::Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::OneLine,
            log_dir: PathBuf::from("logs"),
            log_level: LogLevel::Trace,
            show_source_location: true,
            show_thread_ids: false,
            show_thread_names: false,
            show_with_target: true,
            write_to_file: false,
            rotation: LogRotation::Daily,
            show_span_duration: false,
        }
    }
}

use anyhow::Ok;
use config::config::LogConfig;
use tracing::level_filters::LevelFilter;
use tracing::{Level, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{self, SubscriberBuilder};
use tracing_subscriber::EnvFilter;

pub fn initialize_logging(cfg: &LogConfig) {
    let filter = EnvFilter::new(&cfg.log_level);
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, "./logs", "log_file.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .with_writer(non_blocking)
        .with_file(cfg.file_path)
        .with_line_number(cfg.line_number)
        .with_target(cfg.with_target)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::config::LogLevel;
    use tracing::{self, debug, error_span, info, span, trace, warn, Level};

    #[tokio::test]
    async fn test_logging() {
        initialize_logging(&LogConfig::default());

        let span = span!(Level::INFO, "my_span");
        let _enter = span.enter();

        info!("This event will be recorded in the context of 'my_span'");
        trace!("This event will be recorded in the context of 'my_span'");
        debug!("This event will be recorded in the context of 'my_span'");
        warn!("This event will be recorded in the context of 'my_span'");
        error_span!("This event will be recorded in the context of 'my_span'");
    }
}

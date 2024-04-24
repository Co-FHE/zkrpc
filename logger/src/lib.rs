use config::config::LogConfig;
use config::config::BASE_CONFIG;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;
pub fn initialize_logger(cfg: &LogConfig) -> WorkerGuard {
    let filter = EnvFilter::new(&cfg.log_level);
    let (non_blocking, guard) = if cfg.write_to_file {
        let file_appender = RollingFileAppender::new(
            cfg.clone().rotation.into(),
            BASE_CONFIG.root_path.join(cfg.log_dir.as_os_str()),
            "zklog.log",
        );
        tracing_appender::non_blocking(file_appender)
    } else {
        tracing_appender::non_blocking(std::io::stdout())
    };

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .with_writer(non_blocking)
        .with_file(cfg.show_file_path)
        .with_line_number(cfg.show_line_number)
        .with_target(cfg.show_with_target)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    guard
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::config::EnvironmentKind;
    use tracing::{self, debug, error_span, info, span, trace, warn, Level};

    #[tokio::test]
    async fn test_logger() {
        assert_eq!(BASE_CONFIG.env, EnvironmentKind::Testing);
        let lc = LogConfig::default();
        let _guard = initialize_logger(&lc);

        let span = span!(Level::INFO, "my_span");
        let _enter = span.enter();

        info!("This event will be recorded in the context of 'my_span'");
        trace!("This event will be recorded in the context of 'my_span'");
        debug!("This event will be recorded in the context of 'my_span'");
        warn!("This event will be recorded in the context of 'my_span'");
        error_span!("This event will be recorded in the context of 'my_span'");
    }
}

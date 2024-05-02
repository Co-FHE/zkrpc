use colored::*;
use config::LogConfig;
use config::BASE_CONFIG;
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;

use crate::format::LogFormat;
mod format;
lazy_static! {
    static ref INIT: Mutex<bool> = Mutex::new(false);
}

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
    println!(
        "Initialized `logger` with : {}={}, {}={:?}, {}={}, {}={}, {}={}, {}={}, {}={}, {}={:?}",
        "log_dir".yellow(),
        BASE_CONFIG
            .root_path
            .join(cfg.log_dir.as_os_str())
            .display(),
        "log_level".yellow(),
        cfg.log_level,
        "show_source_location".yellow(),
        cfg.show_source_location,
        "show_thread_ids".yellow(),
        cfg.show_thread_ids,
        "show_thread_names".yellow(),
        cfg.show_thread_names,
        "show_with_target".yellow(),
        cfg.show_with_target,
        "write_to_file".yellow(),
        cfg.write_to_file,
        "rotation".yellow(),
        cfg.rotation
    );
    use tracing_subscriber::fmt::format;
    use tracing_subscriber::prelude::*;

    let formatter = format::debug_fn(|writer, field, value| {
        if field.name() == "message" {
            return write!(writer, "{:?}", value);
        } else {
            write!(
                writer,
                "{}={:?}",
                field.name().yellow(),
                color_eyre::owo_colors::OwoColorize::italic(&value)
            )
        }
    })
    .delimited(", ");
    match cfg.format {
        config::LogFormat::OneLine => {
            let subscriber = fmt::Subscriber::builder()
                .with_span_events(if cfg.show_span_duration {
                    FmtSpan::CLOSE
                } else {
                    FmtSpan::NONE
                })
                .event_format(
                    LogFormat::default()
                        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
                        .with_thread_ids(cfg.show_thread_ids)
                        .with_thread_names(cfg.show_thread_names)
                        .with_source_location(cfg.show_source_location)
                        .with_target(cfg.show_with_target)
                        .with_ansi(true),
                )
                .with_env_filter(filter)
                .with_writer(non_blocking)
                .fmt_fields(formatter)
                .finish();
            let mut init = INIT.lock().unwrap();
            if !*init {
                tracing::subscriber::set_global_default(subscriber)
                    .expect("Failed to set subscriber");
                *init = true;
            }
        }
        config::LogFormat::Pretty => {
            let subscriber = fmt::Subscriber::builder()
                .pretty()
                .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
                .with_thread_ids(cfg.show_thread_ids)
                .with_thread_names(cfg.show_thread_names)
                // .with_source_location(cfg.show_source_location)
                .with_target(cfg.show_with_target)
                .with_ansi(true)
                .with_env_filter(filter)
                .with_writer(non_blocking)
                .fmt_fields(formatter)
                .finish();
            let mut init = INIT.lock().unwrap();
            if !*init {
                tracing::subscriber::set_global_default(subscriber)
                    .expect("Failed to set subscriber");
                *init = true;
            }
        }
    };
    guard
}
#[macro_export]
macro_rules! init_logger_for_test {
    () => {
        $crate::initialize_logger(&config::LogConfig::default())
    };
}
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use config::EnvironmentKind;
    use tracing::{self, debug, debug_span, error_span, info, info_span, span, trace, warn, Level};

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
    #[tracing::instrument]
    fn test(n: u64) {
        std::thread::sleep(Duration::from_secs(n));
    }
    #[test]
    fn test_layer() {
        let outer_span = info_span!("outer", level = 0, other_field = tracing::field::Empty);
        let _outer_entered = outer_span.enter();
        // Some code...
        outer_span.record("other_field", &7);
        {
            let inner_span = debug_span!("inner", level = 1);
            let _inner_entered = inner_span.enter();
        }
        info!(a_bool = true, answer = 42, message = "first example");
    }
    #[test]
    fn test_span() {
        let _guard = init_logger_for_test!();
        use tracing::{event, span, Level};

        // records an event outside of any span context:
        event!(Level::INFO, "something happened");

        let span = span!(Level::INFO, "my_span");
        let _guard = span.enter();

        // records an event within "my_span".
        event!(Level::DEBUG, "something happened inside my_span");
    }
    #[test]
    fn test_nu_ansi_term() {
        use nu_ansi_term::Color;
        println!("{}", Color::Blue.paint("This is a blue message"));
        println!("{}", Color::Cyan.paint("This is a cyan message"));
        println!(
            "{}",
            Color::Green.dimmed().paint("This is a light blue message")
        );
        println!(
            "{}",
            Color::Green.bold().paint("This is a bold green message")
        );
        println!(
            "{}",
            Color::Green.italic().paint("INFO This is a green message")
        );
        println!("{}", Color::Red.paint("This is a red message"));
        println!(
            "{}",
            Color::Red
                .on(Color::Blue)
                .paint("This is an red message on blue")
        );
        println!(
            "{}",
            Color::Rgb(0, 255, 0)
                .dimmed()
                .paint("This is an rgb message")
        );
        println!(
            "{}",
            Color::Fixed(135)
                .on(Color::Fixed(28))
                .paint("This is an fixed color message")
        );
        println!(
            "{}",
            Color::Purple.bold().paint("This is an purple message")
        );
        println!("{}", Color::Magenta.paint("This is an magenta message"));
    }
    #[test]
    fn test_color() {
        use colored::Colorize;

        println!("{}", "this is blue".blue());
        println!("{}", "this is red".red());
        println!("{}", "this is red on blue".red().on_blue());
        println!("{}", "this is also red on blue".on_blue().red());
        println!(
            "{}",
            "you can use truecolor values too!".truecolor(0, 255, 136)
        );
        println!(
            "{}",
            "background truecolor also works :)".on_truecolor(135, 28, 167)
        );
        println!("{}", "you can also make bold comments".bold());
        println!(
            "{} {} {}",
            "or use".cyan(),
            "any".italic().yellow(),
            "string type".cyan()
        );
        println!("{}", "or change advice. This is red".yellow().blue().red());
        println!(
            "{}",
            "or clear things up. This is default color and style"
                .red()
                .bold()
                .clear()
        );
        println!("{}", "purple and magenta are the same".purple().magenta());
        println!(
            "{}",
            "bright colors are also allowed"
                .bright_blue()
                .on_bright_white()
        );
        println!(
            "{}",
            "you can specify color by string"
                .color("blue")
                .on_color("red")
        );
        println!("{}", "and so are normal and clear".normal().clear());
        println!("{}", String::from("this also works!").green().bold());
        println!(
            "{}",
            format!(
                "{:30}",
                "format works as expected. This will be padded".blue()
            )
        );
        println!(
            "{}",
            format!(
                "{:.3}",
                "and this will be green but truncated to 3 chars".green()
            )
        );
    }
    // #[test]
    // fn test_time() {
    //     use tracing::{info, info_span};

    //     // start the span
    //     let start = Instant::now();
    //     let span = info_span!("my_span", ?start);
    //     let _enter = span.enter();

    //     // your code here...

    //     // at the end of the span
    //     info!(duration = %start.elapsed().as_secs_f32());
    // }
}

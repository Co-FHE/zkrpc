//! Formatters for logging `tracing` events.
//!
//! This module provides several formatter implementations, as well as utilities
//! for implementing custom formatters.
//!
//! # Formatters
//! This module provides a number of formatter implementations:
//!
//! * [`Full`]: The default formatter. This emits human-readable,
//!   single-line logs for each event that occurs, with the current span context
//!   displayed before the formatted representation of the event. See
//!   [here](Full#example-output) for sample output.
//!
//! * [`Compact`]: A variant of the default formatter, optimized for
//!   short line lengths. Fields from the current span context are appended to
//!   the fields of the formatted event, and span names are not shown; the
//!   verbosity level is abbreviated to a single character. See
//!   [here](Compact#example-output) for sample output.
//!
//! * [`Pretty`]: Emits excessively pretty, multi-line logs, optimized
//!   for human readability. This is primarily intended to be used in local
//!   development and debugging, or for command-line applications, where
//!   automated analysis and compact storage of logs is less of a priority than
//!   readability and visual appeal. See [here](Pretty#example-output)
//!   for sample output.
//!
//! * [`Json`]: Outputs newline-delimited JSON logs. This is intended
//!   for production use with systems where structured logs are consumed as JSON
//!   by analysis and viewing tools. The JSON output is not optimized for human
//!   readability. See [here](Json#example-output) for sample output.
//!
use nu_ansi_term::{Color, Style};
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FormatEvent};
use tracing_subscriber::fmt::{
    time::{FormatTime, SystemTime},
    FormatFields,
};
use tracing_subscriber::{fmt::FmtContext, fmt::FormattedFields, registry::LookupSpan};
#[derive(Debug, Clone)]
pub struct LogFormat<T = SystemTime> {
    pub(crate) timer: T,
    pub(crate) ansi: bool,
    pub(crate) display_timestamp: bool,
    pub(crate) display_target: bool,
    pub(crate) display_level: bool,
    pub(crate) display_thread_id: bool,
    pub(crate) display_thread_name: bool,
    pub(crate) display_filename: bool,
    pub(crate) display_line_number: bool,
}
impl Default for LogFormat<SystemTime> {
    fn default() -> Self {
        Self {
            timer: SystemTime,
            ansi: true,
            display_timestamp: true,
            display_target: true,
            display_level: true,
            display_thread_id: false,
            display_thread_name: false,
            display_filename: false,
            display_line_number: false,
        }
    }
}
impl<T> LogFormat<T> {
    pub fn with_timer<U>(self, timer: U) -> LogFormat<U> {
        LogFormat {
            timer,
            ansi: self.ansi,
            display_timestamp: self.display_timestamp,
            display_target: self.display_target,
            display_level: self.display_level,
            display_thread_id: self.display_thread_id,
            display_thread_name: self.display_thread_name,
            display_filename: self.display_filename,
            display_line_number: self.display_line_number,
        }
    }
    #[allow(dead_code)]
    /// Do not emit timestamps with log messages.
    pub fn without_time(self) -> LogFormat<()> {
        LogFormat {
            timer: (),
            ansi: self.ansi,
            display_timestamp: false,
            display_target: self.display_target,
            display_level: self.display_level,
            display_thread_id: self.display_thread_id,
            display_thread_name: self.display_thread_name,
            display_filename: self.display_filename,
            display_line_number: self.display_line_number,
        }
    }
    /// Enable ANSI terminal colors for formatted output.
    pub fn with_ansi(self, ansi: bool) -> LogFormat<T> {
        LogFormat { ansi, ..self }
    }

    /// Sets whether or not an event's target is displayed.
    pub fn with_target(self, display_target: bool) -> LogFormat<T> {
        LogFormat {
            display_target,
            ..self
        }
    }

    #[allow(dead_code)]
    /// Sets whether or not an event's level is displayed.
    pub fn with_level(self, display_level: bool) -> LogFormat<T> {
        LogFormat {
            display_level,
            ..self
        }
    }

    /// Sets whether or not the [thread ID] of the current thread is displayed
    /// when formatting events.
    ///
    /// [thread ID]: std::thread::ThreadId
    pub fn with_thread_ids(self, display_thread_id: bool) -> LogFormat<T> {
        LogFormat {
            display_thread_id,
            ..self
        }
    }

    /// Sets whether or not the [name] of the current thread is displayed
    /// when formatting events.
    ///
    /// [name]: std::thread#naming-threads
    pub fn with_thread_names(self, display_thread_name: bool) -> LogFormat<T> {
        LogFormat {
            display_thread_name,
            ..self
        }
    }

    /// Sets whether or not an event's [source code file path][file] is
    /// displayed.
    ///
    /// [file]: tracing_core::Metadata::file
    pub fn with_file(self, display_filename: bool) -> LogFormat<T> {
        LogFormat {
            display_filename,
            ..self
        }
    }

    /// Sets whether or not an event's [source code line number][line] is
    /// displayed.
    ///
    /// [line]: tracing_core::Metadata::line
    pub fn with_line_number(self, display_line_number: bool) -> LogFormat<T> {
        LogFormat {
            display_line_number,
            ..self
        }
    }

    /// Sets whether or not the source code location from which an event
    /// originated is displayed.
    ///
    /// This is equivalent to calling [`Format::with_file`] and
    /// [`Format::with_line_number`] with the same value.
    pub fn with_source_location(self, display_location: bool) -> Self {
        self.with_line_number(display_location)
            .with_file(display_location)
    }

    #[inline]
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result
    where
        T: FormatTime,
    {
        // If timestamps are disabled, do nothing.
        if !self.display_timestamp {
            return Ok(());
        }

        // If ANSI color codes are enabled, format the timestamp with ANSI
        // colors.
        if writer.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(writer, "{}", style.prefix())?;

            // If getting the timestamp failed, don't bail --- only bail on
            // formatting errors.
            if self.timer.format_time(writer).is_err() {
                writer.write_str("<unknown time>")?;
            }

            write!(writer, "{} ", style.suffix())?;
            return Ok(());
        }

        // Otherwise, just format the timestamp without ANSI formatting.
        // If getting the timestamp failed, don't bail --- only bail on
        // formatting errors.
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }
}

trait LevelNames {
    const TRACE_STR: &'static str;
    const DEBUG_STR: &'static str;
    const INFO_STR: &'static str;
    const WARN_STR: &'static str;
    const ERROR_STR: &'static str;

    fn format_level(level: Level, ansi: bool) -> FmtLevel<Self> {
        FmtLevel {
            level,
            ansi,
            _f: PhantomData,
        }
    }
}
impl<T> LevelNames for LogFormat<T> {
    const TRACE_STR: &'static str = "TRACE";
    const DEBUG_STR: &'static str = "DEBUG";
    const INFO_STR: &'static str = " INFO";
    const WARN_STR: &'static str = " WARN";
    const ERROR_STR: &'static str = "ERROR";
}
struct FmtLevel<F: ?Sized> {
    level: Level,
    ansi: bool,
    _f: PhantomData<fn(F)>,
}
impl<F: LevelNames> fmt::Display for FmtLevel<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ansi {
            return match self.level {
                Level::TRACE => write!(f, "{}", Color::Purple.paint(F::TRACE_STR)),
                Level::DEBUG => write!(f, "{}", Color::Blue.paint(F::DEBUG_STR)),
                Level::INFO => write!(f, "{}", Color::Green.paint(F::INFO_STR)),
                Level::WARN => write!(f, "{}", Color::Yellow.paint(F::WARN_STR)),
                Level::ERROR => write!(f, "{}", Color::Red.paint(F::ERROR_STR)),
            };
        }

        match self.level {
            Level::TRACE => f.pad(F::TRACE_STR),
            Level::DEBUG => f.pad(F::DEBUG_STR),
            Level::INFO => f.pad(F::INFO_STR),
            Level::WARN => f.pad(F::WARN_STR),
            Level::ERROR => f.pad(F::ERROR_STR),
        }
    }
}
impl<F: LevelNames> FmtLevel<F> {
    pub(crate) fn style_color_by_level(&self) -> Style {
        if self.ansi {
            match self.level {
                Level::TRACE => Color::Purple.normal(),
                Level::DEBUG => Color::Blue.normal(),
                Level::INFO => Color::Green.normal(),
                Level::WARN => Color::Yellow.normal(),
                Level::ERROR => Color::Red.normal(),
            }
        } else {
            Style::new()
        }
    }
}
struct FmtThreadName<'a> {
    name: &'a str,
}

impl<'a> FmtThreadName<'a> {
    pub(crate) fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl<'a> fmt::Display for FmtThreadName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::sync::atomic::{
            AtomicUsize,
            Ordering::{AcqRel, Acquire, Relaxed},
        };

        // Track the longest thread name length we've seen so far in an atomic,
        // so that it can be updated by any thread.
        static MAX_LEN: AtomicUsize = AtomicUsize::new(0);
        let len = self.name.len();
        // Snapshot the current max thread name length.
        let mut max_len = MAX_LEN.load(Relaxed);

        while len > max_len {
            // Try to set a new max length, if it is still the value we took a
            // snapshot of.
            match MAX_LEN.compare_exchange(max_len, len, AcqRel, Acquire) {
                // We successfully set the new max value
                Ok(_) => break,
                // Another thread set a new max value since we last observed
                // it! It's possible that the new length is actually longer than
                // ours, so we'll loop again and check whether our length is
                // still the longest. If not, we'll just use the newer value.
                Err(actual) => max_len = actual,
            }
        }

        // pad thread name using `max_len`
        write!(f, "{:>width$}", self.name, width = max_len)
    }
}

struct StyleWrap;
#[allow(dead_code)]
impl StyleWrap {
    fn dimmed(ansi: bool) -> Style {
        if ansi {
            Style::new().dimmed()
        } else {
            Style::new()
        }
    }
    fn bold(ansi: bool) -> Style {
        if ansi {
            Style::new().bold()
        } else {
            Style::new()
        }
    }
    fn italic(ansi: bool) -> Style {
        if ansi {
            Style::new().italic()
        } else {
            Style::new()
        }
    }
    fn underline(ansi: bool) -> Style {
        if ansi {
            Style::new().underline()
        } else {
            Style::new()
        }
    }
    fn blink(ansi: bool) -> Style {
        if ansi {
            Style::new().blink()
        } else {
            Style::new()
        }
    }
    fn reverse(ansi: bool) -> Style {
        if ansi {
            Style::new().reverse()
        } else {
            Style::new()
        }
    }
    fn hidden(ansi: bool) -> Style {
        if ansi {
            Style::new().hidden()
        } else {
            Style::new()
        }
    }
    fn strikethrough(ansi: bool) -> Style {
        if ansi {
            Style::new().strikethrough()
        } else {
            Style::new()
        }
    }
}

impl<S, N, T> FormatEvent<S, N> for LogFormat<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        // if the `Format` struct *also* has an ANSI color configuration,
        // override the writer...the API for configuring ANSI color codes on the
        // `Format` struct is deprecated, but we still need to honor those
        // configurations.

        self.format_timestamp(&mut writer)?;
        let fmt_level = Self::format_level(*meta.level(), writer.has_ansi_escapes());
        let level_color_style = fmt_level.style_color_by_level();
        if self.display_level {
            write!(writer, "{} ", fmt_level)?;
        }

        if self.display_thread_name {
            let current_thread = std::thread::current();
            match current_thread.name() {
                Some(name) => {
                    write!(
                        writer,
                        "{} ",
                        level_color_style.dimmed().paint(format!(
                            "[{}{}] ",
                            FmtThreadName::new(name),
                            if self.display_thread_id {
                                format!("({:0>2?})", std::thread::current().id())
                            } else {
                                "".to_string()
                            }
                        ))
                    )?;
                }
                // fall-back to thread id when name is absent and ids are not enabled
                None if !self.display_thread_id => {
                    write!(
                        writer,
                        "{}",
                        level_color_style
                            .dimmed()
                            .paint(format!("[{:0>2?}] ", current_thread.id()))
                    )?;
                }
                _ => {}
            }
        } else if self.display_thread_id {
            write!(
                writer,
                "{}",
                level_color_style
                    .dimmed()
                    .paint(format!("[{:0>2?}] ", std::thread::current().id()))
            )?;
        }
        let dimmed = StyleWrap::dimmed(writer.has_ansi_escapes());
        if let Some(scope) = ctx.event_scope() {
            // let bold = StyleWrap::bold(writer.has_ansi_escapes());

            let mut seen = false;
            let mut first = true;

            for span in scope.from_root() {
                if !first {
                    write!(writer, "{}", level_color_style.underline().paint("::"))?;
                }
                first = false;
                write!(
                    writer,
                    "{}",
                    level_color_style.underline().paint(span.metadata().name())
                )?;
                seen = true;

                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write!(
                            writer,
                            "{}",
                            format!("{}{}{}", "{", { fields }, "}") // .dimmed()
                                                                    // .underline() // StyleWrap::underline(writer.has_ansi_escapes())
                                                                    //     .dimmed()
                                                                    //     .paint(format!("{}{}{}", "{", fields, "}"))
                        )?;
                    }
                }
            }

            if seen {
                writer.write_char(' ')?;
            }
        }

        if self.display_target {
            write!(
                writer,
                "{}{} ",
                level_color_style.dimmed().paint(meta.target()),
                level_color_style.dimmed().paint(":")
            )?;
        }

        let line_number = if self.display_line_number {
            meta.line()
        } else {
            None
        };

        if self.display_filename {
            if let Some(filename) = meta.file() {
                write!(
                    writer,
                    "{}{}{}",
                    dimmed.paint(filename),
                    dimmed.paint(":"),
                    if line_number.is_some() { "" } else { " " }
                )?;
            }
        }

        if let Some(line_number) = line_number {
            write!(
                writer,
                "{}{}:{} ",
                dimmed.prefix(),
                line_number,
                dimmed.suffix()
            )?;
        }

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

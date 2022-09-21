use std::fmt::Result as FmtResult;

use time::format_description::{
    modifier::{Day, Hour, Minute, Month, Second, Year},
    Component, FormatItem,
};
use tracing::{Event, Subscriber};
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling,
};
use tracing_subscriber::{
    fmt::{
        format::Writer,
        time::{FormatTime, UtcTime},
        FmtContext, FormatEvent, FormatFields, Layer,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer as _,
};

pub fn init() -> WorkerGuard {
    // Displays ERROR, WARN, INFO, and DEBUG from shishabot
    // and ERROR, WARN, INFO from dependencies
    let stdout_filter: EnvFilter = "shishabot=debug,info".parse().unwrap();

    let stdout_layer = Layer::default()
        .event_format(StdoutEventFormat::default())
        .with_filter(stdout_filter);

    let file_appender = rolling::daily("./logs", "shishabot.log");
    let (file_writer, guard) = NonBlocking::new(file_appender);

    // Check RUST_LOG in .env, if it's not found it'll
    // display ERROR, WARN, INFO, DEBUG, and TRACE from shishabot
    // and ERROR, WARN, INFO from dependencies.
    let file_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => "shishabot=trace,info".parse().unwrap(),
    };

    let file_layer = Layer::default()
        .event_format(FileEventFormat::default())
        .with_writer(file_writer)
        .with_filter(file_filter);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}

struct StdoutEventFormat {
    timer: UtcTime<&'static [FormatItem<'static>]>,
}

impl Default for StdoutEventFormat {
    fn default() -> Self {
        Self {
            timer: UtcTime::new(DATETIME_FORMAT),
        }
    }
}

impl<S, N> FormatEvent<S, N> for StdoutEventFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> FmtResult {
        self.timer.format_time(&mut writer)?;
        let metadata = event.metadata();

        write!(writer, " {:>5} ", metadata.level(),)?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

struct FileEventFormat {
    timer: UtcTime<&'static [FormatItem<'static>]>,
}

impl Default for FileEventFormat {
    fn default() -> Self {
        Self {
            timer: UtcTime::new(DATETIME_FORMAT),
        }
    }
}

impl<S, N> FormatEvent<S, N> for FileEventFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> FmtResult {
        self.timer.format_time(&mut writer)?;
        let metadata = event.metadata();

        write!(
            writer,
            " {:>5} [{}:{}] ",
            metadata.level(),
            metadata.file().unwrap_or_else(|| metadata.target()),
            metadata.line().unwrap_or(0),
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub const DATE_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Year(Year::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Month(Month::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Day(Day::default())),
];

pub const TIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Hour(<Hour>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Minute(<Minute>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Second(<Second>::default())),
];

pub const DATETIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Compound(DATE_FORMAT),
    FormatItem::Literal(b" "),
    FormatItem::Compound(TIME_FORMAT),
];

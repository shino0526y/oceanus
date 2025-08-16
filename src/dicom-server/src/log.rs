pub struct Formatter {
    is_tty: bool,
}

impl Formatter {
    pub fn new(is_tty: bool) -> Self {
        Self { is_tty }
    }
}

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for Formatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::format::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // 日時
        let now = chrono::Local::now().to_rfc3339();

        // ログレベル
        let level = if self.is_tty {
            match *event.metadata().level() {
                tracing::Level::ERROR => "\x1b[31mERROR\x1b[0m", // red
                tracing::Level::WARN => "\x1b[33mWARN\x1b[0m",   // yellow
                tracing::Level::INFO => "\x1b[32mINFO\x1b[0m",   // green
                tracing::Level::DEBUG => "\x1b[34mDEBUG\x1b[0m", // blue
                tracing::Level::TRACE => "\x1b[90mTRACE\x1b[0m", // gray
            }
        } else {
            match *event.metadata().level() {
                tracing::Level::ERROR => "ERROR",
                tracing::Level::WARN => "WARN",
                tracing::Level::INFO => "INFO",
                tracing::Level::DEBUG => "DEBUG",
                tracing::Level::TRACE => "TRACE",
            }
        };

        write!(writer, "[{now} {level}]: ")?;

        // Span
        if let Some(scope) = ctx.event_scope() {
            if let Some(span) = scope.from_root().last() {
                if let Some(fields) = span
                    .extensions()
                    .get::<tracing_subscriber::fmt::FormattedFields<N>>()
                {
                    write!(writer, "[{} ", span.metadata().name())?;
                    if !fields.is_empty() {
                        write!(writer, "({})] ", fields.as_str())?;
                    }
                }
            }
        }

        // メッセージ本文
        let _ = ctx.field_format().format_fields(writer.by_ref(), event);
        writeln!(writer)
    }
}

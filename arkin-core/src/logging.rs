use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(FmtSpan::NONE)
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(false)
        .with_target(true)
        .with_line_number(false)
        .with_file(false)
        .with_ansi(true)
        .compact()
        .init();
}

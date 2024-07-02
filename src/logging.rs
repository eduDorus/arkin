use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(EnvFilter::from_default_env())
        .with_thread_ids(false)
        .with_target(false)
        // .with_span_events(FmtSpan::NONE)
        .with_line_number(false)
        .with_file(false)
        .with_ansi(true)
        .compact()
        .init();
}

pub fn init_test_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .with_test_writer() // This is the important part
        .compact()
        .init();
}

use tracing::{span, Span};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format},
    layer::SubscriberExt,
    prelude::*,
    EnvFilter,
};

pub fn init_log() -> (WorkerGuard, Span) {
    let file_appender = tracing_appender::rolling::never("./", "tx-coordinator.log");
    let (non_blocking, _file_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .event_format(format().compact())
        .with_writer(non_blocking);
    let std_layer = fmt::layer()
        .event_format(format().compact())
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(file_layer)
        .with(std_layer)
        .init();

    let root = span!(tracing::Level::INFO, "tx-coordinator");

    return (_file_guard, root);
}

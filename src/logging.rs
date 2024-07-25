use tracing::{error, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub(crate) fn enable_logging(verbose: u8) {
    let log_level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        2 => Level::TRACE,
        _ => Level::TRACE,
    };

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(tracing_subscriber::fmt::layer().with_target(false));

    match tracing_journald::layer() {
        Ok(layer) => {
            registry.with(layer).init();
        }
        // journald is typically available on Linux systems, but nowhere else. Portable software
        // should handle its absence gracefully.
        Err(e) => {
            registry.init();
            error!("couldn't connect to journald: {}", e);
        }
    }
}

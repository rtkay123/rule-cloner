use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rule_cloner=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let name = env!("CARGO_PKG_NAME");
    let msg = format!("{name} has started");
    info!("{msg} has started");
}

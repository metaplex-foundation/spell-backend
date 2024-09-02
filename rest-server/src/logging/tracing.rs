use std::{io, sync::Arc};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn set_up_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_thread_names(true)
        .with_writer(Arc::new(io::stdout()));

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_env("LOG_LEVEL"))
        .init();
}

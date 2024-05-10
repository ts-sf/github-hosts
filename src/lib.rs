pub mod consts;
mod dns;

pub mod dns_server;
pub mod executor;

pub mod util;
pub mod version;

#[cfg(test)]
pub fn config_logger(log: &str) {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

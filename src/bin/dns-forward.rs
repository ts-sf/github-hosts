#![allow(unused_parens)]

use std::net::SocketAddr;

use github_hosts::{dns_server, version::VERSION};
use tracing_subscriber::{prelude::*, EnvFilter};

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Target {
    Github,
    File,
}

#[derive(Debug, Parser)]
#[command(name = "dns-forward")]
#[command(about = "A dns forward service", long_about = None, version = VERSION)]
struct Arg {
    /// server at
    #[arg(long,default_value_t = ("127.0.0.1".to_owned()))]
    address: String,

    /// remote dns server
    #[arg(long, default_value_t = ("8.8.8.8".to_owned()))]
    resolver: String,
}

#[tokio::main]
async fn main() {
    let env = EnvFilter::from_env("RUST_LOG");
    tracing_subscriber::registry()
        .with(env)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let arg = Arg::parse();

    let Arg { address, resolver } = arg;

    let server_at = format!("{}:53", address);
    let remote_dns = format!("{}:53", resolver);

    server_at
        .parse::<SocketAddr>()
        .expect("address parse error");
    remote_dns
        .parse::<SocketAddr>()
        .expect("resolver parse error");

    dns_server::server(&server_at, &remote_dns).await.expect("server panic");
}

use anyhow::Error;
use clap::Parser;

use crate::cli::Cli;

mod acme;
mod cache;
mod check;
mod cli;
mod configuration;
mod core;
mod dns;
mod firewall;
mod http;
mod management;
mod metrics;
mod nns;
mod persist;
mod rate_limiting;
mod routes;
mod snapshot;
mod tls_verify;

#[cfg(feature = "tls")]
mod tls;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    core::main(cli).await
}

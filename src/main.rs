//! Supercode - Orchestration system for managing multiple coding agent sessions

mod agent;
mod cli;
mod config;
mod core;
mod db;
mod mcp;
mod session;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "supercode=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Supercode v{}", env!("CARGO_PKG_VERSION"));

    // Run CLI
    cli::run()?;

    Ok(())
}

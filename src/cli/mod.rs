//! CLI module

pub mod commands;

pub fn run() -> anyhow::Result<()> {
    commands::run()
}

use anyhow::Result;
use clap::Parser;
use ferris_agent::{Cli, app};

#[tokio::main]
async fn main() -> Result<()> {
    app::run(Cli::parse()).await
}

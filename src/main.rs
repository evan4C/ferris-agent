use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use ferris_agent::{ChatUsage, Cli, app, Model};

#[tokio::main]
async fn main() -> Result<()> {
    app::exec(Cli::parse()).await
}

/// Prints the assistant's reply inside a bordered block.
fn print_reply(reply: &str) {
    let divider = "─".repeat(60).bright_black();
    println!();
    println!("{}", divider);
    println!("{}", reply.trim());
    println!("{}", divider);
}

/// Prints token usage and estimated cost for the most recent turn, if reported.
fn print_usage(usage: Option<&ChatUsage>, model: &Model) {
    let Some(usage) = usage else { return };
    let cost = usage.cost_usd(model);
    println!(
        "{} {} {} {} {} {} {} {}",
        "tokens:".bright_black(),
        "prompt".dimmed(),
        usage.prompt_tokens.to_string().yellow(),
        "completion".dimmed(),
        usage.completion_tokens.to_string().yellow(),
        "total".dimmed(),
        usage.total_tokens.to_string().yellow(),
        format!("(${:.6})", cost).green().bold(),
    );
}

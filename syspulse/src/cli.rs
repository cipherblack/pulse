use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "syspulse", about = "Advanced system monitoring and process management tool")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Monitor system in real-time with process management, email, and backup
  Monitor {
    #[arg(short, long, default_value_t = 5)]
    interval: u64,
    #[arg(short, long, default_value_t = 600)]
    backup_interval: u64,
    #[arg(long)]
    email: Option<String>,
  },
  
  /// Generate a one-time system report
  Report,
}
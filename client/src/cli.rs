use clap::{Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;
use tracing::Level;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub address: IpAddr,
    #[arg(short, long)]
    pub port: u16,
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short, long, default_value = "info")]
    pub verbose: Level,
}

#[derive(Subcommand)]
pub enum Commands {
    Download {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        directory: PathBuf,
    },
    Upload {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        directory: PathBuf,
    },
    List,
}

use anyhow::Result;
use clap::Parser;
use server::{cli::Cli, server_main};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(args.verbose)
        .init();

    // println!("args: {:?}", &args.address);
    // let srv_path = "127.0.0.1:3000".to_string();
    server_main(&args).await?;


    Ok(())
}

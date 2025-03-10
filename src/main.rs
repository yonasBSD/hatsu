#[cfg(feature = "snmalloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use clap::{Parser, Subcommand};
use hatsu_utils::AppError;
use human_panic::{metadata, setup_panic};

mod commands;
mod utils;

#[derive(Debug, Parser)]
#[command(
    styles = utils::styles(),
    name = "hatsu",
    version = hatsu_utils::VERSION,
    about,
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_panic!(metadata!().homepage("https://github.com/importantimport/hatsu/issues"));

    let args = Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Run => commands::run().await,
        }
    } else {
        commands::run().await
    }
}

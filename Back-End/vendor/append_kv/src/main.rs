mod store;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use store::KvStore;

#[derive(Parser)]
#[command(name = "append_kv")]
#[command(about = "A simple append-only key-value store", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sets a key-value pair
    Set { key: String, value: String },
    /// Gets a value by key
    Get { key: String },
    /// Removes a key
    Rm { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = KvStore::open("data/kv.log")?;

    match cli.command {
        Commands::Set { key, value } => {
            store.set(key, value)?;
            println!("OK");
        }
        Commands::Get { key } => {
            match store.get(&key)? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        Commands::Rm { key } => {
            store.remove(key)?;
            println!("OK");
        }
    }

    Ok(())
}
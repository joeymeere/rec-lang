use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rec")]
#[command(about = "REC configuration file tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a REC file
    Validate {
        /// The REC file to validate
        file: PathBuf,
    },
    /// Convert REC to JSON
    ToJson {
        /// The REC file to convert
        file: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
            let content = fs::read_to_string(&file)?;
            let doc = rec::parse_rec(&content)?;
            rec::validate(&doc)?;
            println!("✓ {} is valid", file.display());
        }
        Commands::ToJson { file } => {
            let content = fs::read_to_string(&file)?;
            let doc = rec::parse_rec(&content)?;
            rec::validate(&doc)?;
            let json = serde_json::to_string_pretty(&doc.root)?;
            println!("{}", json);
        }
    }

    Ok(())
}

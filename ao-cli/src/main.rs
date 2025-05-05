use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};
use std::fs::OpenOptions;

use ao::{init, check, run, build}; // Added build

/// Top-level CLI parser
#[derive(Parser)]
#[command(name = "ao", version = "0.1.0", about = "Analytics Ops CLI orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new modeling project
    Init {
        /// Name of the project to initialize
        name: String,
    },
    /// Run linting and tests on a project
    Check {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run a defined task from ao.toml
    Run {
        /// Name of the task to run
        task_name: String,
        /// Path within the project directory (optional, defaults to current dir)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Build Docker images for the project services
    Build {
        /// Path within the project directory (optional, defaults to current dir)
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() -> Result<()> {
    // Initialize JSON logger to file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ao-cli.log")
        .expect("Failed to open log file");
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => init::run(name)?,
        Commands::Check { path } => check::run(path)?,
        Commands::Run { task_name, path } => run::run(task_name, path)?,
        Commands::Build { path } => build::run(path)?,
    }

    Ok(())
}
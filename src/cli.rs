use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

/// ACVPeppers : Rust ACVP testing framework CLI
#[derive(Parser, Debug, Clone)]
#[command(name = "ACVpeppers", version, about)]
pub struct Cli {
    /// Path to the acvp.toml (TOML)
    #[arg(long, default_value = "src/acvp.toml", env = "ACVP_CFG")]
    pub config: PathBuf,

    /// Capability file (JSON)
    #[arg(long = "cap", default_value = "src/capabilities.json", env = "ACVP_CAP")]
    pub capability: Option<String>,

    /// Increase verbosity (-v, -vv). Increase or decrease debug logs.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (suppresses all of the info logs - cleaner execution)
    #[arg(short, long, action)]
    pub quiet: bool,

    /// Output JSON for commands that support it 
    #[arg(long, action)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Execute ACVP tests using the provided capability file
    Run(RunArgs),

    /// Show or update the configuration
    Config(ConfigCmd),

    /// Validate a certain input (e.g., hash) against ACVP requirements - MAKE SURE IT IS SUPPORTED BY ACVPeppers
    Validate(ValidateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// If true, publishable (prompt=true) session, otherwise demo/non-publish
    #[arg(long, default_value_t = false)]
    pub publish: bool,

    /// Whether to validate immediately if server supports it
    #[arg(long, default_value_t = false)]
    pub immediate: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    /// Algorithm to validate (e.g., SHA2-256)
    #[arg(long)]
    pub algorithm: String,

    /// Hex string or file path with inputs (we keep it simple here)
    #[arg(long)]
    pub input: String,

    /// Expected output (hex) to check pass/fail; if omitted, just compute & show
    #[arg(long)]
    pub expected: Option<String>,
}

/// Subcommands for config
#[derive(Args, Debug, Clone)]
pub struct ConfigCmd {
    /// Show the loaded configuration (default)
    #[arg(long, default_value_t = false)]
    pub show: bool,

    /// Update base URL
    #[arg(long)]
    pub set_base_url: Option<String>,
}
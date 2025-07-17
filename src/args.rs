use clap::{Parser, Subcommand};
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "hotline",
    version = "1.0",
    about = "CLI Tool designed to return hotline information from Sangoma Vegas",
    after_help = "If you need the output of one device, just use --start-ip\nExample: hotline scan -s 192.168.1.10 -u admin -p secret -x output.xlsx"
)]
pub struct Args {
    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-critical logging
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan Vega(s) via IP address(es) and pull configuration
    Scan(ScanArgs),
    /// Parse local config file instead of querying a Vega device
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Start IP address of range (inclusive)
    #[arg(short, long)]
    pub start_ip: Option<IpAddr>,

    /// End IP address of range (inclusive)
    #[arg(short, long, requires = "start_ip")]
    pub end_ip: Option<IpAddr>,

    /// Username for Vega web login
    #[arg(short, long, required = true)]
    pub username: String,

    /// Password for Vega web login
    #[arg(short, long, required = true)]
    pub password: String,

    /// Export results to Excel file
    #[arg(short = 'x', long, default_value = "output.xlsx")]
    pub excel_file: Option<String>,

    /// Optional JSON output file
    #[arg(short, long)]
    pub json_file: Option<String>,
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// Path to Vega config file
    #[arg(short, long, required = true)]
    pub config_file: PathBuf,

    /// Export results to Excel file
    #[arg(short = 'x', long, default_value = "output.xlsx")]
    pub excel_file: Option<String>,

    /// Optional JSON output file
    #[arg(short, long)]
    pub json_file: Option<String>,
}

pub fn get_args() -> Args {
    Args::parse()
}

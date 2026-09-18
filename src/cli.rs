/* Command-line interface for the seed finder. */

use clap::Parser;

/// Command-line arguments for mc-seed-finder
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Searches for Minecraft seeds matching given conditions"
)]
pub struct Args {
    /// Number of threads to use for the search
    #[arg(short, long, default_value_t = 4)]
    pub threads: usize,
}

pub fn parse_args() -> Args {
    Args::parse()
}

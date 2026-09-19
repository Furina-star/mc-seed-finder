//! Command-line argument parsing for mc-seed-finder.
//!
//! No dependencies on other modules (see README § Module Ownership), so this
//! file can be built and reviewed independently of `rng/`, `placement/`,
//! `biome/`, etc.
//!
//! Usage (from README):
//!   mc-seed-finder bench [count] --mode <double|quad>
//!   mc-seed-finder search [count] --mode <double|quad>

use clap::{Parser, Subcommand, ValueEnum};

/// Default number of seeds to scan when `count` is omitted.
const DEFAULT_COUNT: i64 = 10_000_000;

/// Which hut-cluster condition to search for.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Adjacent pair of swamp huts.
    Double,
    /// All four corners of a 2x2 block of swamp huts.
    Quad,
}

#[derive(Parser, Debug)]
#[command(
    name = "mc-seed-finder",
    about = "Searches Minecraft seeds for swamp-hut AFK farm clusters, \
             verified against real biome data.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Pass-1 geometry scan only, no biome check. Use this to measure raw
    /// throughput on your hardware before committing to a long `search` run.
    Bench {
        /// Number of seeds to scan, starting from 0. Fits comfortably up to
        /// 1_000_000_000+.
        #[arg(default_value_t = DEFAULT_COUNT)]
        count: i64,

        /// First seed in the scanned interval.
        #[arg(long, default_value_t = 0)]
        start_seed: i64,

        /// Number of Rayon worker threads.
        #[arg(long, default_value_t = default_threads(), value_parser = parse_threads)]
        threads: usize,

        /// Which hut-cluster condition to search for.
        #[arg(long, value_enum, default_value_t = Mode::Double)]
        mode: Mode,
    },

    /// Pass-1 geometry scan, then a real biome check (via cubiomes) on the
    /// survivors. Prints confirmed matches with block coordinates.
    Search {
        /// Number of seeds to scan, starting from 0. Fits comfortably up to
        /// 1_000_000_000+.
        #[arg(default_value_t = DEFAULT_COUNT)]
        count: i64,

        /// First seed in the scanned interval.
        #[arg(long, default_value_t = 0)]
        start_seed: i64,

        /// Number of Rayon worker threads.
        #[arg(long, default_value_t = default_threads(), value_parser = parse_threads)]
        threads: usize,

        /// Which hut-cluster condition to search for.
        #[arg(long, value_enum, default_value_t = Mode::Double)]
        mode: Mode,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_readme() {
        let cli = Cli::parse_from(["mc-seed-finder", "search"]);
        match cli.command {
            Command::Search {
                count,
                start_seed,
                threads,
                mode,
            } => {
                assert_eq!(count, DEFAULT_COUNT);
                assert_eq!(start_seed, 0);
                assert_eq!(threads, default_threads());
                assert_eq!(mode, Mode::Double);
            }
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn parses_count_and_mode() {
        let cli = Cli::parse_from(["mc-seed-finder", "search", "300000000", "--mode", "quad"]);
        match cli.command {
            Command::Search {
                count,
                start_seed,
                threads,
                mode,
            } => {
                assert_eq!(count, 300_000_000);
                assert_eq!(start_seed, 0);
                assert_eq!(threads, default_threads());
                assert_eq!(mode, Mode::Quad);
            }
            _ => panic!("expected Search"),
        }
    }

    #[test]
    fn bench_parses_too() {
        let cli = Cli::parse_from(["mc-seed-finder", "bench", "50000000", "--mode", "double"]);
        match cli.command {
            Command::Bench {
                count,
                start_seed,
                threads,
                mode,
            } => {
                assert_eq!(count, 50_000_000);
                assert_eq!(start_seed, 0);
                assert_eq!(threads, default_threads());
                assert_eq!(mode, Mode::Double);
            }
            _ => panic!("expected Bench"),
        }
    }

    #[test]
    fn parses_start_seed_and_threads() {
        let cli = Cli::parse_from([
            "mc-seed-finder",
            "search",
            "3500000",
            "--start-seed",
            "1000000",
            "--threads",
            "16",
            "--mode",
            "quad",
        ]);
        match cli.command {
            Command::Search {
                count,
                start_seed,
                threads,
                mode,
            } => {
                assert_eq!(count, 3_500_000);
                assert_eq!(start_seed, 1_000_000);
                assert_eq!(threads, 16);
                assert_eq!(mode, Mode::Quad);
            }
            _ => panic!("expected Search"),
        }
    }
}

fn parse_threads(value: &str) -> Result<usize, String> {
    let threads = value
        .parse::<usize>()
        .map_err(|_| "threads must be a positive integer".to_owned())?;
    if threads == 0 {
        return Err("threads must be greater than zero".to_owned());
    }
    Ok(threads)
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

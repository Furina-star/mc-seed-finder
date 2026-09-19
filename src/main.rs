mod cli;

use clap::Parser;
use cli::{Cli, Command, Mode};
use mc_seed_finder::{
    find_confirmed_double_from, find_confirmed_quad_from, search, DOUBLE_HUT_CONDITION,
    QUAD_HUT_CONDITION,
};

fn main() {
    let cli = Cli::parse();
    println!(
        "Threads: {} (Rayon auto-threaded)",
        rayon::current_num_threads()
    );

    match cli.command {
        Command::Bench {
            count,
            start_seed,
            mode,
        } => {
            validate_interval(start_seed, count);
            println!(
                "Benchmarking Pass-1 (geometry only, no biome filter) over seeds {start_seed}..{} [{mode:?}]...",
                start_seed + count
            );
            let elapsed_secs;
            let match_count;
            match mode {
                Mode::Double => {
                    let r = search::scan_geometry_from(start_seed, count, |seed| {
                        DOUBLE_HUT_CONDITION.evaluate(seed)
                    });
                    elapsed_secs = r.elapsed.as_secs_f64();
                    match_count = r.matches.len();
                    println!("Done in {:.2?}", r.elapsed);
                }
                Mode::Quad => {
                    let r = search::scan_geometry_from(start_seed, count, |seed| {
                        QUAD_HUT_CONDITION.evaluate(seed)
                    });
                    elapsed_secs = r.elapsed.as_secs_f64();
                    match_count = r.matches.len();
                    println!("Done in {:.2?}", r.elapsed);
                }
            }
            println!(
                "Throughput: {:.2} Million seeds/sec",
                (count as f64 / elapsed_secs) / 1_000_000.0
            );
            println!("Pass-1 geometric matches: {match_count}");
        }
        Command::Search {
            count,
            start_seed,
            mode,
        } => {
            validate_interval(start_seed, count);
            println!(
                "Pass 1: scanning seeds {start_seed}..{} for geometry-only {mode:?} hut clusters...",
                start_seed + count
            );
            let total_start = std::time::Instant::now();
            match mode {
                Mode::Double => {
                    let (pass1, confirmed) = find_confirmed_double_from(start_seed, count);
                    print_pass1_summary(pass1.matches.len(), pass1.seeds_scanned, pass1.elapsed);
                    print_total_summary(total_start.elapsed(), pass1.elapsed, count);
                    println!(
                        "\n=== CONFIRMED (geometry + real biome check) ===\n{} of {} candidates are actual swamp at every hut.",
                        confirmed.len(),
                        pass1.matches.len()
                    );
                    for (i, m) in confirmed.iter().take(10).enumerate() {
                        println!("\nMatch #{}: seed {}", i + 1, m.seed);
                        println!("  AFK spot : X: {:.1}, Z: {:.1}", m.afk_x, m.afk_z);
                        println!("  Hut 1    : X: {:.1}, Z: {:.1}", m.hut1.0, m.hut1.1);
                        println!("  Hut 2    : X: {:.1}, Z: {:.1}", m.hut2.0, m.hut2.1);
                    }
                }
                Mode::Quad => {
                    let (pass1, confirmed) = find_confirmed_quad_from(start_seed, count);
                    print_pass1_summary(pass1.matches.len(), pass1.seeds_scanned, pass1.elapsed);
                    print_total_summary(total_start.elapsed(), pass1.elapsed, count);
                    println!(
                        "\n=== CONFIRMED (geometry + real biome check) ===\n{} of {} candidates are actual swamp at every hut.",
                        confirmed.len(),
                        pass1.matches.len()
                    );
                    for (i, m) in confirmed.iter().take(10).enumerate() {
                        println!("\nMatch #{}: seed {}", i + 1, m.seed);
                        println!("  AFK spot : X: {:.1}, Z: {:.1}", m.afk_x, m.afk_z);
                        for (j, (x, z)) in m.huts.iter().enumerate() {
                            println!("  Hut {}    : X: {:.1}, Z: {:.1}", j + 1, x, z);
                        }
                    }
                }
            }
        }
    }
}

fn print_pass1_summary(match_count: usize, seeds_scanned: i64, elapsed: std::time::Duration) {
    println!(
        "Pass 1 done in {:.2?}: {} geometric candidates ({:.0} seeds/sec)",
        elapsed,
        match_count,
        seeds_scanned as f64 / elapsed.as_secs_f64()
    );
}

fn print_total_summary(total: std::time::Duration, pass1: std::time::Duration, seeds_scanned: i64) {
    let pass2 = total.saturating_sub(pass1);
    println!("Pass 2 biome validation: {:.2?}", pass2);
    println!(
        "Total end-to-end: {:.2?} ({:.0} seeds/sec)",
        total,
        seeds_scanned as f64 / total.as_secs_f64()
    );
}

fn validate_interval(start_seed: i64, count: i64) {
    if count <= 0 {
        eprintln!("error: count must be greater than zero");
        std::process::exit(2);
    }
    if start_seed.checked_add(count).is_none() {
        eprintln!("error: start-seed + count exceeds the supported i64 range");
        std::process::exit(2);
    }
}

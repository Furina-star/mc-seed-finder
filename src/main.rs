/* Main entry point for the seed finder application. */

mod cli;

fn main() {
    let args = cli::parse_args();

    // TODO: once conditions/mod.rs is ready, build a condition from args, e.g.:
    // let condition = mc_seed_finder::conditions::DoubleHutCondition::new(...);

    // TODO: once search/mod.rs is ready, call into it:
    // let matches = mc_seed_finder::search::find_seeds(condition, args.thread_count);
    // for seed in matches { println!("{seed}"); }

    println!("mc-seed-finder: pipeline not fully wired up yet niggers");
}

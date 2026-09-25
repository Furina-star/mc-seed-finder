//! Parallel loop over candidate seeds. Depends on `conditions`. Generic
//! over the match type so both DoubleHutMatch and QuadHutMatch (or any
//! future condition) reuse the same Pass-1 scan. Callers still own the
//! biome-confirmation step since that's inherently sequential (one
//! in-process cubiomes generator instance).

use rayon::prelude::*;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Pass1Result<M> {
    pub matches: Vec<M>,
    pub seeds_scanned: i64,
    pub elapsed: Duration,
}

impl<M> Pass1Result<M> {
    /// Throughput for this scan, in seeds/sec — what `bench` reports.
    pub fn seeds_per_second(&self) -> f64 {
        self.seeds_scanned as f64 / self.elapsed.as_secs_f64()
    }
}

/// Pass 1: geometry-only scan across `0..count`, fully parallel. `evaluate`
/// is whatever condition's `.evaluate()` method you're scanning with, e.g.
/// `|seed| DOUBLE_HUT_CONDITION.evaluate(seed)`. No biome filtering here —
/// see `biome::BiomeOracle` for that, run by the caller only against the
/// (much smaller) set of matches this returns.
///
/// Deliberately does not report progress mid-scan: a shared counter
/// touched from every parallel iteration would add cross-thread
/// contention across potentially a billion iterations, directly hurting
/// the throughput `bench` exists to measure. Run `bench` on a smaller
/// count first to estimate how long a full run will take instead.
pub fn scan_geometry<M, F>(count: i64, evaluate: F) -> Pass1Result<M>
where
    M: Send,
    F: Fn(i64) -> Option<M> + Sync + Send,
{
    scan_geometry_from(0, count, evaluate)
}

/// Pass 1 over the explicit interval `start_seed..start_seed + count`.
pub fn scan_geometry_from<M, F>(start_seed: i64, count: i64, evaluate: F) -> Pass1Result<M>
where
    M: Send,
    F: Fn(i64) -> Option<M> + Sync + Send,
{
    debug_assert!(count >= 0, "count should not be negative");
    let end_seed = start_seed
        .checked_add(count)
        .expect("seed interval exceeds the supported i64 range");

    let start = Instant::now();
    let matches: Vec<M> = (start_seed..end_seed)
        .into_par_iter()
        .filter_map(evaluate)
        .collect();
    Pass1Result {
        matches,
        seeds_scanned: count,
        elapsed: start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_correctly_and_counts_scanned_seeds() {
        // Simple stand-in condition: match every seed divisible by 7.
        let result = scan_geometry(100, |seed| if seed % 7 == 0 { Some(seed) } else { None });

        assert_eq!(result.seeds_scanned, 100);
        assert_eq!(
            result.matches,
            vec![0, 7, 14, 21, 28, 35, 42, 49, 56, 63, 70, 77, 84, 91, 98]
        );
    }

    #[test]
    fn zero_count_returns_no_matches() {
        let result = scan_geometry(0, |seed| Some(seed));
        assert!(result.matches.is_empty());
        assert_eq!(result.seeds_scanned, 0);
    }

    #[test]
    fn seeds_per_second_is_computed_from_elapsed_time() {
        let result = scan_geometry(1000, |seed| if seed % 2 == 0 { Some(seed) } else { None });
        // Just confirms the math runs and produces a sane positive value —
        // exact timing isn't asserted since that would make this test flaky.
        assert!(result.seeds_per_second() > 0.0);
    }
}

// Exposes the public API of the library

pub mod biome;
pub mod conditions;
pub mod placement;
pub mod rng;
pub mod search;
pub mod structures;

use biome::BiomeOracle;
use conditions::{DoubleHutCondition, DoubleHutMatch, QuadHutCondition, QuadHutMatch};
use search::{scan_geometry_from, Pass1Result};

/// Swamp-hut AFK farm condition used by `bench`/`search`: two huts within
/// 128m of a shared AFK spot, itself within 512m of spawn.
pub const DOUBLE_HUT_CONDITION: DoubleHutCondition = DoubleHutCondition {
    max_spawn_dist: 512.0,
    max_afk_radius: 128.0,
};

/// Same conditions, but requiring all 4 huts of a 2x2 region block.
pub const QUAD_HUT_CONDITION: QuadHutCondition = QuadHutCondition {
    max_spawn_dist: 512.0,
    max_afk_radius: 128.0,
};

/// Runs Pass 1 (geometry) then Pass 2 (real biome check via the oracle) on
/// the survivors, for each condition. Kept as two typed wrappers rather
/// than one generic function — a generic version needs either a shared
/// trait for `.seed`/`.hut_coords()` across match types or a closure per
/// call site, and with only two conditions right now, that abstraction
/// wasn't worth adding yet. Revisit if a third condition shows up.

/// Double-hut convenience wrapper.
pub fn find_confirmed_double(count: i64) -> (Pass1Result<DoubleHutMatch>, Vec<DoubleHutMatch>) {
    find_confirmed_double_from(0, count)
}

pub fn find_confirmed_double_from(
    start_seed: i64,
    count: i64,
) -> (Pass1Result<DoubleHutMatch>, Vec<DoubleHutMatch>) {
    let pass1 = scan_geometry_from(start_seed, count, |seed| {
        DOUBLE_HUT_CONDITION.evaluate(seed)
    });
    if pass1.matches.is_empty() {
        return (pass1, Vec::new());
    }
    let mut oracle = BiomeOracle::new();
    let confirmed: Vec<DoubleHutMatch> = pass1
        .matches
        .iter()
        .filter(|m| oracle.check_all_swamp(m.seed, &m.hut_coords()))
        .cloned()
        .collect();
    (pass1, confirmed)
}

/// Quad-hut convenience wrapper.
pub fn find_confirmed_quad(count: i64) -> (Pass1Result<QuadHutMatch>, Vec<QuadHutMatch>) {
    find_confirmed_quad_from(0, count)
}

pub fn find_confirmed_quad_from(
    start_seed: i64,
    count: i64,
) -> (Pass1Result<QuadHutMatch>, Vec<QuadHutMatch>) {
    let pass1 = scan_geometry_from(start_seed, count, |seed| QUAD_HUT_CONDITION.evaluate(seed));
    if pass1.matches.is_empty() {
        return (pass1, Vec::new());
    }
    let mut oracle = BiomeOracle::new();
    let confirmed: Vec<QuadHutMatch> = pass1
        .matches
        .iter()
        .filter(|m| oracle.check_all_swamp(m.seed, &m.hut_coords()))
        .cloned()
        .collect();
    (pass1, confirmed)
}

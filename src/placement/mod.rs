//! Region-grid math: given a world seed and a structure's placement config
//! (spacing, chunk_range, salt), compute which chunk within a region that
//! structure attempts to generate at. Depends on `rng`.
//!
//! Takes spacing/chunk_range/salt as plain parameters rather than a
//! `structures::StructureConfig` for now, since that module doesn't exist
//! yet — once it does, add a thin `&StructureConfig` overload rather than
//! rewriting this core logic.

use crate::rng::JavaLcg;

/// Returns the (chunk_x, chunk_z) a structure attempts placement at within
/// the given region, in absolute chunk coordinates.
///
/// `spacing` is the region size in chunks, `chunk_range` is the bound
/// passed to the RNG for the offset within the region, and `salt` is the
/// structure-specific value that keeps different structure types from
/// lining up on the same seed.
#[inline(always)]
pub fn get_structure_chunk(
    world_seed: i64,
    region_x: i32,
    region_z: i32,
    spacing: i32,
    chunk_range: i32,
    salt: i64,
) -> (i32, i32) {
    let region_seed = (region_x as i64)
        .wrapping_mul(341873128712)
        .wrapping_add((region_z as i64).wrapping_mul(132897987541))
        .wrapping_add(world_seed)
        .wrapping_add(salt);

    let mut rng = JavaLcg::new(region_seed);
    let chunk_x = rng.next_int(chunk_range);
    let chunk_z = rng.next_int(chunk_range);

    (
        region_x.wrapping_mul(spacing).wrapping_add(chunk_x),
        region_z.wrapping_mul(spacing).wrapping_add(chunk_z),
    )
}

// Convenience: block-space center of a structure's chunk (chunk origin + 8).
#[inline(always)]
pub fn chunk_to_block_center(chunk_x: i32, chunk_z: i32) -> (f64, f64) {
    (chunk_x as f64 * 16.0 + 8.0, chunk_z as f64 * 16.0 + 8.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real swamp hut config from cubiomes' SWAMP_HUT_CONFIG:
    // { salt: 14357620, regionSize: 32, chunkRange: 24 }
    const SWAMP_HUT_SPACING: i32 = 32;
    const SWAMP_HUT_CHUNK_RANGE: i32 = 24;
    const SWAMP_HUT_SALT: i64 = 14357620;

    #[test]
    fn matches_verified_swamp_hut_placement() {
        // world_seed = 0, region (0,0) — independently computed and verified
        assert_eq!(
            get_structure_chunk(
                0,
                0,
                0,
                SWAMP_HUT_SPACING,
                SWAMP_HUT_CHUNK_RANGE,
                SWAMP_HUT_SALT
            ),
            (0, 18)
        );
        // region (1,0) — confirms the region_x * spacing offset is applied correctly
        assert_eq!(
            get_structure_chunk(
                0,
                1,
                0,
                SWAMP_HUT_SPACING,
                SWAMP_HUT_CHUNK_RANGE,
                SWAMP_HUT_SALT
            ),
            (32, 19)
        );
        // negative region coordinates — confirms sign handling doesn't break
        assert_eq!(
            get_structure_chunk(
                0,
                -1,
                -1,
                SWAMP_HUT_SPACING,
                SWAMP_HUT_CHUNK_RANGE,
                SWAMP_HUT_SALT
            ),
            (-30, -9)
        );
    }
}

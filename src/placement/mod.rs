//! Region-grid math: given a world seed and a structure's placement config,
//! compute which chunk within a region that structure attempts to generate
//! at. Depends on `rng` and `structures`.
//!
//! Verified field-for-field against cubiomes' finders.c getStructureChunkInRegion().

use crate::rng::JavaLcg;
use crate::structures::StructureConfig;

/// Returns the (chunk_x, chunk_z) a structure attempts placement at within
/// the given region, in absolute chunk coordinates.
#[inline(always)]
pub fn get_structure_chunk(
    world_seed: i64,
    region_x: i32,
    region_z: i32,
    config: &StructureConfig,
) -> (i32, i32) {
    let region_seed = (region_x as i64)
        .wrapping_mul(341873128712)
        .wrapping_add((region_z as i64).wrapping_mul(132897987541))
        .wrapping_add(world_seed)
        .wrapping_add(config.salt);

    let mut rng = JavaLcg::new(region_seed);
    let chunk_x = rng.next_int(config.chunk_range);
    let chunk_z = rng.next_int(config.chunk_range);

    (
        region_x.wrapping_mul(config.spacing).wrapping_add(chunk_x),
        region_z.wrapping_mul(config.spacing).wrapping_add(chunk_z),
    )
}

/// Convenience: block-space center of a structure's chunk (chunk origin + 8).
#[inline(always)]
pub fn chunk_to_block_center(chunk_x: i32, chunk_z: i32) -> (f64, f64) {
    (chunk_x as f64 * 16.0 + 8.0, chunk_z as f64 * 16.0 + 8.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::SWAMP_HUT;

    #[test]
    fn matches_verified_swamp_hut_placement() {
        assert_eq!(get_structure_chunk(0, 0, 0, &SWAMP_HUT), (0, 18));
        assert_eq!(get_structure_chunk(0, 1, 0, &SWAMP_HUT), (32, 19));
        assert_eq!(get_structure_chunk(0, -1, -1, &SWAMP_HUT), (-30, -9));
    }
}

//! Static structure-placement data: spacing, separation, and salt per
//! structure type. No dependencies on other modules — this is just data.

#[derive(Clone, Copy, Debug)]
pub struct StructureConfig {
    /// Region size in chunks (structures attempt placement once per region).
    pub spacing: i32,
    /// Max random offset within a region, in chunks (spacing - separation
    /// in vanilla's terms — this crate stores the already-computed range
    /// passed straight to `next_int`).
    pub chunk_range: i32,
    pub salt: i64,
}

/// Swamp Hut (witch hut) structure config for modern (post-1.13) versions.
///
/// NOTE: 14357617 shows up all over older/copy-pasted seed-finding code
/// because pre-1.13, desert pyramids, jungle temples, witch huts and
/// igloos all shared ONE salt. Since 1.13, Mojang split these into
/// per-structure salts and swamp hut's became 14357620. Verified against
/// Cubitect/cubiomes' finders.c (`s_swamp_hut` config), the seed-finding
/// community's reference implementation. Using the pre-1.13 salt on a
/// modern (26.2) world doesn't error — it silently predicts the wrong
/// chunk in every region.
pub const SWAMP_HUT: StructureConfig = StructureConfig {
    spacing: 32,
    chunk_range: 24,
    salt: 14357620,
};

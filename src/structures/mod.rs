//! Static, per-structure placement data: spacing, separation, salt, spread type.

//! This module is pure data and has **no dependencies on other modules** — no
//! RNG, no grid math. `placement/` consumes what's here and does the actual
//! region -> chunk computation. Keeping the split means a wrong constant can
//! only ever be a wrong constant, never a wrong algorithm.

//! Every value below is transcribed from the vanilla `data/minecraft/worldgen/
//! structure_set/*.json` files (via the Minecraft Wiki's default structure set
//! table, cross-checked against cubiomes' `finders.c` where the two overlap).
//! A wrong salt or spacing here produces confident, silently wrong output —
//! see the READMe's note on the swamp hut salt for how easily that happens.

//! Target version: Java Edition 26.3 "Wilderness Bound".

/// How offsets within a region are drawn from the RNG.

/// Both variants consume from the same `JavaLcg`, but `Triangular` burns two
/// `next_int` calls per axis instead of one — so the spread type is not just
/// a distribution choice, it changes the RNG call sequence. Getting it wrong
/// desyncs every subsequent value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadType {
    /// One roll per axis, uniform over `[0, spacing - separation)`.
    Linear,
    /// Two rolls per axis, averaged (floored). Biases offsets toward the
    /// middle of the region, which is why monuments and mansions sit more
    /// centered in their cells than villages do.
    Triangular,
}

/// A structure set's placement algorithm.

/// Vanilla defines exactly two. There is no `dimension_origin` type — if you
/// see one referenced, it isn't in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Grid of `spacing`x`spacing` chunk regions starting at chunk (0, 0),
    /// with one generation attempt placed at a random offset inside each.
    RandomSpread {
        /// Average distance in chunks between neighboring attempts; also the
        /// region size.
        spacing: i32,
        /// Minimum distance in chunks between neighboring attempts. Must be
        /// strictly less than `spacing`.
        separation: i32,
        spread_type: SpreadType,
    },
    /// A fixed number of attempts arranged in rings around the world origin.
    /// Only strongholds use this in vanilla.
    ConcentricRings {
        /// Ring thickness plus gap, in units of 6 chunks.
        distance: i32,
        /// Total attempts across all rings in the dimension.
        count: i32,
        /// Attempts on the innermost ring.
        spread: i32,
    },
}

/// A structure set that must not generate near another set's structures.

/// `placement/` does not currently evaluate exclusion zones; this is carried
/// as data so a future condition can apply it without re-deriving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusionZone {
    /// Radius in chunks. Between 1 and 16 inclusive.
    pub chunk_count: i32,
    /// The structure set ID being avoided.
    pub other_set: &'static str,
}

/// Everything needed to locate one structure set's candidate chunks.

/// `Copy` and allocation-free on purpose: `search/` evaluates this against
/// hundreds of millions of seeds, so a `String` field here would mean a heap
/// allocation per clone in the hot loop. All presets are `const`, so they cost
/// nothing at runtime and their invariants are checked at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StructureConfig {
    /// Human-readable label for output.
    pub name: &'static str,
    /// Namespaced structure set ID, e.g. `"minecraft:swamp_huts"`.
    pub id: &'static str,
    /// Mixed into the region seed. Distinct per set so two sets don't land on
    /// identical coordinates.
    pub salt: i32,
    pub placement: Placement,
    /// Probability the game *attempts* generation at all, in `[0.0, 1.0]`.

    /// Not applied by `placement/` — a structure with `frequency < 1.0` will
    /// be over-reported by pure geometry. Relevant for pillager outposts
    /// (20%), buried treasure (1%) and mineshafts (0.4%).
    pub frequency: f32,
    pub exclusion_zone: Option<ExclusionZone>,
}

impl StructureConfig {
    /// Standard random-spread set: linear spread, no frequency gate.

    /// Panics at compile time (all presets are evaluated in `const` context)
    /// if `separation >= spacing`, which would hand `next_int` a bound of zero
    /// or less.
    pub const fn random_spread(
        name: &'static str,
        id: &'static str,
        spacing: i32,
        separation: i32,
        salt: i32,
    ) -> Self {
        assert!(spacing > 0, "spacing must be positive");
        assert!(separation >= 0, "separation must be non-negative");
        assert!(
            separation < spacing,
            "separation must be strictly less than spacing"
        );
        Self {
            name,
            id,
            salt,
            placement: Placement::RandomSpread {
                spacing,
                separation,
                spread_type: SpreadType::Linear,
            },
            frequency: 1.0,
            exclusion_zone: None,
        }
    }

    /// Concentric-rings set. Strongholds only.
    pub const fn concentric_rings(
        name: &'static str,
        id: &'static str,
        salt: i32,
        distance: i32,
        count: i32,
        spread: i32,
    ) -> Self {
        Self {
            name,
            id,
            salt,
            placement: Placement::ConcentricRings {
                distance,
                count,
                spread,
            },
            frequency: 1.0,
            exclusion_zone: None,
        }
    }

    /// Switch a random-spread set to triangular spread.
    pub const fn triangular(mut self) -> Self {
        if let Placement::RandomSpread {
            spacing,
            separation,
            ..
        } = self.placement
        {
            self.placement = Placement::RandomSpread {
                spacing,
                separation,
                spread_type: SpreadType::Triangular,
            };
        }
        self
    }

    pub const fn with_frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    pub const fn with_exclusion_zone(mut self, chunk_count: i32, other_set: &'static str) -> Self {
        self.exclusion_zone = Some(ExclusionZone {
            chunk_count,
            other_set,
        });
        self
    }

    /// Upper bound (exclusive) for the per-axis offset roll: `spacing - separation`.

    /// This is what gets handed to `JavaLcg::next_int`. `None` for
    /// concentric-rings sets, which don't roll offsets this way.
    pub const fn spread_bound(&self) -> Option<i32> {
        match self.placement {
            Placement::RandomSpread {
                spacing,
                separation,
                ..
            } => Some(spacing - separation),
            Placement::ConcentricRings { .. } => None,
        }
    }

    /// Region size in chunks, for sets that use a grid.
    pub const fn spacing(&self) -> Option<i32> {
        match self.placement {
            Placement::RandomSpread { spacing, .. } => Some(spacing),
            Placement::ConcentricRings { .. } => None,
        }
    }

    /// True if pure geometry over-reports this set — because generation is
    /// probabilistic, blocked near another structure, or both.
    pub const fn needs_extra_filtering(&self) -> bool {
        self.frequency < 1.0 || self.exclusion_zone.is_some()
    }
}

// Ordered alphabetically by structure set ID, to make diffing against the
// vanilla data files straightforward.

pub const ANCIENT_CITY: StructureConfig =
    StructureConfig::random_spread("Ancient City", "minecraft:ancient_cities", 24, 8, 20083232);

/// Frequency 1%, and `/locate` reports it offset by 9 blocks on X and Z.
pub const BURIED_TREASURE: StructureConfig =
    StructureConfig::random_spread("Buried Treasure", "minecraft:buried_treasures", 1, 0, 0)
        .with_frequency(0.01);

pub const DESERT_PYRAMID: StructureConfig = StructureConfig::random_spread(
    "Desert Pyramid",
    "minecraft:desert_pyramids",
    32,
    8,
    14357617,
);

pub const END_CITY: StructureConfig =
    StructureConfig::random_spread("End City", "minecraft:end_cities", 20, 11, 10387313);

pub const IGLOO: StructureConfig =
    StructureConfig::random_spread("Igloo", "minecraft:igloos", 32, 8, 14357618);

pub const JUNGLE_PYRAMID: StructureConfig = StructureConfig::random_spread(
    "Jungle Pyramid",
    "minecraft:jungle_temples",
    32,
    8,
    14357619,
);

pub const MINESHAFT: StructureConfig =
    StructureConfig::random_spread("Mineshaft", "minecraft:mineshafts", 1, 0, 0)
        .with_frequency(0.004);

/// Fortresses and bastion remnants share one structure set: each successful
/// region rolls 40% fortress / 60% bastion. Geometry alone tells you a
/// *nether complex* is at these coordinates, not which of the two — so don't
/// label output from this preset "Nether Fortress" without a further check.
pub const NETHER_COMPLEX: StructureConfig = StructureConfig::random_spread(
    "Nether Complex",
    "minecraft:nether_complexes",
    27,
    4,
    30084232,
);

pub const NETHER_FOSSIL: StructureConfig =
    StructureConfig::random_spread("Nether Fossil", "minecraft:nether_fossils", 2, 1, 14357921);

pub const OCEAN_MONUMENT: StructureConfig = StructureConfig::random_spread(
    "Ocean Monument",
    "minecraft:ocean_monuments",
    32,
    5,
    10387313,
)
.triangular();

pub const OCEAN_RUIN: StructureConfig =
    StructureConfig::random_spread("Ocean Ruin", "minecraft:ocean_ruins", 20, 8, 14357621);

/// 20% frequency, and excluded within 10 chunks of any village.
pub const PILLAGER_OUTPOST: StructureConfig = StructureConfig::random_spread(
    "Pillager Outpost",
    "minecraft:pillager_outposts",
    32,
    8,
    165745296,
)
.with_frequency(0.2)
.with_exclusion_zone(10, "minecraft:villages");

pub const RUINED_PORTAL: StructureConfig = StructureConfig::random_spread(
    "Ruined Portal",
    "minecraft:ruined_portals",
    40,
    15,
    34222645,
);

pub const SHIPWRECK: StructureConfig =
    StructureConfig::random_spread("Shipwreck", "minecraft:shipwrecks", 24, 4, 165745295);

pub const STRONGHOLD: StructureConfig =
    StructureConfig::concentric_rings("Stronghold", "minecraft:strongholds", 0, 32, 128, 3);

/// The salt is `14357620`, **not** the widely copy-pasted `14357617`. That
/// value is the pre-1.13 salt shared by desert pyramids, jungle temples,
/// swamp huts and igloos; 1.13 split them per-structure. Verified against
/// cubiomes' `s_swamp_hut` config. This is the one constant in this file the
/// whole project depends on — `tests/known_seeds/` exists to catch it
/// silently changing.
pub const SWAMP_HUT: StructureConfig =
    StructureConfig::random_spread("Swamp Hut", "minecraft:swamp_huts", 32, 8, 14357620);

pub const TRAIL_RUINS: StructureConfig =
    StructureConfig::random_spread("Trail Ruins", "minecraft:trail_ruins", 34, 8, 83469867);

pub const TRIAL_CHAMBERS: StructureConfig = StructureConfig::random_spread(
    "Trial Chambers",
    "minecraft:trial_chambers",
    34,
    12,
    94251327,
);

pub const VILLAGE: StructureConfig =
    StructureConfig::random_spread("Village", "minecraft:villages", 34, 8, 10387312);

pub const WOODLAND_MANSION: StructureConfig = StructureConfig::random_spread(
    "Woodland Mansion",
    "minecraft:woodland_mansions",
    80,
    20,
    10387319,
)
.triangular();

/// Placeholder. The real abandoned camp salt has not been read out of the
/// game's data files, and no public source lists it.

/// Do not guess. A wrong salt doesn't fail loudly — it produces a full,
/// plausible-looking coordinate list that points at empty ground.
pub const ABANDONED_CAMP_SALT_UNVERIFIED: i32 = 0;

/// Abandoned camp, added in 26.3 (released 2026-09-15).

/// **Deliberately excluded from [`ALL`] until the salt is verified.**

/// Known: generation follows a grid of 37x37 chunk regions centered on the
/// world origin, one attempt per region, across 18 surface biomes. Spacing was
/// widened from 34 to 37 between snapshot 4 and release, so any constant taken
/// from snapshot-era tooling is stale.

/// Not known: the separation and the salt. The `8` below is carried over from
/// the snapshot-era values and is a guess; it is consistent with the wiki's
/// (stale, `[verify]`-tagged) "minimum 8, maximum 60 chunks apart" figures,
/// which imply `spacing = 34, separation = 8`.

/// To fix: pull `data/minecraft/worldgen/structure_set/abandoned_camps.json`
/// out of a 26.3 client jar, replace both values, then confirm one coordinate
/// with `/locate structure minecraft:abandoned_camp` on a real 26.3 server
/// before adding this to [`ALL`].
pub const ABANDONED_CAMP: StructureConfig = StructureConfig::random_spread(
    "Abandoned Camp (UNVERIFIED)",
    "minecraft:abandoned_camps",
    37,
    8,
    ABANDONED_CAMP_SALT_UNVERIFIED,
);

/// Every structure set whose constants are verified and safe to search on.

/// Excludes [`ABANDONED_CAMP`] — see its docs.
pub const ALL: &[StructureConfig] = &[
    ANCIENT_CITY,
    BURIED_TREASURE,
    DESERT_PYRAMID,
    END_CITY,
    IGLOO,
    JUNGLE_PYRAMID,
    MINESHAFT,
    NETHER_COMPLEX,
    NETHER_FOSSIL,
    OCEAN_MONUMENT,
    OCEAN_RUIN,
    PILLAGER_OUTPOST,
    RUINED_PORTAL,
    SHIPWRECK,
    STRONGHOLD,
    SWAMP_HUT,
    TRAIL_RUINS,
    TRIAL_CHAMBERS,
    VILLAGE,
    WOODLAND_MANSION,
];

/// Look up a verified preset by its namespaced structure set ID.
pub fn by_id(id: &str) -> Option<&'static StructureConfig> {
    ALL.iter().find(|config| config.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the invariant `next_int` needs: a positive bound.
    #[test]
    fn every_random_spread_has_a_positive_offset_bound() {
        for config in ALL {
            if let Some(bound) = config.spread_bound() {
                assert!(
                    bound > 0,
                    "{}: spread bound must be positive, got {bound}",
                    config.name
                );
            }
        }
    }

    /// Two sets sharing a salt *and* a spacing would generate on identical
    /// coordinates. Ocean monuments and end cities share salt `10387313` but
    /// differ in spacing and dimension, which is fine; a full collision is not.

    /// Buried treasure and mineshafts are exempt: both are `spacing: 1,
    /// separation: 0, salt: 0`, which is not a mistake. With a region of one
    /// chunk the offset roll is always 0, so placement is decided entirely by
    /// the frequency gate rather than by the salt.
    #[test]
    fn no_two_sets_share_both_salt_and_spacing() {
        let grid_sets: Vec<_> = ALL
            .iter()
            .filter(|c| c.spacing().is_some_and(|spacing| spacing > 1))
            .collect();

        for (i, a) in grid_sets.iter().enumerate() {
            for b in &grid_sets[i + 1..] {
                assert!(
                    !(a.salt == b.salt && a.spacing() == b.spacing()),
                    "{} and {} would generate identically",
                    a.name,
                    b.name
                );
            }
        }
    }

    /// Fails if someone adds the abandoned camp to `ALL` before replacing the
    /// placeholder salt. Strongholds legitimately use salt 0.
    #[test]
    fn no_placeholder_salts_in_all() {
        for config in ALL {
            if matches!(config.placement, Placement::ConcentricRings { .. }) {
                continue;
            }
            if config.salt == 0 {
                assert!(
                    config.id == "minecraft:buried_treasures"
                        || config.id == "minecraft:mineshafts",
                    "{} has salt 0 — is this a placeholder?",
                    config.name
                );
            }
        }
        assert_eq!(
            ABANDONED_CAMP.salt, ABANDONED_CAMP_SALT_UNVERIFIED,
            "abandoned camp salt changed — remove the placeholder and add it to ALL"
        );
    }

    #[test]
    fn swamp_hut_uses_the_post_1_13_salt() {
        // Regression guard for the most copy-pasted wrong constant in seed
        // finding. 14357617 is the desert pyramid's salt.
        assert_eq!(SWAMP_HUT.salt, 14357620);
        assert_ne!(SWAMP_HUT.salt, DESERT_PYRAMID.salt);
    }

    #[test]
    fn triangular_sets_are_marked_triangular() {
        for config in [OCEAN_MONUMENT, WOODLAND_MANSION] {
            assert!(
                matches!(
                    config.placement,
                    Placement::RandomSpread {
                        spread_type: SpreadType::Triangular,
                        ..
                    }
                ),
                "{} must use triangular spread",
                config.name
            );
        }
        assert!(matches!(
            VILLAGE.placement,
            Placement::RandomSpread {
                spread_type: SpreadType::Linear,
                ..
            }
        ));
    }

    #[test]
    fn lookup_by_id_works() {
        assert_eq!(
            by_id("minecraft:swamp_huts").map(|c| c.salt),
            Some(14357620)
        );
        assert!(by_id("minecraft:abandoned_camps").is_none());
        assert!(by_id("minecraft:not_a_structure").is_none());
    }

    #[test]
    fn probabilistic_sets_are_flagged() {
        assert!(PILLAGER_OUTPOST.needs_extra_filtering());
        assert!(BURIED_TREASURE.needs_extra_filtering());
        assert!(!SWAMP_HUT.needs_extra_filtering());
    }
}

//! Defines and evaluates "does this seed match?". Depends on `placement`
//! (to find candidate hut positions) and, indirectly through `search`, on
//! `biome` (to confirm them).

use crate::placement::{chunk_to_block_center, get_structure_chunk};
use crate::structures::SWAMP_HUT;

#[derive(Debug, Clone)]
pub struct DoubleHutMatch {
    pub seed: i64,
    pub afk_x: f64,
    pub afk_z: f64,
    pub hut1: (f64, f64),
    pub hut2: (f64, f64),
}

impl DoubleHutMatch {
    pub fn hut_coords(&self) -> [(f64, f64); 2] {
        [self.hut1, self.hut2]
    }
}

/// A double-witch-hut AFK farm condition: two huts within `afk_radius`
/// blocks of a shared AFK spot, itself within `max_spawn_dist` of spawn.
/// This is Pass 1 — geometry only. Callers still need to confirm the
/// result against `biome::BiomeOracle` before trusting it in-game.
#[derive(Debug, Clone, Copy)]
pub struct DoubleHutCondition {
    pub max_spawn_dist: f64,
    pub max_afk_radius: f64,
}

impl DoubleHutCondition {
    pub fn evaluate(&self, world_seed: i64) -> Option<DoubleHutMatch> {
        let radius_sq = self.max_afk_radius * self.max_afk_radius;

        for rx in -2..=1 {
            for rz in -2..=1 {
                let pairs = [((rx, rz), (rx + 1, rz)), ((rx, rz), (rx, rz + 1))];

                for &((r1_x, r1_z), (r2_x, r2_z)) in &pairs {
                    let (c1x, c1z) = get_structure_chunk(world_seed, r1_x, r1_z, &SWAMP_HUT);
                    let (c2x, c2z) = get_structure_chunk(world_seed, r2_x, r2_z, &SWAMP_HUT);

                    let hut1 = chunk_to_block_center(c1x, c1z);
                    let hut2 = chunk_to_block_center(c2x, c2z);

                    let afk_x = (hut1.0 + hut2.0) / 2.0;
                    let afk_z = (hut1.1 + hut2.1) / 2.0;

                    if (afk_x * afk_x + afk_z * afk_z) > (self.max_spawn_dist * self.max_spawn_dist)
                    {
                        continue;
                    }
                    let dx1 = hut1.0 - afk_x;
                    let dz1 = hut1.1 - afk_z;
                    if (dx1 * dx1 + dz1 * dz1) > radius_sq {
                        continue;
                    }

                    return Some(DoubleHutMatch {
                        seed: world_seed,
                        afk_x,
                        afk_z,
                        hut1,
                        hut2,
                    });
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct QuadHutMatch {
    pub seed: i64,
    pub afk_x: f64,
    pub afk_z: f64,
    pub huts: [(f64, f64); 4],
}

impl QuadHutMatch {
    pub fn hut_coords(&self) -> [(f64, f64); 4] {
        self.huts
    }
}

/// A quad-witch-hut AFK farm condition: all 4 huts at the corners of one
/// 2x2 region block within `afk_radius` of their shared center, itself
/// within `max_spawn_dist` of spawn. Much rarer than the double-hut
/// condition — expect far fewer Pass-1 survivors per seed scanned.
#[derive(Debug, Clone, Copy)]
pub struct QuadHutCondition {
    pub max_spawn_dist: f64,
    pub max_afk_radius: f64,
}

impl QuadHutCondition {
    pub fn evaluate(&self, world_seed: i64) -> Option<QuadHutMatch> {
        let radius_sq = self.max_afk_radius * self.max_afk_radius;

        for rx in -3..=2 {
            for rz in -3..=2 {
                let regions = [(rx, rz), (rx, rz + 1), (rx + 1, rz), (rx + 1, rz + 1)];
                let mut huts = [(0.0, 0.0); 4];
                for (i, &(r_x, r_z)) in regions.iter().enumerate() {
                    let (cx, cz) = get_structure_chunk(world_seed, r_x, r_z, &SWAMP_HUT);
                    huts[i] = chunk_to_block_center(cx, cz);
                }

                let afk_x = huts.iter().map(|h| h.0).sum::<f64>() / 4.0;
                let afk_z = huts.iter().map(|h| h.1).sum::<f64>() / 4.0;

                if (afk_x * afk_x + afk_z * afk_z) > (self.max_spawn_dist * self.max_spawn_dist) {
                    continue;
                }

                let all_within_radius = huts.iter().all(|&(x, z)| {
                    let dx = x - afk_x;
                    let dz = z - afk_z;
                    (dx * dx + dz * dz) <= radius_sq
                });

                if all_within_radius {
                    return Some(QuadHutMatch {
                        seed: world_seed,
                        afk_x,
                        afk_z,
                        huts,
                    });
                }
            }
        }
        None
    }
}

// shim.c
//
// Thin C shim over cubiomes, compiled and linked directly into the Rust
// binary via build.rs (using the `cc` crate). Exposes only what
// biome/mod.rs needs, behind an opaque handle, so the Rust side never has
// to replicate cubiomes' `Generator` struct layout (which contains a
// union and would be fragile to hand-port across platforms/ABIs).
//
// VERSION NOTE: MC_NEWEST here resolves to whatever this vendored copy of
// cubiomes considers newest (1.21 "WD" as of the version cloned for this
// project — see README/comments elsewhere for why: this predates
// Minecraft 26.2 "Chaos Cubed"). Swamp surface placement is assumed
// unaffected by 26.2's (cave-focused) changes, but that's an assumption —
// verify against chunkbase's live 26.2 map before trusting a full run.

#include <stdlib.h>
#include "generator.h"
#include "finders.h"

void *oracle_new(int mc) {
    Generator *g = (Generator *)malloc(sizeof(Generator));
    if (!g) return NULL;
    setupGenerator(g, mc == 0 ? MC_NEWEST : mc, 0);
    return (void *)g;
}

void oracle_apply_seed(void *handle, long long seed) {
    applySeed((Generator *)handle, DIM_OVERWORLD, (unsigned long long)seed);
}

// Returns 1 if the position is a viable Swamp Hut biome, 0 otherwise.
int oracle_is_swamp_hut_viable(void *handle, int x, int z) {
    return isViableStructurePos(Swamp_Hut, (Generator *)handle, x, z, 0);
}

void oracle_free(void *handle) {
    free(handle);
}

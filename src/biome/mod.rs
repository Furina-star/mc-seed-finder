//! Biome checks, via direct FFI into cubiomes (Cubitect/cubiomes),
//! compiled from vendored source and linked straight into this binary by
//! build.rs. See oracle/cubiomes/shim.c for the tiny C surface this binds
//! against — Rust never touches cubiomes' `Generator` struct layout
//! directly, only the opaque handle the shim returns.
//!
//! There IS a real safe Rust binding published as `cubiomes` on
//! crates.io (github.com/villevilli/cubiomes-rs, via bindgen), and it's
//! the more idiomatic long-term choice if this project wants to drop the
//! hand-written shim. It wasn't used here because: (a) it needs `bindgen`
//! + a working `libclang` on every dev machine at build time, which is
//! one more cross-platform variable on top of the C-compiler requirement
//! this project already has, and (b) its exact API surface hasn't been
//! verified end-to-end the way this shim has (see tests/known-seeds).
//! Swapping to it later would only touch this module.

use std::ffi::c_void;
use std::os::raw::{c_int, c_longlong};

extern "C" {
    fn oracle_new(mc: c_int) -> *mut c_void;
    fn oracle_apply_seed(handle: *mut c_void, seed: c_longlong);
    fn oracle_is_swamp_hut_viable(handle: *mut c_void, x: c_int, z: c_int) -> c_int;
    fn oracle_free(handle: *mut c_void);
}

pub struct BiomeOracle {
    handle: *mut c_void,
}

impl BiomeOracle {
    /// mc = 0 selects cubiomes' MC_NEWEST (currently ~1.21 "WD" — see the
    /// version note in oracle/cubiomes/shim.c; this predates Minecraft
    /// 26.2, which is an unverified-but-probably-fine assumption for
    /// swamp placement specifically).
    pub fn new() -> Self {
        let handle = unsafe { oracle_new(0) };
        assert!(!handle.is_null(), "failed to allocate cubiomes generator");
        Self { handle }
    }

    /// Returns true if every coordinate is a viable swamp-hut biome for
    /// this seed, per cubiomes' own isViableStructurePos (which handles
    /// the correct per-version y-sampling internally).
    pub fn check_all_swamp(&mut self, seed: i64, coords: &[(f64, f64)]) -> bool {
        unsafe { oracle_apply_seed(self.handle, seed) };
        coords.iter().all(|&(x, z)| {
            let viable = unsafe { oracle_is_swamp_hut_viable(self.handle, x as i32, z as i32) };
            viable != 0
        })
    }
}

impl Default for BiomeOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BiomeOracle {
    fn drop(&mut self) {
        unsafe { oracle_free(self.handle) };
    }
}

// The Generator this handle points to is only ever touched from one
// thread at a time in this codebase (search runs Pass 1 in parallel,
// then biome-checks the small surviving set sequentially on one
// BiomeOracle) — Send is safe under that usage, but this type is
// intentionally NOT Sync, so sharing one instance across threads
// without synchronization won't silently compile.
unsafe impl Send for BiomeOracle {}

//! Compiles cubiomes (vendored C source under oracle/cubiomes/) plus our
//! shim into a static library and links it into this crate.
//!
//! Using the `cc` crate instead of shelling out to a hardcoded "cc"
//! command is the whole fix for Windows: `cc` finds MSVC's `cl.exe` via
//! the same detection `rustc` itself uses (vswhere/registry), given the
//! MSVC Build Tools this project's README already tells people to
//! install. On Linux/macOS it finds gcc/clang as before. This also drops
//! the old approach of shipping a prebuilt `libcubiomes.a` — that .a was
//! built for x86_64 Linux and simply couldn't link on Windows or macOS
//! regardless of compiler; compiling from vendored source here fixes
//! that at the root instead of patching around it per-platform.

fn main() {
    let src_dir = "oracle/cubiomes";

    cc::Build::new()
        .include(src_dir)
        .file(format!("{src_dir}/noise.c"))
        .file(format!("{src_dir}/biomes.c"))
        .file(format!("{src_dir}/layers.c"))
        .file(format!("{src_dir}/biomenoise.c"))
        .file(format!("{src_dir}/generator.c"))
        .file(format!("{src_dir}/finders.c"))
        .file(format!("{src_dir}/util.c"))
        .file(format!("{src_dir}/quadbase.c"))
        .file(format!("{src_dir}/shim.c"))
        .warnings(false)
        .compile("cubiomes_shim");

    println!("cargo:rerun-if-changed={src_dir}");
}

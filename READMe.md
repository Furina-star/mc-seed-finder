# mc-seed-finder

A Rust command-line tool that searches through candidate Minecraft world seeds
and finds ones matching conditions you specify, inspired by [Chunkbase's Seed
Finder](https://www.chunkbase.com/apps/seed-finder). Currently implements
swamp-hut AFK farm conditions — double and quad hut clusters near
spawn — verified against real biome data, not just structure-placement math.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [How Biome Checking Works](#how-biome-checking-works)
- [Known Seeds / Verification](#known-seeds--verification)
- [Development Workflow](#development-workflow)
- [Module Ownership](#module-ownership)
- [Known Limitations](#known-limitations)
- [Credits & Licensing](#credits--licensing)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before you touch any code, make sure you have these installed:

1. **Git** — [git-scm.com](https://git-scm.com/downloads). Default install options are fine.
2. **Rust (via rustup)** — [rustup.rs](https://rustup.rs/)
3. **A C compiler.** This is not optional, and it's not just a Windows thing
   anymore: `build.rs` compiles a vendored copy of the
   [cubiomes](https://github.com/Cubitect/cubiomes) C library from source on
   every clean build, via the [`cc`](https://crates.io/crates/cc) crate.
   - **Linux:** `sudo apt install build-essential` (or your distro's
     equivalent) gets you `gcc`.
   - **macOS:** `xcode-select --install` gets you `clang`.
   - **Windows:** install the MSVC C++ Build Tools from
     [visualstudio.microsoft.com/visual-cpp-build-tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
     and check **"Desktop development with C++"** in the installer. Do this
     *before* running the Rust installer, or you'll get linker errors later.
     The `cc` crate finds `cl.exe` the same way `rustc` does, so once this is
     installed correctly you don't need to configure anything else.
4. **A code editor** — VS Code recommended, with the **rust-analyzer** extension installed.

You do **not** need to separately install cubiomes, clang/libclang, or
bindgen — the C library's source is vendored under `oracle/cubiomes/` and
compiled straight into the binary.

### Verify everything is installed

```
git --version
rustc --version
cargo --version
cc --version        # or `cl` on Windows, from a "Developer Command Prompt"
```

If any say "not recognized," restart your terminal (or your machine) —
installers often need a fresh session to update your PATH.

---

## Getting Started

### 1. Clone the repository

```
git clone https://github.com/Furina-star/mc-seed-finder.git
cd mc-seed-finder
```

### 2. Build

```
cargo build --release
```

The first build is noticeably slower than a normal Rust project's — it's
also compiling the vendored cubiomes C source (`oracle/cubiomes/`), not just
downloading and compiling Rust dependencies.

### 3. Run

```
cargo run --release -- search 50000000 --start-seed 0 --mode double
```

See [Usage](#usage) below for the full command shape.

### 4. Run the tests

```
cargo test --release
```

This runs the unit and integration tests included in the project. Run it
after pulling any change touching `rng/`, `structures/`, `placement/`, or
`conditions/` — these tests are the fastest way to catch a broken
salt/spacing/RNG constant before it silently produces wrong output.

### 5. Format your code before committing

```
cargo fmt
```

(Optional: in VS Code, enable "Format On Save.")

---

## Usage

```
mc-seed-finder bench [count] [--start-seed <seed>] --mode <double|quad>
mc-seed-finder search [count] [--start-seed <seed>] --mode <double|quad>
```

- `bench` — Pass-1 geometry scan only, no biome check. Use this to measure
  raw throughput on your hardware before committing to a long `search` run.
- `search` — Pass-1 geometry scan, then a real biome check (via cubiomes) on
  the survivors. Prints Pass-1 timing, biome-validation timing, total
  end-to-end timing, and confirmed matches with block coordinates.
- `count` — number of seeds to scan. Defaults to `10,000,000`. The scanned
  interval is `start_seed..start_seed + count`.
- `--start-seed` — first seed in the interval. Defaults to `0`. Use this to
  reproduce the exact same seed range in another implementation.
- `--mode` — which hut-cluster condition to search for (defaults to
  `double`). See [Module Ownership](#module-ownership) for what each one
  actually checks geometrically.

Example:
```
cargo run --release -- bench 10000000 --start-seed 0 --mode quad

# End-to-end search over the same interval
cargo run --release -- search 10000000 --start-seed 0 --mode quad

# Search a later interval
cargo run --release -- search 10000000 --start-seed 10000000 --mode quad
```

`bench` reports Pass-1 geometry throughput only. `search` reports the
end-to-end rate:

```text
total seeds / (geometry time + biome-validation time)
```

This end-to-end rate is the appropriate Rust figure to compare with a tool
that performs its complete search pipeline per seed. The current build uses
Rayon's automatically selected thread pool; an explicit `--threads` option
is not available yet.

---

## Project Structure

```
mc-seed-finder/
├── Cargo.toml
├── .gitignore
├── build.rs                    # Compiles oracle/cubiomes/ + shim.c via the `cc` crate
├── oracle/
│   └── cubiomes/                # Vendored C source (github.com/Cubitect/cubiomes)
│       ├── LICENSE               # cubiomes' MIT license — required to ship alongside the vendored source, see Credits & Licensing
│       ├── *.c / *.h             # cubiomes itself, unmodified
│       ├── tables/                # cubiomes' biome-tree data tables
│       └── shim.c                 # Our thin C wrapper — hides Generator's
│                                   # struct layout behind an opaque handle
├── src/
│   ├── main.rs                  # Thin entry point — calls into the library below
│   ├── lib.rs                    # Exposes all modules; ties search + biome together
│   ├── cli.rs                    # Command-line argument parsing (clap)
│   ├── rng/
│   │   └── mod.rs                 # Java-compatible RNG — the foundation everything else uses
│   ├── structures/
│   │   └── mod.rs                 # Per-structure data: spacing, separation, salt
│   ├── placement/
│   │   └── mod.rs                 # Region-grid math: computes candidate chunks
│   ├── biome/
│   │   └── mod.rs                 # Direct FFI into oracle/cubiomes/ (see below —
│   │                               # NOT the crates.io `cubiomes` crate)
│   ├── conditions/
│   │   └── mod.rs                 # DoubleHutCondition / QuadHutCondition
│   └── search/
│       └── mod.rs                 # Parallel (rayon) loop over candidate seeds
└── tests/
    └── known_seeds/
        └── main.rs                # Verifies output against seeds with known, confirmed hits
```

---

## How Biome Checking Works

`biome/mod.rs` binds directly to the vendored cubiomes C source via a small
hand-written shim (`oracle/cubiomes/shim.c`), compiled and linked into this
binary by `build.rs`. There is a real, published safe Rust wrapper for
cubiomes on crates.io — [`cubiomes`](https://crates.io/crates/cubiomes),
by villevilli, built on `cubiomes-sys` via `bindgen` — and it's the more
idiomatic long-term choice if this project ever wants to drop the hand-rolled
shim. It isn't used currently because:

1. It needs `bindgen` + a working `libclang` on every contributor's machine
   at build time — one more cross-platform variable on top of the C-compiler
   requirement this project already has.
2. Its exact API surface hasn't been verified end-to-end the way this shim
   has (see `tests/known-seeds`).

Swapping to it later should only require changes inside `biome/mod.rs`.

**Version caveat:** the vendored cubiomes source predates Minecraft 26.2
("Chaos Cubed") — its newest supported version is 1.21 "Winter Drop". Chaos
Cubed's known changes (Sulfur Caves biome, sub-Y0 ore/cave adjustments) are
cave-focused, so surface swamp placement is *probably* unaffected — but
that's an assumption, not a verified fact. If you're chasing a specific
seed, spot-check one confirmed coordinate against chunkbase's live 26.2 map
or in-game F3 before trusting a full batch.

**Structure salt:** swamp hut uses salt `14357620`, not the commonly
copy-pasted `14357617`. The latter is the pre-1.13 salt shared by desert
pyramids, jungle temples, witch huts, and igloos together; Mojang split
these into per-structure salts in 1.13, and swamp hut's became `14357620`.
Verified against cubiomes' `finders.c` (`s_swamp_hut` config). See
`structures/mod.rs` for the full note.

---

## Known Seeds / Verification

The condition and RNG tests pin behavior that should remain stable when
placement logic changes. Confirmed search results should also be checked
against cubiomes' real biome logic (not just geometry):

| Seed | Mode | Test |
|---|---|---|
| `2120` | double | `seed_2120_is_a_geometric_double_hut_match` |

If you find and confirm a new seed worth pinning, add an integration test
that asserts the exact hut coordinates the condition returns, with a comment
explaining how it was found and independently confirmed.

---

## Development Workflow

**Nobody commits directly to `main`.** Every change goes through a branch and
a Pull Request.

### Starting work

```
git checkout main
git pull
git checkout -b yourname/task-name
```

### While working

```
git add .
git commit -m "describe what you changed"
```

Commit as often as makes sense — small, focused commits are easier to review
than one giant one at the end.

### Sharing your work

```
git push -u origin yourname/task-name
```

Then open a Pull Request on GitHub comparing your branch into `main`. Have a
teammate review it before merging.

### After your PR merges

```
git checkout main
git pull
git branch -d yourname/task-name
```

Delete the branch on GitHub too, or enable **Settings → General →
"Automatically delete head branches."**

### If you hit a merge conflict

`git status` will list the conflicted files. Open them, resolve the
`<<<<<<< HEAD` / `=======` / `>>>>>>>` markers, then `git add <file>` and
`git commit` to finish.

---

## Module Ownership

| Module | Focus | Depends on |
|---|---|---|
| `rng/` | Core RNG — must exactly match Java's `Random` behavior | none |
| `structures/` | Static data: spacing/separation/salt per structure type | none |
| `placement/` | Region-grid math for locating candidate chunks | `rng/`, `structures/` |
| `conditions/` | Geometry-only match logic — `DoubleHutCondition` (adjacent pair), `QuadHutCondition` (all 4 corners of a 2x2 block) | `placement/`, `structures/` |
| `biome/` | Direct FFI into vendored cubiomes (`oracle/cubiomes/`), NOT the crates.io `cubiomes` crate — see [How Biome Checking Works](#how-biome-checking-works) | `oracle/cubiomes/` (vendored C), `build.rs` |
| `search/` | Parallel (rayon) loop over candidate seeds, generic over match type | `conditions/` |
| `lib.rs` | Ties `search` + `biome` together per condition (`find_confirmed_double/quad`) | `search/`, `biome/`, `conditions/` |
| `cli.rs` | Argument parsing — seed count, `--start-seed`, and `--mode` | none |

`cli.rs` and `structures/` have no dependencies on other modules, so they can
be built independently and in parallel with everything else.

---

## Known Limitations

- **No thread-count flag yet.** Rayon automatically selects the available
  thread pool. The benchmark output identifies the Pass-1 and end-to-end
  rates, but the current CLI cannot cap or select the number of worker
  threads. This will be added separately so benchmark changes remain
  reproducible.
- **Quad-hut has no pinned known-seed test.** It's rare enough (both huts of
  a 2x2 block are far less likely to all be swamp than a pair) that
  a search wasn't run to completion during development. If you run one to
  completion, add the result to `tests/known-seeds/`.
- **The 26.2 version gap** described above — cubiomes here doesn't
  officially know about Chaos Cubed. Watch for this if Mojang ever changes
  Overworld surface climate/biome placement in a future update; nothing here
  would catch that automatically.

---

## Credits & Licensing

This project's biome verification is only possible because of
[**cubiomes**](https://github.com/Cubitect/cubiomes) by
[Cubitect](https://github.com/Cubitect) — a from-scratch C reimplementation
of Minecraft's world generation, and the seed-finding community's reference
implementation for this kind of work. `oracle/cubiomes/` is a vendored,
unmodified copy of its source (plus our own `shim.c`, which is not part of
cubiomes), compiled directly into this binary by `build.rs`. Two specific
things in this project came directly from reading cubiomes' source rather
than from Mojang's game files or our own derivation:

- The corrected swamp-hut salt (`14357620`), found in `finders.c`'s
  `s_swamp_hut` config — see [How Biome Checking Works](#how-biome-checking-works).
- The `isViableStructurePos` call our `shim.c` wraps, which encodes the
  correct per-version y-sampling logic for structure biome checks — this
  project does not reimplement that logic itself.

cubiomes is MIT-licensed (see `oracle/cubiomes/LICENSE`, included verbatim
alongside the vendored source as the license requires). This project's own
code is Rust, in `src/`, and is not itself part of cubiomes — only the
`oracle/cubiomes/` directory is someone else's code.

If this project ever switches to the [`cubiomes` crate on
crates.io](https://crates.io/crates/cubiomes) (see [How Biome Checking
Works](#how-biome-checking-works)), credit for the underlying generation
logic still traces back to the same place: villevilli's crate is itself a
binding over Cubitect's cubiomes.

---

## Troubleshooting

**"cargo: command not found" / "not recognized"**
Rust's installer didn't finish updating your PATH — close and reopen your
terminal, or restart your machine.

**Build fails looking for a C compiler ("cc"/"cl" not found)**
See [Prerequisites](#prerequisites) — a C compiler is required for every
build, not just the first one, since `build.rs` compiles `oracle/cubiomes/`
each time its source changes. On Windows this almost always means the MSVC
Build Tools aren't installed or aren't on PATH — try building from a
"Developer Command Prompt for VS" once to confirm the toolchain itself
works, before troubleshooting Cargo.

**Build fails with `tables/btree18.h: No such file or directory` or similar**
`oracle/cubiomes/tables/` is missing or incomplete — make sure your checkout
actually includes it (it's not something `.gitignore` should ever exclude;
double check if you don't see it after cloning).

**`cargo test` fails on a `known-seeds` test**
Don't just re-run it — this means the RNG, salt, spacing, or condition logic
changed in a way that broke a previously-confirmed real seed. Check what
changed in `rng/`, `structures/`, `placement/`, or `conditions/` since the
last passing run before assuming the test itself is wrong.

**Linker errors when building on Windows**
You're missing the MSVC C++ Build Tools — see [Prerequisites](#prerequisites).

**`git push` fails with "src refspec main does not match any"**
Your local branch isn't actually named `main` (it may still be `master`).
Run `git branch` to check, then `git branch -M main` to rename it.

**`git pull` refuses, saying local changes to `Cargo.lock` would be
overwritten — but `git status` says the working tree is clean**
This is a phantom diff, most likely left over from `.gitattributes` being
added after `Cargo.lock` was already tracked — `status` and the merge
machinery can disagree about whether the file counts as "changed." Fix it
for good, once: `git add --renormalize .` then commit. Until that cleanup
lands, if you hit this: `git fetch origin` then `git reset --hard
origin/main` forces a clean sync (only ever do this on `main` — never on a
branch with real uncommitted work, since `--hard` discards it permanently).

**`git pull` refuses the same way, and `git status` *does* show `Cargo.lock`
as modified**
This is the ordinary version — running `cargo build`/`test`/`check` before
pulling silently updates `Cargo.lock` locally. Fix: `git restore Cargo.lock`,
then `git pull` again. Pull before you build, not after, to avoid this
recurring.

**`Cargo.lock` has actual `<<<<<<<` conflict markers after a merge**
Don't hand-resolve them — it's machine-generated and not meant to be edited
by hand. Resolve `Cargo.toml` normally instead (that one's human-readable),
delete `Cargo.lock` entirely, run `cargo build` to regenerate a correct one
from the resolved `Cargo.toml`, then commit the result.

**A file won't "Run" in your editor**
Config files like `Cargo.toml` aren't code — there's nothing to execute. Use
`cargo run` from the terminal, or open `src/main.rs` and use the ▶ Run link
rust-analyzer shows above `fn main()`.
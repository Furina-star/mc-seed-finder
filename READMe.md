# mc-seed-finder

A Rust command-line tool that searches through candidate Minecraft world seeds
and finds ones matching conditions you specify (e.g. a village within a
certain distance of spawn), inspired by [Chunkbase's Seed Finder](https://www.chunkbase.com/apps/seed-finder).

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Module Ownership](#module-ownership)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before you touch any code, make sure you have these installed:

1. **Git** — [git-scm.com](https://git-scm.com/downloads). Default install options are fine.
2. **Rust (via rustup)** — [rustup.rs](https://rustup.rs/)
   - **Windows only:** you also need the MSVC C++ Build Tools. Install from
     [visualstudio.microsoft.com/visual-cpp-build-tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
     and check **"Desktop development with C++"** in the installer. Do this
     *before* running the Rust installer, or you'll get linker errors later.
3. **A code editor** — VS Code recommended, with the **rust-analyzer** extension installed.

### Verify everything is installed

Open a terminal (Command Prompt, PowerShell, or Git Bash all work) and run:

```
git --version
rustc --version
cargo --version
```

All three should print a version number. If any say "not recognized," restart
your terminal (or your machine) — installers often need a fresh session to
update your PATH.

---

## Getting Started

### 1. Clone the repository

```
git clone https://github.com/Furina-star/mc-seed-finder.git
cd mc-seed-finder
```

### 2. Build and run

```
cargo run
```

This compiles everything under `src/` and runs the resulting program in one
step. The first build will take longer since it also downloads and compiles
dependencies — subsequent builds are much faster.

### 3. Run the tests

```
cargo test
```

This runs both the unit tests inside each module (things like the RNG
correctness check) and the integration tests under `tests/`.

### 4. Format your code before committing

```
cargo fmt
```

Run this before every commit — it auto-formats all `.rs` files to a
consistent style, so four people's code doesn't clash in diffs. (Optional:
in VS Code, enable "Format On Save" in settings so this happens automatically
every time you hit Ctrl+S.)

---

## Project Structure

```
mc-seed-finder/
├── Cargo.toml
├── .gitignore
├── .gitattributes
├── src/
│   ├── main.rs           # Thin entry point — calls into the library below
│   ├── lib.rs             # Exposes all modules so tests/ can use them
│   ├── cli.rs             # Command-line argument parsing (clap)
│   ├── rng/
│   │   └── mod.rs         # Java-compatible RNG — the foundation everything else uses
│   ├── structures/
│   │   └── mod.rs         # Per-structure data: spacing, separation, salt
│   ├── placement/
│   │   └── mod.rs         # Region-grid math: computes candidate chunks
│   ├── biome/
│   │   └── mod.rs         # Wraps the `cubiomes` crate for biome checks
│   ├── conditions/
│   │   └── mod.rs         # Defines what counts as a "matching" seed
│   └── search/
│       └── mod.rs         # Parallel loop over candidate seeds
└── tests/
    └── known-seeds/
        └── main.rs        # Verifies output against seeds with known structure locations
```

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

(The `-u` is only needed the first time you push a given branch — after that,
plain `git push` works.)

Then open a Pull Request on GitHub comparing your branch into `main`. Have a
teammate review it before merging.

### After your PR merges

```
git checkout main
git pull
git branch -d yourname/task-name
```

Delete the branch on GitHub too (there's a button right on the merged PR
page), or enable **Settings → General → "Automatically delete head
branches"** so it happens for you.

### If you hit a merge conflict

`git status` will list the conflicted files. Open them and look for:

```
<<<<<<< HEAD
your version
=======
their version
>>>>>>> branch-name
```

Edit the file to keep whichever version (or combination) is correct, delete
the marker lines, then `git add <file>` and `git commit` to finish.

---

## Module Ownership

| Module | Focus | Depends on |
|---|---|---|
| `rng/` | Core RNG — must exactly match Java's `Random` behavior | none |
| `placement/` | Region-grid math for locating candidate chunks | `rng/` |
| `biome/` | Wraps the `cubiomes` crate for biome/condition checks | external crate |
| `conditions/` | Defines and evaluates "does this seed match?" | `placement/`, `biome/` |
| `search/` | Parallel loop over candidate seeds, collects matches | `conditions/` |
| `cli.rs` | Argument parsing — seed count, conditions, thread count | none |
| `structures/` | Static data: spacing/separation/salt per structure type | none |

`cli.rs` and `structures/` have no dependencies on other modules, so they can
be built independently and in parallel with everything else.

---

## Troubleshooting

**"cargo: command not found" / "not recognized"**
Rust's installer didn't finish updating your PATH — close and reopen your
terminal, or restart your machine.

**Linker errors when building on Windows**
You're missing the MSVC C++ Build Tools — see [Prerequisites](#prerequisites).

**`git push` fails with "src refspec main does not match any"**
Your local branch isn't actually named `main` (it may still be `master`).
Run `git branch` to check, then `git branch -M main` to rename it.

**A file won't "Run" in your editor**
Config files like `Cargo.toml` aren't code — there's nothing to execute. Use
`cargo run` from the terminal, or open `src/main.rs` and use the ▶ Run link
rust-analyzer shows above `fn main()`.
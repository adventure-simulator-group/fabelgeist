# Development build artifacts

Development and test builds omit debug information and incremental compilation
in every Cargo workspace, including the independently built character creator,
character-rig importer (`fabelgeist-mhr`), NumPy array storage library, and
vendored Bevy controller. Tests inherit the development profile. Release
profiles retain their existing settings.

The settings live in each workspace manifest so they also apply when Cargo is
invoked from another directory with `--manifest-path`. The repository Cargo
configuration additionally disables incremental compilation for all profiles.
There is no separate profile that enables debug information.

Cargo features such as `adventuresim-tactical-client/debug` enable developer
tools. They are independent of the profile's compiler debug information.

On Windows with Microsoft Visual C++ (MSVC), Cargo still requests Program
Database (PDB) symbol files when Rust debug information is disabled. The
configured linker command uses the linker shipped with Rust and adds
`/DEBUG:NONE` for outputs under development `debug`
directories, including build scripts and procedural macros. Other output
profiles retain the normal linker arguments. This command uses the existing
Python 3.11 or newer prerequisite (`PYTHON_BIN` can select its executable).
Existing build directories can retain older PDB files. Run
`cargo clean --profile dev` once in each already-built workspace when adopting
these settings to remove the earlier artifacts.
When invoking Cargo from outside the checkout, select the nightly toolchain in
`rust-toolchain.toml` and pass `--config /path/to/checkout/.cargo/config.toml`
to retain the repository's linker policy as well.

## Choose native and database features

The core's `spacetimedb` feature links imports supplied by the database's
WebAssembly host. Build that feature and the persistence module for
`wasm32-unknown-unknown`. Cargo combines feature requests across selected
packages, so `--all-features` in the main workspace activates those imports
and cannot link native tests. Exclude the persistence module and select optional
native features explicitly, using names such as
`adventuresim-tactical-client/tracy`.

Build the database module and browser client in separate Cargo invocations.
Selecting both packages together combines their features even when both target
WebAssembly, and the browser does not provide the database host's imports.

The tactical client's browser build selects `gameplay` and `client-audio`
without `debug`. The standalone art demo uses `--no-default-features`. Build
them separately so Cargo preserves each application's feature selection.

## Measure and retain compiler output

Run `just artifacts` to measure native and cross-compiled development artifacts
across the checkout's Cargo workspaces. The report includes libraries,
executables, examples, test binaries, build-script output, and metadata beneath
their `debug` directories. It reports logical file sizes and bytes counted once
per underlying file in each directory, deduplicating hard links. These are not
filesystem allocation measurements; compression and block overhead can differ.
Pass `--root PATH` to the Python helper to inspect another checkout without
copying the script there.

Run `just artifacts-prune` to preview cleanup. The default policy selects
development profiles whose newest file modification is older than 14 days,
then selects additional profiles from oldest to newest until their combined
file bytes, after deduplicating hard links, fit within 64 GiB. Modification time
is an age heuristic,
not a record of when Cargo last reused an artifact. Adjust the policy with
`just artifacts-prune --days 30 --max-gib 96`. Run it between builds.

Add `--apply` to execute the printed plan. Cleanup uses `cargo clean --profile
dev` for whole selected native or cross-compiled development profiles. It
preserves release profiles and unrelated captures, reports, authoring assets,
and isolated development database directories outside the compiler profiles.
An individual profile can exceed the budget, so cleanup may select the entire
profile; this policy does not evict individual dependency builds.

The helper uses Cargo's workspace discovery for default target directories,
including workspaces that have never been built. It does not create lockfiles.
For configured target directories, it uses locked, offline Cargo metadata and
requires dependencies and lockfile resolution to be available locally. Build
the workspace first if metadata reports an unavailable dependency.
Custom profiles are outside this development-only policy.
Separately configured `build.build-dir` directories are rejected so cleanup
cannot remove intermediate output omitted from the size report. The helper
uses the repository's pinned nightly Cargo to inspect this configuration.

Binding generation and freshness checks use a development WebAssembly build.
Checking schema freshness does not require a separate optimized release build.

For a saved measurement, run:

```sh
python scripts/cargo_artifacts.py report --json target/artifact-report.json
```

Keep measurements and compilation logs in ignored local output such as
`target/artifact-measurement/`. Keep runtime and generated-data retention
separate from compiler-cache cleanup.

## Worktrees and repeat builds

Remove a completed linked worktree once its changes are committed and retained
on the intended branch. Inspect `git worktree list` and `git -C PATH status`
before `git worktree remove PATH`. Removing a worktree also removes its local
build cache; retain worktrees that still contain active work. Use
`git worktree prune` to remove registrations for checkouts already deleted.

Use the ordinary development profile for routine builds. Keep compiler flags,
features, and target selection consistent when the task permits; alternate
combinations and toolchain updates can leave additional cached artifacts.
Build the packages and targets needed for iteration, then perform the complete
validation required by the change. Separate worktrees retain separate caches.

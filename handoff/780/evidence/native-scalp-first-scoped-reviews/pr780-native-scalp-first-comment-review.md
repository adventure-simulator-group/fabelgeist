# Native scalp first-tranche Comment Review

Root HEAD: `4ddddf5856d269d36cf898d59f34af7a30186479`; review is of current dirty bytes below.

Worktree: `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.

Verdict: **CLEAR for the publication/closest/docs eight-file tranche only.**
No new actionable comment finding in the scoped current change. This is not
native-scalp, material, whole-hair, whole-PR, all-thirteen or CI clearance.

## Scope and method

Applied `/home/brunosegovia/.codex/skills/comment-review/SKILL.md` and the
repository/root and crates agent instructions. Freshly read all eight current
files in full, including their existing test source; read the complete current
readback and scalp layout owners. Read the relevant current guide-root/comb
dispatch, CPU hairdo root-selection and canonical batch submission sections.
Existing test bodies were inspected as source and were not executed. No source,
tests, fixtures, helpers, assertions, or documentation examples were added or
changed by this review. No project check, build, scanner, probe or hosted CI
request was run.

Compared against the saved before source under
`target/pr780-audit/native-scalp-preinventory-4ddddf585/source`. The fresh
ledger declares `native_scalp_current_discovery.first_tranche_files`; this
review does not treat the discovery inventory or earlier verdict as clearance.

## Contract evidence and dispositions

1. `gpu/scalp.rs:134-136`: two grouped metadata readbacks accurately describe
   the calls at 191 and 367. `gpu/readback.rs:16-58` records all requested words
   into one readback, submits with `?`, and blocks on its canonical read future.
   The second group contains vertex count, both hairline counts, area and six
   bounds words, rather than separate waits for each item. The final edge batch
   at `gpu/scalp.rs:395` now uses `submit()?` before `Ok(GpuScalp { ... })`.
   This propagates the canonical submission outcome; it does not promise GPU
   completion or a synchronous native verdict on device-handled paths.
   `compute/src/kernel/mod.rs:334-347` explicitly states and implements that
   distinction. No new comment overclaims completion or universal validation.

2. `gpu/wgsl/guides.rs:149-214`: the closest-within-one-centimetre description
   now matches the complete triangle loop for the actual host caller, which
   supplies `max_distance = 0.01` at `gpu/guides.rs:251`. The unsupported
   first-corner rejection and its triangle-size claim are gone. Every triangle
   reaches the same existing closest-point calculation; `d <= best` still
   chooses the later equal-distance triangle. The position, normal, UV and
   found-flag code is retained. This is not admission or numerical correctness
   clearance for malformed, nonfinite or degenerate surfaces, nor a measured
   performance claim. CPU hairdo edge roots still use `Surface::closest(p, 0.01)`
   (`hairdo.rs:218-224`); the Surface query documentation is now attached to
   `surface.rs:141`, instead of to `boundary` at 114.

3. The orphan Rustdoc was removed from the end of `gpu/grow.rs` and attached
   immediately to the actual `record_set_map` method at `gpu/grow/maps.rs:5-7`.
   That method records the map assignment described by its documentation.
   No implementation moved or changed with the documentation.

4. `gpu/mod.rs:9-11`: placing roots in one batch, reading counts, then recording
   the remaining groom in another matches `gpu/grow.rs:140-164,319`. The former
   one-batch/one-submit claim is gone. Current source does not claim that the
   recording API itself makes the whole grooming call asynchronous.

5. `gpu/buffers.rs:11-12`: scratch reuse while requests fit and growth by at
   least half again match `Scratch::fit` at 19-56. This no longer claims repeated
   grooming makes no allocations. Returned roots, attributes, positions and
   guides are separately allocated by the actual grow/guide producers, so their
   stated independent ownership matches those retained outputs.

6. `gpu/scalp.rs:1-3` and the existing `gpu/mod.rs` example now require the same
   head geometry, hairline and sampling recipe for reuse. `Head::scalp`,
   `ScalpGrid::new` and head-shape upload depend on these inputs; hairline alone
   was insufficient. The wording is a reuse condition, not an admitted device
   or geometry-identity proof. Those authority/admission repairs remain open.

7. `gpu/wgsl/guides.rs:9` now says one workgroup per guide. `guide_comb` uses
   `@workgroup_size(8)`, obtains the guide from `workgroup_id.x`, and samples
   seven field probes through its lanes; `gpu/guides.rs:347-360` dispatches
   `[count, 1, 1]`. This matches the actual comb scheduling.

8. `head.rs:7-8`: approximate signed field and estimated outward normal match
   its component ellipsoid approximation, smooth combination and central
   differences (`head.rs:39-55,146-173`). The unsupported exact-distance-anywhere
   wording is removed. No new precision, timing or hardware comparison is added.

The previous discovery findings NS-C1 through NS-C8 are resolved in this
tranche on the above fresh source evidence. No historical verdict supplies
additional coverage.

## Exact comparison evidence

All eight current SHA-256 fingerprints are recorded below. For each complete
file, the executable/noncomment bytes equal the saved source after reversing
only the two declared semantic edits: final edge `submit()?` and removal of the
first-corner rejection. The six other files have comment-only changes.

For the WGSL owner, restoring the exact removed five-line prune makes the
entire imports-through-EOF region byte-exact to the saved source. All declarations
before `edge_closest` and from the following guide-selection declaration through
EOF are already byte-exact. This covers the retained triangle traversal, tie
condition, closest/barycentric mathematics and all other generated native bodies.
Complete existing test modules in `gpu/mod.rs`, `head.rs` and `surface.rs` are
byte-exact to the saved source.

Machine-readable source comparison:
`/tmp/pr780-native-scalp-first-source-comparison.json`.

| Source | Saved SHA-256 | Current SHA-256 |
|---|---|---|
| `crates/fabelgeist-hair/src/gpu/scalp.rs` | `8497dc68d32bee89caf41854e8cb0492416acb664f9546b98b5f027e530b0b06` | `40486b2fcb0f2a2742a0a43ac588c71ba46f47495236def99dd6f0b27c821447` |
| `crates/fabelgeist-hair/src/gpu/grow.rs` | `6b746c55f7cc60f88cdac24dbd2b2486d56a439ac5b49dc5c689a07b2f18d0b0` | `b4907574e60ff614aef176c62a138331eb3bc047cb799db0fe6f82d5f92b6c26` |
| `crates/fabelgeist-hair/src/gpu/grow/maps.rs` | `7ff8d6ba755cf671762b1aa0dae9c9f876f074813459b5e4ae2989874e18157d` | `7b8b6157448397c687e7da311bb8ca6a933455934080093196854e20c1c0c0bc` |
| `crates/fabelgeist-hair/src/surface.rs` | `0c54c21fb94e4b1df748a29cdbef562802d717acfc810211072be30d236a3298` | `0cf3b1a2d8fa1e50f2494291f44e803aff7501f7d5f92763c87ef54a5db0ac0d` |
| `crates/fabelgeist-hair/src/head.rs` | `5d1dcadadba0fa88cbd668791a3a995a7ef83af5fdbaa3b699b828531f339335` | `c95b6fcaaec095658c85f5c325781ecd1c349ddefb7b7eecd018ab61b500f709` |
| `crates/fabelgeist-hair/src/gpu/buffers.rs` | `6ebf50208593b3f1c16352bc3665e35f74cb0b01c83db56807bf3c0b83da568a` | `e82d568414119b0fd6077ab911224dc9eee4eefb2d6a5cf7ff38c98dea9a78e1` |
| `crates/fabelgeist-hair/src/gpu/mod.rs` | `f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22` | `f5696d4762b8a0414cd89406639a49edeabbd1acbd330df25eeee220291cac9a` |
| `crates/fabelgeist-hair/src/gpu/wgsl/guides.rs` | `bcd60aee22ae82fae270b38599aca660f0432c0a6baf72d58241a88da4cfa840` | `7eb85b1be589db745f6411247e2b22ee076a63ebf49f97dcec29fab7caf14043` |

Relevant receiving-owner fingerprints (read extent described above):

| Source | Current SHA-256 |
|---|---|
| `crates/fabelgeist-hair/src/gpu/readback.rs` | `e78544e5d43fd63fa4be775f79e0488d0e35e952982c7d94ce8d0dac2487d7ea` |
| `crates/fabelgeist-hair/src/gpu/scalp/layout.rs` | `1934e3eae9d1344b28b6cfd751a0fb691b3a81c64c1a18f2aea41a65f4e3f036` |
| `crates/fabelgeist-hair/src/gpu/guides.rs` | `f4c9e231efc1a770670e14030140640b5d22d12e26814c7c6613259395e4e71f` |
| `crates/fabelgeist-hair/src/hairdo.rs` | `7adbce20aa4fb6390748d4a83ad91e4c31a976db817f0a26c321f5c415393f14` |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74` |

## Explicit limits and remaining work

The whole native count/Grid/source/device/logical-prefix migration, guide/root
admission and error handling remain open. In particular this report does not
clear unchecked grid products, source geometry consistency, empty/degenerate
surface admission, retained scalp device identity or manual logical buffer
trimming. The separate frozen fifteen-file material/scalp asynchronous progress
tranche and the generated Map/shared-signature changes are outside this verdict.

Full files were read, but unchanged simulation/LOD algorithms and their complete
receiving graphs were not re-audited merely because the existing tests live in
`gpu/mod.rs`. CPU/GPU exact geometry claims were assessed against the inspected
producer/order implementation and the existing limited parity fixtures; no
universal numerical identity or runtime success was established. Unchanged
external provenance references were not independently reverified in this narrow
tranche. Final complete reviews and hosted required checks on one final pushed
revision remain pending.

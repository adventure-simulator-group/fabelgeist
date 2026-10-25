# Incoming Scalp chain Comment Review

One retained wording finding at canonical
`c6659f6cfa4610f066c659411ade6636e21bd3b5`
(tree `12f2d808ab9d659999aaec627e02e594897c8beb`). All nine owning files were
read fully and match the manifest. Fourteen contract dependencies were
also read fully; additional bounded/carried coverage is explicit in the JSON.
Peer mapping/readback helpers differ from Root visibility/poll_complete and are
unlisted dependencies: preserve Root native helpers during composition.

SCALP-C1 (P2), `hair/gpu/mod.rs:3-5`: the unqualified claim that kernels run
about ten times faster than every CPU core lacks hardware/workload/benchmark
bounds. Source inspection and existing fixture GPU timing output do not support
that general comparison. Remove the multiplier while retaining independent GPU
strand sweeps, or cite an already existing bounded observation. No benchmark,
test or numerical behavior change is requested.

The fixed padded mip layout and visible-texel/shadow-block distinctions match
actual allocations, dispatches and unchanged WGSL addresses. Host chain reads
explicitly check the creating Device, read the exact prefix and strip row
padding. Bevy upload correctly documents configured native handling without a
synchronous verdict; it does not promise rollback, retries or completion.
Canonical async completion and existing Stage adapters retain their limits.

Root Assets differs as a whole, but its complete scalp_images function is
byte-exact to peer. There is no missing scalp receiver at that function. Do not
overwrite unrelated geometry/strip functions based on the whole-file hash.
The existing render fixture differs from parent99d only by its import and Stage
payload type adapters; WGSL is exact. No cases, helpers, assertions or oracles
were added or run.

Root has not received this handoff. Canonical native counts/stride/layouts,
physical/generated geometry/Grid/producer/caller/Readback/lock composition and
separate native GpuGroomer completion cleanup remain required. Preserve Root
native policy docs. No source edits, project checks, runtime, push or CI were
performed. Full all13 waves, final independent reviews and hosted gates remain
required on one final pushed revision; no global or final clearance.

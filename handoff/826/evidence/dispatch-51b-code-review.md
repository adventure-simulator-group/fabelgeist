# Independent dispatch successor Code Review

Revision: `51b21447649f0aa53be19a20cdfc066863617deb`.
Tree: `114c758d7049d1af96fc1d257a8be52bd81ea56e`.
Parent: `cf7646a96ac5ba89696eac925f14593e0b13c884`.
Comparison: `65e8aebedb6490bc2945eac6d48479ea6a2cd10a`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
Peer proof: `95f9d31b7d9fbb56aa8330c72fffb46fb8e0c0be`.

No new owned actionable correctness finding in this bounded review. DG65-C1
is resolved: shape admission constructs the grid and returns an empty grid
before imposing nonempty y/z restrictions. Zero invocations produce `[1,0,1]`
for every admitted nonzero shape, preserving no-op admission.

The complete nonempty padded rectangle remains checked with u64 products
and at most `2^32` lanes, so its greatest address fits u32. Ordinary covering
and exact rejected-axis diagnostics are preserved. The lower receiver checks
all axes against the actual receiving device. SDK29.0.4 performs the same
all-axis test, including an excessive axis paired with another zero axis.
The result alias preserves the concrete host error; actual native failures
retain their original wgpu cause. Receiving-device documentation is precise.

The peer's complete native scan source establishes a different safe contract:
its workgroup-uniform `block >= block_count` guard runs before lane products,
sums writes and every barrier. For u32 count and BLOCK256, valid block indices
are below `2^24`, so the greatest valid lane fits u32. The folded group index
itself stays below `65535*257`; padded groups return before unsafe lane math.
The recursive producer carries that exact count to both shaders. Typed grid
geometry therefore suffices here. The earlier proposed public group-input
projection is unnecessary and its speculative requirement is withdrawn.

All five dispatch files were freshly read completely. The JSON records exact
source hashes, fresh affected-body reads, unchanged source identity carried
from65e, complete peer scan production/native-source proof and SDK excerpts.
Both existing fixture files and the transferred compile-fail block remain
byte-identical to canonical95; no tests/cases/helpers/assertions were added.

#780's actual-device guard in KernelBatch remains required after its empty
return and before FastPath preparation. Its fallible Kernel/caller migration,
remaining native producer/layout/source/count/creator families and lock
integration also remain required. No shared caller source was edited here.

This is source/Git/provenance review only: no project execution, check,
build, test, scanner, generator, runtime, push or CI. It establishes no
compilation, whole-family, all13-lens, whole-PR or final clearance. Final
reviews and hosted gates remain required on the same final pushed revision.

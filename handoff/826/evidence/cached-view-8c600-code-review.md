# Cached-view commands Code Review

Head `8c600f727d1eecf21dd241f02739b825edef5000`; tree `1c141f5f1a28cb532cb51b472ba7e8fcc01486ae`;
parent `47f37a07687bc47347cedb9e274c2f273f9deae6`.

No actionable implementation finding in the bounded cached-view blit/clear repair. Source-only review does not establish combined source compilation or final/global clearance.

Texture2d::blit observes unchanged Blitter::new (both shader/pipeline programs,
sampler/layouts), native encoder creation and actual Blitter::blit
bind/pass/draw operations under one immediate observation. View.clear likewise
observes encoder creation and pass lifetime including drop before closure
returns.

Both closure operations are infallible in their Rust return shape and contain no
?/early application Err. observe_validation always pops after normal completion
and returns the first actual native Validation cause. Thus no nested body-
primary/secondary resolver is needed here; genuine native Rust panic/unwind is
not converted into an invented application error.

An observed error prevents submission of that encoder and returns
ComputePassError::Native(Arc<actual wgpu::Error>). Existing Error::source
exposes the original native object. Any deferred finish error and subsequent
queue error are separately observed by unchanged WgpuContext::submit. Neither
stage waits for completion or promises rollback of prior native work.

Immediate observer pushes/pop only on native contexts with blocking_validation
enabled. It contains no device.poll or GPU completion wait. Browser/borrowed
modes retain configured provider error handling and no synchronous validation
verdict; non-Validation classes retain native handling.

No new GPU state/cache publication or callbacks are introduced. Local Arc clones
keep view/device lifetime through the call. No source/native allocation port is
newly required for cached-view clear/blit.

The old clear policies are equivalent: all integer arms scale each f32 channel
by255 before its f64 native adapter; depth uses1 only for exact zero RGB, else
color.x. Shared View.clear preserves cached-view/device error priority and
allocation-independent operation. Diagnostic labels now name View.clear.

All thirteen bounded preinventory contracts are source-reconciled in the JSON.
Full Blitter/native shader source and existing2d fixture suffix independently
match the parent exactly. No new source finding was identified.

Current canonical ResourceDescriptor/storage/value/coordinate/ResourcePipeline/r
ecipe/stride/BoundaryMode and every affected caller integration remain required;
Root current unintegrated API graph is not compile-accepted.

Peer-owned Advect selected-view/sRGB repair remains separate required connected
work.

Held extraction/native producer/creator/application/manifest-lock families
remain required, with all final independent Code/Rule/Comment Reviews and hosted
gates on same final pushed SHA. No merge.

Read-only source/Git/provenance only; no source edits, project
build/check/lint/test/scanner/runtime/generator or new
tests/cases/helpers/assertions, push/CI/external posting.

Both complete changed files, complete actual Blitter module and current native
policy/error helpers freshly read; unchanged selected-view/format/error-
declaration/Cargo owners exact-hash carried with explicit prior coverage.

Malformed resource/native-error timing reviewed against already pinned SDK29
sources and unchanged helper contracts, not runtime reproduced.

Existingfixture suffix proof is byte identity, not an executed test; no
wholePR/all13global/sourcecompile/final clearance.


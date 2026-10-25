# Independent Comment Review: Advect decode and clamp delta

HEAD: `fff01705f4eaae22f98242ce64e538e02ac76e86`, with only
`crates/fabelgeist-compute/src/advect/mod.rs` dirty.

Current SHA-256:
`55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a`.

Verdict: CLEAR for the narrow two-defect source/comment delta. No actionable
comment finding remains here. Broader generated-program-family audit and final
combined reviews remain open.

Read all current Advect source and its diff. Read canonical
`fabelgeist-compute/src/boundary.rs` directly, not the nonexistent
advect/boundary.rs. Its SHA-256 is
`31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8`.
Read the selected-format discovery report and the thirteen-lens
advect_decode_clamp_preinventory entry as scope and evidence context; those
historical records are not themselves clearance.

Source-backed results:

- The false claim that textureLoad does not automatically convert sRGB is
  removed with the duplicate decoder and its generated Boolean. The pinned
  local wgpu-types 29.0.4 texture/format.rs:101-117 explicitly states that shader
  loads/stores apply format conversion and that sRGB storage yields linear
  shader intensity. The helper now directly interpolates loaded values and
  returns them, with no competing prose or transfer-function implementation.
- BoundaryMode::Clamp promises the nearest sample on the grid edge. The 3D
  quantity address now takes a final integer minimum with dimensions minus one.
  The normalized upper endpoint therefore selects the last texel rather than
  index equal to dimension. The actual Texture3d constructor on the selected
  producer path clamps every axis to at least one; the subtraction is meaningful
  for admitted initialized textures. This source repair matches the canonical
  comment without changing the documented policy to native out-of-bounds
  behavior.
- No new explanatory comment is necessary for the local min-address guard or
  the direct interpolation return. The code states the operation clearly, and
  the boundary policy already owns its contract. No new claim is made about
  successful native validation, device health, GPU completion, arbitrary mixed
  descriptor compatibility, 3D filtering, or runtime acceptance.
- The existing `AdvectSignature.dimension` inline comment is unchanged and is
  not treated as admission proof or as the execution dimension authority in
  this narrow review. Broader signature/descriptor review remains open.

No source edits, project checks, tests, cases, helpers, assertions, documentation
samples, CI requests or external posts were performed. Only this local report
was written. This source-only verdict does not supply runtime test evidence or
final-pushed-revision clearance.

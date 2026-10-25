# Complete renderer integration Rule Review

Pinned `2b646df899ee73045795b129bfbade5dec7d4b84`

Tree `8ff9d17406c41db64dd32c901ad1bf054caf5475`; canonical dependency pin
`e2162708fcf3885193db434300ed6c2999ab7607`.

Two actionable closure gaps remain after a fresh complete read of all18 renderer
files, including all shaders and the parent fixture. Earlier receiving-only
receipts do not grant this complete family clearance.

P1 — lod/policy.rs:6 and lod/presets.rs:6 import fabelgeist_math::Ratio. Actual
Root and exact canonical peer math/lib.rs expose only units::Ratio. Those policy
producers supply the renderer’s Form/Blend/Representation values, so fixing
draws alone leaves unresolved imports after composition. Use units::Ratio at
both producers; add no compatibility root export. This follows the canonical
owner/propagation rules in crates/AGENTS.md:28–57,69–71.

P2 — render/mod.rs:53 retains an unused DeviceExt import. All native upload
calls now live in focused modules with their own trait imports. The required
justfile759 and hosted rust-quality workflow150 Clippy gates deny warnings.
Remove the parent import; no suppression or forwarding alias is needed. This is
source evidence, not a reported lint execution.

The three previous corrections hold: the canonical HairGpuError enum exclusively
owns Atlas conversion, draws uses units::Ratio and frame_uniforms uses
units::Metres. Full explicit renderer From pairs were compared with canonical
error derives; feature/export routes for cards, native layouts/Sim/Source, mesh
counts and color native mapping were inspected. No further duplicate conversion
or unsupported forwarding API was found in that bounded graph.

Source traces in the JSON cover actual host/native source Arcs, immutable host
frames, native buffer identities, maximum resident binding population,
independent guide arrays, logical versus padded bindings, actual
usage/storage/copy limits, prior rest preservation, candidate publication and
actual causes. Unchanged draw/frame/readback and native numerical/shader roles
were read in full. Default Renderer error policy remains device-handled; no GPU
completion, global health or rollback guarantee is claimed.

Existing fixtures/POD/shaders are exact, no new
tests/cases/helpers/assertions/doctests or production unwrap/unsafe/suppression
was added, and ceilings were not raised. Required baseline reductions remain
hosted/final work. No project
checks/build/lint/tests/scanners/generators/runtime or source edits were
performed. The two findings, complete canonical
producer/caller/Creator/physical/lock integration, and all13/reviews/hosted
checks on the same final pushed SHA remain required.

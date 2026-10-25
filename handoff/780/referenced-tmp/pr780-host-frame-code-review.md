# Independent Code Review: host SimFrame, binding and settings delta

Base and HEAD: `a2ca851015b9649097635f9f580e409028682ffa`.
Reviewed the current uncommitted delta; captured source fingerprints below.
Read only `host_shape_sim_frame_current_discovery` from the ignored ledger.

## Findings

No actionable correctness defect demonstrated in the requested delta.

This is source-only review, not compilation/runtime evidence, global audit
clearance or final independent review of a combined pushed revision. No source
edits, builds, tests, project checks, new tests/cases/helpers/assertions, hosted
CI requests, pushes or external posts were performed.

## Source-backed contracts

- `sim/build.rs::HairSim::new` retains the actual first drawn level and the
  actual nearest-stride sampling level from the same sealed HairLods collection.
  Its unsigned `abs_diff` selection avoids the previous signed-cast/absolute
  overflow and preserves first-level precedence on ties. The only other source
  construction route, `from_level`, passes the same Arc for both roles.
  `from_sources` is private, so arbitrary unrelated same-count levels cannot be
  publicly combined into a fabricated binding.
- The common sealed HairLods producer establishes matching population/order
  across those levels. The simulated count is the requested prefix capped by
  available strands, with minimum one only for populated sources. The simulated
  stride is at least two, including zero-strand inputs. Prefix slicing fits the
  sampled positions; empty inputs slice zero records without inventing a guide.
- `from_sources` checks guide count fits u32 before PointGrid construction and
  ID casts, and checks both doubled stride and doubled total sample count before
  allocating the rod shape/frame arrays. Owned initial arrays use fallible
  reservation. The installed binding shape remains independent of subsequent
  mutation of requested settings counts.
- `SimBinding::build` retains immutable actual source and sampling Arcs, the
  checked simulation layout and full-source guide/weight arrays. Its direct
  prefix mapping is justified by common shuffled source order. Simulated roots
  come from sampled rest points; other drawn roots now come from the actual
  first drawn level. That corrects multilevel target-root selection without
  changing single-source numerical behavior. Compared with base, guide grid
  spacing, nearest-four selection, inverse-square weighting and normalization
  retain their exact arithmetic and per-strand ordering.
- `sim/frame.rs::SimFrame` has a private HairSim borrow and no public constructor.
  Its rest, positions, head, layout, offset and actual binding therefore refer
  to one live immutable simulation borrow. The borrow prevents stepping or
  mutating settings while that frame is live. Binding fields are private to
  simulation construction and exposed as borrowed slices; an independently
  cloned binding Arc cannot mutate those arrays or replace the installed record.
- `sim.rs` derives all physics chunk strides from the installed checked layout.
  The positive stride and prechecked doubled extents cover shape/frame chunks;
  equal private array lengths follow construction. Compared complete reset,
  shape, orientation, step, air and pose bodies with base: numerical operations
  and their ordering remain unchanged except extracting the existing Metres
  value at the rod numerical boundary. `pose.rs` preserves rigid transforms and
  blend arithmetic. Native constructors/collision/inspection likewise propagate
  existing typed counts, with raw values extracted only at their existing
  numerical/native ports.
- `sim/settings.rs` preserves authored defaults and SimLevel identities while
  using existing StrandCount/PointsPerStrand leaves. The canonical Metres owner
  stores the original f32 unchanged, so negative, nonfinite and ordinary offset
  values retain their existing policy; no new positivity or finite clamp was
  introduced. Native execution explicitly projects `.metres()` at its WGSL port.
- Static Rust API inspection found no new visibility, sibling-module access,
  generic inference or temporary-lifetime defect. In particular, frame accessors
  return references tied to the HairSim borrow, so the existing CPU-reference
  case can keep `bound.frame().binding()` after the temporary frame value ends.
  Existing rustdoc paths referenced by the extraction resolve to current owners.
  Fixture adaptations preserve the same cases, numerical inputs and assertions;
  the standalone existing helper now supplies a checked one-strand source level.

## Coverage and limits

Read all four new modules, complete current production simulation bodies and
base counterparts; the native changed constructor/collision/execution/inspect
sites; existing host/native fixture adaptations; canonical shape/level invariants,
PointGrid production source, and canonical Metres owner. Renderer and Creator
receiving composition remain known separate dependencies, not new defects
attributed to this extraction. Broader physical/grid math remains pending and is
not cleared by count/array admission. No source-only conclusion establishes
successful compilation or hosted runtime validation. Final complete audit waves,
combined independent reviews and required checks remain necessary.

## Captured working-delta fingerprints

```json
[
  {
    "path": "crates/fabelgeist-hair/src/gpu/mod.rs",
    "sha256": "2b2bc94e4178a098e4ae5dac1794f82380d037922fce244ce833a0d73482b7fe"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim.rs",
    "sha256": "413f88a20d71eac80a2028f44826ba8f910df760078175d6eba83321233943bc"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/collision.rs",
    "sha256": "432b9250fbd2810918ab7ca663e1596d715b4cd6a7f4491b109b3dfd6cc8fe6e"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/execution.rs",
    "sha256": "09689764502410c3fcd3bede867e66b30941a329ebfc280438f822c9b928c3cd"
  },
  {
    "path": "crates/fabelgeist-hair/src/gpu/sim/inspect.rs",
    "sha256": "f019273dd3c0d3c216f8e2b48b7ecf49fd9d8333f5b26f81b8e3f936777d1597"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim.rs",
    "sha256": "61f1db429d25290869a495bcfa270d73d92a203fb5de405f255384f9e8081632"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim/build.rs",
    "sha256": "badb20a3e56c8fe3a51a2c629a6bf9b5fcdbac0c9f9f52774ebabcf660d18d5d"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim/frame.rs",
    "sha256": "0bec2f5a59de545d2968c962ab28be07e91d685dc097fa0a17af5db09312adc3"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim/pose.rs",
    "sha256": "0cc72e5d058745c9d41c7de3f31fba92c839ae85845dd599bcf3c1fe50c892f4"
  },
  {
    "path": "crates/fabelgeist-hair/src/sim/settings.rs",
    "sha256": "8f900903b50fd6f9ba4ca81c01c2205a4be919589899569594db60a4041d3b32"
  }
]
```

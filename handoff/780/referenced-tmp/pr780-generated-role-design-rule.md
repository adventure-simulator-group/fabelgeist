# Generated-role representation: independent Rule design review

Root worktree: /home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator.
HEAD 99d70e7c3f0bcf8db563c602f39869b91a9f23bf; nine unrelated scalp-family
paths are dirty. The generated-family files below remain unchanged from the
complete source discovery: all 51 recorded fingerprints match current bytes.
Fresh targeted reads cover all five signature/parser producers, shared native
value/coordinate owners, Gather/Scatter/Stream call selection, Stencil mode and
binding/call emitters, and both DistanceField wrapper bodies. The prior complete
discovery supplies the unchanged remaining caller/native-boundary coverage;
its historical verdict is not clearance for future implementation.

The proposed representation fits the current source. Two details must be explicit:
only actual declared return types are retained separately; role-derived resource
and input types are queries. Resource membership and native wrapper syntax must
be classified before ResourceBaseType cleaning erases their outer structure.
No implementation, project checks, tests, cases, helpers, assertions, scanner,
build, runtime, CI or external post occurred. This is design acceptance input,
not final source approval or a substitute for Root's unfinished all-thirteen
exact-API preinventory.

## Representation contracts

1. GPU signature.rs is the lowest shared owner of ShaderParameter: a native
   name plus ResourceBaseType, private fields and read-only getters. Existing
   CoordinateParameter already owns name plus admitted coordinate shape; reuse
   it directly for coordinates. Neither native leaf asserts full WGSL validity,
   identifier validation or alias resolution. Preserve open Custom identifiers.
   No new dependency is required for this record.

2. One private ordered Vec<SourceParameter<local role>> per signature replaces
   all raw-argument/name/user/index/size projections and Stream presence flags.
   SourceParameter owns the original trimmed declaration text and its admitted
   role together. The role owns its name through ShaderParameter,
   CoordinateParameter, or the neighborhood payload; do not add an outer name.
   Expose no mutable vector, declaration setter or role setter. Derive authored
   argument order, coordinate/user/resource iterations and presence predicates
   from that vector. Temporary borrowed partitions are valid; independently
   retained columns or new positional (name,type) tuples are not.

3. The shared Compute lexer should return named ephemeral borrowed slices of
   declaration/name/native type, respecting the existing angle-depth comma
   splitting and required colon pair. It has no algorithm-role authority and
   should not become a general WGSL parser. Preserve native custom/pointer/
   Resource/array spelling. DistanceField::generate_min and generate_smooth_min
   (distance_field.rs:31-166) extend the original declarations and call original
   names in order. Rebuilding these from ResourceBaseType::as_str loses lexical
   information, because clean_type_name (signature.rs:94-134) strips wrappers.
   Verbatim text is a justified immutable emission projection, not a second
   independently mutable signature or source cache.

4. Retain a private output ResourceBaseType for Map/Gather/Stencil's actual
   return declaration. Scatter and Stream are void writers: their output type
   currently comes from the last resource parameter, falling back to F32
   (scatter.rs:105-153; stream/mod.rs:88-143). Derive that result from resource
   roles rather than also storing it. Likewise derive input type from the last
   value/resource or neighborhood role, with the same existing F32 defaults.
   Do not silently change last-role selection to first-role selection. Native
   validation still rejects unsupported combinations; these are source holders,
   not proof that every admitted signature is an executable shader.

5. Map's primary is contextual: build_pipeline:147-161 selects the first
   ordinary parameter only when an actual input descriptor exists. Remaining
   parameters split by actual secondary_resources membership, preserving authored
   binding order. Do not freeze primary/resource/uniform roles from native type
   text at parse time. A generator with no primary input remains valid.
   SecondaryResources sorting is cache identity, not binding/call ordering.

6. Gather has index/resource semantics, not an ordinary in_val feature.
   Its emitter (gather/mod.rs:253-261) passes only the selected index; do not
   start forwarding unsupported extra parameters as values. Scatter/Stream
   select an explicit supported index before considering an unsigned alias.
   Only without that explicit role does the old first eligible unsigned alias
   fallback apply; Stream offset is excluded and remains exactly u32. All
   remaining ordinary unsigned parameters retain value roles. This corrects
   Scatter's dropped unsigned value and Stream's earlier-value/later-index
   disagreement without adding a DSL feature. Float-only no-index values and
   Stream programs with no input value remain valid. Multiple ordinary values
   continue using the existing in_val semantics, not new per-value resources.

7. Resource roles need the original native type slice at admission. Cleaned
   F32/Custom alone cannot distinguish a scalar from Resource/ptr metadata.
   Decode actual outer native Resource/ptr/texture forms once; do not treat an
   arbitrary Custom name containing Resource, ptr or Neighbors as a protocol
   role. Preserve MC/DC's _res pointer metadata, native global output writer,
   already-authored globals and reflected custom record stride. Do not rename
   _res into a local writer parameter or synthesize new function arguments.

8. Stencil neighborhood payload owns native name, closed 1D/2D/3D/4D dimension
   and element type. CoordinateShape is not the 4D neighborhood owner. Derive
   neighborhood mode/dimension/input type from the admitted role rather than
   storing another bool/tag/dimension/name combination. Optional center is
   legitimate. Preserve explicit center/val precedence, then the current first
   non-coordinate, non-neighborhood fallback only when no explicit center exists
   (stencil.rs:156-188). Reject duplicate neighborhood or center roles explicitly;
   record that earlier error timing instead of claiming all old parse outcomes
   identical. Exact supported Neighbors1D<T> through Neighbors4D<T>, with their
   existing whitespace/native element forms, select the closed dimension.
   Unknown reserved family forms must not silently default to 2D; arbitrary
   unrelated Custom identifiers remain open. Preserve all dimension field aliases,
   boundary modes, native conversion/lookup formulas and current optional-center
   call ordering. Emitter behavior must consume the admitted roles, not reparse
   declaration text to recover them.

## Connected acceptance and thirteen-lens check

| Lens | Design requirement before implementation can clear |
| --- | --- |
| Producer propagation | All five parsers, builders, execute/descriptor paths, Stream call helper, DistanceField and existing fixture API accesses consume the final owner directly |
| Existing owners | Reuse ResourceBaseType, CoordinateParameter/Shape, TextureValue, ResourceFetch, BoundaryMode, ResourcePipeline and canonical native launch owners |
| Admission | Private aggregates; exact duplicate/reserved-role policy; no new unchecked public mutation or fabricated successful-program guarantee |
| Invariant operations | One ordered sequence drives binding/call selection; preserve last-role/F32 fallbacks, explicit-index precedence and numerical emission |
| Aggregates | No independent name/type/coordinate arrays, mode fields, presence flags, cached role-derived input/output types or cached signature beside mutable Definition.code |
| Ownership | GPU owns reusable native leaf; Compute owns lexer, declaration association and local algorithm roles; no all-algorithm role enum or competing native leaf |
| Redundant authority | Immutable original declaration is lexical data only; query semantic roles and reparse current Definition.code on use |
| Behavior | Preserve generators, resource globals, pointers, aliases, authored order, native programs and existing fixture cases/oracles; explicitly name intended earlier errors and unsigned fixes |
| Enum shapes | Local payload-bearing roles, optional center, exact closed neighborhood dimension; boolean query predicates are derived, not retained flags |
| Derivability | Derive equivalent Definition Defaults; retain code-only or code-plus-boundary PartialEq and generic cache Default; no unjustified new error dependency |
| Granularity | Smallest useful shared leaf and local roles; no new identifier branding, fake numeric bounds, compatibility aliases or temporary primitive adapters |
| Strings | Classify reserved source protocol once; preserve genuine native strings/declarations; no mutable prose/Debug dispatch |
| Numbers | Preserve workgroup declarations, lane math, binding numbers, neighbor offsets and native kernel policy; no sorting count used for launch quantities |

Drop unused constructor context arguments directly and update actual calls;
Map::new already has no context. Remove rather than forward any obsolete API.
Add no tests/cases/helpers/assertions/doctests; adapt existing helpers' API calls
only. Keep fixture inputs and oracles byte-identical. Do not use new helpers as
an implementation-validation workaround.

Extract real parser/schema, neighborhood emission and cohesive binding/call
responsibilities before expanding Map or Stencil. The original Stencil file is
1082 lines and its build_pipeline body spans roughly584; Map has495 lines.
Each resulting module/function must satisfy the existing no-growth rules,
ordered imports/mods/aliases/types/impls/functions/tests-last, without include
fragments or pass-through splits. Relevant baseline reductions remain required
through authorized hosted project checks; none ran in this review.

Keep the separately inventoried coordinate/address Result-alias/item-order
findings and adjacent unused signatures/defaults explicit. GPU has no direct
thiserror dependency; derive-based error cleanup remains a coordinated manifest/
lock/ownership decision, not an assumed available local derive. Preserve exact
messages and original causes if changed. Root reports native signature/coordinate/
value/address owners agreed; peer native pipeline/pass functions remain untouched.

Stencil's missing-axis 64 Buffer defaults and native i32 extent arithmetic,
physical/Grid admission, incoming GPU composition and all remaining PR families
stay open. This design does not authorize their speculative rewrite or claim
runtime correctness. Root must complete all-thirteen exact API/contracts/
dependencies/acceptance inventory before source repair, then review the complete
implemented graph and final same-pushed-revision hosted results.

## Current SHA-256 identities

- `crates/fabelgeist-compute/src/map/mod.rs`: `6bac7f1ff95532004d88a383de80e559f550f4e8d3223d5adbbffd50a8907071`

- `crates/fabelgeist-compute/src/gather/mod.rs`: `95da5cd7a5e390cb9ea21c4dd61be55d324726a9d1dfef9c9bc769629318bfd9`

- `crates/fabelgeist-compute/src/scatter.rs`: `20e9e3f15f7e286e98018ec8bba3d8e4d430e24a13d800d2fd8246f8f0c50932`

- `crates/fabelgeist-compute/src/stream/mod.rs`: `5d8fdfbb20b5107dc061d22f3bae3611904ea4828ec4ee38fd47ce3518e5ba6e`

- `crates/fabelgeist-compute/src/stream/call.rs`: `a164da4c7d62a4eb24a6a3b0e1d6f0a15851ed90747e5af07dd9f43b08b941b9`

- `crates/fabelgeist-compute/src/stencil.rs`: `a5552583fdd75934fc3f6f9623ca45fc7de5f4ae2def11544c19d984dc09cdf6`

- `crates/fabelgeist-compute/src/distance_field.rs`: `2e0eea32b384b061a6b1bbb3162b077479bfefde32e196a8978e66d6c3b84b69`

- `crates/fabelgeist-compute/src/map/tests/helper.rs`: `307a22684a265bcbdd41fc0b7b038b9cdec839b1988eed7f66b8390fee2e6a5d`

- `crates/fabelgeist-compute/src/gather/tests/mod.rs`: `6a9360ebc741c55a8068c062857298ce9ee455bb10ce99202fd8f03cb71ee160`

- `crates/fabelgeist-compute/src/stream/tests/mod.rs`: `16f48fa721c0981cf7799288e7e4e898e1f6e2481b312bce73487c10070da743`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs`: `991f1eebf5b1a9cf7f8fa7314793b2610a78fa7f5b01236a19c84ea9f73773fd`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs`: `505a16459267c2280842514d42eb3aded371b6a2a84584ba6faf9ebb11c4ca2f`

- `crates/fabelgeist-gpu/src/data/gpu/compute/signature/texture_value.rs`: `79124150e675e45b70b38b8287f82d80b7eda48202e0d2fa6e24f880bb153de6`

- `crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs`: `686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836`

- `crates/fabelgeist-gpu/Cargo.toml`: `707cc7a7a86fd855e779b3233b4dd97d3fd2be4496aa550a4388a1dec80d8a58`

- `crates/fabelgeist-compute/src/boundary.rs`: `31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8`


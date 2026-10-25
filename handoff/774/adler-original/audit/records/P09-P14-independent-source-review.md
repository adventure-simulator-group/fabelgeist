# Independent P09–P14 source review

Reviewed the connected increment `6aee89ebf17bf3e823ff9003bca82596f608da0f`
against `8f2b901881f8aa8035a03cd99dbfefb7bbdf8926`, then reviewed the exact
repair `8be48645f4a92ab41723240ea2e957472706fc39` against `6aee89e` on the
same task. The source worktree was clean after the repair.

**Verdict:** no open actionable finding after the repair. The reviewed
production paths meet the scoped P09–P14 exit requirements from source. This is
not a compilation, numerical, shader, test, visual, or runtime acceptance
verdict. P09 was incomplete at the initial head because one existing fixture
receiver was missed; its exact source repair closes that identified gap.

The review followed the rule-review skill and its bespoke-type reference. Root
and scoped instructions were read, including `crates/AGENTS.md`, the Creator
guide, the shared armor guide, and the core guide. Explicit user restrictions
superseded their executable validation guidance. No project commands, Cargo,
checks, tests, formatters, scanners, Rust parsers, generators, numerical probes,
or runtime probes were run. No source was changed by this reviewer. Only Git,
source reads/searches, and standard-library JSON metadata inspection were used.

## Findings and repair disposition

1. **[P1, closed] Complete the existing clothing admission fixture's typed
   operands.** At the initial head,
   [clothing.rs:694](SNAPSHOTS/rig-quality/crates/adventuresim-character-creator/src/clothing.rs:694)
   and the following name/state operands still supplied raw arrays to
   `generate_clothing_shells`, whose parameters at lines 112 and 114–115 had
   become `PackedRigSkin<8>` and `RigJointTable` references. The existing
   `absent_landmark_keeps_its_identity_and_error_classification` test could not
   type-check in Creator test/all-target builds. This was a concrete missed
   receiver, consistent with the producer-propagation requirement in
   [crates/AGENTS.md:36](SNAPSHOTS/rig-quality/crates/AGENTS.md:36)
   (lines 36–42). Repair `8be48645f` admits the unchanged zero words against
   authored extent 1 and constructs the unchanged ROOT/state rows as canonical
   tables. The missing-C_NECK assertion and all numerical inputs remain intact.
   This closes the static type mismatch; it is not an executed test pass.

2. **[P3, closed] Place the new prefix-level record before functions.** At the
   initial head, `skel_state.rs:118–121` introduced
   `PrefixMultiplicationLevel` after free functions. That violated the explicit
   user item ordering and
   [crates/AGENTS.md:90](SNAPSHOTS/rig-quality/crates/AGENTS.md:90)
   (lines 90–94). Repair `8be48645f` moves only the declaration and attached
   documentation above the functions. No behavior changed.

Both findings were sent promptly to the coordinator and original implementor.
No additional demonstrated correctness defect was found. No speculative
strengthening of public aggregate APIs is presented as a requirement of this
increment.

## Source-backed family traces and completeness

### P09 — joint columns and packed skins

**Complete for the reviewed scoped migration after the fixture repair.**
`RigJointTable<T>` owns rig-order columns (`fabelgeist-rig/src/table.rs:11–94`),
while `PackedRigSkin` owns unchanged native u32 rows plus the actual source rig
extent (`fabelgeist-rig/src/skin.rs:6–16,108–226`). Admission checks every word,
including zero-weight slots; replacement/push encode before mutation, selection
retains the extent, and append rejects incompatible nonempty bounds before
extending. The type does not claim equal counts establish rig identity.

The FBX traversal produces names, parents, translations, rotations and object
IDs together (`fabelgeist-mhr/src/character/skeleton.rs:69–135`). Cluster lookup
retains ordinals, admits words against the inverse-bind column count, and
`Mhr::new` projects checked native gather indices only at TensorData creation
(`character/skin.rs:25–112`; `model.rs:274–360`). Creator GPU readback establishes
the actual joint count before constructing retained generated columns
(`generation.rs:59–82`). The glTF runtime path waits for joint assets, builds
name/state columns, checks matching extents, then admits every skin word
(`adventuresim-tactical-client/src/equipment/runtime_equipment/body.rs:21–35,
79–138`).

The retained chain was inspected through RuntimeBody, Wearer/FitBody,
forearm/torso/hair inputs, clothing, generated armor, morph residuals, garment
transfer, GPU correspondence and underlayer readback, trim/construction,
compaction, preview, and export. Upload receivers check the admitted extent
against the real column before raw GPU upload (`device_frames.rs:109–130`;
`device_fit.rs:49–72`; `fabelgeist-armor/src/gpu/body.rs:44–71`). Readback carries
the original body extent (`gpu/body.rs:104–183`; `device_underlayer.rs:371–391`).
Trim and embedded construction retain typed influence rows and the source
extent (`trim/split.rs:180–226`; `tiling/embedded.rs:48–108`). Actual native
ports are GPU buffers, JSON output, Bevy skin attributes/entity lists, and glTF
buffers (`animation_preview/binding.rs:15–57`; `export/skinning.rs:4–23`;
`runtime_equipment/meshes.rs:78–135`). Checked u16 conversion replaces narrowing
casts at those ports.

The distinct preview slot-sort/zero-total behavior remains separate from the
duplicate-combining export/runtime reducer. ClothBinding retains weighted
payloads across the ignore-weights physics toggle; constructor and weld paths
retain row correspondence, and weighted playback guards absence or differing
matrix extent (`animation_preview.rs:116–153,319–353`;
`animation_preview/cloth_render.rs:6–42`). The authorized unweighted fixture
adaptation preserves physical assertions. Numerical fixtures' explicit extents
do not establish coherence of their unused empty name/state metadata.

### P10 — optional topology and prefix levels

**Complete for the reviewed scoped migration.** The FBX builder creates
`Option<RigJointOrdinal>` directly and always appends a child after its parent
(`character/skeleton.rs:70,101–118`). Bind pose, preview ancestry, underlayer
proportion propagation and export retain that representation
(`character/skeleton.rs:44–65`; `animation_preview.rs:218–253`;
`underlayer_equipment/proportions.rs:22–42`; `export/skeleton.rs:4–42,60–73`).
Prefix source/target records retain ordinals until checked i32 conversion at the
Burn gather boundary (`skel_state.rs:123–156`; `model.rs:274–291`). Export checks
that every present parent precedes its child before serializing native node
indices (`export/validation.rs:23–37`). glTF graph-node ordinals remain a
different role in runtime FittingRig. No production -1 parent reconstruction
remains in the inspected family. Existing raw numerical topology fixtures are
literal adapters, not a second production path. The migration does not make
all public skeleton columns immutable or independently prove aggregate rig
identity; the canonical builder establishes the inspected production relation.

### P11 — MHR parameter layouts

**Complete for the reviewed scoped migration.** MHR owns exact model/set names,
joint/channel rows, distinct model columns, typed dense tables and the matrix
adapter (`model_def/layout.rs:9–56`). `.model` text admission resolves exact
joint labels and closed channel tokens before constructing rows; contribution
records retain both identities (`model_def.rs:190–291`). Dense accumulation,
parameter-set masks, limits and blend-column expansion retain these types
(`model_def.rs:93–129,295–338`). Matrix append copies each old row into the new
stride, and transpose projects raw values only for the actual native tensor
(`layout.rs:233–278`; `model.rs:258–272`). Creator joint-basis extraction uses
typed joint/channel rows and model columns (`proportions.rs:31–69`).

The parser's insertion order, duplicate accumulation, reference expansion,
offset assignment, first-name lookup, set/limit selection and blend zero-column
behavior were compared with the parent. No changed behavior was identified.
No interchange of raw application row and column ordinals remains in the
reviewed retained parameter records. Native model parameter vectors and tensors
remain explicit numerical interfaces.

### P12 — garment offsets and upright anchors

**Complete for the reviewed scoped migration.** Canonical `SignedLength` is
reused from the integration worktree's shared geometry owner
(`fabelgeist-geometry/src/measurement.rs:19,33–78`): finite signed metre admission,
validated deserialization, unchanged u16 millimetre promotion/division, and
fallible finite scaling. Garment specifications retain it through initial fit
and refit to the position-plus-normal numerical kernel
(`clothing.rs:38,81–97,314–319`; `clothing/fitting.rs:13–18,32–47`).
`JointHeightAnchor` and `JointReachTarget` retain named joint identities,
SignedLength offsets and the deliberately unrestricted Ratio
(`device_garment.rs:131–150`). Torso and skirt span producers construct those
records directly (`device_garment_torso.rs:298–312`;
`device_garment_skirt.rs:408–457`). `record_upright` projects them at Word::F /
joint-uniform binding (`device_garment.rs:206–240`); the unchanged shader owns
the resulting frame validation. No internal retained distance tuple remains
on this path. Dependency availability was checked against the coordinator's
integration reference; the rig branch's omitted prerequisite commits are not a
finding.

### P13 — candidate presence in garment shaders

**Complete for the reviewed scoped searches.** BAND owns explicit band,
cursor and candidate presence, preserving vertex order, earliest-index ties,
the old finite ceiling and reduction identities
(`device_garment/candidates.rs:19–100`). Torso/tube/fauld consumers guard payload
use; parallel failure flags are folded into existing fit status before later
cage use (`device_garment_torso.rs:66–81,341–359`;
`device_garment_tube.rs:214–221,410–427`;
`device_garment_skirt.rs:262–279`).

Skirt envelope entries explicitly exclude unsupported skin; nearest-station
selection guards absent candidate payloads and records absent support before
radius evaluation (`device_garment_skirt/stations.rs:18–38,58–127`). The recorded
status is folded before DRAPE_FINISH/WRAP, which guard the failed fit
(`device_garment_skirt.rs:108–135,293–298,371–378`). Chausses use the same explicit
cursor/candidate/section distinction and atomic part failure
(`device_chausses.rs:94–174,211–215,267–280`). Literal zeros still used behind an
absent record or for numerical reductions are not presence sentinels. Shader
compilation, dispatch ordering on a device and numerical equivalence were not
executed.

### P14 — bound catalogue records and retained Creator views

**Complete for the reviewed scoped migration.** Core's private-field
`CatalogItem`/`CatalogPlacement` bind a placement to its actual parent record.
Exact lookup keeps the first duplicate; record admission uses actual reference
membership; enumeration preserves authored record and placement order
(`equipment_assets/selection.rs:35–75,112–169`). OwnedCatalogPlacement clones
one immutable item snapshot into a shared Arc and retains a private ordinal
created from its exact placement range. Later borrowing does not decode a key
again (`selection.rs:78–109`).

Creator catalogue enumeration, article resolution, loadout, design selection,
parametric generation, material/socket output, review export and retained
armory/editor paths were traced (`equipment_catalog.rs:73–105`;
`inventory.rs:99–136`; `inventory/loadout.rs:54–84`;
`item_design.rs:91–142`; `parametric_equipment.rs:68–109`;
`equipment_export.rs:185–215,289–320`; `armory.rs:400–457`;
`armory_ui.rs:159–205`). The original duplicate export/first-match policies and
dedicated generator/family checks are retained. Pure fitting kernels receive
FittingPlacement/Side after their owning catalogue classification
(`armor_frames.rs:17–43`).

The runtime chain admits replicated/ECS item-plus-placement text at
LayerSelection, retains the bound selection through FitKey/outfit support keys,
generation, mesh material and sockets, and projects IDs for cache lexical keys
or serialized/display output (`runtime_equipment/layers.rs:14–135`;
`runtime_equipment/generation.rs:65–87,156–218`;
`runtime_equipment.rs:121–194`). Rejected admission removes stale FitKey and
hides the failed presentation (`generation.rs:356–378`). Static asset lookup
requires the canonical static selection and preserves lexical default placement
(`adventuresim-core/src/equipment_assets.rs:47–98`). Runtime clothing eligibility
classification remains separate from catalogue membership and preserves the
original authored-document filtering and role predicate. No arbitrary unrelated
item/placement pair or internal key reconstruction was found in these reviewed
paths.

## Coverage limits

This was the connected P09–P14 review, not another twenty-family or thirteen-lens
audit, and not a Comment Review. No fresh verdict is granted to unrelated
families. Canonical geometry was read only as a dependency reference. The
coordinator's completed dependency/manifest edges are accepted integration
context; unfinished P16 dimensions/grip work is outside this verdict.

The review inspected owners and materially distinct production paths above,
related unchanged receivers, and changed fixture/native adapters. It did not
exhaustively evaluate every unrelated body-fitting, decoration, rendering,
serialization or physics algorithm contained in the 148 affected files, nor
every configuration and all-target instantiation. Broad existing source-quality
debt was not reclassified as new increment defects. Remaining build/lint/format,
shader, numerical, visual and hosted acceptance are unverified. Source inspection
cannot establish those outcomes, and no checks or CI were requested.

# fabelgeist-mhr

Native Burn implementation of [MHR](https://github.com/facebookresearch/MHR)
(Momentum Human Rig), Meta's parametric 3D body model.

The crate reads the released assets directly — the binary FBX rig, the momentum
`.model` parameter transform, and the `.npz` pose correctives — and runs the
whole forward pass on Burn tensors. There is no Python, no FBX SDK, no
`pymomentum`, and no C dependency anywhere in the path.

## Model

| Input | Shape | Meaning |
| --- | --- | --- |
| `identity` | `[batch, 45]` | 20 body, 20 head, 5 hand shape coefficients (roughly -3..3) |
| `model_parameters` | `[batch, 204]` | joint angles (radians), translations, scales |
| `expression` | `[batch, 72]` | facial expression coefficients (roughly -1..1) |

| Output | Shape | Meaning |
| --- | --- | --- |
| `vertices` | `[batch, V, 3]` | posed vertices, centimetres |
| `skeleton_state` | `[batch, 127, 8]` | global joint `[tx, ty, tz, qx, qy, qz, qw, s]` |

The pipeline is the reference one: blend shapes give a rest mesh, the parameter
transform maps model parameters to 7 joint channels each, forward kinematics
produces global joint states, a small MLP adds pose-dependent corrective
offsets, and linear blend skinning poses the result.

Vertices per level of detail: 73 639 (LOD 0), 18 439, 10 661, 4 899, 2 461, 971,
595 (LOD 6). The 127 joints, the parameter layout and the 117 blend shapes are
the same at every LOD.

Runtime loading and export admit only `CharacterLod::Detailed` (4), `Reduced`
(5), and `Minimal` (6). The shared type owns their topology counts, asset
selection, command-line admission, and numeric serialization. The denser source
levels belong to offline baking. Unsupported numbers and malformed spellings
return `CharacterLodError` with a stable classification and the original
spelling or numeric input; parse failures retain their cause.

`MhrConfig` carries this detail and `PoseCorrectivePolicy::Enabled` or
`Disabled`. The default is detailed with correctives enabled. The creator
initially disables correctives. Loading, controls, cache identity, and export
retain these nominal choices; numeric detail and boolean policy convert only at
CLI, UI, and serialization boundaries.

## Assets

Download `assets.zip` from the
[MHR releases](https://github.com/facebookresearch/MHR/releases) and unpack it.
`Mhr::from_files` takes `MhrAssetDirectory`, constructed from a native
`PathBuf`.
It accepts that directory or its parent with an `assets/` subdirectory:

```
compact_v6_1.model                  parameter transform, sets, limits
lod{0..6}.fbx                       rig, mesh, skin weights, 117 blend shapes
corrective_activation.npz           sparse activation layer, shared by all LODs
corrective_blendshapes_lod{0..6}.npz    corrective basis for that LOD
```

`mhr_model.pt` is not used; it is the reference TorchScript build.

`Mhr::from_uri` takes an admitted `fabelgeist_fs::ResourceLocator`. Its asset
reads retain `FileContents`, and the model definition is admitted as `FileText`
before parsing. `Mhr::from_asset_bytes` accepts those same nominal payloads. The
file representations are converted at the FBX, model-definition, and NPZ decoder
constructors. `MhrAsset` owns the rig, definition, activation, and
detail-specific basis roles and their exact filenames. `MhrAssetReadError`
retains the role, native directory or both resource lookup attempts, and
concrete causes. Invalid model-definition text preserves its rejected bytes.
`MhrAssetDirectoryError` distinguishes missing definitions from failed native
inspection. Address construction does not prove filesystem existence or access.

Native loading decodes the rig before reading the definition and admits the
corrective basis before activation. Resource loading reads and admits the
definition before decoding the rig and reads activation before the basis. These
existing admission orders determine which failure the caller receives. Rig
construction always reads the blend shapes required by the model.

`CharacterDecodeError` classifies FBX decoding, missing rig roles, incomplete
coordinate arrays, polygon framing, skin references, mismatched skin arrays, and
missing or empty weights. It retains nominal array/corner counts and vertex
slots, mesh coverage, and object provenance. `RigObjectContext` snapshots the
object's shared `fabelgeist_fbx::FbxObjectId`, normalized diagnostic name,
record kind, and class; its
spelling does not select an error or confer lookup authority. The original
concrete FBX error remains available through the cause chain.

`MhrLoadError` retains directory, asset, rig, corrective archive/network,
definition, blend-column growth, and model-layout stages with their concrete
causes. Native archive failures retain the basis or activation role. Callers can
inspect variants directly instead of searching prose or downcasting a generic
loader result. Display includes the stage and cause; polygon and skin
diagnostics retain their existing wording. `MhrModelLayoutError` reports the
actual `RigBlendShapeCount`, distinct from appended `BlendShapeParameterCount`
columns. The model admits that count before projecting data to device tensors.

Skeleton traversal and skin-cluster resolution retain that same FBX identity
through joint lists and lookup maps. Raw signed IDs are admitted by the FBX
reader; graph consumers do not unwrap them into primitive lookup helpers.
Identity does not establish object existence or joint membership. Record, class,
and property selectors use the shared `FbxRecordName`, `FbxClassName`, and
`FbxPropertyName` owners. Skeleton traversal consumes `ModelRole`; root grouping
nulls remain traversable, while collision nulls and nested locators are skipped.
Diagnostic provenance retains the same nominal record and class identities.

Rig interpretation keeps existing numeric casts, fan triangulation, bottom-up UV
conversion, joint/connection order, strongest-influence selection, clamping,
normalization, and NaN-weight behavior. This error migration adds no
finite-value policy or changed admission order. Skeleton/mesh representations,
geometric helper ports, FBX numeric/name roles, and tensor mathematics remain
recorded semantic migration debt. The crate's loading and decoding results use
concrete errors and no longer depend directly on `anyhow`.

## Usage

```rust
use burn::tensor::{Device, Tensor};
use fabelgeist_mhr::{Mhr, MhrAssetDirectory, MhrConfig, ModelBatchSize, NUM_IDENTITY_BLEND_SHAPES};

let device = Device::default();
let assets = MhrAssetDirectory::from(std::path::PathBuf::from("target/mhr-assets/assets"));
let model = Mhr::from_files(&assets, MhrConfig::default(), &device)?;

let identity = Tensor::zeros([1, NUM_IDENTITY_BLEND_SHAPES], &device);
let output = model.forward(identity, model.zero_parameters(ModelBatchSize::from(1)), None)?;
```

`ModelBatchSize` owns evaluation rows. `ModelParameterCount` counts all named
transform columns; `PoseParameterCount` snapshots the definition's pose/scale
columns before identity blend columns are appended. The transform's
`num_parameters` and model's `num_model_parameters` retain these distinct
counts. Native extents convert directly at collection or tensor SDK calls.
`zero_parameters` takes the batch type, so a parameter count cannot serve as the
number of evaluations.

`MhrEvaluationError` classifies identity width/rows, pose width, and expression
width/rows with nominal rejected quantities. Identity admits an exact batch or
one broadcast row. Expression requires an exact batch. Empty layouts retain
their existing admission meaning. Identity width and rows are checked before
pose width; expression checks follow identity's rest-shape multiplication, as
before. Width failures precede row failures. Display text remains diagnostic;
callers should inspect variants. Input admission does not add coefficient bounds
or change the numerical forward pass. Primitive tensor mathematics and FBX
scene/property interfaces remain recorded migration debt.

## Character creator

The separate `adventuresim-character-creator` workspace provides the Bevy studio
and character/equipment exports. Its `--lod` argument admits the shared runtime
detail type before loading. The studio selects the same variants and corrective
policy directly.

```bash
cargo run --release --manifest-path crates/adventuresim-character-creator/Cargo.toml -- --assets target/mhr-assets/assets --lod 4
```

## Memory

The corrective basis stores `3000 x V x 3` floats and dominates model memory.
Set `MhrConfig::pose_correctives` to `PoseCorrectivePolicy::Disabled` when only
the linear model is needed. `Mhr::pose_corrective_availability` reports whether
the loaded model contains a network; it is separate from evaluation policy.

Corrective loading takes `CorrectiveRigTopology::from(&character)` instead of
independent joint and vertex counts. It checks basis rank, coordinate width,
mesh vertex coverage, hidden components, sparse coordinate/weight pairing, and
row/column bounds before uploading tensors. Network dimensions use checked
arithmetic; a rig missing the two skipped joints returns a structured failure.
Sparse densification preserves archive order, transposition, and the last
assignment when duplicate coordinates occur.

Sparse admission retains nominal NumPy values, counts and ordinals. Signed
coordinates keep their cast values and native conversion causes. Admitted
coordinates and activation coefficients remain typed through transposed
assignment. The resulting matrix owns its checked network layout and converts to
`TensorData` with that layout, so upload cannot supply unrelated dimensions.
Empty sparse arrays retain a shaped zero matrix; float coordinate arrays keep
their existing integer-cast-before-bound-check behavior.

`CorrectiveDecodeError` distinguishes archive and array roles, basis topology,
network dimension overflow, sparse layout, and rejected coordinates. It retains
the concrete `ZipReadError` or `NpzArrayError`, including nested NumPy decoder
causes. Diagnostic coordinates retain their original signed values. A missing
basis array still means that the model has no corrective network. This admission
layer has a concrete error API, retained by `MhrLoadError`. Raw mathematical
interfaces and remaining FBX numeric/name roles remain recorded migration debt.

## Model definitions

`ParameterTransform::from_definition` takes the admitted `FileText` and a
`Skeleton`. It returns `ModelDefinitionError`, retaining source lines, rejected
tokens, numeric roles and parse causes, or a layout/limit classification.
`ModelParameterName`, `RigJointName`, and `ParameterSetName` have distinct exact
identities. Parameter lookup returns `ModelParameterIndex`; row access takes
`JointParameterRow`. `JointParameterChannel` owns the seven serialized channels
and their row construction from `RigJointOrdinal`. Convert a model slot to a
native index only at the collection SDK boundary.

The parser retains parameter introduction order, contribution order, additive
duplicate assignments, offsets, and references to earlier joint contributions.
References copy a snapshot and do not copy offsets. Repeated supported sections
retain their contents in order. The supported grammar still ignores unknown
sections, nonnumeric bare terms, unsupported limit kinds, and unknown names in
sets or limits. Right-side reference spelling retains its existing whitespace
rules. These choices are separate from numeric admission errors.

`ParameterBounds` rejects NaN and reversed endpoints; infinite endpoints remain
valid. `SolverLimitWeight` is a distinct quantity and does not affect clamping.
Parsed limit representations are private. Malformed minmax syntax, numeric
bounds, and solver weights fail admission instead of being discarded or silently
defaulted. Only an absent solver weight receives the default `1.0`.

`append_blend_shapes` takes `BlendShapeParameterCount`, checks layout
arithmetic, and preserves the existing matrix and set membership when growth
fails. The built model exposes its parameter transform through an immutable
accessor; callers cannot mutate host metadata behind the cached device
projection. The creator's `ProportionBasisError` distinguishes missing
parameter/limit, shared-limit mismatch, and nontranslation drivers. CPU tests
cover its portable translation projection and centimetre-to-metre conversion
without model assets.

The old free parsing and blend-column functions are removed. Numeric transform
storage, count/row-value helpers, coefficient vectors, and FBX scene/property
interfaces still require migration. They remain visible in the inventory; this
parser increment does not establish that the MHR mathematical API is complete.

## Verification

Unit tests cover detail admission and serde wire shapes, native directory
selection, asset role/error context, rejected definition bytes, parameter
parsing, limit admission, corrective topology and sparse admission,
deterministic duplicate assignment, cause chains, and host/tensor transform
agreement. Creator CPU tests cover portable proportion projection. Its ignored
skeletal proportion and measured garment/armor tests require the model assets
and a compute-capable GPU. They compile with the shared configuration but must
be run explicitly in that environment.

Parameter evaluation retains its broadcast multiply and reduction. Type
migrations must preserve its arithmetic, tensor dtype, units, and admission
order. Raw geometric, scene/property and mathematical interfaces remain
migration debt; structured loading errors do not complete their audit.

## Known gaps

- Locators, collision geometry and FBX animation stacks are skipped; only what
  the body model itself needs is read from the rig.
- Momentum's non-`minmax` parameter limits (`linear`, `ellipsoid`, `halfplane`)
  are ignored. The MHR rig uses none of them.
- Posed normals are recomputed from the deformed triangle geometry; this loader
  does not expose a separate authored-normal reconstruction policy.

## Rig label ownership

Skeleton labels and model-definition joint tokens use
`fabelgeist_rig::RigJointName` directly. Joint ordinals also belong to that
lightweight shared crate; MHR no
longer defines or forwards equivalent identities. FBX namespace normalization
occurs before rig-label admission. Skeleton and parameter binding preserve the
first duplicate label, while FBX object-ID lookup independently keeps the last
duplicate ID. Blend-shape metadata retains decoded FBX object labels, and rig
errors retain the source object's normalized identity. Device and math layout
interfaces that still expose primitive quantities remain migration debt.

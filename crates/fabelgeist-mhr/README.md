# fabelgeist-mhr

Native Burn implementation of [MHR](https://github.com/facebookresearch/MHR)
(Momentum Human Rig), a parametric body model.

The crate reads binary FBX rigs, Momentum `.model` parameter definitions and
NumPy pose-corrective archives. Its forward pass combines blend shapes, joint
parameters, forward kinematics, optional pose correctives and linear skinning.
Vertex normals are recomputed from the deformed triangles.

## Model inputs and outputs

| Input | Shape | Meaning |
| --- | --- | --- |
| `identity` | `[batch, 45]` | body, head and hand shape coefficients |
| `model_parameters` | `[batch, P]` | pose and scale coefficients named by the model definition |
| `expression` | `[batch, 72]` | optional facial expression coefficients |

| Output | Shape | Meaning |
| --- | --- | --- |
| `vertices` | `[batch, V, 3]` | posed vertices, centimetres |
| `normals` | `[batch, V, 3]` | normals accumulated from the deformed triangles and normalized |
| `skeleton_state` | `[batch, J, 8]` | global joint `[tx, ty, tz, qx, qy, qz, qw, s]` |

`P`, `V` and `J` come from the admitted model. `num_model_parameters()` excludes
identity blend-shape columns appended to the full parameter transform.
`MhrConfig` supports levels of detail 4 through 6 and loads pose correctives by
default. Set `pose_correctives` to `false` to evaluate only the linear model.

## Loading and evaluation

Unpack the assets from the
[MHR releases](https://github.com/facebookresearch/MHR/releases). The native
loader accepts either the asset directory or its parent containing `assets/`.
It reads `compact_v6_1.model`, `lod{lod}.fbx` and, when enabled,
`corrective_activation.npz` plus `corrective_blendshapes_lod{lod}.npz`.

```rust
use burn::tensor::{Device, Tensor};
use fabelgeist_mhr::{Mhr, MhrConfig, NUM_IDENTITY_BLEND_SHAPES};

let device = Device::default();
let model = Mhr::from_files("D:/AI/Models/mhr", MhrConfig::default(), &device)?;

let identity = Tensor::zeros([1, NUM_IDENTITY_BLEND_SHAPES], &device);
let output = model.forward(identity, model.zero_parameters(1), None)?;
```

`from_asset_bytes` accepts the FBX, model-definition text and optional named
`CorrectiveArchiveBytes`. `from_uri` loads those same inputs through
`fabelgeist-fs`, including native paths, HTTP/blob URLs, browser filesystem
handles and project URIs. Loading and forward failures propagate through the
existing `anyhow::Result` context.

The character-creator application is maintained in
[`adventuresim-character-creator`](../adventuresim-character-creator/).

## Skeleton and vertex ownership

`Character::skeleton` retains labels as `fabelgeist_rig::RigJointName`.
`Skeleton::joint_index` accepts that label and returns `RigJointOrdinal`,
retaining the first match when names repeat. FBX decoding constructs
`FbxQualifiedName` without the class suffix, then derives `FbxObjectName` by
removing everything through the last colon. Skeleton admission converts that
normalized owner directly into the shared rig label without further changes.
`BlendShapes::names` retains the normalized FBX owner for each shape.
See the [shared rig guide](../fabelgeist-rig/README.md) for lookup and native
conversion boundaries.

`Mesh::vertex_count` and `Mhr::num_vertices` return `MeshVertexCount`;
`Mhr::num_joints` returns `RigJointCount`. Their explicit native projections are
`native_vertex_count` and `native_joint_count`. Equal counts do not establish
mesh or rig instance identity.

`MeshVertexTable` owns values in mesh vertex order. Count-based allocation accepts the
vertex count; indexed traversal yields `MeshVertexIndex`. An index identifies
a candidate's role. Checked `get` and `get_mut` establish membership in the
selected table and return its actual entry. Mesh positions and retained skin
weights use this collection. `SkinWeights` privately retains paired packed
joint indices and weights, with checked vertex influence selection.
Blend-shape storage carries no duplicate vertex count.

## Parameter transform

`JointChannels` owns the seven authored roles: translation XYZ, rotation XYZ
and scale. `JointParameterTable` checks a candidate joint against its actual
rows before channel selection. The parameter transform retains its source
joint order and exposes borrowed joint/channel views. Selected coefficient
rows retain their source transform, and checked column access returns the
actual coefficient. Named lookup keeps the first duplicate label; ordered
traversal includes every original slot.

The dense matrix admits its joint-channel extent, complete element product and
native float allocation layout before allocation. Column extension and tensor
prefix transposition return errors for unrepresentable or invalid extents.
`NpyDimension` carries the model-column array extent through parameter tables
and MHR state until the native tensor boundary. `ModelParameterTable<f32>`
retains column roles without imposing a new scalar range or finite-value
policy.

## Corrective archive admission

`CorrectiveArchiveBytes` keeps activation and basis inputs in named fields
through URI loading and NumPy archive admission. It does not validate encoded
archive contents. Corrective import checks basis axes, mesh vertex count and
hidden-unit count before tensor construction. It admits joint subtraction and
tensor-extent products with checked arithmetic, then pairs the two sparse index
columns with their ordered weights. A mismatch returns `CorrectiveImportError`
through the loader's error context. Archive and NumPy causes remain available.

The private `CorrectiveLayout` owns the admitted `[components, vertices, 3]`
basis shape: components equal the hidden width and vertices equal the supplied
mesh count. `CorrectiveBasis` retains that layout with its array until upload.
Counts become generic array extents at this boundary; they do not certify rig
identity, mesh topology or geometry validity.

Sparse coordinates retain distinct `CorrectiveHiddenIndex` and
`CorrectiveFeatureIndex` roles after the native-width cast. Their native
projections support structured diagnostics without claiming membership. The
layout checks every coordinate before dense allocation. Admitted sparse
contributions retain checked offsets and borrow that layout; upload cannot
substitute another layout or repeat membership admission. Transposition,
coefficient order and last-write behavior for duplicates are preserved.

Corrective evaluation checks the input tensor's exact joint count and seven
channels before slicing or matrix operations. The admitted input retains the
network layout through rotation-feature extraction. Values must already use
the model's channel order; shape admission does not prove rig identity or
value provenance. `CorrectiveForwardError` distinguishes count and channel
mismatches and retains a concrete `NpyLayoutError` source for batch-dependent
layout overflow. No new finite-value or positive-batch restriction is imposed.

The corrective basis holds one float per component, vertex and coordinate.
Disabling correctives avoids loading that basis and the activation network.
Representable layout admission does not promise a memory budget or execution
of every empty layout supported by array storage.

## Format limits

The FBX importer reads the skeleton, mesh, skin and blend shapes needed by the
body model. It skips locators, collision geometry and animation stacks.
Momentum's non-`minmax` parameter limits are ignored. The NumPy reader's
supported dtype, storage-order and archive policies are documented in
[`fabelgeist-numpy-storage`](../fabelgeist-numpy-storage/README.md).

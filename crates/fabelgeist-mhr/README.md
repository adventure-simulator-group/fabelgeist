# fabelgeist-mhr

Native Burn implementation of MHR (Momentum Human Rig), Meta's parametric 3D
body model. The library reads the released FBX rig, Momentum parameter
transform, and NumPy pose-corrective archives directly. It does not require
Python, the FBX SDK, or the Momentum runtime.

## Configuration and assets

`CharacterLod` owns supported runtime/export mesh level of detail: `Detailed`
(LOD 4, 2,461 vertices), `Reduced` (5, 971), and `Minimal` (6, 595). Denser release
meshes are reserved for offline baking. The type owns native asset filenames and returns
`MeshVertexCount` for authored topology size. Unsupported numeric detail and
malformed CLI spellings return `CharacterLodError`, retaining the rejected
input, stable violation class, and any integer parse cause.

`MhrConfig` carries detail and `PoseCorrectivePolicy::Enabled` or `Disabled`.
Its default is detailed with correctives enabled. The studio starts with them
disabled. `PoseCorrectiveAvailability` separately describes whether the loaded
model contains a network. `forward_with` can skip an available network; an
enabled policy only applies one that was loaded.

CLI, serde, and native UI adapters admit primitive encodings and immediately
retain these bespoke types. Serde preserves numeric detail and a boolean
corrective policy. Studio model state, body-cache identity, and exports keep
the admitted configuration. No duplicate creator-only detail type is needed.

Install the pinned assets through `just init-mhr-assets`. The creator's default
asset directory is `target/mhr-assets/v1.0.1/assets`. Native `Mhr::from_files`
accepts either that directory or its parent; `from_uri` also supports the
providers exposed by `fabelgeist-fs`. Expected assets include:

```text
compact_v6_1.model
lod{4..6}.fbx
corrective_activation.npz
corrective_blendshapes_lod{4..6}.npz
```

Optional correctives are installed separately; see the
[character creator guide](../adventuresim-character-creator/README.md).
Disabling the network avoids loading its much larger corrective basis. Native
file/resource failures and deeper FBX/NumPy/parser errors retain their existing
APIs; these are separate migration families.

## Evaluation

Identity has 45 coefficients (20 body, 20 head, five hands), model parameters
have 204 pose/scale columns, and expression has 72 coefficients. A single
identity row broadcasts; expression rows must match the pose batch exactly.
Output positions are in centimetres; each joint state contains translation,
quaternion, and scale in eight channels. Mesh normals are recomputed from the
posed triangles with area weighting.

```rust
use burn::tensor::{Device, Tensor};
use fabelgeist_mhr::{Mhr, MhrConfig, NUM_IDENTITY_BLEND_SHAPES};

let device = Device::default();
let model = Mhr::from_files("target/mhr-assets/v1.0.1/assets", MhrConfig::default(), &device)?;
let identity = Tensor::zeros([1, NUM_IDENTITY_BLEND_SHAPES], &device);
let output = model.forward(identity, model.zero_parameters(1), None)?;
```

The pipeline applies blend shapes, the parameter transform, forward kinematics,
optional pose-dependent corrections, and linear blend skinning. Joint parameter
projection deliberately uses a broadcast product and reduction to preserve
float32 precision on backends that promote matrix multiplication to tf32.

The [character creator](../adventuresim-character-creator/README.md) is a
separate optional workspace with studio controls and rigged glTF export.

## Supported subset

Locators, collision geometry, and FBX animation stacks are skipped. The model
parser supports the released rig's `minmax` parameter limits; other limit kinds
are ignored. Asset-dependent evaluation and visual fitting require the pinned
assets and GPU provider in addition to ordinary unit and documentation tests.

# Model-definition parser errors

`ParameterTransform::from_model_definition(text, &skeleton)` reads Momentum
model-definition text with the joint hierarchy required to resolve assignment
targets. It returns `Result<ParameterTransform, ModelDefinitionError>`.
`ModelDefinitionError` is available from both the crate root and `model_def`.
There is one associated representation constructor; no free parser entry or
compatibility forwarding API remains.

The bespoke error distinguishes six existing rejections:

- `MissingHeader`: no meaningful version-header line was present.
- `InvalidHeader`: the first meaningful line was not the supported header.
- `InvalidTarget`: an assignment target lacked its joint/channel separator.
- `UnknownJoint`: a target joint was absent from the supplied skeleton.
- `UnknownChannel`: a target channel was outside `tx, ty, tz, rx, ry, rz, sc`.
- `InvalidExpressionCoefficient`: the first factor of a two-factor term could
  not be parsed as a native `f32`.

Text fields preserve the rejected line and target/token after existing comment
removal and trimming. They carry diagnostic provenance rather than parameter
or joint identity. They are not physical source-line numbers or the untouched
source file. Unicode, whitespace within tokens and control characters retain
that same native processing. Display keeps the existing diagnostic wording;
Debug now reports the structured classification and its provenance.

`InvalidExpressionCoefficient` exposes the original `ParseFloatError` through
`std::error::Error::source`. Other variants have no subordinate cause. The MHR
asset loader keeps its `parsing the MHR model definition` context and preserves
the concrete parser error for `anyhow` downcasting and source-chain inspection.
The native float cause adds one chain element to coefficient errors.

```rust
use fabelgeist_mhr::{ModelDefinitionError, ParameterTransform, Skeleton};

let result = ParameterTransform::from_model_definition("# no header", &Skeleton::default());
assert!(matches!(result, Err(ModelDefinitionError::MissingHeader)));
```

Grammar and admission stay the same. Missing equals signs, unparseable bare
constants and terms with unexpected factor counts are ignored. Unsupported
sections and limit kinds are ignored. Assignment ordering, duplicate
accumulation, earlier-channel reference expansion, sets and native numeric
bits retain their behavior. The first rejected assignment still stops parsing.

Limits and solver weights keep their existing admission policy here. Invalid
intervals can still reach `f32::clamp`, and a malformed supplied solver weight
still defaults to 1.0. Checked interval and solver-weight admission are separate
responsibilities; their errors should compose into this canonical parser owner
when integrated. Channel, rig, parameter, set and layout identities also remain
separate. Native numeric buffers, triplets and section text remain existing
representation debt rather than new owners introduced by this error family.

The public model-loader regression uses an authored binary FBX rig with joints,
mesh geometry and skin data. Rejection occurs before tensor construction; it
proves context and source identity without claiming GPU execution or released
MHR asset acceptance.

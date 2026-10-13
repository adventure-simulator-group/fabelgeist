# Parameter-limit solver weights

A Momentum Human Rig (MHR) model definition can attach a solver-enforcement
weight to a supported `minmax` parameter limit. For example:

```text
limit bend minmax [-1, 1] 0.25
```

The trailing `0.25` describes how strongly a solver should enforce that bound.
`model_def::SolverLimitWeight` retains this role through
`ParameterLimit.weight`,
copying, cloning and public transform metadata. `apply_limits` uses only the
checked endpoints; the weight does not change clamping.

Admit a native value with `SolverLimitWeight::from(value)` at a numerical input
boundary. Every `f32` bit pattern is accepted, including negative values,
infinities, signed zero and NaN (not a number) payloads. Native construction
preserves those bits. Use `f32::from(weight)` only for an API that requires the
native numerical representation or for presentation. Debug formatting forwards
to the native float, including its width, alignment and precision settings.

## Model text admission

An omitted trailing token receives `SolverLimitWeight::DEFAULT`, whose native
value is `1.0`. Existing whitespace trimming and comment removal happen before
admission, so blank trailing text or a comment after the bounds also omits the
weight.

A present trailing token must parse as a native float. Text such as
`limit bend minmax [-1, 1] heavy` produces a `SolverLimitWeightAdmissionError`
through the existing public parser error boundary. Its private fields retain
the admitted `ModelParameterName`, rejected trailing token, stripped source
line and native `ParseFloatError`. The `parameter`, `rejected_token` and `line`
accessors provide that context; `std::error::Error::source` exposes the native
cause. Malformed present text no longer silently receives the omitted-token
default. No positivity or finite-value policy is added.

Unknown parameter names and unsupported limit kinds retain their existing skip
behavior. Malformed bounds retain their existing parsing rules. Checked bounds
are admitted before the weight, so NaN or reversed endpoints still produce the
existing `ParameterBoundsAdmissionError` first, with the same parameter context
and native cause. The weight neither admits intervals nor corrects invalid
endpoints. Parameter ordering, transforms and file-order clamping remain
unchanged.

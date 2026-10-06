# Parameter-limit solver weights

Momentum Human Rig (MHR) model definitions attach an optional solver weight to
each supported `minmax` limit. `model_def::SolverLimitWeight` keeps that role
through `ParameterLimit.weight`, copying, cloning and public transform metadata.
It describes how strongly a solver should enforce a bound. `apply_limits`
continues to clamp solely against the endpoints, independently of the weight.

Construct a weight explicitly with `SolverLimitWeight::from(value)` at a native
numeric boundary. Every `f32` value is admitted, including negative values,
NaN, infinities, signed zero and extreme values. Native construction preserves
all bits, including NaN payloads. `f32::from(weight)` projects the
representation for numeric inspection or an external representation boundary.
Public debug metadata retains the native float spelling.

## Model text admission

A declaration such as `limit bend minmax [-1, 1] 0.25` supplies an explicit
solver weight. An absent trailing token receives `SolverLimitWeight::DEFAULT`,
whose native value is 1.0. The parser's existing whitespace trimming and comment
stripping happen before weight admission. A comment after the bounds therefore
leaves the weight absent.

A present trailing token must parse as a native float. Malformed text such as
`limit bend minmax [-1, 1] heavy` returns
`SolverLimitWeightAdmissionError` through the existing public parser error
boundary. The error retains the rejected trailing text, parameter name,
stripped source line and native `ParseFloatError` cause. Malformed present text
no longer silently receives the absent-token default. No positivity or finite
value policy is added.

Other limit grammar, interval admission, parameter ordering and transform
arithmetic retain their existing behavior. The weight has no responsibility for
admitting bounds or correcting invalid-interval clamp behavior. Focused tests
cover accepted float bits, absence versus malformed text, structured error
context, native construction, cloned metadata and clamp independence.

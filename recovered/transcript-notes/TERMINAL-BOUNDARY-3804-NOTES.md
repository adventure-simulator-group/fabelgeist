# Terminal profile and boundary follow-up

Selected source revision: `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Prospective source registrations:
`REGISTRATION-TERMINAL-PROFILES-L13-3804.json` and
`REGISTRATION-TERMINAL-STRUCTURE-L13-3804.json`.

- Every meter's terminal loss is 1.0, while `add_bounded` retains values in
  [-1, 2]. Baseline values are derived in approximately [-0.02, 0.02].
- There are 14 authored intervention profiles. Their selected meter effects
  include negative therapeutic values and positive adverse values. The
  combined state applies all disease vectors first, then intervention effects,
  clamping after each addition. Specific simultaneous administrations,
  dose, ordering and patient history remain to be qualified.
- Disease structural points include ordinary incubation, rise, peak and
  recovery phase boundaries and every authored special-disease curve point.
  Critical disease events add other points. The inspected structural builder
  does not explicitly enumerate intermediate clamp crossings.
- This supports a precise source-level qualification only: saturation can
  create a slope change inside an interval, but the available evidence does
  not establish an admitted case that misses a terminal crossing. No runtime
  behavior, repair or contract closure is claimed.

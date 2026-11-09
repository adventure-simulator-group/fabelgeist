# Singleton terminal preview caller follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
`REGISTRATION-TERMINAL-ZERO-CALLERS-CORRECTED-L13-3815.json` covers the
bounded caller reads. The first registration accidentally requested lines
1-100 of a 43-line foraging file; the initial output stopped with an index
error. It remains preserved as invalid. A corrected registration was created
before both complete bounded reads; it grants no new population credit.

The inspected foraging and ingredient-preparation helpers first preview injury
and pass `injury.elapsed` into the disease terminal preview. Both then treat
`injury.terminal` as a terminal boundary independently of the disease boolean.
Consequently an injury-terminal zero elapsed result does not by itself show an
unsafe completed action due to the singleton disease search. Whether either
helper is invoked with an initial zero duration, whether its planning succeeds,
and what its caller does with that result remain open. This comparison does
not resolve the separate positive-duration preview / zero-settlement concern.
No product execution, new finding, or contract closure is claimed.

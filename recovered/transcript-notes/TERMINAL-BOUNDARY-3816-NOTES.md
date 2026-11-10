# Terminal preview positive-duration callers

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Registrations: `REGISTRATION-TERMINAL-DURATION-L13-3816.json`,
`REGISTRATION-TERMINAL-DURATION-GUARD-L13-3816.json`,
`REGISTRATION-TERMINAL-DURATION-BOUNDS-L13-3816.json`,
`REGISTRATION-TERMINAL-PREP-DURATION-L13-3816.json`,
`REGISTRATION-TERMINAL-INJURY-L13-3816.json`, and
`REGISTRATION-TERMINAL-BLOOD-L13-3816.json`.

The selected foraging reducer validates requested duration before calling the
terminal preview: it must be a whole hour from 60 to 1,440 minutes. The
selected ingredient-preparation authority derives its duration with
`physical_preparation_minutes`, which clamps the result to at least one
minute before its terminal preview. Both preview injury first; the selected
blood simulation increments elapsed before either returning terminal or
finishing a positive request. Therefore, if these inspected operations reach
the disease preview normally, its requested interval is positive, even when
injury ends the interval early. A singleton caused by a zero requested duration
is not established for these two selected callers. Saturating strategic-time
addition at the numeric maximum remains a distinct equal-endpoint possibility.

This narrows the singleton question and does not establish runtime behavior,
other callers, clock maximum reachability, or a general zero-duration policy.
The separate positive-preview / zero-settlement contract remains independent.

# Terminal singleton interval and prior zero-preview comparison

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
`REGISTRATION-TERMINAL-ZERO-L13-3814.json` covers zero-duration native
callers. The shared search and structural builder were registered at
checkpoint 3802.

- `first_private_terminal` starts its structural list with `from` and `to`;
  every additional point is filtered to that inclusive range. When both
  endpoints are equal, sorting and deduplicating yields a single point. The
  shared search loops over pairs and returns `None` without evaluating that
  singleton state. Thus, if this caller is reached with equal endpoints, the
  source-derived result cannot report a terminal meter, independently of
  whether the sampled state would have been terminal.
- The disease clip path immediately returns `(0, None)` for a zero request.
  The side-effect-free preview has no corresponding early return and calls
  `first_private_terminal` with the equal endpoints. Planner errors or other
  prerequisites may prevent that path from reaching the search. The separate
  medication commit-at-boundary helper directly evaluates the current state
  rather than invoking the interval search.
- The earlier `ZERO-PREVIEW-L13-3742.md` describes a *positive-duration*
  preview whose crossing occurs at its left endpoint and yields zero elapsed,
  followed by zero-duration settlement. That claim is not a zero-request
  preview; the singleton observation does not contradict it. Its downstream
  outcome, native reachability and intended zero-duration policy remain open.
- Retain entry 2291 for the shared terminal search; compare the related
  zero-time lifecycle entry before any additional unique finding. No product
  execution, runtime result or closure is claimed.

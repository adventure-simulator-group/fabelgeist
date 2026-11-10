# Terminal administration boundary follow-up

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
`REGISTRATION-TERMINAL-ADMIN-L13-3805.json` records the inspected dose,
effect, private variation and administration paths. Existing registrations
`REGISTRATION-TERMINAL-BOUNDARY-L13-3802.json` and
`REGISTRATION-TERMINAL-PROFILES-L13-3804.json` cover the search, native
structural builder and authored profiles.

- A standalone administration requires a living authorized relationship,
  individual patient-owned item, matching profile and route, patient clock,
  and a dose from 1 through 8,000 milliunits. The inspected insertion does not
  check for an already-active administration. The food-component path also
  inserts a nonzero oral profile dose. Both call the current-minute terminal
  check after insertion. Transaction behavior and broader admission paths
  remain separate questions.
- Effects are active before the earlier of stop and profile end, and zero at
  or after that minute. Native structural points include the end itself, but
  the inspected builder does not explicitly add the preceding minute. The
  shared search skips an interval when both its left and right sampled states
  are nonterminal, even if a minute inside it is terminal.
- Thus, if an adverse effect is active while disease rises through terminal,
  and expires before the next sampled structural point such that the state at
  expiration is nonterminal, the search can skip the interior crossing. This
  is a conditional source-derived mechanism; exact simultaneous disease,
  dose, phenotype, other boundaries, prior survival and native runtime
  reachability have not been established. Current-minute insertion checks do
  not by themselves settle a later rising-disease scenario.
- This discontinuity question is distinct from the earlier clamp-induced
  slope question, although both affect the same search contract. No unique
  finding, repaired behavior, runtime result or whole-contract closure is
  claimed pending alias and admissibility qualification.

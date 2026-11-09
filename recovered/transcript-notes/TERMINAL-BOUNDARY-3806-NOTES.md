# Conditional authored-parameter expiration example

Sources selected at `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`:
`REGISTRATION-TERMINAL-INFLUENZA-L13-3806.json`,
`REGISTRATION-TERMINAL-PROGRESSION-L13-3803.json`,
`REGISTRATION-TERMINAL-PROFILES-L13-3804.json`,
`REGISTRATION-TERMINAL-ADMIN-L13-3805.json`, and
`REGISTRATION-TERMINAL-BOUNDARY-L13-3802.json`.

This is arithmetic on selected source parameters, not product execution or a
constructed native patient. It assumes zero immunity (giving severity 1.85),
oxygen phenotype multiplier 1.0, oxygen baseline 0, sensitivity 0, and
adverse variation 1,500 basis points. The consistency and reachability of
those exact secret-derived values have not been established.

For influenza, the source gives a one-day incubation, a 2,880-minute rise,
and 0.55 peak oxygen loss. Profile 9 has a 240-minute duration and oxygen
adverse coefficient 0.16. At an 8-standard-dose administration, the assumed
adverse contribution is 8 * 0.15 * 0.16 = 0.192. Administer at 2,200 minutes
into the rise and assume no other episodes or administrations. Approximating
with the source formulas gives:

| Sample | Rise minute | Oxygen loss |
| --- | ---: | ---: |
| Administration | 2,200 | 0.969257 |
| One minute before expiry | 2,439 | 1.053695 |
| Expiry, effect no longer active | 2,440 | 0.862049 |

The ordinary peak is rise minute 2,880. The standalone respiratory critical
event occurs at progress 1 / (0.55 * 1.85), approximately 0.9828, later than
this interval. Thus neither inspected disease point splits this interval.
The exact integer rounding, other terminal meters, secret-derived correlations,
patient access, prior survival, actual transaction and runtime behavior remain
to be checked. This example narrows the conditional hypothesis; it does not
prove an admissible live missed crossing or close the audit contract.

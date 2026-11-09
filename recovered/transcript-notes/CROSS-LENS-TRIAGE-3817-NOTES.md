# Cross-lens scope triage

Current queue `CLOSURE-QUEUE-4151.json` contains 3,267 entries not marked
source-discovery closed. These overlap and are not unique findings. The
largest open kinds are 1,306 pending contracts/revisits and 924 exact
registered-scope own-lens qualifications; 189 registered unread/extension
obligations are also present. Counts classify queue metadata, not remaining
work hours or a completion percentage.

For example, entry 2708 asks Lens 4 to qualify all lines 1-827 of the frozen
`errantry.rs` object SHA-256 `c745b617...65969858`. The inherited Lens 4
`contract-groups-671.json` in the locally available handoff ref identifies
that exact object but records only lines 62-98 and marks the added material as
pending own-lens disposition. It therefore cannot by itself satisfy the
whole-file obligation. Historical original-baseline coverage may exist, but
its exact source, attribution and ranges still need proof before reuse. No
foreign-lens or partial-body credit is inferred. This triage changes no
selected state or ledger closure.

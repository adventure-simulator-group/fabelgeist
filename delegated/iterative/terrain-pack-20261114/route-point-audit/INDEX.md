# Route points: turn-retention audit checkpoint

The [connected ledger](LEDGER-v19.json) carries the inherited assessments and
a new bounded review of route-point turn retention at product commit
`0c9330f2a0824477b98be321920857772744b03a`. The preceding evidence checkpoint
is `3b13553419f5e7e74d687cf78006833e8032fb40`. The new assessment records a
local, validated repair; its recorded product commit was not yet published
when these inputs were frozen. Its incremental bundle requires the parent
product commit and its object closure.

The selected regression shows that positive-zero direction classification can
omit real grid turns. The local repair retains turns using grid-axis ordering
when the retained sequence fits within the existing 512-point limit. The
checked generic-grid test passed after the change; broader terrain, formatting
and lint gates passed on matching uncommitted bytes before the local commit.
These witnesses do not establish a deployed terrain-pack incident, and the
existing over-cap geometry policy remains open. Frame, endpoint, interpolation,
grid-stability and related questions remain separate in the ledger.

The original checkpoint 3821 queue and ledger remain missing. Selected
reconstructed q120 and q194 rows retain null recorded revision, Git path and
named reader; q194 has zero body or population credit. Original Adler, f2c and
95e reader and revision attributions remain distinct. SOURCE-004 proposal
bytes were separately recovered, while native qualification and receipt
obligations remain open. The 73 historical nontext/no-positive-EOF obligations
remain open. Reconstructed 059 q603 has empty source and receipt arrays and
zero population credit; its exact alias is unproved. Neither earlier records
nor new assessments are promoted to checkpoint 3821.

Recovery requires this archive, the immutable preceding archives, their
separately held keys, and prerequisite product and evidence/checkpoint Git
objects. The first failed v18 ciphertext is nested in the retry archive and
requires its own key. Same-machine copies are not an independent backup.
The encrypted evidence registry binds the selected complete source collections;
capture and publication records produced after its cutoff require later
custody. Hash and guard comparisons are point-in-time observations, not locks.

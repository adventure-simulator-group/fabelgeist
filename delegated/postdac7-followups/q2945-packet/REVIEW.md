# Independent q2945 Lens 2 source review

Reader: independent reviewer; exact attribution is held in the restricted original. Scope: original/reconstructed 059 entry 2945,
`OBLIGATION-ROOT-L3-EXT-0020-LENS-02`, and the registered full 1–209
`crates/adventuresim-tactical-core/src/city_layout/grounding/enclosure/binding.rs`
at 6533e444fa37552927070cc8d8190907fd3b413e,
e2755a732c84e67296b5e3441a4259a84f461b2f, and pinned
95e63628f9d70b8dd6e6fede64334696c6995009. Each exact source transcript was
prospectively registered and independently displayed in full in this review.
All three paths resolve through verified raw commit/tree chains to Git blob
05defe7da97ee80aecb4eb57602cbc4a7fba6541, SHA-256
9db0cfc4b4c3053884d835faaca49a404c9942f7f0599b3069b89729fd8d7ae3,
EOF 209. Identity is for this file, not whole trees. The current body was
recovered from the separately attributed q2368 packet after the local Git
object database lacked the 95e commit.

## Focused assessment

No new existing-owner mismatch is established within the selected file and
shown adjacent boundaries. `binding.rs:6–15,51–72,74–124,175–209` retains
separate CityPropertyId, PropertyMembers, SupportElevation, descriptor
digest, BoundarySupportElement, mesh and typed error causes. `compound.rs:17–19,
27–34,58–86` and `scene_input/buildings/identity.rs:1–33` establish different
property and physical-building identity roles, despite both using u64.
`members.rs:21–50` admits nonempty, ordered, nonzero, unique building IDs;
`binding.rs:109–124` adds the local exactly-two and nonzero-property rules.
Neither public identity newtype independently enforces nonzero; the evidence
does not establish that every use of either owner must prohibit zero.

`grounding.rs:73–108` owns finite scene support elevations, including zero
and negative finite values. `binding.rs:17–48` hashes the descriptor's wall
count and float-bit encodings and obtains the gate yaw from the existing
BuildingOrientation adapter (`buildings.rs:27–55,71–76`); the binding does
not retain those scalar adapters as separate unowned coordinate fields. The
digest's 32 bytes have a distinct descriptor-identity role; no equivalent
existing owner with the same full contract was established in this bounded
search. `compound/boundary.rs:27–61,125–178` associates the `usize` wall
indices with their actual enumerated wall/cap identities; this does not prove
an existing index owner should replace them or settle unrelated lens scopes.

`binding.rs:134–155` checks selected property/front, ordered fixed elements
and the descriptor digest; `binding.rs:157–172` checks count and two-way
element membership. `enclosure.rs:24–56,194–217` admits individual cells and
then applies the binding's element checks, while `scene_input/compounds.rs:
31–53,69–89` checks scene association and equal gate datum. `projection.rs:
4–63` constructs the binding and returns the same typed gate value. Those
checks do not by themselves establish the geometric/material correspondence
of every cell to the descriptor or the supported producer and native outcome.
Do not infer an actual reachable defect, alias, repair count or a runtime result.

## History and relationships

Original and reconstructed 059 q2945 byte spans match exactly, while their
whole queues differ. q2945 recorded pending own-lens qualification with zero
body credit. Independently recovered 059 q2368 named Lens 2 evidence for the
same source: hash-matched ROOT-L2-EXT-0021 registration and Root's fresh
6533 1–209 receipt; e275 ROOT-L2-CURRENT-0039 records same-Lens reuse of
1–209, with no fresh e275 personal ranges or new population credit. These
are a concrete historical qualification candidate for q2945's named range,
not a checkpoint 3821 reconstruction. My three fresh displays are separately
attributed and do not transfer the historical Root or Adler Lens 3/11 reads.

Historical q2368 retains its scoped no-existing-owner-mismatch disposition
without global closure; q2369 concerns support-prism native representation.
Historical q1493 is a separate Lens 12 descriptor/hash and gate relationship.
Immutable dac7d ledger row 1082
`relationship-enclosure-descriptor-cell-owner-95e` connects q2368 and q1493
while separately naming `ADLER-L11-SUPPORT-BINDING-ERROR-ROLE`; policy,
supported reachability, native outcomes and aliases remain open. Ledger row
320 is a separate Lens 5 contact-only aggregate question. No new unique count
is asserted. Any later repair direction follows that ledger's trace of
supported descriptor, cell and consumer producers and remains unimplemented.

## Recovery and display limits

All 73 nontext/no-positive-EOF obligations, SOURCE004, reconstructed 440,
774 normalized-versus-missing-pre-export originals, other lens and population
obligations, and the missing checkpoint 3821 remain open or qualified as in
the recovered handoff. No product code, parser, build, test, fixture, repair
or runtime probe was executed. The 95e source Git command and tree-listing
commands failed because that commit/root were unavailable in the local Git
object database; the first path-absence inference was retracted and is not
evidence. Their exact stdout/stderr and successful independent raw-tree proof
are preserved. A broad metadata display was truncated after its full result
was saved; a later broad q2368 JSON display was truncated and was not separately
captured in full, although the exact original selected record is retained
and a narrower follow-up was registered and displayed. The initial commit
anchor metadata was read before its separate registration; it gives no source
body credit and was subsequently independently registered and verified.

The complete 059 queues and dac7d ledger are separately held external anchors;
the packet contains exact selected spans and self-contained source tree proof,
not those complete streams. An evidence-only verifier checks optional full
queue/ledger membership when their separately held anchors are supplied.

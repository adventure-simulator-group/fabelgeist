# Independent Lens 4 review: tactical material and fade-band operations

Reviewer: `final_guard_peer`. This is a bounded independent peer review of
`mhr_character`'s q2744 source assessment at product revision
`95e63628f9d70b8dd6e6fede64334696c6995009`, tree
`1eec97de340d9b7debbb30ca0c0fb2240a73b93f`. Evidence checkpoint
`8caeb2003d10d08320566206817aa0c694231778` supplies the queue and ledger
snapshot. The original q2744 Lens 4 status is
`own-lens-proof-qualification-or-read-pending`; its 8ca wrapper is unassessed.
This review proposes no adoption, unique-defect count, semantic alias or lens
closure. It is later than both process-custody cutoffs.

The assigned `presentation/materials.rs:1–40` directly registers material and
presentation plugins. It contains no direct numeric transformation; that
bounded observation does not establish that registered plugins or runtime
rendering are free of invariant failures. The selected shared chunk path groups
positions using division by `ChunkSize` and computes bounds from `InstanceReach`
and largest instance scale (`presentation/chunk_instancing/spawn.rs:16–38,
45–125`). The size and reach fields do not themselves validate their floats.
The selected pebble and vista producers use positive fixed chunk sizes 24 and
256 metres. Selected pebble scales are 0.58 or 1.0; the selected vista scale
formula uses clamped variation and a positive multiplier. This does not prove
all positions, asset-derived card heights, other callers, bounds arithmetic or
native device outcomes.

The selected pebble LOD fade-out ranges are 5–6, 20–24 and 160–180 metres;
the corresponding bands enter `FadeBand::new`. The selected vista path uses
`FadeBand::until` and either 1600–1900 or 4600–5200 metres for fade-out
(`presentation/ground_scatter/loose_stone.rs:144–165`,
`presentation/ground_scatter/loose_stone/batches.rs:100–115`,
`presentation/vista/tree_cards.rs:118–160`,
`presentation/vista/details.rs:272–292`). These selected fade-outs have
positive widths. The pebble hero and vista zero-start fade-ins become a small
positive start band through the constructor; this is not a claim about every
material or shader path.

A separately scoped shared-band operation has a representable counterexample.
`FadeBand::new` raises the fade-in end using the start plus 0.001 and leaves
fade-out unchanged (`presentation/chunk_instancing/instance.rs:27–77`). At the
exactly representable f32 value 32768.0, the next larger f32 is 0.00390625
away; 0.001 is less than half that upward spacing. Under ordinary round-to-nearest f32
arithmetic, adding 0.001 can therefore leave an equal start and end at
32768.0, despite the nearby nonzero-width intent. This is an internal API
counterexample, not an observed render result.

The separately installed grass plugin is connected support, not one of the
three assigned pebble/vista registrations. Its graphics validator accepts
finite, nonnegative, nondecreasing tier endpoints, including equality
(`presentation/config.rs:207–212,242–345`). Other tier fields have additional
checks (`config.rs:347–373`). If a decoded tier has fade-in [0.0, 0.001] and
fade-out [1.0, 1.0], and its other configuration fields pass, these endpoint
checks allow the equal fade-out and the constructor retains it. A fade-in of
[32768.0, 32768.0] followed by fade-out [32769.0, 32770.0] illustrates the
rounding case under the same conditions. The displayed shipped tier endpoints
have positive widths (`assets/config/tactical-graphics.yaml:35–65`); no YAML
parser or fixture was run.

The selected source accepts a native graphics-config path and provides wasm
YAML entry points, which call parsing and validation before passing graphics
settings to the presentation plugin (`crates/adventuresim-tactical-client/src/
main.rs:134–160,176–183,191–196,282–307,365–375,445–455`). Headless mode
switches grass off. The grass batch skips empty tiers and otherwise copies the
configured endpoints into the band (`presentation/ground_scatter/
instanced_grass.rs:455–468`, `instanced_grass/batches.rs:28–65`). Nonempty
chunk buffers upload `shader_range`, the pipeline defines visibility dither,
and the selected grass shader calls a function that divides by the selected
end minus start (`presentation/chunk_instancing/draw.rs:163–197`,
`pipeline.rs:74–90`, `chunk_instancing.wgsl:62–67`,
`assets/shaders/tactical_grass_instanced.wgsl:218–220`).

A zero denominator in that source expression therefore remains conditional on
successful decoding and all other validation, a nonempty grass tier, chunk
and pipeline availability and visibility, and selection of the relevant
camera-distance branch. Actual configuration reach, native or browser backend
result, visible effect, retry behavior and intended hard-cut policy remain
unproved. The two examples share a denominator question; they are not asserted
as separate accepted defects. Any future repair planning must decide whether
equal endpoints mean a hard cut or rejection and connect admission, f32 band
construction and shader behavior while preserving valid transitions. No repair
or product validation has begun.

## Historical records and relationships

The original 059 q2744 row names 6533 and its 1–35 source requirement, with no
own-Lens 4 assessed range. The recovered historical 6533 receipt credits Root
with a fresh personal Lens 3 read of 1–35. A separate Adler Lens 11 receipt
records its own connected read. The e275 Root Lens 3 receipt records qualified
reuse, no fresh personal e275 reread, and
`source_adoption_pending_for_frozen18_19: false`. Its conditional wording about
other source adoption does not prove those other conditions were met. None of
these historical readers supplies this peer or q2744 Lens 4 source credit.
The 6533 and e275 files match one another and the archived object; the 95e file
differs. These are per-file identities, not whole-tree equivalence.

Earlier records keep their distinct first questions. q49 and ledger 201 concern
preserving terminal texture-load failure; q2701 and ledger 1084 concern
recipe/channel admission. The original q2701 has no status field, while its
8ca wrapper records a null historical status. q2747's original Lens 7 row asks
for same-lens proof or a read; later material-lifecycle work relates to ledger
251. q2748 and q2750 similarly retain their own Lens 8 and Lens 10 proof
questions; later render-subapp startup work relates to ledger 1088. These are
connections for central reconciliation, not a global alias decision or an
accepted extra count. Their original readers, later authors and dispositions
remain distinct.

Seven selected original and reconstructed 059 records match, while the two
complete streams differ. The original checkpoint 3821 queue and ledger remain
missing. Protected historical extraction was read only; replay needs the
separately held full streams, extraction and Git objects on the same machine.
No independent backup, complete Git closure or global alias inventory follows.

## Review and process limits

I personally displayed 29 prospectively registered 95e windows across 20
files. The initial 22 windows, six supplemental configuration-entry windows,
and one validator window were displayed in nine smaller untruncated batches.
The supplementary registrations preceded their displays. Full source blobs
were captured for identity only. Personal display is reviewer-attested, not
independently telemetered; test-annotated source and shader text were read as
text, with no product, test, parser, fixture, shader or renderer execution.

The author's captured process preserves its metadata-selection KeyError, prior
report displayed before its specific registration and later redisplay, and an
ineffective copy-time manifest comparison followed by exact verification. Its
first restricted verifier used a nonexistent stale capture filename; the
corrected successor retained the check, and a further successor corrected a
reported prior-report count from two to four. Those process corrections did not
change the source assessment.

My first source registration paraphrased the original status; a separate
correction preserves the exact original string above without changing the
registered source views. My initial independent history check sought the
literal `own-lens` in three questions that say `same-lens`; the 16-of-17
result and corrected 17-of-17 successor are both retained. Two attempted
metadata file lookups used nonexistent filenames; their diagnostics were
tool-visible only and the later correct files were read. No product execution,
build, test, fixture, runtime probe, implementation or product validation was
performed.

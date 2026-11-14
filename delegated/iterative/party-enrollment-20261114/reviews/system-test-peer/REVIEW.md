# PartyEnrollment zero-grace system test: static review

The proposed system-level test exercises `check_terminal_combat_outcome` with a
loading entity, then a matching loaded Party combatant, then neither. It tests
the system's resulting mission behavior rather than testing an object in
isolation. Inspecting mission state as an observation does not turn this into a
standalone object unit test. `crates/AGENTS.md` prohibits object unit tests
unless expressly requested.

On the pinned unmodified production source, the first call begins enrollment
for the loading entity and the zero threshold yields Abandoned, so the first
running-state assertion is expected to fail. For a guard that only abandons
when the party is actually empty, the loading and present calls should remain
running, the loaded member seals enrollment, and the final empty call should
produce a standalone Failed resolution and Defeat presentation. This is a
source trace, not an observed test result; the proposed production guard was not
available for review.

The test supplies `MissionState` and `TacticalConsequenceAccumulator`, uses
`MinimalPlugins` and `AdventureSimulatorNetPlugins`, and follows the existing
standalone system test's setup. The latter plugin includes replication and
registers `TacticalOutcomeResponse`. The optional SpacetimeDB connection is
absent, selecting the local-resolution branch. Compilation and framework
resource initialization have not been verified by execution.

The test's scope is one loading-to-present-to-empty sequence, with zero time
delta and no live database connection. It manually advances the terminal
presentation facade to observe Defeat; it does not run the scheduled presentation
system or cover connected submission, every enrollment/loading cross-product,
positive grace boundaries, or parser conversion. Those limits do not invalidate
this specific regression test.

The captured patch is relative to source revision
`95e63628f9d70b8dd6e6fede64334696c6995009`. Its identity was captured after
initial inspection and the worktree file was equal before and after the capture;
this does not prove a freeze against concurrent changes. This review is separate
from the earlier four-lens source assessment. No product build, test, fixture,
probe, repair, or source edit was performed.

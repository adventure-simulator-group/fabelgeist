# Independent source review of historical q2305

Reader: `/root/verifier_scope`. This is a bounded current-source assessment of
recovered 059 queue entry 2305, not a new defect count, ledger edit, global
alias decision, audit closure or product validation. Current code was read at
`95e63628f9d70b8dd6e6fede64334696c6995009`; historical records and their
original e275 reviewer attributions remain separate. Registrations, displayed
source ranges and unread complements are retained in this packet.

## Source observations

1. On the shown raised-step path, after the local chain is finalized, the
   coordinator snapshots each available foot and computes a terrain/mode-based
   support observation. If present, it writes that observation into both the
   raised-footwork support value and the IK memory; the memory receives the
   optional value even when observation is unavailable. The local snapshots,
   earlier solver and orientation work, publication checks and writes are at
   `animation/procedural/ik/mod.rs:880-1025`. Missing snapshots or failed
   validation can take other paths. This is not evidence that the raised value
   is always a nominal or pre-solve constant.
2. The registered presentation chain schedules leg IK before transform
   propagation and `refresh_raised_support_after_propagation` after it
   (`animation/presentation/plugin.rs:70-114`). With one terrain, applicable
   owner/rig and IK state, no disqualifying error or authored-locomotion
   continuation, an initialized raised step and an available foot global,
   the observer computes support from the propagated ankle and updates the IK
   memory and raised solve target, but not the raised support value
   (`ik/observation.rs:104-117,170-234,239-257`). These are source gates and
   schedule registration, not proof of an observed frame or an unmodified
   running schedule.
3. `LegIkState::diagnostics` exposes memory support options
   (`ik/state.rs:443-474`). Capture computes its required IK/cadence admission
   first (`capture/admission.rs:86-115`); if accepted, an initialized raised
   state supplies general left/right support in preference to those admitted
   values (`capture.rs:638-656`). The sample separately retains optional IK
   support (`capture.rs:727-739`); initialized means the step is not
   `Uninitialized` (`ik/raised_footwork.rs:56-78,179-181`). Capture is
   registered in `Last`, while the observer is registered in `PostUpdate`;
   the separately attributed prior metric assessment cites the pinned default
   Bevy schedule. No running schedule or alternate scheduling configuration
   was examined here.
4. A concrete source-level conditional follows: if a selected initialized
   raised value is one, a later admitted IK observation for that same foot is
   zero, and no intervening writer changes either before capture, the general
   and optional IK sample fields differ. This condition does not prove that
   such a combination is reachable in a supported rig, scenario or frame.
   One downstream slip computation uses the minimum of the general support
   values across two samples, and only when its terrain/guard, threshold,
   body-turn and bone-availability gates hold
   (`report/metrics/support_slip.rs:6-55`). Whether those fields should mean
   final propagated visual support, a distinct logical role, or another
   explicit policy remains open. No report or rendering outcome was observed.

## Historical and current relationships

- Original 059 q2305 names
  `ADLER-L11-RAISED-SUPPORT-OBSERVATION-DISPATCH#post-local-cache-versus-propagated-support-qualified`
  and records `documented-conditional-policy-question`, with its named scope
  and parent both incomplete, no population credit and no runtime pass. Its
  parent is original 059 q45,
  `ADLER-L11-RAISED-SUPPORT-OBSERVATION-DISPATCH`, which remains open.
- All 16 named historical PLAN, READ, root-adoption, VERIFY and proposal
  records were recovered as exact bytes from the user's read-only extraction
  and independently matched the hashes named by q2305. Their digest/size/path
  identities also appear in the published original-transport inventory;
  that does not prove independent off-machine custody. The historical
  `lens11_reconcile` reads and subsequent root adoptions are distinct: the
  original reader attribution says adoption pending, while the later root
  mappings adopt PL11-0247, 0248, 0258 and 0275 without a root reread.
- The exact q2305 entry slice is equal in the original 059 queue and the
  separately path-redacted public reconstruction; the full stream identities
  differ. Neither selected slice is an original standalone queue. The parent
  q45 display is a reconstructed selection from its identified original queue.
- The separately attributed committed
  `ASSESSMENT-REPORT-METRIC-SUPPORT-95E.json` already connects this parent to
  the current conditional support/metric chain. The checkpoint 9 delegated
  capture observation and checkpoint 10A `P17-SLIP-SUPPORT-SOURCE-AUTHORITY`
  report are related observations with their own authors and qualifications;
  the latter does not establish an exact alias or defect. These assessments
  do not themselves close q2305, all consumers or all lenses.
- The e275 and 95e IK coordinator files differ in full-file identity, though
  the compared 985-1011 byte range matches. The historical observation,
  capture and admission file blobs match their 95e counterparts. Those checks
  do not establish surrounding-context, whole-tree or runtime equivalence,
  and no historical e275 body read is claimed by this reviewer.

## Open work and qualifications

Determine the intended role and authority of each general and optional IK
support field; trace whether any supported initialized raised step, solver,
post-IK writer, native hierarchy and frame can produce different values;
reconcile all applicable consumers, source aliases and missing-observation or
terrain-disabled behavior. Confirm relevant asset/rig and report policy before
any later authorized validation. These remain questions, not claims of an
actual false metric or product defect.

One registered state display requested through line 485 although its file ends
at 479. The initial display is preserved, and the state lines were credited
only after an exact corrected registration and redisplay. A separate Git
identity command failed because its relative repository path was used from
`/tmp`; its error output was truncated and is separately recorded. Neither
incident is credited as an extra read. No missing checkpoint 3821 queue or
ledger is reconstructed or inferred. No product code, tests, fixtures, macros,
builds, runtime probes or repairs were run, and no publication or protected
extraction file was modified.

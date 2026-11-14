# PartyEnrollment: four bounded audit lenses

This is a source-only assessment by the four-lens reviewer at source commit
`95e63628f9d70b8dd6e6fede64334696c6995009`, tree
`1eec97de340d9b7debbb30ca0c0fb2240a73b93f`. It covers behavior preservation
(Lens 8), derivability (10), raw strings (12), and raw primitives (13) for
`PartyEnrollment` and selected grace configuration and reconnect consumers.
Paths below are relative to `crates/adventuresim-tactical-server/src/` unless
otherwise noted. Bounded source and test text were read; no product code, test,
fixture, build, or runtime probe ran. Findings are conditional source analysis,
not evidence of supported input reach, an observed player outcome, or a closed
lens. No defect count or ledger alias is added.

## Reconnection and behavior preservation

The stated purpose of reconnect grace is to tolerate an empty party while
clients reload (`mission/mod.rs:60–64`, `main.rs:102–104`). The code does not
settle whether a zero grace value should be rejected or should expire immediately
when the party is actually empty. In `mission/enrollment.rs:70–110`, Awaiting
returns before the timeout comparison; Enrolling or Sealed accumulates elapsed
time only while no party member is loaded and no player is loading, resetting
it otherwise. The subsequent unconditional `empty_for >= reconnect_grace`
comparison returns Abandoned when the normalized grace is zero even after a
present/loading party reset it. `mission/systems.rs:175–232,52–67` routes that
effect into a frozen Failed terminal submission; it does not establish that an
external submission, database update, or player-visible result occurs. This is
the existing q3134 / ledger 406 first operation, also independently analyzed
under q2982 Lens 4, not a newly counted defect.

`main.rs:102–106,168–175` declares a floating-point seconds option, supplies a
default of ten seconds from `mission/systems.rs:24–27`, and converts using
`Duration::from_secs_f32`; the selected declaration has no explicit positive
range. The installed rust-src matching the declared `nightly-2026-07-13`
toolchain documents zero and the positive value `1e-20_f32` both converting to
zero, rounding to nanoseconds, and a panic for negative, nonfinite or overflowing
inputs (core `time.rs:966–999`). Thus, if policy rejects zero durations, checking
only the raw float for positivity would be insufficient; check the normalized
duration as well. The [upstream Duration documentation](https://doc.rust-lang.org/stable/std/time/struct.Duration.html#method.from_secs_f32)
describes the same contract. Whether the actual argument parser accepts any
particular spelling or out-of-range value, and whether such inputs are supported,
was not established. The rust-src copy and Git object store are same-machine
sources, not an independently authenticated toolchain distribution.

The disconnected-player snapshot has a separate 30-second capability retention
period (`player_projection.rs:373–379,484,1754–1829,1889–1911,1957–1990`).
A join carrying a capability requires a matching, unclaimed and unexpired
snapshot; a failed supplied capability is rejected before the no-token path
(`player_projection.rs:950–1029`). Snapshot expiry calls `leave_mission` and
retains the entity for a later retry when that call fails. This timer has a
different role from the mission's default ten-second empty-party timer. Their
interaction when configured durations, loading, expiry, and server scheduling
vary is a behavior-preservation and policy question; the source alone does not
prove a runtime sequence or require the two values to be equal. The standalone
join/disconnect branch differs (`player_projection.rs:1050–1095,1854–1882`).

## Existing coverage and regression requirements

`mission/enrollment.rs:130–187` contains source tests for enrollment/loading,
sealing, known and unseen members, and positive-ten-second expiry immediately
before and at the threshold. A zero delta appears with positive grace; no
selected test supplies zero grace or a positive float rounded to zero.
`player_projection.rs:2407–2507,2509–2579` contains observer-order/rebind and
capability, rotation, duplicate-claim, and expiry-boundary tests. These are
source text only, and do not establish end-to-end timing or outcome coverage.

Before choosing a future change, decide whether zero normalized grace is
admitted and what should happen for an actually empty party. Regression cases
should distinguish Awaiting from Enrolling and Sealed; loaded, loading, and
truly empty conditions; just-before/equal/after positive deadlines; timer resets
and zero delta; zero and tiny positive-to-zero input; known versus unseen roster
members; and terminal-effect ordering. If invalid values are rejected, check
the converted duration and preserve intended diagnostics. Separately assess
reconnect before and after capability expiry, differing mission and snapshot
grace periods, successful versus failed leave, supplied versus absent tokens,
and live versus standalone execution. These are prospective test requirements,
not tests performed or a claim that every combination is reachable.

## Derivability, strings, and primitive boundaries

The complete `mission/enrollment.rs:1–124` implementation has ordinary derived
traits and semantic state-machine methods; this bounded file review found no
handwritten trait or serialization implementation to replace with a derive.
It does not rule out implementations elsewhere. The local
`unreachable!("begin transitions awaiting enrollment")` at line 59 is a
diagnostic, not an observed branch on a magic string. Selected system messages
at `mission/systems.rs:196–220` are diagnostic text; the review does not claim a
complete audit of every adjacent CLI or network string.

The external seconds option is an `f32`; the retained value and elapsed timer
use `Duration`. Expected party size is already `NonZeroU32`; member identity
uses `CharacterId`, and the roster uses a set (`mission/enrollment.rs:1–75`).
The selected `u32` loaded count and loading boolean drive the empty predicate;
zero is a meaningful count there. Their producer bounds and supported scheduler
behavior are separate questions, not resolved by a blanket primitive replacement.
The 10s mission default and 30s snapshot lifetime are distinct named constants,
not evidence of a required common value.

## History and evidence limits

The five inspected source files have identical per-file bytes at historical
`6533e444fa37552927070cc8d8190907fd3b413e`,
`e2755a732c84e67296b5e3441a4259a84f461b2f`, and 95e. That is file identity,
not whole-tree equality, an independently performed historical read, or proof
of checkpoint 3821 recovery. The original q2982 obligation had status
`own-lens-proof-qualification-or-read-pending`, no own-Lens-4 assessed ranges,
and required historical mission-module lines 1–220; its f2c wrapper remained
unassessed. Root's 6533 Lens 3 read was fresh for that range. The e275 receipt
records qualified reuse of that earlier read, no fresh e275 read, with frozen
source adoption pending. Original q3134 repeats a broader personal-attribution
label; the receipt supplies the narrower qualification. Its original row has no
status field; its f2c wrapper is partial. The original and reconstructed selected
rows match as recorded, but their complete streams differ. Earlier q2982 author
and peer reports, their source readers and their test-text/process corrections
remain separately attributed; this reviewer previously assembled their reports
and separately authored q2990/q3062 and reviewed q2980/q2989. None of those
roles transfers reading credit to this new four-lens slice.

Registered personal displays are reviewer-attested, not independently
telemetered. Test bodies were read as source, never run. A clipped combined
guide display was followed by a complete crate-guide read; the root guide was
provided in full by the user. Original source streams, same-machine Git,
installed rust-src, and separately preserved prior receipts remain distinct
anchors. Missing checkpoint 3821 has not been recovered or reconstructed here.

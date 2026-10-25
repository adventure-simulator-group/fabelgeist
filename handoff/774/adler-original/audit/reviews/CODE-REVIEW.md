# Independent Code Review for PR #774

Review the resulting implementation as a proposed change by another engineer.
This review supplements all thirteen bespoke-audit lenses, Rule Review and
Comment Review. It does not replace any of them or the mandatory twenty-family
implementation plan. Use a reviewer independent of the implementation work.

Pin the actual final head, comparison base and any working changes. Compare the
complete PR against the merge base with current `origin/main`; do not reuse the
old comparison SHA from an earlier review. Include staged, unstaged and untracked
files when they are in scope. Include affected unchanged producers, retained
state, operations and receiving bodies. Recovery snapshots are not automatically
integrated source or proof of final correctness.

The user's constraints override generic review defaults: add no tests, cases,
fixture helpers or assertions; do not run local project checks or runtime
validation; do not trigger hosted CI while known connected repairs are unfinished.
Read existing tests and available source/hosted evidence. Review-only does not
authorize source edits, external posts or merging. Remediation follows the
handoff's implementation ownership and acceptance requirements.

## Findings

Respond in normal Markdown, not JSON, XML or a structured findings schema.
Focus on discrete, actionable issues the author would likely fix once aware.
Prefer no findings over speculative or low-signal feedback. An issue should:

1. Materially affect correctness, performance, security or maintainability.
2. Be discrete and actionable.
3. Be introduced by the reviewed change, with a source-backed scenario.
4. Be something the author would likely fix.
5. Avoid unstated assumptions about intent.
6. Identify the affected behavior and consequence clearly.

Separate demonstrated defects, incomplete migrations, proposed strengthening and
unverified coverage. A static representable counterexample is not automatically
an observed runtime failure. A search result, passing CI or previous verdict is
not sufficient source evidence. Review contracts across all affected families;
do not silently convert unfinished work into a follow-up.

For feedback attached to a changed line, emit one directive per issue:

```text
::code-comment{title="[P2] Concrete issue" body="Scenario, consequence, correction and applicable rule reference." file="/actual/checkout/path.rs" start=123 end=124 priority=2}
```

Use the shortest useful range and actual checkout paths. Required attributes
are `title`, `body` and `file`; `start`, `end` and `priority` are optional.
Explain the relevant file and line or function in visible prose too. Use severity
labels when helpful. Emit no directives when no actionable inline issue exists.

## Repository rule attribution

Read applicable root and scoped instruction files with ordinary precedence:
`AGENTS.override.md`, `AGENTS.md`, then any configured fallback, choosing at most
one file per directory. More-specific guidance wins on conflict; user review
scope and style instructions take precedence.

Review the diff independently. Deduplicate by changed location and defect/remedy,
retaining the union of real rule support. A finding is rule-supported only when
applicable guidance materially supplies repository-specific scope, an invariant,
remedy, convention or confirmation behavior beyond generic correctness advice.
Do not omit ordinary correctness findings or invent findings because a rule file
exists. For each rule-supported finding, verify the exact instruction file and
smallest supporting line range, then include a compact Markdown/local-file
reference in its visible body. Do not fabricate citations or hidden metadata.

## Verdict and re-review

State the exact reviewed head/base, meaningful coverage and verification limits.
If there are no actionable issues, say so briefly. Identify unread paths rather
than granting them clearance. Correctness and migration completeness are separate
verdicts. Keep the Code Review, Rule Review, Comment Review, thirteen audit lenses
and required hosted CI evidence tied to the same final pushed revision. After
repairs, repeat affected reviews and audits and complete the full audit waves
required by the handoff. Do not merge or mark overall issue #726 complete.

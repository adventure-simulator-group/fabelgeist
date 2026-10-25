---
name: clippy-review
description: Run and review a Rust repository's Clippy gates, inspect lint inheritance and suppressions, and fix lint findings when requested. Use for explicit Clippy reviews or lint failures.
---

# Clippy review

Read the applicable repository rules, pinned toolchain, Cargo workspaces,
standalone manifests, and authoritative lint recipes or CI jobs. Use their
actual targets and features, including separate native or WASM gates. Do not
assume `cargo clippy` on the default package covers the repository.

For review-only work, run the requested/applicable checks and inspect findings.
For authorized fixes, use one implementer for each reproduce/fix/check loop.
Capture bounded logs in ignored output; diagnose the earliest failed contract
before examining downstream noise. Distinguish toolchain, dependency, binding,
and system-library failures from lint failures. Missing prerequisites are not
a passing check. Do not alter tracked build configuration merely to disguise a
local environment limitation; record any local override used.

Review the lint policy as well as diagnostics:

- Check workspace inheritance and any required standalone policy copies.
- Inspect file-level suppressions, lint priorities, and feature-specific paths.
  Narrow an exception to the responsible item and give its concrete reason.
- Follow the repository's `expect` versus `allow` policy. With `expect`, check
  the targets where the lint is emitted and where the expectation is fulfilled.
  Generated code and macro expansion can require different treatment.
- Distinguish an idiomatic readability fix from an API, authority, or performance
  redesign. Do not satisfy a lint by adding compatibility paths or weakening
  invariants. Do not introduce optimization without a demonstrated need.
- Treat suggested automated fixes as patches to inspect. Avoid broad
  `cargo clippy --fix` runs that can rewrite unrelated user changes.

Run narrow package checks during implementation, then the prescribed final
matrix. Run behavioral tests when a change affects behavior or a domain
invariant; do not add tests that merely reproduce constants or source text.
For generated binding findings, change the owner and regenerate through the
repository workflow rather than patching the generated output.

Report commands, targets/features, outcomes, remaining diagnostics, and any
blocked gate. Separate pre-existing debt from changes caused by the patch.
A clean Clippy run does not establish compliance with semantic or architectural
rules that Clippy does not enforce.

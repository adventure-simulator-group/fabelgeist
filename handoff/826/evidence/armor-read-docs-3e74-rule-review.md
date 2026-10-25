# Armor read documentation Rule Review

Pinned `3e74eea62f4138fb95e001bf93e9438966d12612`, tree
`a83c132e94bab1db7691468d553465081b7432fb`, against
`d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`.

No actionable finding in this bounded documentation change. Both complete
owning files, the exact diff and all four read contracts were read. Only
Rustdoc changes; every non-Rustdoc line and every other tree path remains
byte-identical. Added doc lines are at most 79 columns. No new type, validator,
alias, method body, exception, fixture or runtime token is introduced.

Direct ArmorGpu::read/read_async now state the same actual creating-device,
paired-queue and external mapping obligations as Buffer::read. Those are
caller preconditions; no new identity check or device-brand guarantee is
claimed. WgpuContext::from_parts already requires its queue to belong to the
supplied device and preserves borrowed-context error handling.

Grouped read_staged/read_staged_async require sources on that same device and
available for GPU copies under their mapping ownership. This accurately differs
from direct-read exclusive mapping access: the grouped owner allocates and maps
its own staging buffer. Canonical b11 Readback derives its allocation and
polling device from the batch, records each source copy, and maps after the
producing submission succeeds. Source/ticket session identity does not establish
native device provenance. The docs make no such claim and preserve native
progress/browser callback limits.

This follows the governing requirement to keep public contracts synchronized
with implementation and reuse existing semantic/native owners. Module order,
error carriers, method bodies and meaningful production-line ceilings are
unchanged. The prior complete native-family source receipt carries only where
exact bytes match; no whole-generator coverage is inferred.

Canonical Readback/count/type receiving, generator/caller/cache/lock work and
the final all-thirteen/review/hosted gates remain required. This documentation
receipt does not establish compilation, native behavior, family completeness or
final clearance. No source edits or project scanner, build, check, lint, test,
generator or runtime execution occurred. Exact hashes and source accounting are
in the adjacent JSON report.

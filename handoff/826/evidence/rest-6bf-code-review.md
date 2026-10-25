# Independent resident-rest checkpoint Code Review

Revision: `6bf03a8d04bf246c996bbbfa17c1925ebd150c38`.
Tree: `daddbb27e119b442e4bc77b30de8d0c84b357fd8`.
Parent: `10cc1a69b1a9a5f4e54daa112477cc56b7487f45`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

No new owned actionable correctness finding in this bounded source review.
Both changed files were freshly read completely, alongside the full native
level receiver and deformation shader. Exact source hashes and fresh versus
carried coverage are recorded in the JSON companion.

First rest construction allocates the canonical positive binding extent and
copies exactly the logical resident position bytes. Zero data produces no
copy, while the native storage binding retains its positive dummy record.
Nonempty byte extents retain vec4 alignment and fit both the admitted output
and rest allocation. Rest and output bindings explicitly expose that resident
extent, leaving physical allocation suffixes outside these bindings.

Existing rest reuse clones the actual native buffer handle. It neither
allocates nor captures already deformed output again. The private level/groom
invariant supports this association: both setters reset dynamics before they
replace the resident level vector; there are no other level replacements.
Repeated dynamics binding therefore reuses each same level's original rest.

Replacement builds candidates while retaining old Dynamics and publishes only
after native submission returns normally. Clear likewise restores exactly
each resident logical prefix, skips zero copying and removes state only after
normal submission. Frame/deformation Pending and temporal publication remain
unchanged. The updated no-work obligation comment now matches empty handling.

This ordering provides no GPU-completion, observed native-validation or
transactional rollback promise. SDK copying still requires actual device,
usage, allocation and alignment admission. Those native failures follow the
existing device-handled policy. The new native input record packages framework
bindings; it does not validate a simulation source.

Guide/id/weight/body source associations and extents, device provenance and
copy usages remain required connected work. The remaining simulation slots
still bind entire buffers. Canonical sealed producer APIs, native cards,
caller/count/creator and manifest-lock integration remain required before
source-complete status. This bounded repair does not clear those paths.

Both existing fixture modules, all shader sources and native POD fields remain
exact parent bytes. No tests, cases, helpers or assertions were added. No
source edits or project execution, build, check, tests, scanners, generators,
runtime, push or CI occurred. This establishes no compilation, full renderer,
all13-lens, whole-PR or final clearance. Final reviews and hosted gates remain
required on the same final pushed revision.

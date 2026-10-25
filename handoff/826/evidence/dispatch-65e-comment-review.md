# Dispatch-grid checkpoint Comment Review

Reviewed exact `65e8aebedb6490bc2945eac6d48479ea6a2cd10a`, tree
`24cf778b808d5be97c829afcc4c6bebbdbcfa64a`, against parent
`b874e611826c27085da897903cacfc31998d0210`; main remains
`990430b77a9f63548ffcb642074185f61ddb3322`.

Fully read all five changed files from pinned Git objects, plus the current
lower-pass error/submission documentation and complete Kernel owner module.
The JSON records every source hash and distinguishes bounded SDK reads. A
working-tree successor edit to shape.rs was excluded.

One small actionable precision finding remains: **65E-C1 (P3)**,
`compute/pass/grid.rs:15`, says “allocating device.” The grid allocates nothing;
`validate_device` uses the receiving device. Replace that phrase with
“receiving device.” The implementation already uses the correct authority.

The substantive dispatch documentation matches this source: folding constructs
geometry only; declared-shape admission checks padded u32 lane addresses;
receiving-device admission checks each actual native axis. Pipeline, resources,
finalization, submission and completion remain distinct. The lower receiver
checks grid limits before pipeline/binding work. Its native-error wording
allows deferred finalization, and submission docs promise no GPU completion.
The 65535 layout policy is not a universal device-maximum claim.

Both transferred fixture files and the compile-fail example are byte-identical
to `9a91b1203d6b9571fde84b67acf06b5b472db393`. They retain 36 covering records,
six occupancy records, two existing tests and four assertion macro sites. The
short provenance token `95f` did not resolve locally; its full pin was requested.
No tests or documentation examples were executed or authored.

Required follow-ups remain open: typed folded WorkgroupCount projection for
scan, and zero-work handling before rejecting a multidimensional declaration.
The current checked folded operation rejects that declaration even for zero
items. Kernel/Prefix/FastPath and the entire agreed producer/count/layout/native
receiver/caller/cache/lock family still require owner integration and review;
current Kernel raw geometry does not consume this new checked boundary.

No source edits, local project execution, tests, builds, scanners, generators,
runtime, push or CI occurred. This is bounded source evidence, not completed
migration, all-thirteen-lens, compiled, hosted or final whole-PR clearance. All
final reviews and required hosted checks must pass on the same final pushed
revision.

# Static review of corrected preinstallation scripts

The separately retained successors correct the previously identified blocking
scan-script digest mismatch. Both the runner and capture now contain the
64-character SHA-256 of the captured scan script; their three scanner input
identities agree. Capture also binds the exact successor runner, its new path
and its receipt schema. The installer successor changes only its capture-file
path. No further blocker was identified under the stated same-machine,
nonconcurrent assumptions. This is a static review, not an observed scan.

The exact runner differs from its predecessor in three changes: the scan
digest, preflight of its directory and the six named scan outputs, and receipt
schema. The exact capture differs in four changes: corrected scan digest,
runner identity, runner path and receipt schema. The installer has one path
change. Previously reviewed stage, peer-manifest, candidate and full local mode
checks remain byte-identical; Git still preserves owner executable state rather
than full local modes. The first successor-generation assertion failed before
destinations were created; its author records a tool-visible-only traceback,
not a separately preserved original transcript. The blocked predecessor and its
independent review remain separately preserved.

The runner checks that its own directory contains exactly the script and
derivation and that the six named scanner outputs do not exist before launch.
The latter check uses `Path.exists()`, which does not detect a dangling
symlink. Its checks are sequential, so a concurrent change can still invalidate
the preflight. A separate immediate check and failure preservation remain useful
mitigations; they are not atomic guarantees. Runner, scan, capture and install
can leave partial outputs on failure. File synchronization does not establish
directory crash durability. Local Windows-path positive controls test an equal
captured regular expression and flags, not actual scanner invocation.

This reviewer authored the predecessor review and participated in earlier
evidence custody and source reviews; the coordinator authored these successors.
Inputs were prospectively registered and their bytes and modes rechecked on the
same machine. No reviewed script or product code was executed; final scan,
postinstall placement and commit were not observed. Finite scans, independent
backup, complete Git closure, original checkpoint 3821 recovery, source
semantics, audit completion and product behavior are not established.

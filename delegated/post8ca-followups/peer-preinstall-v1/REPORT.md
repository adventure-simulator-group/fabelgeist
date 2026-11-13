# Static review of the prospective capture and installation template

Reviewer: `final_guard_peer`. The reviewed evidence-only `CAPTURE.py` has
SHA-256 `edfc8cd0eb24a0845b3fe36126d1afb2ecc6ef6f4255c277c890dc5f3b4c1e4a`;
`INSTALL-TEMPLATE.py` has SHA-256
`28693b95d536159771a2ca77b68d6636d8031178f081d54560eec62ec0a0cadd`. This
review is static: neither script, the future scan, the guard, nor product code
was executed. The final scan output and post-install placement were not
observed. The template still contains a capture-hash placeholder; a derived
installer requires its own identity and review after capture.

Four enforcement gaps remain before these scripts can provide their intended
independent evidence bindings:

1. `CAPTURE.py:44–73` reads the staged `PINS.json` and checks the hashes it
   supplies, but does not independently pin that file or require the guard's
   exact 26-name set. The staged receipt records a pins hash, but the capture
   does not compare it. The eventual guard checks the name set while taking the
   expected hashes from that same pins file. A coordinated change to a pin and
   its stage input can therefore be captured without the original pins identity
   being enforced. The provisional pins file does match the receipt and all 26
   staged inputs at this review cutoff; the gap concerns later enforcement.
2. `CAPTURE.py:16–26,44–73` pins the guard-v2 and installer-v3 peer manifest
   bytes, and reconstructs the guard correction exactly from a pinned
   derivation. It does not validate those peer manifests' listed member bytes,
   or compare the staged guard-v1 peer manifest with the v1-reviewer hash in
   the correction. The eventual guard does not invoke those peer verifiers.
   A member could change under an unchanged manifest and be newly captured.
   The provisional v1/v2 guard and installer-v3 packages all match their
   manifests and separately retained public intakes; no current mismatch was
   found in those snapshots.
3. `CAPTURE.py:78–90` compares the current stage's complete file/directory
   bytes and modes with the copied scan tree, excluding the added scan receipt,
   and checks the zero counters plus wrapper exit and stderr. It does not
   require exactly the two expected check labels, seven scanned root files,
   the scanned-path count or the Windows-control flag; `all` accepts an empty
   check list. It does not bind the wrapper logs to the exact scanner scripts or
   recompare the scanned root files with the seven pinned candidates. The
   intended `SCAN-RUN.py` checks some of those conditions if it actually runs,
   but this capture alone does not establish runner provenance. Its local
   single/doubled Windows controls use the same regex and flags as the captured
   pattern-scanner source, yet they do not call that scanner, and the scanner
   dependencies are not hash-pinned by the wrapper. The scan was not run here.
4. `INSTALL-TEMPLATE.py:37–55` compares every pre-captured row's bytes and full
   local mode, the complete recorded directory set and modes, and stage root
   mode, in both stage and copied destination. The newly generated
   `STAGE-CAPTURE.json` is checked only by bytes, not full mode, and
   `ADOPTION-STATUS.json` is added after those comparisons without a recorded
   mode. A claim that every installed stage file's full local mode was compared
   would therefore exceed this script. Git records its 100644/100755 distinction
   using the owner-executable bit, not full POSIX permissions; later index and
   commit verification remain separate.

At this provisional cutoff, a read-only comparison found the eight fixed stage
hashes, the predecessor guard and root wrapper, canonical correction and pinned derivation
consistent. The v1/v2 guard and v3 builder peer packages each matched the
captured stage inventory and their separately retained public intake manifests.
All 26 current pins matched their staged inputs. The four candidate before
images matched the pinned 8ca Git objects, whose tree modes are 100644; all
seven v3 after/new rows matched their captured bytes, SHA-256, Git blob and
Git-mode identities. These observations do not bind the later final stage.
The provisional stage inventory had 363 files and 41 directories, including
its root. Its contents may gain this review before the final scan.

The source uses exclusive creation for the capture and new root files, checks
for absent destinations, rejects symlinks and special files in stage traversal,
and rejects absolute or parent-traversing pin paths. The installer checks the
pinned 8ca HEAD/tree, named branch, ordinary Git status and before images before
writing. These are sequential same-machine operations, not an atomic snapshot
or protection against adversarial concurrent path replacement; ignored files,
trusted ancestor directories and Git object availability remain separate.

Capture writes its private and staged capture copies sequentially. A failure
between writes, or after a write before the final checks, can leave partial
artifacts and block a simple rerun. Installation copies a stage, creates new
files, prepares replacements, then replaces four files sequentially. A failure
may leave a partial destination, new files or temporary replacements; no
rollback is implemented. A log or result failure after a write can leave the
write present without a corresponding final success record. File fsyncs do not
by themselves prove parent-directory crash durability. Later placement, action
logs, final modes, index/commit, publication and recovery require separate
checks.

The independent static metadata check passed 28 comparisons, and a separate
scanner-source AST check passed five. My first checker failed on its own
path-component index; its raw failure and additive corrected successor are
retained. The source snapshots and rehashes are sequential, not atomic. The
reviewer previously helped build the same-machine custody packages, reviewed
q2545/q2744 source evidence and earlier checkpoint reconciliation and guard
work. Independence is from these two script authorship decisions, not from all
underlying evidence; source-read credit does not transfer.

The original checkpoint 3821 queue and ledger remain missing. Recovered
records, reconstructed records, historical summaries and new assessments keep
their revisions and readers. Prior ciphertexts, key, original evidence and Git
objects remain same-machine dependencies; no independent backup, complete Git
closure, product behavior, repair, unique-defect count or lens closure is
established. No product build, test, fixture, runtime probe or validation was
performed.

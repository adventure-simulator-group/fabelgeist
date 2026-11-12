# Adversarial review of the frozen-checkpoint successor design

This is a fresh adversarial review by `/root/verifier_scope`, the same author
as the earlier published `delegated/live-state-successor-design/REPORT-v2.md`.
It is not an independent-author review. A different reviewer must assess the
adoption conditions and evidence before the design can be treated as reviewed.
No implementation, historical replay or product validation was performed.

A separately frozen, exactly materialized checkpoint can preserve the checks
and qualifications that its unchanged public driver actually applies while a
separate live namespace advances. That conclusion is conditional: the
checkpoint, its entire input namespace, tool environment, optional checks and
recovery evidence must be bound and verified. A passing public driver does not
turn reconstructed public evidence into original private bytes, prove every
optional check ran, or establish privacy, independent custody or live-state
correctness. The proposed live namespace and closure rules remain unimplemented
and unapproved.

## Concrete observations and failure modes

- The original proposal was registered and displayed from fixed published
  checkpoint `c7dc76868c73991af5209e61cc664652b0a8d025`. A later observation
  found both the publication and separate `remote-restore` checkout at
  `a9d979e4f4fd84b797da89a33263d78604fe9224`, with clean ordinary Git status.
  This was a point-in-time observation, not an instruction to move either
  checkout. The recovery Git directory is separate, shallow and has no
  alternates file; the publication Git directory has an alternates file.
- The fixed c7dc tree has 711 root `verify_*.py` names and 54 native verifier
  names. The then-current recovery checkout had 713 root names: the extra
  `verify_postc7dc_followups.py` and `verify_postc7dc_optional_replay.py` would
  join the old driver's root glob if that driver were run there as observed.
  Root checkpoint-glob and native-verifier name sets matched in that one
  comparison, and no untracked ignored files were reported. This does not
  compare their bytes or modes, and other dynamically chosen paths may differ.
- With lazy fetch disabled, the recovery object store could traverse the c7dc
  tree and report headers for all 16,122 distinct blobs on its 24,242 tracked
  paths. Their path-summed sizes total 3,082,916,243 bytes; this is not disk
  usage. All c7dc entries reported regular Git modes 100644 or 100755. These
  checks do not rehash complete blob contents, compare the materialized
  checkout, prove all older history or external inputs, or establish
  off-machine custody. The recovery checkout's later index and files are not
  a c7dc materialization merely because its object store has those headers.
- The candidate driver discovers `verify_*.py` and
  `native-evidence/*/verify.py` dynamically and substitutes qualified
  successors from two data files plus named recovered/city successors
  (`check_published_evidence_privacy.py:10-37`). A tracked-tree reconstruction
  selects 763 invocations: 709 root invocations and 54 native, including 22
  qualified substitutions. `DRIVER-SUITE-INVENTORY.json` binds their order,
  selected paths, modes, Git blobs, byte counts and SHA-256 values. It is a
  predicted inventory, not an executed run or a complete dynamic-input graph.
  Extra ignored or untracked files, symlinks, added checkpoint JSONs, changed
  crosswalks or archive/member lists can change behavior even if four state
  files remain frozen. The qualified checkpoint-13 link verifier also builds
  an overwrite-on-later-key map from all matching root checkpoint documents
  (`qualified-059-early-links/.../verify_delegated_links_13_059_public.py:20-29`).
- The old checker asserts checkpoint-specific incomplete lens and queue
  counters (`verify_main_advance_evidence.py:141-156`). Running it against a
  later live queue is not an equivalent historical replay. Other checks bind
  exact directory sets, input records and data-directed paths. Conversely,
  changing only a live namespace does not establish that the new row states,
  connected ledger or counters are correct.
- An exact read-only mount is insufficient for the unchanged driver as written:
  its recovered verifier creates a temporary native Git clone under the
  evidence root (`verify_recovered_public_redacted_plus2.py:7131-7151`), and
  another verifier creates a temporary object-only Git repository in the
  system temporary area (`verify_p17_patch_recovery.py:32-39`). Interrupted
  runs, Python bytecode, Git configuration, replace refs, filters, concurrent
  writers and temporary outputs need explicit control and observation. A
  writable scratch root and controlled external temporary space can be used
  only if their effects and environment are recorded and the checkpoint bytes
  are rechecked afterwards. Do not assume the currently shared recovery
  checkout is free to reset or overwrite.
- The default driver does not pass every optional argument. For example,
  `verify_evidence_verifier_evolution.py:13-19` and checkpoint recovery
  verifiers record that their optional Git comparisons were not repeated;
  `verify_path_digest_correction_proposal.py:30-40` records whether private
  manifests were supplied and says their substitution replay was not repeated.
  The driver invokes qualified reconstructed-byte paths and labels its output
  accordingly. Retain those limits; do not relabel a default pass as a private
  original-byte replay or a remote recovery proof.

## Minimum binding and before/after gates

1. Choose a single immutable published checkpoint C after current publication
   work has settled. Bind its commit and root tree, product-source authority,
   full path/mode/blob inventory and SHA-256 of each materialized unfiltered
   file, including symlink targets if any. Record exact original and qualified
   reconstruction identities, source-proof and archive dependencies, object
   store/alternates, shallow boundary, Git/Python versions and environment.
   Verify all required objects and data-driven inputs locally without lazy
   fetching; a commit and root tree alone do not prove closure.
2. Reserve an isolated writable replay checkout. Before any replay compare its
   entire actual filesystem to C, including ignored/untracked paths, names,
   modes, bytes and all driver discovery sets; bind the ordered selected
   scripts and qualified replacement/crosswalk/argument mappings. Check
   separately that external inputs and optional private or Git checks are
   available and authorized. Coordinate the existing recovery checkout's
   ownership instead of treating a clean status as permission to repurpose it.
3. Run only the unchanged evidence driver in that controlled snapshot, record
   complete stdout/stderr, exit status, environment and every skipped or
   optional check. Independently verify resulting discovery and filesystem
   names, modes and bytes against the pre-run inventory, inspect temporary and
   external effects, and report failures or residue. A crash or changed input
   invalidates an unqualified replay claim. This review did not run that step.
4. Bind a separate authoritative live manifest to C and to all changed, added,
   deleted or mode-changed paths, not just four JSON documents. It must identify
   original 059 pointers and revision, previous and current dispositions,
   reviewer attribution, exact evidence objects, cross-lens ledger links,
   unresolved scope, corrected/reopened events and derived counters. Bind the
   current product revision and any later source delta explicitly. A reviewed
   live verifier must validate the actual committed live bytes and closure
   criteria independently of old checkpoint assertions. Make the live landing
   authoritative while retaining the old landing as a dated C observation.
5. Preserve privacy history and qualifications separately. At c7dc the
   15-file path-digest correction remains unapplied; its 522 corpus matches
   and 273 semantically selected fields have different evidentiary bases.
   A restricted C replay can retain those exact bytes, while a public live
   successor may need separately reviewed redaction/crosswalks. Do not call the
   frozen tree privacy-clean, erase earlier Git history by implication, or
   treat a bounded scan or public reconstruction as a private replay. Recheck
   publication/recovery, manifests and scan boundaries at each new checkpoint.

## What a different reviewer must establish

Independently reconstruct the driver discovery and all transitive/dynamic input
and output paths, including qualified successors and optional arguments; verify
C's complete Git/object and byte custody with an appropriately isolated actual
replay; review live schema, atomic change/correction semantics, source and
repair-ledger links and closure rules; and re-evaluate privacy crosswalks,
restricted originals and public history. The lexical inventory in this packet
is only an aid and can miss indirect behavior. Preserve original reviewer and
revision attribution, the missing 3821 queue/ledger, and the difference
between recovered, reconstructed, historical and newly assessed work.

The earlier proposal's author also performed this challenge, so its conclusions
remain a same-author review even where new observations are independently
repeatable. The registered design-metadata display was truncated; its full
saved projection is retained, and unseen material was not credited. A later
focused selection was displayed. No repairs, builds, tests, fixtures, runtime
probes, ledger edits, publication changes or checkout resets were performed.

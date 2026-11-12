# Independent review of the live-state successor proposal

Reviewer: `/root/mhr_character`. Proposal author: `/root/verifier_scope`. This is
an independent, bounded evidence-tool source review; no product or historical
evidence driver was executed and no live-state mechanism was implemented.

The split is feasible as a design, subject to proving its isolation and
recovery requirements. It is not yet an equivalent verifier or evidence that
historical verification can be replayed. The proposal correctly limits itself
to a candidate. The historical checkpoint **C** remains to be selected; the
published proposal is at publication commit `a9d979e4f4fd84b797da89a33263d78604fe9224`,
while its original source investigation was at the earlier publication commit
`a9b8331ae347542b1ea707b1b04eb8092735d1bd`. The product-source authority
`95e63628f9d70b8dd6e6fede64334696c6995009` is a different identity. None of
these restores checkpoint 3821.

## What the checked evidence establishes

1. The published privacy-qualified driver selects root `verify_*.py` names and
   `native-evidence/*/verify.py` dynamically. It replaces selected invocation
   names with qualified successors and runs each child with `check=True`.
   `check_published_evidence_privacy.py:10-38`.
2. The qualified checkpoint-13 link checker discovers all matching root
   checkpoint JSON files and overwrites duplicate manifest-key mappings with
   later entries: `delegated/qualified-059-early-links/successors/verify_delegated_links_13_059_public.py:19-29,54-62`.
   Its qualified crosswalk also walks the *whole* evidence root and rejects
   symlinks outside `.git`, not merely old packet paths:
   `delegated/qualified-059-successors/successors/qualified_crosswalk.py:22-46`.
   Therefore a live directory inside an old checker's root can affect behavior
   even when no historical JSON is edited and no new root checker is added.
3. A tracked-name census independently reproduced the proposal's time-bound
   a9b driver selection: 700 raw root verifier filenames, of which two optional
   names are excluded, yielding 698 root invocation slots plus 54 native slots.
   At a9d these figures are 713 raw names, 711 root slots and 54 native slots;
   thirteen root names were added. These are selected names, not successful or
   even reached invocations: the driver can stop at the first failing child.
   Substitutions change the actual script paths, and untracked or ignored names
   are outside this tracked-name census. See `discovery-census-07.json` and
   `discovery-count-clarification-07.json`. This is a concrete example of
   checkpoint drift, not a correction of the proposal's explicitly a9b count.
4. `verify_main_advance_evidence.py:126-157` reads frozen predecessor snapshots
   and asserts exact historical source-delta, incomplete lens, transition-cell,
   and queue counts. An advancing current record can be correct yet fail this
   old historical check. `CONTINUATION.md:3-14` is also a time-bound landing
   whose wording must not silently claim that frozen counts remain current.
5. The driver dispatches a qualified recovered verifier rather than the
   historical original-byte check. Its crosswalk separately binds original and
   public queue/ledger identities; it explicitly does not prove private-to-public
   transformation: `qualified_crosswalk.py:1-6,49-69,73-89,106-116` and
   `CROSSWALK-059.json:2-32`. A successful public replay is consequently a
   qualified public-evidence result, not proof that reconstructed public bytes
   equal private originals.
6. A read-only checkout is insufficient for the unchanged selected verifier:
   `verify_recovered_public_redacted_plus2.py:7141-7150` creates a temporary
   local bare clone under its root; `verify_p17_patch_recovery.py:32-36` creates
   a temporary bare repository elsewhere and writes Git objects to it. Their
   actual creation was not attempted in this review. The latter explicitly
   restricts system and global Git configuration at lines 1-4; that one example
   is not a complete environmental guarantee for every selected child.
7. Some verifiers have optional Git checks the default driver does not pass to
   them. For example `verify_checkpointa9b_recovery.py:16-20` makes its Git
   parent comparison optional, and the driver normally invokes children without
   arguments except for explicit substitutions. Likewise,
   `verify_evidence_verifier_evolution.py:13-20` separately reports whether Git
   source bytes were re-read. A driver-success line cannot by itself claim
   those optional checks or independent checkout custody.
8. Several inspected verifiers rely on Python `assert`, including the driver at
   lines 14,17,19, `qualified_crosswalk.py:49-68`, and
   `verify_checkpointa9b_recovery.py:6-15`. The driver's child processes inherit
   its environment. If Python optimization is enabled through the environment,
   these assertions can be disabled; a zero exit alone is then insufficient.
   This is a conditional source-level weakness, not a claim that a prior replay
   used optimization or produced a false success.

## Failure modes and minimum safeguards

- **Checkpoint boundary and actual bytes.** Select an immutable, actually
  published C, record its commit and tree IDs, qualified driver identity and
  per-path raw bytes/modes, and independently verify the full required object
  closure and custody. The proposal's tree listing is an identity inventory,
  not the blobs themselves or proof of remote availability. Create a separate
  recovery root with no live namespace inside it. Inventory tracked, untracked,
  ignored and temporary paths, symlinks and file types before and after; Git
  status or a Git tree alone cannot show ignored files, empty directories,
  filesystem metadata or all external dependencies. Verify unfiltered bytes,
  not just checkout status. Record observed repository/common-object locations
  and ensure scratch writes cannot modify the protected extraction, the live
  publication, or a shared historical object/ref store. A standalone object
  store or a separately justified read-only shared store can meet that goal.
- **Reproducibility of the actual driver.** Before any future authorized replay,
  record the selected exact scripts and substitutions, external and private
  inputs, interpreter/Git versions, flags, environment, configuration, working
  directory, allowed writable roots, optional checks and failure policy. Check
  that Python assertions remain enabled for every child, including inherited
  environment; minimize module-path and Git configuration surprises without
  claiming those settings reproduce an unknown historical environment. Capture
  a complete exit/skip/error record and before/after inventories. Preserve
  private raw logs separately if privacy requires a public reconstruction.
  Execute optional object or private-original checks separately when available;
  label unavailable ones and do not upgrade a public-only check into a private
  or independently recovered result. If replay fails or prerequisites are
  missing, report that rather than claiming preservation.
- **Binding evolving state.** Freeze a coherent live snapshot for each published
  checkpoint, compare all additions, modifications, deletions, modes and
  inherited references with C, and verify the *actual target bytes*, not only a
  scratch overlay. Include namespace boundaries and canonical paths; reject or
  account for links and aliases. Resolve the manifest's self-reference by an
  explicit excluded manifest field/path or a separately anchored successor;
  do not silently treat a manifest as hashing itself. Serialize updates or bind
  one immutable snapshot so readers cannot combine an old ledger with a new
  queue. A tree hash proves content consistency, not author identity, remote
  availability, access control, append-only enforcement or chronology.
- **Authoritative state and progress.** Make the current landing point to one
  live baseline and clearly label C's old landing as historical. Identify each
  original 059 row by its original revision and source pointer, and link each
  new or corrected live row to the named reader, actual registered/displayed
  source ranges, evidence revision, safeguards, open conditions and event.
  Derive counts from distinct identified rows and a documented definition;
  independently check membership and evidence rather than trusting arithmetic
  alone. Keep original historical dispositions, inferred/reconstructed records,
  newly assessed work and aliases separate. Duplicate or superseded findings
  must retain their relationships without becoming extra defect counts.
- **Closure and reopening.** A lens or source-level audit may advance only under
  an independently reviewed scope and closure rule that accounts for all
  required roles/queue entries, unread ranges, source deltas, cross-lens links,
  historical and current provenance, competing interpretations and unresolved
  reachability or policy. Record explicit corrections and reopenings; do not
  assume progress is monotone. A source-only audit disposition is not a repair
  or product validation. No implementation or product validation is authorized.
- **Privacy and custody.** Compare stored, decoded, and public-reconstructed
  representations separately, preserving original whole-file identities and
  restricted exact originals without publishing sensitive values or individual
  private path/prefix digests. Bound recursive scans and list omissions; inspect
  logs, Git objects/history, nested archives, paths and metadata as applicable.
  A public redacted successor can be recoverable as public bytes without giving
  the public enough information to recover restricted originals. A forward
  correction cannot erase old commits or prior copies, and a privacy scan is
  not proof of complete absence.

## Limits and attribution

The source windows, original author receipts and truncation qualifications are
recorded in the registrations and read receipts bundled here. Full copied
source bodies and tree hashes are not full personal-read credit. We inspected
selected tools and did not establish their full dynamic dependency graph, all
substitution targets, or the whole 3 GB historical object closure; a metadata
census is not a full content rehash. We did not run or modify the old driver,
perform a checkout, publish a successor, decide an alias or unique defect
count, close a queue/lens, or perform product verification. Existing prior
readers retain their own attribution, and the original 3821 records remain
missing. Git-tree membership proofs are bundled; binding a named revision to
its tree still requires the original commit object, which is not copied here in
order to avoid unnecessary commit-author metadata. The owner must separately
assess public privacy and independent-machine custody.

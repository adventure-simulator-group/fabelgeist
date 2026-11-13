# Evidence-guard review, successor record

This is a bounded source inspection and evidence-only replay. The application
candidate was uninstalled at this review cutoff. Installation, publication,
independent backup, product behavior and semantic audit closure are separate.

The original v2 guard checked seven unique root paths without naming them.
The current root capture has the expected seven names. Guard v3 added an exact
set assertion, while preserving v2 and its correction. Guard v4 changes only
the index-correction chain to include the additive application-custody
navigation. Its SHA-256 is:

`ac6b8ff0879d806710925af66420e5d350ffdd8f73b9d817c94f92d4a5cfc33f`

The saved v3 bytes match the reviewed predecessor. The v3-to-v4 correction and
the index v4-to-v5 correction bind their exact predecessor and successor
hashes. Index v5 adds only the application custody pointer and its recorded
12-collection, 109-file, 20-directory restoration and later-control limit.
The separate application custody note retains key, Git, prior-ciphertext,
finite-scan and same-machine qualifications.

The initial application receipt wrapper passed without a key, but failed when
supplied the original prior-inventory file because it requested `nonce_hex`
instead of the recorded `nonce` member. The isolated failure and exact
stdout and stderr are separately retained. Wrapper v2 preserves its predecessor
verifier and manifest, corrects the field and requires the exact `PEER-FULL`
and `PEER-PUBLIC` labels. Independent no-key and original-inventory checks of
the isolated successor both exited zero with empty stderr. Its manifest
SHA-256 is:

`f5052a426f01e16d8109ed9239799e684b2fcf0eecd168fe395cc3a8786968ae`

The staged ciphertext package and original receipt and scan files match the
sealed source bytes. Comparing new nonces to a supplied inventory does not by
itself prove that inventory's provenance. The receipt records an earlier
same-machine authenticated restore; the no-key verifier does not repeat it.

Guard `VERIFY.py:10–24` checks the captured regular-file set, Git executable
bits, exact seven root names and correction-chain identities. It does not
check all directory modes or the complete directory set. The installer
`INSTALL-TEMPLATE.py:9–17` compares captured directories and local modes before
copying. The guard requires the four closing files to be all present or all
absent; `verify_post76fe_checkpoint.py:6–9` invokes their separate verifier.
The guard binds the parent Git tree, but replaying a clean exact predecessor
requires its optional predecessor input and historical objects.

The capture and installer retain the sequential, nontransactional operation
and partial-failure qualifications. The reviewed scripts do not by themselves
establish a trusted final checkpoint: final PINS had not been generated at the
review cutoff, and later closing capture, exact commit anchoring and separately
held custody remain necessary. No installer, product code, repair, build, test
or runtime probe was run as part of this review.

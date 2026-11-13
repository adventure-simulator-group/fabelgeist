# Independent review of the post-76fe evidence guard

Reviewer: `final_guard_peer`, separate from the guard and installer author.
This bounded review uses the same staged material and the separately held
prior nonce inventory used by the earlier `verifier_scope` review. It is not
independent evidence custody. At this review cutoff, the candidate had not
been installed: the publication checkout was clean at checkpoint 76fe and
its expected tree.

No concrete blocker was found in the reviewed evidence-only installation
path. The later stage capture, installation, closing checks, commit,
publication and recovery remain separate work; they were not performed by
this reviewer.

- Guard v4 is
  `ac6b8ff0879d806710925af66420e5d350ffdd8f73b9d817c94f92d4a5cfc33f`.
  Its v2-to-v3 change asserts the exact seven root names; v3-to-v4 extends
  the index correction chain. Both transitions and all four index correction
  links match their recorded predecessor and successor hashes.
- The final PINS file is
  `0ebd265d0e5c62d6ae42a71317f346e24395ecafccae9887a0972d5ba0cdfe14`.
  Its 27 names match the guard's required set and all staged hashes match.
  Twenty-five values also match earlier received identities; the remaining
  two bind the current index and root-candidate capture. The seven candidate
  root images independently match their recorded size, SHA-256 and Git blob.
- The corrected application custody wrapper replaces the erroneous
  `nonce_hex` lookup with `nonce` and requires the two named replay labels.
  Independent no-key and supplied-inventory invocations both exited zero with
  empty stderr. The supplied inventory has 626 distinct twelve-byte values;
  this comparison does not independently establish its provenance. The
  no-key replay checks identities and recorded results, not authenticated
  decryption. The initial wrapper failure remains an earlier separate record.
- The guard checks the captured regular-file set, content identities and Git
  executable bits before subordinate evidence verifiers. It allows either all
  four closing files or none. It does not compare the complete directory set
  or full local modes. The capture records those modes and directories; the
  installer compares them for captured paths and the stage root before and
  after the copy. The newly written capture file's mode and later status and
  closing files are outside those captured mode comparisons. Git does not
  preserve directory modes or non-executable permission bits.
- The installer checks the exact branch, parent and tree, before-images, root
  candidates and absent destinations before proceeding. Its Git status check
  covers tracked and nonignored untracked paths, not all ignored files.
  It then writes sequentially; the log and status do not make it atomic.
  An interruption can leave a partial checkout that must be inspected. The
  final guard checks the parent and source Git tree identities but does not
  independently assert the current checkout's HEAD. Its predecessor replay is
  optional and requires a clean 76fe checkout and historical Git objects.
- The root verifier calls the guard and a separate closing verifier. That
  closing verifier did not exist in the reviewed stage and was not reviewed.
  Capture and status are self-consistency records until later closing and
  commit anchoring. The recorded main-branch observation is point-in-time;
  a fresh observation and separate remote recovery remain necessary.

Same-machine custody, the separately held key, historical objects, earlier
ciphertexts and later post-build controls remain dependencies. Finite scans
cannot prove the absence of unknown private data. This review does not recover
checkpoint 3821, close a lens or establish product behavior. It did not run an
installer, product code, repair, build, test or runtime probe.

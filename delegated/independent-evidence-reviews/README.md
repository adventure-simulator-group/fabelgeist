# Independent review preservation

This additive packet preserves five separately attributed independent reviews:
checkpoint 35, checkpoint 37, checkpoint 38, and the initial and successor
checkpoint 7 tar reconstructions. The exact frozen original review packets are
retained in a separate local-only collection. This packet retains safe bytes
unchanged and replaces genuine local paths in six checkpoint 38 files with
consistent placeholders. `REVIEW-IDENTITIES.json` maps every original and
public member hash, size, and redaction; it also records both checkpoint 7
tar archives and their 372 individual entries, including decoded gzip hashes.

The original review manifests remain exact historical records. In checkpoint
38 they refer to original private-local files and do not describe the six
reconstructed public copies. The new root manifest describes this public
packet. Redacted historical scripts are provenance, not self-contained
reproduction tools, and should not be run as evidence checks. Their external
inputs remain necessary.

`DEPENDENCIES.json` hash-links the external checkpoint 37 checked-stage
snapshot. Its 12,541 files were independently verified against the recorded
snapshot manifest, but neither that stage nor private original archives are
included here. The two checkpoint 7 archives are distinct, already
reconstructed public evidence artifacts; their private source archive and the
earlier damaged opaque public archive have separate hashes. The initial and
successor review findings belong to their respective observed artifacts.

`python3 verify-public.py` checks this packet read-only, including nested
archives and contextual privacy scanning. An optional local-original root
permits comparison with the exact frozen review packets and reconstruction
operations. These checks do not execute product code or validate product
behavior. Contextual privacy detection is bounded and cannot establish that
every possible sensitive value has been found.

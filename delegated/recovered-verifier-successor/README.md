# Qualified recovered-evidence verifier successor

This packet preserves the exact original verifier and deterministically builds a
separately named successor. Only three already-identified public objects receive
qualified crosswalk checks. Their original hash-named paths and historical
manifests remain historical identifiers; those public files contain redacted
bytes with distinct hashes. The three untouched historical manifest bytes are
also pinned. All other assertions are retained by the unique-anchor transform.

`sample-publication/` contains only the three redacted objects, their three
historical manifests, and the exact crosswalk. It is not a repository snapshot.
The successor expects the crosswalk at the same relative path shown there when
placed alongside the complete publication evidence. This packet does not alter
the publication checkout.

Run `python3 -B verify-evidence.py` for bounded read-only checks. The optional
`--originals-root` argument verifies the crosswalk's transformation against the
separately retained private originals; use `--original-home-prefix` when the
originals came from a different user home. `--publication-root` additionally
checks the observed original verifier, manifests, original objects and the
Lens 12 parent reference. No full verifier or product code is run by this check. Run
`python3 -B verify-privacy.py DIRECTORY` to scan the packet with bounded
contextual path and common-secret patterns.

The original aggregate runner invokes the original verifier and discovers other
`verify_*.py` files. Merely adding this successor while leaving that runner
unchanged would still run the original and fail on the three redacted objects.
Central integration must explicitly select the successor and avoid also invoking
the original, while preserving the original as historical evidence.

Private originals, old Git blobs, archives and the separately identified 059
raw-object exposure are not removed here. The successor itself cannot prove
original private bytes from their redacted public reconstructions. Full
integration verification remains unperformed.

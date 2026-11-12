# Privacy and verification limits

The predecessor was withheld because its crosswalk included digests of removed
local-path values. This successor records only whole-file hashes, byte offsets,
removed lengths and opaque aliases. It does not disclose hashes of the removed
values or make the private original recoverable without its separately held copy.

Preliminary scans found no candidate home, temporary, Windows-user, email or
credential patterns, and no match to a separate restricted 40-digest corpus. A
private corpus of 191 removed path and nontrivial prefix values was also checked
in both text-hex and raw-digest forms across raw and bounded JSON/base64-decoded
views; it found no matches. The corpus values and their digests are not included
in this successor. Final manifest-bound scan and replay receipts are held
separately and included with the portable delivery.

These checks cover only the specified patterns, formats, recursion depth and
view-size limits. They do not clear unknown secrets, encodings or archive types,
Git history, symlinks, or separately held original packets and external anchors.
Whole-file hashes and replay establish byte correspondence to supplied evidence,
not independent custody, semantic truth, intended policy or runtime behavior.

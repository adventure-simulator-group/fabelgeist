Before the manifest existed, a verifier preflight intentionally disabled the
manifest check in memory. Its generic PASS line did not mean a manifest was
checked. The separately sealed final scan receipt records the normal verifier
run after manifest creation; no verifier code was changed by the preflight.

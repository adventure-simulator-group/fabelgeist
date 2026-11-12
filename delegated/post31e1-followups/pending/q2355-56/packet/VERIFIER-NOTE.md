Use `VERIFY-EVIDENCE-v2.py`. The retained predecessor incorrectly assumed that
all original source captures contained an e275 equality flag. Version 2
checks that flag only when present and separately checks all 15 original
source identities against their recorded 6533/e275/95e hashes. The predecessor
was not used for the sealed final verification. A preflight of version 2
deliberately bypassed the not-yet-created manifest; its PASS line is not a
manifest check. The final scan receipt captures the ordinary manifest check.

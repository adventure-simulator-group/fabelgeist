# Supplemental registration source restoration

These 43 source identities are cited by recovered registrations and were
verified against their pinned Git bytes. They are absent from the existing
373-entry source manifest, but may overlap other restoration collections.
This does not establish source-read credit, current authority or audit closure.

Run `python restore_sources.py REPOSITORY DESTINATION` from this directory.
The destination must not exist. Each source hash is checked, and an incomplete
marker remains if restoration fails.

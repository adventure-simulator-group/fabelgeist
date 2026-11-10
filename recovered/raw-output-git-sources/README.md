# Git sources identified in recorded outputs

The manifest contains 248 revision, path and SHA-256 identities matched to
Git bytes and complete JSON row fragments in preserved outputs. They are
outside five previously checked source manifests but may overlap other
collections. The scan also found two external Burn identities whose matching
bytes are separately preserved in `../raw-output-external-sources/`.

Run `python restore_sources.py REPOSITORY DESTINATION` from this directory.
The destination must not exist. Each hash is checked and an incomplete marker
remains on failure. Source preservation does not establish read attribution,
audit credit or the final audit authority.

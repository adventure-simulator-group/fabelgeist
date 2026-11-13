# Evidence-only application

`APPLICATION-EVIDENCE.tar.gz` contains four predecessor images, four
prospective after-images, two new root records, the transform and the candidate
receipt. Tar ownership and timestamps are normalized. The report's adopted
wording becomes operative only after exact later installation.

Before installation, replay with `python3 -B VERIFY.py --publication CHECKOUT
--candidate CANDIDATE`. After installation, omit `--candidate`. An optional
`--predecessor` names a separate clean 8ca checkout and runs its evidence guard;
that guard requires the separately held historical and pinned-source Git
objects, such as the documented minimal process-local alternate.

These checks establish captured evidence identities and deterministic
application only. Source semantics, same-machine key and object dependencies,
independent custody, missing checkpoint 3821, audit closure and product
behavior remain open. No product build, test or runtime probe is performed.

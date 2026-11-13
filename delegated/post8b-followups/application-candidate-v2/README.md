# Five-row evidence application candidate

This package preserves the reviewed proposal, the evidence-only transform and
its exact before and candidate after images. It does not itself install them.
The report's adoption statement is prospective until those exact images are
installed in a reviewed successor checkpoint. Counts are planning and partial
assessment counts, not lens closure or unique defects.

Run `python3 -B VERIFY.py` to check the transform and assembly review-member
identities. After installation, `--publication CHECKOUT` also checks the exact
root files. `--predecessor CHECKOUT --ancestor CHECKOUT` additionally checks
the exact 8b parent files and runs its guard against a separate 8ca predecessor.
Without both arguments that older guard is explicitly skipped. The separate
checkouts and any historical Git object alternate remain external dependencies.
The connector observation is point-in-time; its raw response is separately
held. No check here runs product code or establishes product behavior.

# Replaying the post-8b evidence guard

Run `verify_post8b_checkpoint.py` from the installed evidence checkout. Without
options it checks the installed staged bytes and the selected evidence
verifiers; it explicitly reports that the earlier predecessor replay was not
run. `--predecessor` supplies the separately recovered 8b checkout and
`--ancestor` supplies the separately recovered 8ca checkout. The latter causes
the older guard to run with its own predecessor. The still earlier 76fe
predecessor remains a separate check.

The separately held minimal Git object alternate used during this audit can be
supplied with all three options `--object-alternate`, `--object-inventory` and
`--object-inventory-sha256`. The first two are absolute, canonical paths; the
third is the lowercase SHA-256 of the exact inventory file. Run
`OBJECT-ALTERNATE-INVENTORY.py --objects <absolute-object-directory> --output
<new-private-absolute-file>` to produce the specified inventory format, keeping
that file outside the object directory. The guard clears inherited `GIT_*`
variables, checks the supplied inventory digest and every listed file and
directory against that alternate, and sets the alternate environment to that
directory for its child checks. Git may also follow on-disk alternates
registered by those checkouts. The guard checks that each supplied predecessor
is the Git checkout root.

The caller-supplied digest binds this invocation to the supplied inventory; it
does not independently establish where those bytes came from. Compare it to a
separately retained trusted record when one is available. The predecessor
checkouts also rely on their separately held shared Git object stores. This is
not a full Git object closure or an independent backup, and does not repeat the
authenticated restoration of private evidence. The supplemental inventory
captured for this audit and its generation record were created after the fourth
fixed custody cutoff and are separately held. No product execution or semantic
audit closure follows from these evidence checks.

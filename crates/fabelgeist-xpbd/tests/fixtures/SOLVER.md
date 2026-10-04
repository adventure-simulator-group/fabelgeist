# Native solver state fixture

`solver.bin` was captured from the original scalar solver before changing its
interfaces. Twelve inputs run through whole-step submission, interleaved
submission, and direct substep recording, in that order. Each state retains
position, previous-position and velocity buffer bytes, followed by before/after
hook phase words and the substep's exact binary32 word. Words are little-endian.

Inputs cover an ordinary interval, zero substeps, positive and negative zero,
a negative interval, a NaN with payload `0x7fc01234`, both infinities, zero and
three constraint sweeps, empty particles, and a minimum subnormal interval
subdivided four times. Other cases use two substeps and one sweep. Nonempty
cases contain one free and one pinned particle connected by a compliant
constraint. Empty cases omit the constraint during recording.

The fixture preserves existing differences in admission: full steps skip
nonpositive intervals and zero substeps, while direct recording invokes both
hook phases even for those inputs. NaN admission differs between the solver
and the shell's interactive driver. Do not regenerate from the implementation
under test to update expectations.

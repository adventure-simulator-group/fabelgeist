# Privacy scan and output correction

The preceding guide packet and initial scan receipt are preserved unchanged.
Its email-like pattern was over-escaped and did not detect a synthetic address;
its reported zero email-like count is invalid. The original scanner program
was not retained as a separate file, so the reproduction is reconstructed from
the tool transcript. This successor runs a corrected bounded scanner whose
source and receipt are retained. It tests its pattern on a synthetic address.

The preceding guide also stated too broadly that the driver never prints local
paths. Normal output is label-only, but Python exceptions and tracebacks can
contain paths. Treat all driver and helper output as private. Neither scan
inspects encrypted archives or restricted evidence, or establishes that all
possible secrets or path values are absent.

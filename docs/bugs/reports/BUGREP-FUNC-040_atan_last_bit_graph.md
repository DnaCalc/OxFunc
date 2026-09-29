# BUGREP-FUNC-040: ATAN last-bit arithmetic graph

Status: `triaged`; canonical stream `BUG-FUNC-061`; owner `oxf-mwue.28.12`.

ATAN public FPATAN candidate matches 1224 discovery rows and 297 independent typed cases. A frozen independent 29573-row numeric cohort exposes two one-ULP counterexamples. Preserve the failed candidate, repeat the exact inputs and discriminate arithmetic graphs; do not promote whole-function or alternate-platform parity.

The canonical stream records retained evidence, current reference axes and open
scope. Whole-function completion is not asserted.

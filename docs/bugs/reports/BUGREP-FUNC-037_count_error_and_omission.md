# BUGREP-FUNC-037: COUNT and COUNTA error and omitted-slot behavior

Status: `triaged`; canonical stream `BUG-FUNC-059`; owner `oxf-mwue.28.10`.

The 2026-09-29 W111 typed-origin packet reproduces this behavior on Excel 16.0
build 20430, 64-bit, Compatibility Version 2. Reference and computed-array origins
remain distinct. Evidence and known admission limitations are retained under
`docs/function-lane/evidence/w111-broad-20260929/count-family/`.
Baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`. Candidate and independent replay remain separate.

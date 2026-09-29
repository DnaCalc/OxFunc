# BUGREP-FUNC-039: conditional prepared-value behavior

Status: `triaged`; canonical stream `BUG-FUNC-060`; owner `oxf-mwue.28.11`.

The W111 live Excel typed packet exposes coercion, array shape, per-cell masking
and missing-value differences in IF/IFERROR/IFNA. See the canonical stream and
`docs/function-lane/evidence/w111-broad-20260929/conditional/` for exact replay and
qualification. HO-FN-023 keeps evaluator scheduling and reference selection open.

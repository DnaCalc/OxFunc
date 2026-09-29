# NOT logical-text observations

Status: `execution_state=in_progress`, `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: missing-argument evaluator admission,
and full evaluator/seam integration. This packet makes no whole-function parity claim.

Excel 16.0 build 20430, workbook Compatibility Version 2, produced 542 observable
rows in `cases.json` and `excel.jsonl`. A 543rd formula, `NOT()`, was rejected at
formula entry and is explicitly retained as a harness-limited observation.

The same rule applies to direct values, dereferenced cells, and each cell of row,
column, or reference arrays: ASCII-case-insensitive `TRUE` and `FALSE` text are
logical values; every other text, including numeric text, empty text, surrounding
whitespace and Unicode near-spellings, returns `#VALUE!`. Blank cells return TRUE.
Numeric zero returns TRUE, other numbers FALSE, logical values are negated, and
worksheet errors propagate per cell. The capture contains all 48 ASCII case
variants of the two recognized spellings, five origins, and seven error codes.

The former scalar path accepted numeric text through numeric coercion and rejected
logical text. The former array path rejected all text. Both now use the existing
`parse_excel_logical_text` helper without trimming. The rule differs from AND/OR/XOR,
which ignore text from reference/array origins and ignore unrecognized direct text.

`w111_not_live_replay.rs` binds IDs, function/formula text, exact typed results,
array dimensions, and reference fixtures; its 542 admitted rows pass. Four existing
NOT unit tests remain relevant. Lean `NotFn` now contains the prepared-cell rule,
array mapping, text binding and error-preservation theorem; its focused build passes.

The independent mixed-array holdout generator is `gen_not_holdout.py`, seed
2026092910. All 120/120 heldout rows match the unchanged frozen candidate, retained
as `frozen-not-fn.rs` (SHA256 `76241853772a9056e24ec1f2f3d196cb0354b916e076b500d62c13ab87f19868`).
Its case-set metadata stores that source hash before capture. Both replay tests
pass, totaling 662 admitted observations plus the explicit formula-entry limit.
Canonical contract notes should reflect the origin-independent logical text rule
and the distinct blank-versus-empty-text behavior. Root owns canonical contract
and evaluator-impact reconciliation.

# HO-FN-024: text shapes, defaults and compatibility context

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.13`, BUG-FUNC-062.

The retained text-slice campaign has 34,267 admitted exact observations on Excel
16.0 build 20430, workbook Compatibility Version 2. Its typed and raw UTF16
captures distinguish omitted defaults, integer conversion, per-cell errors,
multiple-array broadcasting and surrogate handling. See the
[evidence and assessment](../function-lane/evidence/w111-broad-20260929/text-slice/README.md).
Seven empty-call entry failures and 264 NUL-input transformations are withheld.

LEFT/RIGHT/LEFTB/RIGHTB default an absent count to one; an explicit missing slot
is zero. Their positive count conversion differs from MID/MIDB. Negative raw
counts are rejected before conversion. MIDB addresses raw UTF16 units while
the observed CV2 MID and LEFT/RIGHT variants treat valid surrogate pairs as one
position. The retained trailing-unpaired-surrogate cases further constrain
LEN, MID and LEFT fallback behavior; replacing them with a generic Unicode
string iterator would change observed results.

Local array lifting now preserves shapes and applies ordered scalar coercion
per coordinate, with missing coordinates represented as #N/A. Existing metadata
that declares no lift, and ENCODEURL's surface dependency declaration, remain
unchanged pending this assessment. The evidence is for the recorded non-DBCS
locale; it does not establish a different locale or compatibility version.

Receiving-side work: replay the retained typed and raw-unit cases through the
evaluator, preserve explicit missing slots and UTF16 units, assess lift and
reference-dependency declarations against actual preparation, and supply the
declared workbook/locale context for version-dependent behavior. Record the
acknowledged declaration changes and their integration evidence before promotion.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: compatibility/DBCS context,
remaining text policies and carriers, declaration assessment, receiving
acknowledgment and evaluator integration. Filing opens the dependency.

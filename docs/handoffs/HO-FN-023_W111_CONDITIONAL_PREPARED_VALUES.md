# HO-FN-023: conditional coercion, shape and missing-value corrections

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.11`, BUG-FUNC-060.

The 2026-09-29 public-interface Excel campaign exposed 324 differences in a
716-case IF/IFERROR/IFNA discovery packet. The revised prepared-value adapters
match 1,992 admitted observations:812 discovery/control,520 qualified first
independent and 660 fresh second independent rows. The first holdout exposed
24 direct one-cell array failures, which were repaired before the second freeze;
140 malformed prepared-array inputs remain withheld. Reference: Excel 16.0 build 20430, 64-bit,
Workbook Compatibility Version 2; update channel is unverified.
The [impact assessment](../function-lane/evidence/w111-broad-20260929/conditional/IMPACT_ASSESSMENT.md)
binds the observations to exact fixture/formula readback and retained replay.

## Observed function behavior

- IF accepts ASCII-case-insensitive `TRUE` and `FALSE` condition text, but rejects
  numeric text and surrounding whitespace. Conditions resolving to arrays apply
  coercion and errors per cell.
- Scalar IF selects the chosen branch's shape. Array IF uses the coordinatewise
  maximum of the condition and both branch shapes, including an unused branch.
  Singleton dimensions broadcast. Missing coordinates of non-singleton axes
  produce `#N/A`; an unselected coordinate cannot replace a selected value.
- Array IFERROR/IFNA include fallback dimensions even when current primary cells
  need no fallback. Padding in the primary is a catchable `#N/A`. IFERROR catches
  all worksheet errors; IFNA catches only `#N/A`.
- Selected blank cells and explicitly missing arguments publish numeric zero.
  An absent third IF argument still yields logical FALSE on the false branch.

The candidate preserves public signatures and metadata. Scalar selection retains
selective reference preparation; array selection prepares both shape-bearing
branches. This changes the values and dimensions the evaluator receives.

## Receiving-side assessment requested

1. Replay the typed coercion, missing-slot and broadcasting cases through the
   evaluator, preserving argument origins and exact error values.
2. Assess how array shape discovery interacts with selective branch scheduling,
   volatility, effects and dependency discovery. The local replay receives
   already evaluated values and establishes none of those scheduling properties.
3. Preserve reference-returning selection where the surrounding expression
   requires it. `COUNTBLANK(IF(FALSE,A100,A100))` observes the chosen reference;
   it is not equivalent to a literal/computed numeric IF result. The ordinary
   value adapter's tests do not establish that reference-valued expression seam.
4. Acknowledge and record integration before either repo promotes the seam claim.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: expression scheduling and side effects, reference-returning selection, uncommon
reference forms and host publication, receiving acknowledgment and integration.
Filing this packet opens the dependency.

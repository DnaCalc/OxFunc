# IF / IFERROR / IFNA prepared-value corrections

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

The live W111 Excel build 20430, workbook Compatibility Version 2 packet observes
716 discovery cases and 96 additional uniform-selector shape controls. Before
the repair, 324 discovery outcomes differed. The revised prepared-value adapter
matches all 812 exact typed outcomes. The original local outcomes are retained.
`candidate-freeze.json` records the source before an independent 660-case packet.
That frozen candidate matched 496 rows, with 24 semantic differences and 140
invalid local inputs. The generator incorrectly allowed syntactic omitted CHOOSE
arguments as Missing cells in already evaluated arrays; all 140 remain retained
and excluded from semantic comparison. Preserving direct 1x1 array selector
identity resolves the 24 valid failures: all 520 admitted rows now match.
`refined-candidate-freeze.json` precedes a corrected fresh 660-case packet.
That second independent packet matches all 660 observations without further
source changes, giving 1,992 admitted exact observations across the family.

## Function-facing corrections

- IF accepts only ASCII-case-insensitive `TRUE` and `FALSE` text as conditions.
  Numeric text and whitespace-decorated logical text return `#VALUE!`, including
  reference and array origins. `=IF("TRUE",7,9)` returns 7;
  `=IF("2",7,9)` returns `#VALUE!`.
- Reference conditions resolving to arrays use the same per-cell rule. A cell's
  condition error affects its result cell, rather than collapsing the whole array.
- Scalar IF selects the chosen branch's shape. For array conditions, output
  dimensions are the coordinatewise maxima of condition and both branch shapes,
  including dimensions of an unused branch. Singleton dimensions broadcast.
  An absent non-singleton coordinate reads as `#N/A`; an unselected branch's
  absent coordinate does not override the selected value.
- Direct 1x1 arrays retain array-selector behavior until output dimensions are
  determined; a one-cell reference follows scalar reference preparation.
  Thus `=IFERROR(CHOOSE({1},1),{11,12})` returns `{1,1}`, while the corresponding
  scalar primary returns scalar 1. A final 1x1 result publishes its scalar cell.
- IFERROR and IFNA select fallback per cell, catching every error or only `#N/A`
  respectively. Array primaries use the coordinatewise maximum of primary and
  fallback shapes, even when existing primary cells need no fallback. Padding
  introduced by a missing primary coordinate is itself a catchable `#N/A`.
  `=IFERROR({1,#N/A},{11,12,13})` returns `{1,12,13}`.
- Explicit missing selected arguments and blank cells publish numeric zero:
  `=IF(TRUE,,9)` and `=IFERROR(#N/A,)` return numeric zero.
  Omitted IF false arguments remain logical FALSE: `=IF(FALSE,7)`.

All examples above are represented by exact typed equivalents in the retained
case/oracle files; their actual formula strings use fixtures for numbers and
CHOOSE expressions for typed arrays. Formula text, function identity and case
identity are bound during replay.

## Evaluator seam and limits

The local runner supplies already evaluated argument values. This packet proves
function-level coercion, selection, masking and shape behavior. It does not prove
that Excel avoids evaluating an unselected expression, or establish timing,
volatility, effects, dependency discovery or branch evaluation order. Scalar
adapters retain selective preparation of reference arguments; array selection
prepares both shape-bearing branches to reproduce the observed spill dimensions.
OxFml must assess how these prepared-value rules interact with its own evaluator
scheduling. No function metadata or public type/API declarations change here.

A concrete outer-context qualification is reference-returning selection:
`COUNTBLANK(IF(FALSE,A100,A100))` can observe a selected reference, whereas
`COUNTBLANK(IF(TRUE,0,0))` receives a computed value and returns `#VALUE!`.
The campaign's COUNTBLANK packet separately observed this distinction. Replacing
the selected reference with an already dereferenced scalar would erase that
provenance; this packet's values-only return path does not establish that seam.

The IF and IFERROR canonical contracts need their earlier phase-completion
statements qualified, their scalar-only array descriptions expanded, and their
missing-value rules corrected. In particular, IFERROR's missing fallback is zero
in these observations, contradicting its earlier `#VALUE!` statement. Root owns
the canonical contract updates and HO-FN-023 handoff registration.

Open lanes: evaluator scheduling and reference side effects; uncommon reference
forms and host publication; reference-returning selection; cross-repository
acknowledgement and integration. Locale and alternate Excel-version sweeps are
separate validation axes.

# OxFunc Function Status Map

Generated: 2026-05-28T20:22:06.2954090Z

This map is the reproducible derived view of where each of the published OxFunc surfaces stands against the bit-exact Excel parity goal (CHARTER.md §4.1). Rebuild with `smart-fuzzer/tools/Build-FunctionStatusMap.ps1`.

Inputs joined:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`
3. `docs/bugs/BUG_STREAM_REGISTER.csv` plus the curated stream→surface map in this script
4. `smart-fuzzer/runs/*/rollup.json` (array_rollup schema) plus broad_scalar comparisons for `scalar_swept_only` coverage

## Tally

| Status | Count | Meaning |
| --- | ---: | --- |
| `deferred` | 17 | in W050; not part of the 517 in-scope rows |
| `structural_bug_open` | 87 | open BUG-FUNC stream with structural severity (kind/error/shape/array-lift) |
| `numeric_drift_open` | 77 | open BUG-FUNC stream with numeric drift severity (1 or >1 ULP) |
| `mixed_or_open` | 28 | genuine non-match rows in the latest run, no linked stream — needs triage |
| `harness_blocked` | 0 | latest run only harness-blocked / generator-invalid — needs a better probe, not a function bug |
| `harness_pending` | 18 | poked, but needs a richer/different harness to judge honestly (reference-identity/host context, or statistical RAND harness) — not a bug |
| `excluded` | 20 | deliberately not value-comparable and not planned for a harness (volatile clock / host / locale / callable) |
| `bit_exact_observed` | 287 | covered by ≥1 array_rollup run and never produced a non-match row |
| `scalar_swept_only` | 0 | swept only by the broad-scalar numeric runner; structural axes still unswept |
| `unswept` | 0 | never observed in any run and no open stream targets it |
| **total** | 534 | published snapshot rows |

## Caveats

1. `bit_exact_observed` is **not** a closure claim. It only means that across the sampled invocation rows in array_rollup runs, no non-match was seen. Unswept axes (locale, alternate version, broader array, edge values not in the manifest seeds) remain unexplored on those surfaces.
2. The stream→surface mapping is curated from the open BUG-FUNC stream titles and W097 records. Broad streams like BUG-FUNC-021 (statistical) and BUG-FUNC-027 (broad scalar) list explicit witness families; non-listed surfaces that share a kernel family may still be affected.
3. `scalar_swept_only` means the broad-scalar numeric runner exercised the surface but the structural axes (array / error / blank / coercion via the array-tranche runner) have not. It is coverage, not closure. Broad-scalar non-match findings flow into the bug-stream column (BUG-FUNC-027) rather than the per-surface status.
4. The status uses the CHARTER §4.1 severity vocabulary. A `numeric_drift_open` surface is still a bug — `excel_imprecision_witness` rows remain in the numeric-drift bug count, not outside it.

## structural_bug_open (87)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ADDRESS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 27 | 05/04/2026 |
| `ODDFPRICE` | BUG-FUNC-032(structural/open/oxfunc) | 3 | 05/28/2026 |
| `OCT2DEC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NUMBERVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NOT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NORMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `NETWORKDAYS.INTL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NETWORKDAYS` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `MULTINOMIAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ODDFYIELD` | BUG-FUNC-032(structural/open/oxfunc) | 3 | 05/28/2026 |
| `MINIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `LOG` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `LCM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISOWEEKNUM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISODD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISNONTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISLOGICAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISEVEN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `MAXIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `ISERR` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `QUOTIENT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `SQRTPI` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `YIELD` | BUG-FUNC-031(structural/open/oxfunc) | 3 | 05/28/2026 |
| `YEARFRAC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WORKDAY.INTL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WORKDAY` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WEEKNUM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WEEKDAY` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `VALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `UNICODE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ROMAN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TIMEVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLYIELD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLPRICE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLEQ` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TAKE` | BUG-FUNC-026(structural/handed_off/seam) | 34 | 05/09/2026 |
| `SWITCH` | BUG-FUNC-004(structural/validated_local/oxfunc); BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `SUMIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `SUMIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `STANDARDIZE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `OP_UNARY_PLUS` | BUG-FUNC-029(structural/open/oxfunc) | 1 | 05/28/2026 |
| `INDEX` | BUG-FUNC-003(structural/handed_off/seam) | 0 |  |
| `GESTEP` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DECIMAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DCOUNTA` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DCOUNT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DBCS` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DAVERAGE` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DATEVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `COUNTIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COUNTIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DELTA` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `COMPLEX` | BUG-FUNC-018(structural/validated_local/oxfunc) | 26 | 05/04/2026 |
| `BINOMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `BIN2OCT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `AVERAGEIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `AVERAGEIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `ASC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ARRAYTOTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `AREAS` | BUG-FUNC-003(structural/handed_off/seam) | 7 | 05/09/2026 |
| `ARABIC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `CLEAN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `IFS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `DGET` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DMIN` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `GCD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `GAMMALN.PRECISE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `FIXED` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `FACTDOUBLE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ERFC.PRECISE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ERFC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ERF.PRECISE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ERF` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DMAX` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `EOMONTH` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DVAR` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSUM` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEVP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEV` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DPRODUCT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DOLLARFR` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `DOLLARDE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DOLLAR` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DVARP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `OP_UNION_REF` | BUG-FUNC-003(structural/handed_off/seam) | 0 |  |

## numeric_drift_open (77)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ACCRINT` | BUG-FUNC-030(numeric/open/oxfunc) | 3 | 05/28/2026 |
| `PERCENTRANK.EXC` | BUG-FUNC-021(numeric/open/oxfunc) | 0 |  |
| `PERCENTRANK` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `NORMSINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `NORMSDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `NORM.S.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `NORM.S.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `NEGBINOMDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `NEGBINOM.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `MROUND` | BUG-FUNC-027(numeric/open/oxfunc) | 2 | 04/29/2026 |
| `MOD` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `MINVERSE` | BUG-FUNC-025(numeric/open/oxfunc) | 17 | 05/04/2026 |
| `KURT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `IPMT` | BUG-FUNC-015(numeric/validated_local/oxfunc) | 0 |  |
| `HYPGEOMDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `HYPGEOM.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `PERCENTRANK.INC` | BUG-FUNC-021(numeric/open/oxfunc) | 0 |  |
| `PERMUTATIONA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `PMT` | BUG-FUNC-015(numeric/validated_local/oxfunc) | 1 | 05/04/2026 |
| `POWER` | BUG-FUNC-027(numeric/open/oxfunc) | 7 | 05/09/2026 |
| `TDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `TANH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `TAN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `T.INV.2T` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.DIST.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.DIST.2T` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMALN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `T.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `SKEW` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `SINH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `SIN` | BUG-FUNC-027(numeric/open/oxfunc) | 11 | 05/04/2026 |
| `SECH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `SEC` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `RATE` | BUG-FUNC-009(numeric/validated_local/oxfunc) | 0 |  |
| `PPMT` | BUG-FUNC-015(numeric/validated_local/oxfunc) | 0 |  |
| `SKEW.P` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `TINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `GAMMAINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `GAMMA.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CHISQ.INV.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CHISQ.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CHISQ.DIST.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CHISQ.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CHIINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `CHIDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `BETAINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `BETADIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `BETA.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `BETA.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `BESSELY` | BUG-FUNC-024(numeric/open/oxfunc) | 10 | 05/04/2026 |
| `ATANH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `ATAN2` | BUG-FUNC-027(numeric/open/oxfunc) | 2 | 04/29/2026 |
| `ACOTH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `ACOSH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COMBIN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COMBINA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `CONFIDENCE` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `CONFIDENCE.NORM` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMA.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `FISHERINV` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `FINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `FDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `F.INV.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `F.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMADIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `F.DIST.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CSCH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `CSC` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COTH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COT` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COSH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COS` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `CONFIDENCE.T` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `F.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `Z.TEST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |

## mixed_or_open (28)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ACOT` |  | 1 | 05/28/2026 |
| `XNPV` |  | 3 | 05/28/2026 |
| `TRIMRANGE` |  | 1 | 05/28/2026 |
| `TREND` |  | 1 | 05/28/2026 |
| `QUARTILE.INC` |  | 1 | 05/28/2026 |
| `QUARTILE.EXC` |  | 1 | 05/28/2026 |
| `PHI` |  | 1 | 05/28/2026 |
| `PERCENTILE.INC` |  | 1 | 05/28/2026 |
| `PERCENTILE.EXC` |  | 1 | 05/28/2026 |
| `NPV` |  | 3 | 05/28/2026 |
| `NPER` |  | 3 | 05/28/2026 |
| `LOGEST` |  | 1 | 05/28/2026 |
| `LINEST` |  | 1 | 05/28/2026 |
| `JIS` |  | 1 | 05/28/2026 |
| `IRR` |  | 1 | 05/28/2026 |
| `HYPERLINK` |  | 1 | 05/28/2026 |
| `GROWTH` |  | 1 | 05/28/2026 |
| `GAUSS` |  | 1 | 05/28/2026 |
| `FTEST` |  | 3 | 05/28/2026 |
| `FORECAST.LINEAR` |  | 3 | 05/28/2026 |
| `FORECAST` |  | 3 | 05/28/2026 |
| `F.TEST` |  | 3 | 05/28/2026 |
| `CUMPRINC` |  | 3 | 05/28/2026 |
| `CONVERT` |  | 3 | 05/28/2026 |
| `CHITEST` |  | 3 | 05/28/2026 |
| `CHISQ.TEST` |  | 3 | 05/28/2026 |
| `YIELDDISC` |  | 3 | 05/28/2026 |
| `YIELDMAT` |  | 3 | 05/28/2026 |

## harness_blocked (0)

_(none)_

## harness_pending (18)

| Surface | Reason |
| --- | --- |
| `AGGREGATE` | AggregateReferenceContext host info: OxCalc integration lane |
| `OP_TRIM_REF_BOTH` | newest range-trim syntax + spill context: OxCalc integration lane |
| `OP_SPILL_REF` | spill-anchor host context: OxCalc integration lane |
| `OP_RANGE_REF` | reference-materialisation: OxCalc integration lane |
| `OP_INTERSECTION_REF` | reference-materialisation: OxCalc integration lane |
| `OP_IMPLICIT_INTERSECTION` | formula-position host context: OxCalc integration lane |
| `XLOOKUP` | reference-return: OxCalc integration conformance lane |
| `SUBTOTAL` | AggregateReferenceContext host info: OxCalc integration lane |
| `SHEETS` | workbook/sheet host context: OxCalc integration lane |
| `SHEET` | sheet-identity host context: OxCalc integration lane |
| `RANDBETWEEN` | statistical-profile harness built (v0 consistent); not bit-exact by nature |
| `RANDARRAY` | statistical-profile harness built (v0 consistent); not bit-exact by nature |
| `RAND` | statistical-profile harness built (v0 consistent); not bit-exact by nature |
| `OFFSET` | reference-return: OxCalc integration conformance lane |
| `ISFORMULA` | cell formula metadata: OxCalc integration lane |
| `FORMULATEXT` | cell formula metadata: OxCalc integration lane |
| `OP_TRIM_REF_LEADING` | newest range-trim syntax + spill context: OxCalc integration lane |
| `OP_TRIM_REF_TRAILING` | newest range-trim syntax + spill context: OxCalc integration lane |

## excluded (20)

| Surface | Reason |
| --- | --- |
| `BAHTTEXT` | locale-specific Thai baht text; deterministic but excluded per scope decision |
| `RTD` | host: real-time data provider |
| `REGISTER.ID` | host: external DLL registration |
| `REDUCE` | callable (lambda) argument |
| `PIVOTBY` | callable / pivot aggregation form |
| `NOW` | volatile clock (TimeDependent); not bit-comparable per evaluation |
| `MAP` | callable (lambda) argument |
| `MAKEARRAY` | callable (lambda) argument |
| `LET` | special binding syntax; not a flat value-vector call |
| `LAMBDA` | callable formation; needs a lambda fixture (W089 deferred) |
| `ISOMITTED` | only meaningful inside a LAMBDA |
| `INFO` | host / system environment state |
| `INDIRECT` | reference-from-text; needs reference/host resolution |
| `IMAGE` | host: image / web resource |
| `GROUPBY` | callable / pivot aggregation form |
| `CALL` | host: external DLL call |
| `BYROW` | callable (lambda) argument |
| `BYCOL` | callable (lambda) argument |
| `SCAN` | callable (lambda) argument |
| `TODAY` | volatile clock (TimeDependent) |

## unswept (0)

_(none)_

## scalar_swept_only (0)

_(none)_

## bit_exact_observed (287)

ABS, RANK.EQ, RANK.AVG, RANK, RADIANS, QUARTILE, PV, RECEIVED, PROPER, PROB, PRICEMAT, PRICEDISC, PRICE, POISSON.DIST, POISSON, PRODUCT, PI, REGEXEXTRACT, REGEXTEST, SERIESSUM, SEQUENCE, SECOND, SEARCH, SEARCHB, RSQ, RRI, REGEXREPLACE, ROWS, ROUNDUP, ROUNDDOWN, ROUND, RIGHT, RIGHTB, REPT, REPLACE, REPLACEB, ROW, SIGN, PERMUT, PERCENTILE, MODE, MMULT, MIRR, MINUTE, MINA, MIN, MODE.MULT, MID, MIDB, MDURATION, MDETERM, MAXA, MAX, MATCH, LOWER, MEDIAN, PERCENTOF, MODE.SNGL, MUNIT, PEARSON, PDURATION, OR, ODDLYIELD, ODDLPRICE, ODD, MONTH, OCT2HEX, NORMINV, NORM.INV, NORM.DIST, NOMINAL, NA, N, OCT2BIN, LOOKUP, SLN, SMALL, WRAPCOLS, WEIBULL.DIST, WEIBULL, VSTACK, VLOOKUP, VDB, WRAPROWS, VARPA, VARA, VAR.S, VAR.P, VAR, VALUETOTEXT, UPPER, VARP, UNIQUE, XIRR, XOR, OP_PERCENT, OP_NOT_EQUAL, OP_NEGATE, OP_MULTIPLY, OP_LESS_THAN, OP_LESS_EQUAL, XMATCH, OP_GREATER_THAN, OP_EQUAL, OP_DIVIDE, OP_CONCAT, OP_ADD, ZTEST, YEAR, OP_GREATER_EQUAL, SLOPE, UNICHAR, TTEST, SUMSQ, SUMPRODUCT, SUM, SUBSTITUTE, STEYX, STDEVPA, SUMX2MY2, STDEVP, STDEV.S, STDEV.P, STDEV, SQRT, SORTBY, SORT, STDEVA, TYPE, SUMX2PY2, SYD, TRUNC, TRUE, TRIMMEAN, TRIM, TRANSPOSE, TOROW, SUMXMY2, TOCOL, TEXTSPLIT, TEXTJOIN, TEXTBEFORE, TEXTAFTER, T.TEST, T, TIME, OP_POWER, LOGNORMDIST, LOGNORM.DIST, COVARIANCE.P, COVAR, COUPPCD, COUPNUM, COUPNCD, COUPDAYSNC, COVARIANCE.S, COUPDAYS, COUNTBLANK, COUNTA, COUNT, CORREL, CONCATENATE, CONCAT, COUPDAYBS, COLUMNS, CRITBINOM, DATE, DURATION, DROP, DISC, DEVSQ, DEGREES, DEC2OCT, CUMIPMT, DEC2HEX, DDB, DB, DAYS360, DAYS, DAY, DATEDIF, DEC2BIN, EDATE, COLUMN, CHOOSEROWS, BESSELJ, BESSELI, BASE, AVERAGEA, AVERAGE, AVEDEV, BESSELK, ATAN, ASIN, AND, AMORLINC, AMORDEGRC, ACOS, ACCRINTM, ASINH, CODE, BIN2DEC, BINOM.DIST, CHOOSECOLS, CHOOSE, CHAR, CELL, CEILING.PRECISE, CEILING.MATH, BIN2HEX, CEILING, BITRSHIFT, BITOR, BITLSHIFT, BITAND, BINOM.INV, BINOM.DIST.RANGE, BITXOR, LOGNORM.INV, EFFECT, EVEN, IMSUB, IMSQRT, IMSINH, IMSIN, IMSECH, IMSEC, IMSUM, IMREAL, IMPOWER, IMLOG2, IMLOG10, IMLN, IMEXP, IMDIV, IMPRODUCT, IMCSCH, IMTAN, INTERCEPT, LOGINV, LOG10, LN, LEN, LENB, LEFT, LEFTB, LARGE, INT, ISREF, ISO.CEILING, ISNUMBER, ISNA, ISERROR, ISBLANK, INTRATE, ISPMT, ERROR.TYPE, IMCSC, IMCOSH, FREQUENCY, FLOOR.PRECISE, FLOOR.MATH, FLOOR, FISHER, FIND, FINDB, FV, FILTER, FACT, EXPONDIST, EXPON.DIST, EXPAND, EXP, EXACT, FALSE, IMCOT, FVSCHEDULE, HARMEAN, IMCOS, IMCONJUGATE, IMARGUMENT, IMAGINARY, IMABS, IFNA, GEOMEAN, IFERROR, HSTACK, HOUR, HLOOKUP, HEX2OCT, HEX2DEC, HEX2BIN, IF, OP_SUBTRACT

## deferred (17)

COPILOT, STOCKHISTORY, PHONETIC, GETPIVOTDATA, FILTERXML, EUROCONVERT, ENCODEURL, TRANSLATE, DETECTLANGUAGE, CUBESETCOUNT, CUBESET, CUBERANKEDMEMBER, CUBEMEMBERPROPERTY, CUBEMEMBER, CUBEKPIMEMBER, CUBEVALUE, WEBSERVICE


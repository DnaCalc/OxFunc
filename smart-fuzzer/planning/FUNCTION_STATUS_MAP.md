# OxFunc Function Status Map

Generated: 2026-05-28T17:50:05.6063254Z

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
| `structural_bug_open` | 79 | open BUG-FUNC stream with structural severity (kind/error/shape/array-lift) |
| `numeric_drift_open` | 76 | open BUG-FUNC stream with numeric drift severity (1 or >1 ULP) |
| `mixed_or_open` | 22 | genuine non-match rows in the latest run, no linked stream — needs triage |
| `harness_blocked` | 76 | latest run only harness-blocked / generator-invalid — needs a better probe, not a function bug |
| `bit_exact_observed` | 227 | covered by ≥1 array_rollup run and never produced a non-match row |
| `scalar_swept_only` | 5 | swept only by the broad-scalar numeric runner; structural axes still unswept |
| `unswept` | 32 | never observed in any run and no open stream targets it |
| **total** | 534 | published snapshot rows |

## Caveats

1. `bit_exact_observed` is **not** a closure claim. It only means that across the sampled invocation rows in array_rollup runs, no non-match was seen. Unswept axes (locale, alternate version, broader array, edge values not in the manifest seeds) remain unexplored on those surfaces.
2. The stream→surface mapping is curated from the open BUG-FUNC stream titles and W097 records. Broad streams like BUG-FUNC-021 (statistical) and BUG-FUNC-027 (broad scalar) list explicit witness families; non-listed surfaces that share a kernel family may still be affected.
3. `scalar_swept_only` means the broad-scalar numeric runner exercised the surface but the structural axes (array / error / blank / coercion via the array-tranche runner) have not. It is coverage, not closure. Broad-scalar non-match findings flow into the bug-stream column (BUG-FUNC-027) rather than the per-surface status.
4. The status uses the CHARTER §4.1 severity vocabulary. A `numeric_drift_open` surface is still a bug — `excel_imprecision_witness` rows remain in the numeric-drift bug count, not outside it.

## structural_bug_open (79)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ADDRESS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 27 | 05/04/2026 |
| `OCT2DEC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NUMBERVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NOT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NORMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `NETWORKDAYS.INTL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `NETWORKDAYS` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `MULTINOMIAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `QUOTIENT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `MINIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `LOG` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `LCM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISOWEEKNUM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISODD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISNONTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISLOGICAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `MAXIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `ISEVEN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ROMAN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `STANDARDIZE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `YEARFRAC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WORKDAY.INTL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WORKDAY` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WEEKNUM` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `WEEKDAY` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `VALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `UNICODE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `SQRTPI` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TIMEVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLYIELD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLPRICE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TBILLEQ` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `TAKE` | BUG-FUNC-026(structural/handed_off/seam) | 34 | 05/09/2026 |
| `SWITCH` | BUG-FUNC-004(structural/validated_local/oxfunc); BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `SUMIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `SUMIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `TEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `OP_UNARY_PLUS` | BUG-FUNC-029(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ISERR` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `IFS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `DCOUNT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DBCS` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DAVERAGE` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DATEVALUE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `COUNTIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COUNTIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COMPLEX` | BUG-FUNC-018(structural/validated_local/oxfunc) | 26 | 05/04/2026 |
| `DCOUNTA` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `CLEAN` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `BIN2OCT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `AVERAGEIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `AVERAGEIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `ASC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `ARRAYTOTEXT` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `AREAS` | BUG-FUNC-003(structural/handed_off/seam) | 7 | 05/09/2026 |
| `ARABIC` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `BINOMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `INDEX` | BUG-FUNC-003(structural/handed_off/seam) | 0 |  |
| `DECIMAL` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DGET` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `GESTEP` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `GCD` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `FIXED` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `FACTDOUBLE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `EOMONTH` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DVARP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DVAR` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DELTA` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DSUM` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEV` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DPRODUCT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DOLLARFR` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `DOLLARDE` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DOLLAR` | BUG-FUNC-028(structural/open/oxfunc) | 1 | 05/28/2026 |
| `DMIN` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DMAX` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEVP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `OP_UNION_REF` | BUG-FUNC-003(structural/handed_off/seam) | 0 |  |

## numeric_drift_open (76)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ACOSH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `PERCENTRANK.INC` | BUG-FUNC-021(numeric/open/oxfunc) | 0 |  |
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
| `PERMUTATIONA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `HYPGEOM.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `PMT` | BUG-FUNC-015(numeric/validated_local/oxfunc) | 1 | 05/04/2026 |
| `PPMT` | BUG-FUNC-015(numeric/validated_local/oxfunc) | 0 |  |
| `TDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `TANH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `TAN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `T.INV.2T` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.DIST.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.DIST.2T` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `T.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `SKEW.P` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `SKEW` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `SINH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `SIN` | BUG-FUNC-027(numeric/open/oxfunc) | 11 | 05/04/2026 |
| `SECH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `SEC` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `RATE` | BUG-FUNC-009(numeric/validated_local/oxfunc) | 0 |  |
| `POWER` | BUG-FUNC-027(numeric/open/oxfunc) | 7 | 05/09/2026 |
| `GAMMALN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `GAMMAINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `GAMMADIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `COMBIN` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
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
| `COMBINA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `CONFIDENCE` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `CONFIDENCE.NORM` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CONFIDENCE.T` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `GAMMA.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMA.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `GAMMA` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `FISHERINV` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `FINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `FDIST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `F.INV.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `TINV` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |
| `F.INV` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `F.DIST` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `CSCH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `CSC` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COTH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COT` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COSH` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `COS` | BUG-FUNC-027(numeric/open/oxfunc) | 0 |  |
| `F.DIST.RT` | BUG-FUNC-021(numeric/open/oxfunc) | 9 | 05/04/2026 |
| `Z.TEST` | BUG-FUNC-021(numeric/open/oxfunc) | 24 | 05/04/2026 |

## mixed_or_open (22)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ACOT` |  | 1 | 05/28/2026 |
| `RANDARRAY` |  | 1 | 05/28/2026 |
| `QUARTILE.INC` |  | 1 | 05/28/2026 |
| `QUARTILE.EXC` |  | 1 | 05/28/2026 |
| `PHI` |  | 1 | 05/28/2026 |
| `PERCENTILE.INC` |  | 1 | 05/28/2026 |
| `PERCENTILE.EXC` |  | 1 | 05/28/2026 |
| `MAP` |  | 1 | 05/28/2026 |
| `MAKEARRAY` |  | 1 | 05/28/2026 |
| `LOGEST` |  | 1 | 05/28/2026 |
| `LINEST` |  | 1 | 05/28/2026 |
| `LAMBDA` |  | 1 | 05/28/2026 |
| `JIS` |  | 1 | 05/28/2026 |
| `IRR` |  | 1 | 05/28/2026 |
| `HYPERLINK` |  | 1 | 05/28/2026 |
| `GROWTH` |  | 1 | 05/28/2026 |
| `GROUPBY` |  | 1 | 05/28/2026 |
| `GAUSS` |  | 1 | 05/28/2026 |
| `BYROW` |  | 1 | 05/28/2026 |
| `BYCOL` |  | 1 | 05/28/2026 |
| `TREND` |  | 1 | 05/28/2026 |
| `TRIMRANGE` |  | 1 | 05/28/2026 |

## harness_blocked (76)

ACCRINT, RANK.EQ, RANK.AVG, RANK, PV, PROB, PRICEMAT, PRICEDISC, PRICE, PIVOTBY, PERCENTOF, PDURATION, ODDLYIELD, ODDLPRICE, ODDFYIELD, ODDFPRICE, RECEIVED, NPV, REGEXEXTRACT, REGEXTEST, YIELDDISC, YIELD, XNPV, XIRR, WEIBULL.DIST, WEIBULL, TTEST, TEXTSPLIT, TEXTBEFORE, TEXTAFTER, T.TEST, SEARCH, SEARCHB, RRI, RIGHT, RIGHTB, REPLACE, REPLACEB, REGEXREPLACE, NPER, NOMINAL, MIRR, DATEDIF, CUMPRINC, CUMIPMT, COUPPCD, COUPNUM, COUPNCD, COUPDAYSNC, COUPDAYS, COUPDAYBS, CONVERT, CHITEST, CHISQ.TEST, AMORLINC, AMORDEGRC, ACCRINTM, DAYS360, DISC, DURATION, EFFECT, MID, MIDB, MDURATION, LOOKUP, LET, LEN, LENB, LEFT, LEFTB, ISPMT, YIELDMAT, INTRATE, FV, FTEST, FREQUENCY, FORECAST.LINEAR, FORECAST, FIND, FINDB, F.TEST, FVSCHEDULE, ZTEST

## unswept (32)

AGGREGATE, OP_TRIM_REF_BOTH, OP_SPILL_REF, OP_RANGE_REF, OP_INTERSECTION_REF, OP_IMPLICIT_INTERSECTION, XLOOKUP, VLOOKUP, TODAY, SUBTOTAL, SHEETS, SHEET, RTD, REGISTER.ID, RANDBETWEEN, RAND, OFFSET, NOW, ISFORMULA, INFO, INDIRECT, IMAGE, IFNA, HLOOKUP, FORMULATEXT, COLUMN, CHOOSE, CELL, CALL, BAHTTEXT, OP_TRIM_REF_LEADING, OP_TRIM_REF_TRAILING

## scalar_swept_only (5)

ERF, ERF.PRECISE, ERFC, ERFC.PRECISE, GAMMALN.PRECISE

## bit_exact_observed (227)

ABS, PI, POISSON, POISSON.DIST, PRODUCT, PROPER, QUARTILE, RADIANS, REDUCE, REPT, ROUND, ROUNDDOWN, PERMUT, ROUNDUP, ROWS, RSQ, SCAN, SECOND, SEQUENCE, SERIESSUM, SIGN, SLN, SLOPE, SMALL, SORT, ROW, PERCENTILE, PEARSON, OR, LOGNORM.INV, LOGNORMDIST, LOWER, MATCH, MAX, MAXA, MDETERM, MEDIAN, MIN, MINA, MINUTE, MMULT, MODE, MODE.MULT, MODE.SNGL, MONTH, MUNIT, N, NA, NORM.DIST, NORM.INV, NORMINV, OCT2BIN, OCT2HEX, ODD, SORTBY, LOGNORM.DIST, SQRT, STDEV.P, VAR.P, VAR.S, VARA, VARP, VARPA, VDB, VSTACK, WRAPCOLS, WRAPROWS, XMATCH, XOR, VAR, YEAR, OP_CONCAT, OP_DIVIDE, OP_EQUAL, OP_GREATER_EQUAL, OP_GREATER_THAN, OP_LESS_EQUAL, OP_LESS_THAN, OP_MULTIPLY, OP_NEGATE, OP_NOT_EQUAL, OP_PERCENT, OP_ADD, VALUETOTEXT, UPPER, UNIQUE, STDEV.S, STDEVA, STDEVP, STDEVPA, STEYX, SUBSTITUTE, SUM, SUMPRODUCT, SUMSQ, SUMX2MY2, SUMX2PY2, SUMXMY2, SYD, T, TEXTJOIN, TIME, TOCOL, TOROW, TRANSPOSE, TRIM, TRIMMEAN, TRUE, TRUNC, TYPE, UNICHAR, STDEV, OP_POWER, LOGINV, LN, COLUMNS, CONCAT, CONCATENATE, CORREL, COUNT, COUNTA, COUNTBLANK, COVAR, COVARIANCE.P, COVARIANCE.S, CRITBINOM, CODE, DATE, DAYS, DB, DDB, DEC2BIN, DEC2HEX, DEC2OCT, DEGREES, DEVSQ, DROP, EDATE, ERROR.TYPE, DAY, CHOOSEROWS, CHOOSECOLS, CHAR, ACOS, AND, ASIN, ASINH, ATAN, AVEDEV, AVERAGE, AVERAGEA, BASE, BESSELI, BESSELJ, BESSELK, BIN2DEC, BIN2HEX, BINOM.DIST, BINOM.DIST.RANGE, BINOM.INV, BITAND, BITLSHIFT, BITOR, BITRSHIFT, BITXOR, CEILING, CEILING.MATH, CEILING.PRECISE, EVEN, LOG10, EXACT, EXPAND, IMLN, IMLOG10, IMLOG2, IMPOWER, IMPRODUCT, IMREAL, IMSEC, IMSECH, IMSIN, IMSINH, IMSQRT, IMEXP, IMSUB, IMTAN, INT, INTERCEPT, ISBLANK, ISERROR, ISNA, ISNUMBER, ISO.CEILING, ISOMITTED, ISREF, LARGE, IMSUM, IMDIV, IMCSCH, IMCSC, EXPON.DIST, EXPONDIST, FACT, FALSE, FILTER, FISHER, FLOOR, FLOOR.MATH, FLOOR.PRECISE, GEOMEAN, HARMEAN, HEX2BIN, HEX2DEC, HEX2OCT, HOUR, HSTACK, IF, IFERROR, IMABS, IMAGINARY, IMARGUMENT, IMCONJUGATE, IMCOS, IMCOSH, IMCOT, EXP, OP_SUBTRACT

## deferred (17)

COPILOT, STOCKHISTORY, PHONETIC, GETPIVOTDATA, FILTERXML, EUROCONVERT, ENCODEURL, TRANSLATE, DETECTLANGUAGE, CUBESETCOUNT, CUBESET, CUBERANKEDMEMBER, CUBEMEMBERPROPERTY, CUBEMEMBER, CUBEKPIMEMBER, CUBEVALUE, WEBSERVICE


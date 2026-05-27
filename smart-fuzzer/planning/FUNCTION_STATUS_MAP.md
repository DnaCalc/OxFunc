# OxFunc Function Status Map

Generated: 2026-05-27T23:39:06.8421932Z

This map is the reproducible derived view of where each of the published OxFunc surfaces stands against the bit-exact Excel parity goal (CHARTER.md §4.1). Rebuild with `smart-fuzzer/tools/Build-FunctionStatusMap.ps1`.

Inputs joined:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`
3. `docs/bugs/BUG_STREAM_REGISTER.csv` plus the curated stream→surface map in this script
4. `smart-fuzzer/runs/*/rollup.json` (array_rollup schema)

## Tally

| Status | Count | Meaning |
| --- | ---: | --- |
| `deferred` | 17 | in W050; not part of the 517 in-scope rows |
| `structural_bug_open` | 31 | open BUG-FUNC stream with structural severity (kind/error/shape/array-lift) |
| `numeric_drift_open` | 76 | open BUG-FUNC stream with numeric drift severity (1 or >1 ULP) |
| `mixed_or_open` | 0 | covered by runs and produced non-match rows but no linked stream |
| `bit_exact_observed` | 146 | covered by ≥1 array_rollup run and never produced a non-match row |
| `unswept` | 264 | never observed in an array_rollup run and no open stream targets it |
| **total** | 534 | published snapshot rows |

## Caveats

1. `bit_exact_observed` is **not** a closure claim. It only means that across the sampled invocation rows in array_rollup runs, no non-match was seen. Unswept axes (locale, alternate version, broader array, edge values not in the manifest seeds) remain unexplored on those surfaces.
2. The stream→surface mapping is curated from the open BUG-FUNC stream titles and W097 records. Broad streams like BUG-FUNC-021 (statistical) and BUG-FUNC-027 (broad scalar) list explicit witness families; non-listed surfaces that share a kernel family may still be affected.
3. Run-BroadScalarExploration cycles (broad_scalar_run_rollup schema) are not summarized here — per-function detail lives in their comparisons.jsonl and failure_packets/. Their findings flow into the bug-stream column instead.
4. The status uses the CHARTER §4.1 severity vocabulary. A `numeric_drift_open` surface is still a bug — `excel_imprecision_witness` rows remain in the numeric-drift bug count, not outside it.

## structural_bug_open (31)

| Surface | Streams | Runs seen | Last seen |
| --- | --- | ---: | --- |
| `ADDRESS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 27 | 05/04/2026 |
| `SWITCH` | BUG-FUNC-004(structural/validated_local/oxfunc); BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `SUMIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `SUMIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `NORMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `MINIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `MAXIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `INDEX` | BUG-FUNC-003(structural/handed_off/seam) | 0 |  |
| `IFS` | BUG-FUNC-018(structural/validated_local/oxfunc) | 29 | 05/04/2026 |
| `DVARP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DVAR` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSUM` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEVP` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DSTDEV` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `TAKE` | BUG-FUNC-026(structural/handed_off/seam) | 34 | 05/09/2026 |
| `DPRODUCT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DMIN` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DMAX` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DGET` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DCOUNTA` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DCOUNT` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `DAVERAGE` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COUNTIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COUNTIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `COMPLEX` | BUG-FUNC-018(structural/validated_local/oxfunc) | 26 | 05/04/2026 |
| `BINOMDIST` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
| `AVERAGEIFS` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `AVERAGEIF` | BUG-FUNC-004(structural/validated_local/oxfunc) | 0 |  |
| `AREAS` | BUG-FUNC-003(structural/handed_off/seam) | 7 | 05/09/2026 |
| `DOLLARFR` | BUG-FUNC-018(structural/validated_local/oxfunc) | 24 | 05/04/2026 |
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

## mixed_or_open (0)

_(none)_

## unswept (264)

ACCRINT, RAND, RANDARRAY, RANDBETWEEN, RANK, RANK.AVG, RANK.EQ, RECEIVED, REDUCE, REGEXEXTRACT, REGEXREPLACE, REGEXTEST, REGISTER.ID, REPLACE, REPLACEB, RIGHT, RIGHTB, ROMAN, RRI, RSQ, RTD, SCAN, SEARCH, SEARCHB, SHEET, SHEETS, SLOPE, SMALL, SQRT, SQRTPI, STANDARDIZE, STDEV, STDEV.P, RADIANS, STDEV.S, QUOTIENT, QUARTILE.EXC, NETWORKDAYS.INTL, NOMINAL, NOT, NOW, NPER, NPV, NUMBERVALUE, OCT2DEC, ODD, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, OFFSET, OR, PDURATION, PEARSON, PERCENTILE.EXC, PERCENTILE.INC, PERCENTOF, PERMUT, PHI, PIVOTBY, PRICE, PRICEDISC, PRICEMAT, PROB, PRODUCT, PV, QUARTILE.INC, NETWORKDAYS, STDEVA, STDEVPA, XIRR, XLOOKUP, XNPV, XOR, YEARFRAC, YIELD, YIELDDISC, YIELDMAT, ZTEST, OP_ADD, OP_CONCAT, OP_DIVIDE, OP_EQUAL, OP_GREATER_EQUAL, OP_GREATER_THAN, OP_IMPLICIT_INTERSECTION, OP_INTERSECTION_REF, OP_LESS_EQUAL, OP_LESS_THAN, OP_MULTIPLY, OP_NEGATE, OP_NOT_EQUAL, OP_PERCENT, OP_POWER, OP_RANGE_REF, OP_SPILL_REF, OP_SUBTRACT, OP_TRIM_REF_BOTH, OP_TRIM_REF_LEADING, WORKDAY.INTL, STDEVP, WORKDAY, WEIBULL, SUBTOTAL, SUMSQ, T, T.TEST, TBILLEQ, TBILLPRICE, TBILLYIELD, TEXT, TEXTAFTER, TEXTBEFORE, TEXTSPLIT, TIMEVALUE, TODAY, TREND, TRIMRANGE, TRUE, TTEST, UNICODE, VALUE, VALUETOTEXT, VAR, VAR.P, VAR.S, VARA, VARP, VARPA, VLOOKUP, WEEKDAY, WEEKNUM, WEIBULL.DIST, MULTINOMIAL, MODE.SNGL, MODE.MULT, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, COVARIANCE.P, COVARIANCE.S, CUMIPMT, CUMPRINC, DATEDIF, DATEVALUE, DAYS360, DBCS, DECIMAL, DEGREES, DELTA, DEVSQ, DISC, DOLLAR, DOLLARDE, DURATION, EFFECT, EOMONTH, ERF, ERF.PRECISE, ERFC, ERFC.PRECISE, EVEN, EXP, COUPDAYBS, F.TEST, CORREL, CONCAT, ACCRINTM, ACOS, ACOT, AGGREGATE, AMORDEGRC, AMORLINC, ARABIC, ARRAYTOTEXT, ASC, ASINH, ATAN, AVEDEV, AVERAGEA, BAHTTEXT, BIN2OCT, BITAND, BITLSHIFT, BITOR, BITRSHIFT, BITXOR, BYCOL, BYROW, CALL, CELL, CHISQ.TEST, CHITEST, CHOOSE, CLEAN, COLUMN, CONVERT, FACT, FACTDOUBLE, FALSE, ISODD, ISOMITTED, ISOWEEKNUM, ISPMT, ISREF, ISTEXT, JIS, LAMBDA, LARGE, LCM, LEFT, LEFTB, LEN, LENB, LET, LINEST, LN, LOG, LOG10, LOGEST, LOOKUP, MAKEARRAY, MAP, MAX, MAXA, MDURATION, MEDIAN, MID, MIDB, MIN, MINA, MIRR, ISNONTEXT, ISNA, ISLOGICAL, ISFORMULA, FIND, FINDB, FISHER, FIXED, FORECAST, FORECAST.LINEAR, FORMULATEXT, FREQUENCY, FTEST, FV, FVSCHEDULE, GAMMALN.PRECISE, GAUSS, GCD, GEOMEAN, OP_TRIM_REF_TRAILING, GESTEP, GROWTH, HARMEAN, HLOOKUP, HYPERLINK, IFNA, IMAGE, INDIRECT, INFO, INT, INTERCEPT, INTRATE, IRR, ISERR, ISEVEN, GROUPBY, OP_UNARY_PLUS

## bit_exact_observed (146)

ABS, MUNIT, N, NA, NORM.DIST, NORM.INV, NORMINV, MONTH, OCT2BIN, PERCENTILE, PI, POISSON, POISSON.DIST, PROPER, QUARTILE, OCT2HEX, REPT, MODE, MINUTE, IMSUB, IMSUM, IMTAN, ISBLANK, ISERROR, ISNUMBER, MMULT, ISO.CEILING, LOGNORM.DIST, LOGNORM.INV, LOGNORMDIST, LOWER, MATCH, MDETERM, LOGINV, ROUND, ROUNDDOWN, ROUNDUP, TOCOL, TOROW, TRANSPOSE, TRIM, TRIMMEAN, TRUNC, TIME, TYPE, UNIQUE, UPPER, VDB, VSTACK, WRAPCOLS, WRAPROWS, UNICHAR, TEXTJOIN, SYD, SUMXMY2, ROW, ROWS, SECOND, SEQUENCE, SERIESSUM, SIGN, SLN, SORT, SORTBY, STEYX, SUBSTITUTE, SUM, SUMPRODUCT, SUMX2MY2, SUMX2PY2, IMSQRT, IMSINH, IMSIN, IMSECH, COLUMNS, CONCATENATE, COUNT, COUNTA, COUNTBLANK, COVAR, CODE, CRITBINOM, DAY, DAYS, DB, DDB, DEC2BIN, DEC2HEX, DATE, CHOOSEROWS, CHOOSECOLS, CHAR, AND, ASIN, AVERAGE, BASE, BESSELI, BESSELJ, BESSELK, BIN2DEC, BIN2HEX, BINOM.DIST, BINOM.DIST.RANGE, BINOM.INV, CEILING, CEILING.MATH, CEILING.PRECISE, DEC2OCT, XMATCH, DROP, ERROR.TYPE, IMCOS, IMCOSH, IMCOT, IMCSC, IMCSCH, IMDIV, IMCONJUGATE, IMEXP, IMLOG10, IMLOG2, IMPOWER, IMPRODUCT, IMREAL, IMSEC, IMLN, IMARGUMENT, IMAGINARY, IMABS, EXACT, EXPAND, EXPON.DIST, EXPONDIST, FILTER, FLOOR, FLOOR.MATH, FLOOR.PRECISE, HEX2BIN, HEX2DEC, HEX2OCT, HOUR, HSTACK, IF, IFERROR, EDATE, YEAR

## deferred (17)

COPILOT, STOCKHISTORY, PHONETIC, GETPIVOTDATA, FILTERXML, EUROCONVERT, ENCODEURL, TRANSLATE, DETECTLANGUAGE, CUBESETCOUNT, CUBESET, CUBERANKEDMEMBER, CUBEMEMBERPROPERTY, CUBEMEMBER, CUBEKPIMEMBER, CUBEVALUE, WEBSERVICE


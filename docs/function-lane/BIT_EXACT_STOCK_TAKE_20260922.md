# OxFunc bit-exact stock take (2026-09-22)

Collation only. No new oracle runs. Sources: `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`
(canonical, reconciled 2026-09-15), `docs/function-lane/W109_*` resume docs and
`W109_WALL_CLUES_LEDGER.md` (live evidence to 2026-09-12, build 20326),
`smart-fuzzer/planning/FUNCTION_STATUS_MAP.md` (2026-07-02 snapshot),
`smart-fuzzer/runs/*/rollup.json`, code comments in `crates/oxfunc_core/src`,
commit log since 2026-06-01, and the Handbook evidence records
(`../ExcelFunctionsHandbook/content/evidence/records`, basis `473efa3`, 2026-07-25).

Comparison policy throughout is `exact_typed_bit_match_no_tolerance`.

## Reclassification note (2026-09-22, same day)

The A/B/C/D scale below was reviewed and rejected as mixing two axes. The rule now
(ODR-FN-005): confidence measures evidence for exactness; any known differing row
means the function is `KnownDivergent`, confidence zero. Read the tables with that
correction:

- Section A splits into `SweptMatch` (held-out fresh corpus + kernel story) and
  `SuiteMatch` (100% on the fitted suite only). Rows with any noted miss, even one
  (WEIBULL.DIST 5,999/6,000, GAMMALN 1709/1711, ACOS(0.5), TANH 8/11, ODDFPRICE on
  actual bases, MINVERSE(5) shape), move to `KnownDivergent{LastBit}` or
  `{Structural}`.
- Sections B and C are all `KnownDivergent`; the difference between them is
  severity, which orders the campaign but does not soften the verdict.
- Section D is `Unverified`, except CUBE*, WEBSERVICE, STOCKHISTORY (`Deferred`).
  Everything else listed as "excluded" (NOW, TODAY, INDIRECT, LAMBDA family,
  RAND family, host seams) is in scope and needs an oracle route.

The original scale is kept below so the evidence citations stay readable.

## Confidence scale (superseded, see note above)

- **A. High**: held-out or large fresh corpus at 100% (or one wall-1 microdetail miss),
  catalog row closed/signed off, kernel identified.
- **B. Medium / partial**: kernel structurally identified, exact on a stated sub-domain,
  known residual band elsewhere.
- **C. Low / known non-match**: open catalog row with a measured shortfall, or kernel unknown.
- **D. Unknown**: no oracle evidence either way (harness-blocked, excluded, deferred,
  or only a single witness).

## A. High confidence

| Function(s) | Evidence | Note |
|---|---|---|
| EXP, LN, LOG10, LOG(x,base) | x87 CRT chain, 249/249 harvested witnesses incl. 36 near-midpoint; POISSON-derived internal exp 34,000/34,000 held-out; live `EXP(-z*z)` 1065/1065 | x86_64 only; non-x86 fallback is ~0.5 ULP glibc port |
| POWER, `^` | 715/715 live; b24 11,045/11,045; pow chain re-race 33,145/33,145 | Operator side has no separate count |
| SIN, COS, TAN, SEC, CSC, COT | 1020/1020 held-out (fFSIN); trig reduction G4-01 5425/5425; COS 2561/2561 | |
| ACOS, ACOT, COT, SEC, CSC | 34/34, 28/28, 10/10, 8/8, 8/8 (2026-09-12 live) | ACOS(0.5) libm 1 ULP noted 2026-09-12; small corpora |
| ATANH | 20,780/20,780, `closed_signed_off` | |
| ACOTH | 268,769/268,769 + held-out 66,552/66,552, `closed_signed_off` | |
| COSH, SINH, SECH, CSCH, COTH | 40/40, 37/37, 40/40 + 5/5, 5/5, 28/28 | TANH only 8/11 (B) |
| SQRTPI, MOD | SQRTPI = pow(n·π, 0.5) landed; MOD 11/11 + 8/8 after exact IEEE remainder | |
| log1p substrate | CR, 0/1350 confound-free divergences | |
| XNPV | 1530/1530 + 175/175 error rows | |
| NPER | 1286/1286 + 7/7 | |
| FV, PV | 149/149 (incl. adversarial), 48/48; kernel 96/96 | |
| YIELDMAT | 1250/1250 | |
| TBILLYIELD | 2156/2156 | Same matrix the repair was fitted on |
| YIELDDISC | three-way replay all bit-exact | Small corpus |
| ACCRINT | 146,850/146,850 (catalog) | Handbook's older 25,407/25,410 is stale |
| EFFECT, RRI, NOMINAL | 160/160, 5536/5536, identified | |
| PDURATION | 80/80 split-LN form | |
| ODDFPRICE (30/360 basis) | 10/10 + 5/5 | Actual bases not yet exact |
| CONVERT | 10,418/10,418 + 34,189/34,189 | |
| COMBIN, COMBINA | 22,242/22,242; 40,330/40,330 (+64,767 replay) | |
| PERMUT | 702/702 | Held-out split not published |
| FACT (≤170), FACTDOUBLE | reverse native product landed | |
| MINVERSE (numeric) | 1599/1599 across three corpora, Doolittle LU | `MINVERSE(5)` / `MMULT(5,2)` publication shape is Cat-1 open |
| WEIBULL.DIST, EXPON.DIST | 5,999/6,000 held-out; 4,000/4,000 held-out | one wall-1 microdetail miss |
| POISSON k=0 | 34,000+ consecutive exact; b26 4000/4000 | k=1 / small λ is C |
| PHI | 764/764 | |
| FORECAST, FORECAST.LINEAR, SLOPE, INTERCEPT | 65/65; 4/4; 4/4 | |
| BESSELI, BESSELK (x≥8), BESSELJ composition | 18/18; 794/794 | BESSELY/J order≥1 x≥8 is C |
| T.DIST df=1 PDF, df=2 PDF; T.INV.2T df=1 | 20/20; 15/15; 9/9 | CDFs are C |
| GAMMA exact slices | integers 1..88; half-integers through 171.5 (172/172); tiny-x ≤1e-16; landed rational-residue product-first and peel chains (quarters, eighths, sixteenths, fifths, sevenths, ninths, tenths, elevenths, twelfths, thirteenths, seventeenths, selected negatives) | Everything outside the landed slices is B/C |
| GAMMALN x≥8, [4,8) | 1709/1711 (two known 1-ULP rows); [4,8) worst 1 ULP | B1/B2 are C |
| Operators, broadcast, logical, text, lookup, reduction families | 288 `bit_exact_observed` surfaces (July map); operator broadcast 21-case sweep closed; REGEX* 40/40; array-lift 34/34 | `bit_exact_observed` is not a closure claim; snapshot is 2026-07-02 |
| Blank-cell scalar coercion (W110-1) | 32/32 truth table, `validated_local` | Not yet checked against live Excel |

## B. Medium / partial (identified, exact on a sub-domain)

| Function(s) | Best current rate | Where it stops |
|---|---|---|
| PMT | live 10,801/13,752; combsweep 100%; held-out family ceiling ~57% | expm1 `|tau|<1` double-rounding wall; expm1 substrate itself 17,996/18,000 (99.978%), PMT-adversarial ceiling 165/234 |
| IPMT, PPMT, CUMIPMT | inherit PMT; PMT = IPMT+PPMT 10/10; CUMIPMT vs sum 1/3 | same wall + timing-1 publication |
| NORMSDIST, NORM.S.DIST, NORM.DIST CDF, GAUSS | wrapper 64/64; GAUSS 14/14; `NORMSDIST(-1)` 1 ULP | body inherits open ERFC |
| ERF, ERF.PRECISE (|z|<0.5) | 774/1033 composition; production 405/1033 max 3 ULP | small-z body unknown ("relative-grain" comb) |
| ERFC, ERFC.PRECISE | positive (0,0.5] complement slice 1033/1033 landed; mid best 2389/7741 (+3414 at 1 ULP); tail 1324 | F-body: "constraint, not an identity; do not land"; all Lentz cubes done, campaign exited |
| GAMMA (general args) | patchwork above; n≥89 integers 4/82 (max 8 ULP); general peel 93/128 | product-rounding wall; extended internal lgamma unknown |
| GAMMALN B2 [1.5,4) | 549/1200 fresh; live 95/256 max 51 | coefficients + ≥2 internal roundings |
| BINOM.DIST | k=n 40/40 and k=0 slices landed; general-k 37.9%, overall 49.59%; perfect n≤15 | degrades n>35 |
| GAMMA.DIST CDF / CHIDIST / CHISQ.DIST CDF | 337/446; 152/195; b26 1,615/4,100 | bgrat tail body; ln-amplification a≥3 |
| BETA.DIST (BRATIO) | 293/671 held-out; spec-identity 20,008/20,008 | bgrat op-graph wall 4 |
| PRICE (Actual/360, Actual/365) | 564/600 production, leader 571/600, all residuals exactly −1 ULP | private coupon-sum accumulator |
| DURATION, MDURATION | 237/264 (max 3 ULP) | accumulator graph |
| YIELD | forward kernel 15/15 + 10/10; 19-ULP end-to-end witness | solver plateau publication |
| NPV (worksheet) | 636/900 (max 4 ULP) | no exact candidate |
| TREND, LINEST, LOGEST | LOGEST 270/358; TREND 1/12 shipped, 7/12 candidate | coefficient graph |
| GROWTH | 666/1240 | kernel unknown |
| F.TEST, FTEST | 33/48 | variance schedule + tail |
| CHISQ.TEST, CHITEST | df=1 route 154/154 | rest inherits GRATIO + ERFC |
| TANH | honest 8/11 | as SINH/COSH |
| POISSON k≥2 | exact at λ ≳ 14 (Loader dpois) | small λ open |
| XIRR | solver on `xnpv_kernel_raw`; large-root precision inventory open | |
| ODDFPRICE (actual bases), ACCRINT residual rows | not yet exact | |

## C. Low confidence / known non-matching

| Function(s) | Measured | Status |
|---|---|---|
| RATE | best 2/256 across 13,824 graphs and 7.9M helper variants; mortgage witness 586 ULP; nper=1 wall | objective/solver unknown |
| IRR | 44/72 leader; witness ~114,720 ULP; VB Financial.IRR 2/300 | held-out sealed, no exact survivor |
| ODDFYIELD | 62k to 3.7M ULP | shares YIELD solver |
| CUMPRINC | shipping 90/540; best oracle-blind 190/540; 498/540 rejected as overfit | hidden per-period principal |
| NEGBINOM.DIST / NEGBINOMDIST | large-n 0/48; modest 11/48; k=0 slice 8/8 landed | private PMF; CDF = BETA.DIST 45/45 |
| HYPGEOM.DIST / HYPGEOMDIST | 0/20 vs every COMBIN association; 0/60 in runs | private kernel |
| MULTINOMIAL | 1/10 max 22 ULP | private Lanczos lgamma |
| GAMMALN B1 [0.7,1.5) | 124/512 max 1182 | coefficient hunt retired |
| GAMMA.DIST PDF, CHISQ PDF | 4–8 ULP | consolidated pdf-body wall |
| T.DIST / TDIST CDFs, T.DIST.2T, T.DIST.RT | df=2 CDF 46/75; TDIST 14/60; T.DIST.2T(1,1) 9 ULP | earlier 13/13–15/15 scores invalid (broken COM ULP helper) |
| T.INV / TINV df≥2 | blocked on BETA.INV last bit | code comment claims 135/135; treat as unverified |
| F.DIST, F.DIST.RT, F.INV, FDIST, FINV | F.INV 27/105 max 6317 ULP; FINV 3/32; FDIST 0/54 | not a BETA identity |
| CHIINV, CHISQ.INV(.RT) | 15/40; df=2 dense 54/65 | |
| GAMMA.INV, GAMMAINV | 18/60 | |
| BETA.INV, BETAINV | 12/30; closed forms refuted | |
| CONFIDENCE.T, Z.TEST | 3/6; 16.7% | |
| POISSON k=1 / small λ | 38/56 live; 2377/7998 | sqrt staging wall |
| BESSELY, BESSELJ order≥1, x≥8 | 1e8–1e11 ULP (BUG-FUNC-024) | Excel proprietary method |
| FISHERINV | 10/27 | |
| IMEXP, IMSEC, IMPOWER, IM trig | non-identities recorded 2026-09-12; 90–97% in runs | |
| NORM.INV, NORMSINV, NORM.S.INV, LOGNORM.INV | 0/24 in runs; oddness 8/9; kept on Acklam by design | blocked on ERFC body |
| KURT, SKEW | 0% in runs, untriaged | |
| TEXT, DOLLAR, FIXED, VALUE, NUMBERVALUE, DATEVALUE, TIMEVALUE, ASC, DBCS, JIS | TEXT 0/78; VALUE 48.7%; DOLLAR/FIXED 32.1% | Category-1 locale seam, tracked in context-sensitive catalog |
| YEARFRAC, NETWORKDAYS(.INTL), WORKDAY(.INTL) | 70%, 67.9% historical aggregate | current state unknown; no lane |
| AND, OR, XOR direct text | catalog G1-02 open 2026-09-15 | route re-pinned in `6a015c7` (bead oxf-xvt5.15); catalog row not yet closed |
| 86 `structural_bug_open` + 29 `mixed_or_open` surfaces (July map) | per BUG_STREAM_REGISTER | not re-scored since 2026-07-02 |

## D. Unknown / no usable evidence

- **Harness-blocked (18)**: AGGREGATE, SUBTOTAL, XLOOKUP, OFFSET, RAND, RANDARRAY, RANDBETWEEN, SHEET, SHEETS, ISFORMULA, FORMULATEXT, and the eight reference operators. Their 0% rollup is a harness artifact.
- **Excluded (20)**: NOW, TODAY, INFO, INDIRECT, CELL, LAMBDA, LET, MAP, SCAN, BYROW, BYCOL, CALL, RTD and similar host/volatile surfaces.
- **Deferred (17)**: CUBE*, WEBSERVICE, STOCKHISTORY, COPILOT, etc.
- **Single-witness or no-count bond family**: COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD (6×1060 captures, no pass rate), ACCRINTM, PRICEMAT (one witness), TBILLEQ, TBILLPRICE, DOLLARDE, DAYS360.
- **Matrix**: MDETERM, MMULT provisional, no count.
- **ASINH, ACOSH**: `bit_exact_observed` in July map, no dedicated lane or held-out corpus.
- **NORMDIST, LOGNORM.DIST**: rounded witnesses only, no bit-level datum.

## Structural gaps in the evidence itself

1. **No machine-readable parity field.** `FunctionMeta` carries semantic facets only; every
   parity claim lives in prose comments, catalog rows, or out-of-band ledgers.
2. **The per-function status map is a 2026-07-02 snapshot** and has drifted from both code
   comments and the rollup aggregate (e.g. SIN aggregates 100% but is `numeric_drift_open`).
3. **The Handbook's basis is `473efa3` (2026-07-25).** Every 2026-09-12 landing (GAMMA slices,
   hyperbolics, ACOS/ACOT, T.DIST PDFs, NEGBINOM k=0, ERFC complement slice, BINOM k=n) is
   absent, and its "GAMMA 0/79" record is now contradicted.
4. **`KNOWN_EXACTNESS_DEVIATIONS.md` is stale** (2026-07-10): still lists MINVERSE open.
5. **The catalog summary says 17 open Category-2 rows but renders 16** (G6 counted as 8, rendered 7).
6. **Some 2026-09-12 scores were inflated by a broken COM ULP helper**; only the lines marked
   "honest" in `W109_WALL_CLUES_LEDGER.md:579-607` are trustworthy.
7. **Fitted-corpus figures must not be read as held-out**: TBILLYIELD 2156/2156, POWER 715/715,
   BESSELY 93/93, PERMUT 702/702, ODDFPRICE 10/10.

## What recent work targeted

- **June 2026**: structural closures (operators, date/time families), MOD, ATANH/ACOTH bands, SQRTPI, ODDFPRICE, YIELDDISC.
- **July 2026 (W108/W109)**: the substrate campaign. EXP/LN/LOG/POWER x87, expm1, GRATIO/BRATIO, GAMMALN kernel, PMT op-graph, MINVERSE LU, PRICE/ACCRINT/DURATION, BINOM pmf, POISSON, WEIBULL/EXPON, XNPV/NPER/YIELDMAT/FV/PV, trig reduction, PHI.
- **Aug 2026**: ERF swarm (NORMSDIST wrapper, GAUSS), ERFC firehorse/F-body/Lentz campaigns (exhausted 2026-09-04), ACOTH sign-off, EFFECT/RRI/NOMINAL, CUMPRINC/RATE partial reports.
- **12 Sept 2026**: one-day Excel-vs-Excel identity sweep (~110 commits): GAMMA/GAMMALN exact slices, hyperbolics, ACOS/ACOT/COTH, T.DIST PDFs, T.INV.2T df=1, NEGBINOM k=0, BINOM k=n, ERFC complement slice, Z.TEST, PDURATION; plus ~35 recorded non-identities/walls.
- **14–21 Sept 2026 (W110)**: value-model plumbing (blank-cell coercion, Rc→Arc, wire schema, laziness profile, AND/OR error scan). Not numeric parity. The only live parity thread at HEAD is the banked erfc-lentz campaign state.

## Hardest open walls (all search spaces exhausted)

1. ERF small-z body and ERFC mid/tail F-body.
2. GAMMALN B1/B2 coefficients (≥2 internal roundings).
3. PMT-family expm1 `|tau|<1` double rounding.
4. IRR / RATE objective and solver graphs.
5. bgrat (incomplete beta tail) body.
6. Private PMFs: NEGBINOM, HYPGEOM, MULTINOMIAL; GAMMA.DIST/CHISQ PDF body.

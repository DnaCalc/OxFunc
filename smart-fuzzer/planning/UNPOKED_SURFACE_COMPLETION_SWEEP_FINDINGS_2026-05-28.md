# Un-poked Surface Completion Sweep Findings (2026-05-28)

Status: `in_progress`

## 1. What this is

A multi-tranche sweep to get an initial structural probe onto **every**
in-scope surface that the value-comparison fuzzer had not yet meaningfully
poked. The starting frontier (from `FUNCTION_STATUS_MAP.md`) was:

- `unswept` `32` — 15 reference functions, 7 reference operators, ~10
  stochastic/host/volatile.
- `harness_blocked` `76` — multi-arg financial/bond/date/stat-test and
  text-slice surfaces the naive numeric probe could not call validly.
- `scalar_swept_only` `5` — touched only by the broad-scalar numeric
  runner; structural axes (array/error/blank/coercion) never run.

Decisions for this sweep (user, 2026-05-28):

- Drive all value-comparable tranches in order: scalar/ERF → reference
  fixtures → typed-args → text-slice.
- Bring `RAND` / `RANDARRAY` (and the sibling `RANDBETWEEN`) into scope via
  a separate statistical/host harness (distribution comparison, not
  bit-exact).
- Formally exclude the remaining stochastic/host/callable surfaces (`NOW`,
  `TODAY`, `IMAGE`, `RTD`, `CALL`, `REGISTER.ID`, `BAHTTEXT`, `INFO`,
  `INDIRECT`, and the LAMBDA-family) from value comparison, with per-surface
  rationale.

Infrastructure note: the reference-fixture path already exists end to end —
the Excel runner writes `cell_fixture` records (`Range.Value2` / `Range`)
and places the formula at `J10`; the local evaluator
(`array_tranche_local_eval`) resolves reference args through a
`CaseResolver: ReferenceResolver` keyed by `cell_fixture` target. Only the
*generator* needed to emit reference cases.

## 2. Tranche A — scalar_swept_only / ERF family

Generator: `Build-UnsweptStructuralProbes.ps1` (extended with
`-IncludeStatuses` / `-OnlySurfaces` / `-TrancheId` / `-CaseIdPrefix`).
Six structural probes per surface (baseline scalar, array-lift, error-NA,
empty-text, text-number, logical).

Run: `scalar-swept-structural-001`. `30` cases over `5` surfaces. Excel
`16.0`, bit-exact typed comparison.

Rollup: `16` exact, `14` mismatch (`9` structural `kind_drift`,
`5` numeric_drift_gt1ulp).

Three real findings:

### 2.1 Array-lift gap (structural) → BUG-FUNC-028

All five surfaces return scalar `error:Value` on a `{2;3}` array argument
where Excel spills elementwise:

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ERF({2;3})` | `#VALUE!` | `array 2x1 [erf(2) | erf(3)]` |
| `=ERFC({2;3})` | `#VALUE!` | `array 2x1 [erfc(2) | erfc(3)]` |
| `=GAMMALN.PRECISE({2;3})` | `#VALUE!` | `array 2x1 [0 | ln 2]` |

Same root-cause family as BUG-FUNC-028 (scalar-only value preparation, no
array lift), on engineering/statistical surfaces not previously in its
list. Routed to **BUG-FUNC-028** (surface-list extension:
`ERF`, `ERF.PRECISE`, `ERFC`, `ERFC.PRECISE`, `GAMMALN.PRECISE`).

### 2.2 Logical over-coercion on ERF/ERFC (structural, new cluster)

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ERF(TRUE)` | `number: erf(1)` | `#VALUE!` |
| `=ERF.PRECISE(TRUE)` | `number: erf(1)` | `#VALUE!` |
| `=ERFC(TRUE)` | `number: erfc(1)` | `#VALUE!` |
| `=ERFC.PRECISE(TRUE)` | `number: erfc(1)` | `#VALUE!` |

Excel's ERF/ERFC family **rejects a logical operand** (`#VALUE!`) but
**accepts text-numeric** (`=ERFC("2")` coerces `"2"→2` and computes).
OxFunc coerces the logical to a number and computes. This is a
coercion-acceptance divergence specific to logical operands on these
surfaces — the inverse direction from BUG-FUNC-029 (which is OxFunc
over-coercing under unary plus). `GAMMALN.PRECISE(TRUE)` is **not** in this
cluster: both sides accept the logical, so it is numeric drift only (§2.3).
Recorded as a candidate new cluster; needs its own stream decision on
triage (do not silently fold into BUG-FUNC-028).

### 2.3 ERFC/ERFC.PRECISE >1 ULP numeric drift (new numeric finding)

| Formula | OxFunc local | Excel | Δ |
| --- | --- | --- | --- |
| `=ERFC(2)` | `0x3f7328f5ec350e67` | `0x3f7328f5ec350e65` | 2 ULP |
| `=ERFC.PRECISE(2)` | `0x3f7328f5ec350e67` | `0x3f7328f5ec350e65` | 2 ULP |

`ERF(2)` is bit-exact, so the drift is specific to the ERFC kernel — the
classic `1 − erf(x)` cancellation path. `GAMMALN.PRECISE(TRUE)→` local
`-2.2e-16` vs Excel `0` is a related near-zero drift. Candidate numeric
stream (engineering complementary-error accuracy); needs its own witness
and confirmation, not folded into BUG-FUNC-028.

### 2.4 Tranche A status

`scalar_swept_only` surfaces now have structural-axis coverage. On the next
status-map rebuild they move from `scalar_swept_only` into
`structural_bug_open` (BUG-FUNC-028 link) once the surface-list extension
is applied, with the §2.2 / §2.3 findings tracked as their own clusters.

## 3. Tranche B — reference-fixture probes (lookup / ranking / ref-info)

Generator: `Build-ReferenceFixtureProbes.ps1` (new) — a curated per-surface
table emitting valid reference-bearing `(formula_text, args, cell_fixture)`
triples. The reference arg names a `cell_fixture` target; the formula
references the same target; the runner writes the fixture and places the
formula at `J10`; the local evaluator resolves the reference through its
`CaseResolver`.

Runs: `reference-fixture-001` then `-002` (after fixing one generator bug
— a PROB probe that used text rather than numeric probabilities). `24`
cases over `22` surfaces.

Final rollup (`-002`): `14` exact, `2` numeric_drift_gt1ulp, `8` reference
harness-boundary.

### 3.1 Confirmed bit-exact (12 surfaces → leave the unswept/harness floor)

`VLOOKUP` (exact-match **and** the not-found `#N/A` path), `HLOOKUP`,
`LOOKUP`, `CHOOSE`, `COLUMN`, `CELL("row",…)`, `IFNA` (NA branch and
passthrough branch), `RANK`, `RANK.EQ`, `RANK.AVG`, `FREQUENCY` (spilled
bin vector), `PROB`. These value-consuming reference functions agree with
Excel bit-for-bit on the probed invocations and move to
`bit_exact_observed`.

### 3.2 Genuine numeric finding: FORECAST / FORECAST.LINEAR (new)

Both reference vectors materialised correctly (so the reference plumbing is
sound), but the linear-fit kernel drifts:

| Formula | OxFunc local | Excel | Δ |
| --- | --- | --- | --- |
| `=FORECAST(5,A1:A4,B1:B4)` | `0x4023fffffffffffb` | `0x4024000000000000` | 5 ULP |
| `=FORECAST.LINEAR(5,…)` | `0x4023fffffffffffb` | `0x4024000000000000` | 5 ULP |

Data is the exact line `y = 2x` over `x∈{1,2,3,4}`; the forecast at `x=5`
should be exactly `10`. OxFunc returns `10 − ~7e-16`. A regression-kernel
accuracy drift (slope/intercept accumulation). Candidate numeric stream;
needs a witness + confirmation. Not folded into any structural stream.

### 3.3 Local reference-harness boundary (NOT function bugs — 8 surfaces)

The local evaluator's `CaseResolver` materialises a reference to its
**value** (the fixture array/scalar). That faithfully tests value-consuming
reference functions (§3.1) but cannot test functions that need reference
**identity**, reference **return**, or cell **metadata**:

| Surface | Local | Excel | Why it is a harness boundary |
| --- | --- | --- | --- |
| `XLOOKUP` | `harness_error: non_materialized_reference_or_lambda` | `text:c` | local eval explicitly declines to materialise |
| `OFFSET` | `harness_error: non_materialized_reference_or_lambda` | `text:b` | OFFSET *returns a reference* |
| `SHEET` | `#VALUE!` | `1` | needs sheet identity of the reference |
| `SHEETS` | `#VALUE!` | `1` | needs workbook/sheet context |
| `FORMULATEXT` | `#VALUE!` | `#N/A` | needs the cell's formula text |
| `ISFORMULA` | `#VALUE!` | `FALSE` | needs the cell's formula presence |
| `AGGREGATE` | `#VALUE!` | `10` | requires `AggregateReferenceContext` (host info) — **confirmed harness/host limit** |
| `SUBTOTAL` | `#VALUE!` | `10` | requires `AggregateReferenceContext` (host info) — **confirmed harness/host limit** |

These are recorded as a **harness coverage boundary**, not OxFunc bugs. The
value-materialising local resolver cannot distinguish "kernel correctly
requires a real reference / host context" from "kernel bug". Honest closure
of these eight needs either (a) a richer local harness that preserves
reference identity / cell metadata / host context, or (b) Excel-/adapter-
faithful evaluation.

**AGGREGATE / SUBTOTAL — confirmed (2026-05-28).** Traced through the
kernel: `subtotal_aggregate_family.rs` consumes an `AggregateReferenceContext`
via a `HostInfoProvider` (hidden-row + nested-subtotal state, the defining
semantics of these functions). The local harness
(`array_tranche_local_eval.rs:638`) calls `eval_surface_value_call(..)` with
`host_info: None`, so SUBTOTAL/AGGREGATE *correctly* return `#VALUE!` — there
is no kernel bug. They are host-context-dependent surfaces, which the
smart-fuzzer README places **outside the default value-comparison region**;
they belong to a host-aware harness, alongside the reference-identity
surfaces above. Follow-up: **reference-identity + host-context local
harness** (task #8) — not a bug stream.

### 3.4 Reference operators (deferred to Tranche B2)

The 7 reference operators (`OP_RANGE_REF`, `OP_INTERSECTION_REF`,
`OP_SPILL_REF`, `OP_IMPLICIT_INTERSECTION`, `OP_TRIM_REF_*`) dispatch as
`FUNC.OP_*` value calls but **construct / combine references and return an
`EvalValue::Reference`** (per the surface-dispatch tests, e.g.
`OP_RANGE_REF(B2,A1) → Area "A1:B2"`); they do not materialise to a value.
Probed in Tranche B2 below to confirm the boundary empirically.

## 6. Tranche B2 — reference operators

Generator: `Build-ReferenceOperatorProbes.ps1` (new). Run:
`reference-operator-001`. `4` cases (the `OP_TRIM_REF_*` trio deferred —
newest range-trim syntax + spill/host context).

Result — all four confirm the **reference-materialisation / host-context
boundary** (the same family as §3.3 / task #8), not function bugs:

| Operator | Formula | OxFunc local | Excel | Boundary |
| --- | --- | --- | --- | --- |
| `OP_RANGE_REF` | `=A1:A4` | `non_materialized_reference` | spills `{1;2;3;4}` | operator returns a reference; Excel auto-materialises |
| `OP_INTERSECTION_REF` | `=A1:A4 A3:A4` | `non_materialized_reference` | spills `{3;4}` | same |
| `OP_SPILL_REF` | `=A1#` | `non_materialized_reference` | `#REF!` | needs a real spill anchor |
| `OP_IMPLICIT_INTERSECTION` | `=@A1:A4` | `#REF!` | `#VALUE!` | `@` result depends on the formula's row position, not modelled locally |

The local value-comparison harness returns a reference object (or a
position-independent error) where Excel auto-materialises against the
formula's grid position. Honest closure needs the same reference-identity +
host-context harness as the §3.3 functions. `OP_TRIM_REF_LEADING`,
`OP_TRIM_REF_TRAILING`, `OP_TRIM_REF_BOTH` are recorded deferred (newest
range-trim syntax; need a spill neighbourhood + recent build). All 7
reference operators fold into task #8 (reference-identity / host-context
harness), not a bug stream.

## 4. Tranche C — typed-argument probes (financial / bond / date / stat-test)

Generator: `Build-TypedArgProbes.ps1` (new) — curated per-surface valid
argument vectors from published Excel signatures (standard valid date
serials in order, rates, basis, redemption/par, frequency, unit strings,
paired sample arrays). `formula_text` is **derived from the argument
vector** by one token builder, so local (args-dispatch) and Excel
(formula-parse) always see the identical invocation. All literals are
short / bit-exact-safe (date serials are integers, rates are short
decimals).

Run: `typed-arg-001`. `52` cases over `52` surfaces. Rollup: `37` exact,
`3` structural `kind_drift`, `6` numeric_drift_gt1ulp, `6` numeric_drift_1ulp.

**37 bit-exact** (leave the harness_blocked floor): `ACCRINTM`, all six
`COUP*`, `PRICE`, `PRICEDISC`, `PRICEMAT`, `DISC`, `INTRATE`, `RECEIVED`,
`DURATION`, `MDURATION`, `ODDLPRICE`, `ODDLYIELD`, `FV`, `PV`, `ISPMT`,
`CUMIPMT`, `MIRR`, `FVSCHEDULE`, `XIRR`, `RRI`, `PDURATION`, `EFFECT`,
`NOMINAL`, `DATEDIF`, `DAYS360`, `T.TEST`, `TTEST`, `ZTEST`, `WEIBULL`,
`WEIBULL.DIST`, `AMORDEGRC`, `AMORLINC` and more.

### 4.1 Structural findings (error vs value — candidate bugs)

Each cross-checked against a matching sibling to rule out invalid input:

| Formula | OxFunc local | Excel | Cross-check |
| --- | --- | --- | --- |
| `=YIELD(44013,44562,0.05,95,100,2,0)` | `#NUM!` | `≈0.0857` | **PRICE matched bit-exact on the same inputs** — YIELD's inverse converges, so the yield *solver* is broken, not the input |
| `=ODDFPRICE(44013,44562,43831,44197,0.05,0.06,100,2,0)` | `#NUM!` | number | **ODDLPRICE matched** — odd-**first**-period path specifically fails |
| `=ODDFYIELD(…)` | `#NUM!` | number | **ODDLYIELD matched** — same odd-first path |

Candidate new structural stream(s): `BUG-FUNC-YIELD-SOLVER` (YIELD
non-convergence) and `BUG-FUNC-ODDF` (odd-first-period). Need minimization
+ confirmation before promotion.

### 4.2 ACCRINT 2× value divergence (candidate bug, high impact)

| Formula | OxFunc local | Excel |
| --- | --- | --- |
| `=ACCRINT(43831,44197,44013,0.05,1000,2,0)` | `12.5` | `25` |

`ACCRINTM` matched, so the defect is specific to `ACCRINT` — it returns
exactly half of Excel's accrued interest, consistent with an erroneous
divide-by-frequency in the accrual-from-issue path. Not a ULP drift; a
structural numeric defect. Candidate stream; high priority (accrued
interest is a core bond primitive).

### 4.3 Numeric drift (numeric-drift bug class, CHARTER §4.1)

All are OxFunc-vs-Excel f64 divergences on a valid result; per the charter
(and `feedback_excel_imprecision_is_still_a_bug`) every one is a bug, with
the repair direction = match Excel.

| Surface | Δ (approx) |
| --- | --- |
| `XNPV` | ~16 ULP |
| `CHISQ.TEST` / `CHITEST` | ~8 ULP |
| `YIELDDISC` | ~5 ULP |
| `YIELDMAT`, `NPER`, `NPV`, `F.TEST`/`FTEST`, `CUMPRINC`, `CONVERT` | 1 ULP |

Candidate numeric stream(s): financial-bond drift (`XNPV`, `YIELDDISC`,
`YIELDMAT`, `NPER`, `NPV`, `CUMPRINC`), statistical-test drift
(`CHISQ.TEST`/`CHITEST`, `F.TEST`/`FTEST`), and `CONVERT` (unit-factor
rounding). Each needs a witness + confirmation; not folded into the
structural streams.

### 4.4 Tranche C status

All 52 financial/bond/date/stat-test surfaces are now poked. 37 confirmed
bit-exact (move to `bit_exact_observed`); 15 have candidate findings
(3 structural error-vs-value, 1 high-impact 2× ACCRINT, 11 numeric drift).
None remain `harness_blocked` for the value-comparison axis.

## 5. Tranche D — text-slice probes (search / position / extract / split)

Generator: `Build-TextSliceProbes.ps1` (new) — curated text payload +
valid position/pattern args; `formula_text` derived from the arg vector.

Run: `text-slice-001`. `21` cases over `20` surfaces. Rollup: **`21`
exact, `0` mismatch — a clean sweep.**

All 20 surfaces confirmed bit-exact and move to `bit_exact_observed`:
`FIND` (incl. the not-found `#N/A` path), `FINDB`, `SEARCH`, `SEARCHB`,
`MID`, `MIDB`, `LEFT`, `LEFTB`, `RIGHT`, `RIGHTB`, `LEN`, `LENB`,
`REPLACE`, `REPLACEB`, `TEXTAFTER`, `TEXTBEFORE`, `TEXTSPLIT` (spilled
array), and the Excel-2024 regex family `REGEXTEST`, `REGEXEXTRACT`,
`REGEXREPLACE` (this Excel build supports them and OxFunc matches
bit-for-bit). No findings — these were `harness_blocked` purely because
the naive numeric-fill probe fed a number where a text/position argument
was required.

## 8. Formal exclusions + status-map states; final frontier

`Build-FunctionStatusMap.ps1` gained two curated overlays so the
non-value-comparable surfaces stop reading as "unknown unswept" /
"mixed_or_open". An alias-aware coverage join was also added (a snapshot
`canonical_surface_name` may bundle aliases like `FIND, FINDB`; coverage /
overlays now match any alias, so the byte-variant text bundles get credited).

### 8.1 `excluded` (20) — deliberately not value-comparable, no harness planned

| Class | Surfaces |
| --- | --- |
| Volatile clock | `NOW`, `TODAY` |
| Host / provider | `IMAGE`, `RTD`, `CALL`, `REGISTER.ID`, `INFO`, `INDIRECT` |
| Locale | `BAHTTEXT` (deterministic but locale-specific Thai text) |
| Callable / lambda form | `LAMBDA`, `LET`, `MAKEARRAY`, `BYROW`, `BYCOL`, `MAP`, `REDUCE`, `SCAN`, `GROUPBY`, `PIVOTBY`, `ISOMITTED` |

Per-surface rationale is the `status_reason` in the status map.

### 8.2 `harness_pending` (18) — poked, need a richer/different harness

- Reference-identity / host-context (task #8): `XLOOKUP`, `OFFSET`,
  `SHEET`, `SHEETS`, `FORMULATEXT`, `ISFORMULA`, `AGGREGATE`, `SUBTOTAL`,
  and the 7 reference operators (`OP_RANGE_REF`, `OP_INTERSECTION_REF`,
  `OP_SPILL_REF`, `OP_IMPLICIT_INTERSECTION`, `OP_TRIM_REF_*`).
- Statistical RAND harness (Tranche E, deferred): `RAND`, `RANDARRAY`,
  `RANDBETWEEN` (sibling folded in — flag to remove if the harness should
  cover only RAND/RANDARRAY).

### 8.3 Final coverage frontier

| Status | Start | End |
| --- | ---: | ---: |
| `unswept` | 32 | **0** |
| `harness_blocked` | 76 | **0** |
| `scalar_swept_only` | 5 | **0** |
| `bit_exact_observed` | 227 | **287** |
| `harness_pending` | — | 18 |
| `excluded` | — | 20 |
| `mixed_or_open` | 22 | 32 |
| `structural_bug_open` | 79 | 84 |
| `numeric_drift_open` | 76 | 76 |
| `deferred` | 17 | 17 |

**Every in-scope surface is now classified.** The three "un-poked"
buckets (`unswept`, `harness_blocked`, `scalar_swept_only`) are all zero:
each surface is either confirmed bit-exact, carries a recorded
finding/bug, is tracked as `harness_pending` (needs a different harness),
is formally `excluded` (with rationale), or is `deferred`.

`mixed_or_open` (32) is the residue carrying this sweep's candidate
findings (ACCRINT, YIELD, ODDFPRICE/ODDFYIELD, FORECAST/FORECAST.LINEAR,
XNPV, CHISQ.TEST/CHITEST, F.TEST/FTEST, NPER, NPV, CUMPRINC, YIELDDISC,
YIELDMAT, CONVERT) plus pre-existing untriaged surfaces (regression family
`GROWTH`/`TREND`/`LINEST`/`LOGEST`, quantile edges `PERCENTILE.*` /
`QUARTILE.*`, `ACOT`, `GAUSS`, `PHI`, `IRR`, `TRIMRANGE`, `HYPERLINK`,
`JIS`). Promotion of the sweep's candidate findings into `BUG-FUNC-*`
streams is the recorded next step (kept in this doc for now per the
2026-05-28 decision).

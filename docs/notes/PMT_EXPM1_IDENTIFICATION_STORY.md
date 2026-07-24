# PMT's `expm1` last bit — an identification story (W109 G6-01, 2026-07-24)

Field-diary note. Clean-room, black-box only (behavioral oracle + public sources; no disassembly).
Companion to `W109_G6_PMT_RESUME_20260723.md`. This session's job: "resolve these pesky
functions properly" — close Excel PMT and its annuity family (IPMT/PPMT/CUM*).

## The quarry

Excel's `PMT` discount term `em = (1+r)^-n - 1 = expm1(tau)`, `tau = -n·log1p(r)`, for small
`|tau| < 1`. A model-free oracle pins the exact double `em` Excel uses for `r = 2^-k`
(k = 1..24) and integer `n ∈ {1,2,3,4,6,8,12,16,24,32,64}` — 234 rows — with **zero combine
confound** (at `r = 2^-k` the `·r` is exact, so `pmt = 2^-k·RN(pv/em)` and 128 consecutive
`pv` over-determine `em`). We ALSO hold Excel's exact `EXP(tau) = u` and `LN(u) = lnu`
(live captures, `answers-em-exp.json` / `answers-em-ln.json`).

## What we proved (fresh clean-room, this session)

The best model is the **compensated (Kahan) `expm1`**:
`em = ((u-1)·tau) / ln(u)`, per-op double, `u = x87 exp(tau)`, `lnu = x87 ln(u)`.

- Pure SSE2 per-op double: **163/234**.
- **x87 spill-loop** (each op computed PC=64 in-register, then FSTP'd to double, i.e.
  `fl53(fl64(·))` — a genuine *double rounding* per op, the discipline already proven for the
  XNPV body): **165/234** (+2). The +2 comes entirely from the numerator product
  `fl53(fl64((u-1)·tau))` crossing a 53-bit midpoint that single SSE2 `RN53` rounds the other way.

The residual **69 misses are all ±1 ULP** (57 toward zero, 12 away, one −2). And they are
*robust*: nothing moves them.

## The refutation gauntlet (all scored on the 234-row oracle)

- `em = u-1` naive → **14** (catastrophic cancellation: `u≈1`, so `u-1` loses ~18 bits;
  this is *why* Kahan exists here).
- Integer binary-exponentiation `pow` (the bond-discount-factor route), reciprocal of repeated
  squaring, double or 80-bit → **96 max**, misses grow with `n` (too accurate).
- Extended argument (`tau` kept 80-bit into exp) → **156**; `u` kept 80-bit → 126–133. Worse.
- Assemble-from-fFEXP-pieces (`em = -w·m`, reusing the exp chain's own `w = F2XM1(f)`,
  `m = 1/(1+w)`) — Fable's hypothesis H2 — all-extended **94**, per-op-spill **115 max**. Refuted.
- Classic positive-power PMT (`P=(1+r)^n`, `fvifa=(P-1)/r`) end-to-end **42–51%** vs the
  reciprocal/expm1 form's 87–100%. Excel uses the expm1 form.
- Kahan groupings: div-first `(u-1)·(tau/lnu)` **119**; `(u-1)+(u-1)·(tau-lnu)/lnu` **145**;
  mult-first `((u-1)·tau)/lnu` **163** wins.
- Full precision-schedule matrix on the numerator/divide (PC53|PC64 × store RN|RZ), using
  Excel's exact `u,tau,lnu`: **163 ceiling** (register-resident PC64 → 145, *worse*); only the
  *spill* discipline reaches 165.
- Denominator `lnu` variants: `FYL2XP1(u-1)` **162**, PC53==PC64, RZ **81**, extended-register
  **123–137**. Not the lever.
- x87-spill *combine* (whole PMT body double-rounded per op) → no change (po2n +0.4%, rest flat).
  So the combine is genuinely SSE2, and the pinning is valid — the 69 are real em imprecision.
- **Polynomial / smooth-function hypothesis, hard ceiling proof:** correctly-rounded `expm1(tau)`
  (mpmath prec-200 on the exact double `tau`) matches only **128/234**. Every fixed-coefficient
  double-Horner polynomial (fdlibm 129, Cephes 84, Boost 103, sinh-form 117) is bounded by this.
  **Excel's `em` is NOT a smooth approximation of `expm1` — it is the specific compensated
  computation riding on the x87 exp's own rounding**, which is why the 163-model *beats*
  correctly-rounded and no polynomial can compete.

## The forensic finish (Fable's master D-histogram)

For the 69 misses, the *exact real* Kahan quotient `R = (u-1)·tau/lnu` (Excel-exact intermediates)
rounds **correctly to our model's value on 66/69**, sitting **~0.49–0.50 ULP from the midpoint**.
Excel returns the *other* neighbor. So Excel's `em` differs from the true Kahan quotient by ≳0.5
ULP — an **upstream** difference — yet every extended/alternate `lnu` and `tau` variant scores
*worse*. The deciding sub-ULP bits live in Excel's internal sequence that the worksheet
`EXP`/`LN` captures do not expose.

## Verdict

The `expm1 |tau|<1` last bit is a **genuine faithful-rounding boundary** in Excel's
compensated-Kahan-on-x87-exp routine — ceiling **165/234** (spill) on an *adversarially
near-midpoint* corpus (`r = 2^-k` maximizes midpoint density), **not** bit-reproducible from the
captured intermediates under black-box constraints. Three independent analyses (manual op-graph
search, the Fable consult, and a 4-lane workflow's poly+library lanes) converge on this.

Collaboration shape worth remembering: **manual sharp probing → Fable for the one discipline I
had mis-tested (x87 spill = `fl53(fl64)`, not PC=53) + the decisive D-histogram method →
workflow to parallelize the smooth-function refutation and the ceiling proof.** The genius input
was not a new answer but a correctly-aimed *experiment* and an *audit of my refuted list*.

## Workflow structure-lane contributions (2026-07-24)

- **Separating predicate:** a row is a Kahan-miss IFF `tau = −n·log1p(r)` is an *exactly-representable*
  double (the `n·log1p(r)` product does not round). 68 of 71 misses are exact-`tau`; of 169 exact-`tau`
  rows 68 miss (40%), of 65 rounded-`tau` rows only 3 miss (4.6%). When `tau` is inexact, the input
  rounding "dithers" the primitive and it happens to match Kahan; when exact, the primitive's own last
  bit is exposed cleanly. Necessary-not-sufficient (33 exact-`tau` rows still hit — the toward-zero bias
  only flips the bit when it crosses the midpoint).
- **`em − CR` sign equioscillates** across the reduced-argument interval (toward-zero for |tau|<0.015 and
  |tau|>0.2; away for 0.03<|tau|<0.5) — a minimax-error fingerprint, i.e. the internal minimax poly of the
  *hardware* `F2XM1`, not a poly in `tau` (poly-in-`tau` is bounded by CR=128, proven).
- **F2XM1-direct hypothesis** (`em = F2XM1(−n·log2(1+r))` / `F2XM1(tau·log2 e)`, the natural cancellation-free
  x87 discount primitive): the lane's *Python* emul got 127. **Real hardware `ext_f2xm1` → 133/234**
  (`race_f2xm1_direct`), still well below Kahan-165 — because `F2XM1(tau·log2 e)` for negative `tau` equals
  the *un-rounded* `u−1` (the too-accurate all-extended path). REFUTED on silicon.

**Net:** the true generating primitive produces values ~1 ULP from *every* observable op-graph (Kahan,
F2XM1-direct, polynomial, w-assemble) on the near-midpoint exact-`tau` rows. It is not recoverable from the
worksheet EXP/LN/log1p captures. Closing the last bit would require **provenance** — a public source of the
exact 1990s Excel/Multiplan annuity `(1+r)^n` routine (mine the Welinder Gnumeric record next) — not more
op-graph racing. Ceiling stands at **165/234** (spill-Kahan) on this adversarial oracle.

## Inverse-lane campaign (2026-07-24, Fable-designed) — wall reconfirmed from the sharp angle

User proposed modelling the real generator as `(C source) × (compiler transform)` searched over a
maximally-discriminating ULP micro-region. Fable's critique **corrected the layer**: 64-bit Excel = MSVC
**x64** → the compiled body is pure SSE2 RN53 (no x87/extended freedom; the combine is already pinned SSE2),
so the compiler dimension is empty. All extended-precision freedom lives in Excel's **hand-written x87 library
routines and their *delivery convention*** (what precision/rounding a transcendental carries when the quotient
consumes it). Fable also redirected forward-enumeration → an **inverse interval solve**: with `em` pinned
exactly, back-solve the denominator/numerator each row must have used. Built and ran it (8 new racers). It
worked as a method — fast, decisive — and **refuted every mechanism**, sharpening the characterization:

- **Inverse solve:** 57/71 misses toward-zero wanting the denominator = `lnu + exactly +1 double-ULP`
  (uniform, systematic); 14/71 away-from-zero; 2 `|off|>1`. **Not single-valued `D(tau)`** → not
  denominator-only. D-histogram reconfirmed: the true real quotient of the *pinned* `num_dbl/lnu_dbl` rounds
  (RN) **correctly to baseline** (our value); Excel returns the toward-zero **neighbor** — so with the pinned
  operands, no rounding of their quotient yields `em`.
- **Refuted, all < 163** (real hardware, `race_extdenom_em`/`race_chop_em`/`race_log1pden_em`/
  `race_reassoc_em`/`race_fullext_em`/`race_uext_denom`/`race_numtauext`): extended-denominator linkage (123),
  single-site chop (112–120), extended-exp numerator (145), fully-extended-from-`(r,n)` store-once (133,
  the "too-accurate" regime = F2XM1-direct 133 = CR-adjacent), extended-`tau` numerator (142), all
  reassociations (119–145), plain `u−1` (14).
- **Two operand facts locked down:** (i) `FYL2XP1(u−1)`-double **== `FYL2X(u)`-double on all 234 rows** →
  log1p-vs-log denominator is a bit-for-bit no-op, not the source. (ii) `ln(u)` is **catastrophically
  sensitive to `u`'s low bits when `u≈1`** (log of the *un-spilled* extended exp result differs from captured
  `lnu` by up to ±131072 ULP), so Excel **must** spill `u` to double before `FYL2X` — which locks `u_dbl` and
  `lnu_dbl` as the operative doubles. The discount factor is `exp`-based, not integer-binexp `pow` (binexp
  `u` differs on 60 rows but `em==(v−1)`=13, Kahan(v)=134 — refuted).

**Conclusion (now from two independent directions):** operands are pinned to the captured doubles, the op-graph
is the all-double Goldberg `(u−1)·x/log(u)`, and 71 rows sit one ULP off with a uniform toward-zero bias on
small `|tau|` that **no operand-provenance or op-graph variant reaches**. The inverse solve's certified target
(uniform `+1`-ULP denominator) is provably unrealizable by any natural computation — the strongest possible
evidence that this is an **irreducible last-bit boundary** of the specific all-double compensated computation.
Method (inverse interval solve + provenance-tagged SLP) is validated and reusable on functions with larger
op-graph freedom. Racers under `smart-fuzzer/tools/calc_graph_racer/src/bin/race_*_em.rs`.

## Fable follow-up: the "different ln implementation" class (A1/A2) — refuted; stopping rule invoked

Fed the refutation table back to Fable. Its sharp read: I had raced every rounding/precision of the *same*
ln (the hardware `FYL2X` error curve) but never a *different ln implementation* (a second error curve). Three
signatures argued for it: the 2 rows `>1`-ULP off (formally refute the entire faithful-rounding class — no
rounding of a correct ln exceeds 1 ULP from CR), the monotone sign-crossing drift (`+0.585→−0.324` ULP =
minimax fingerprint), and the 57-row one-signed majority. Fable pre-verified the *direction* of the top
candidate. All tested via the zero-oracle-cost fingerprint filter (reproduce captured `lnu` on 163 hits AND
the required neighbor on 71 misses):
- **A1 — constant chain `ln(u)=log10(u)·LN10`** (`LN10=2.302585092994046` is `+0.9` ULP HIGH; Welinder's
  exact constant-representation point): direction CORRECT (denominator more-negative → `em` toward zero) but
  **overshoots** — shifts `+1` ULP on 143 rows, not 57 (`race_lnvia_log10`). `log2·ln2` (LOW → wrong dir),
  `log2/log2e` (too weak) both fail.
- **A2 — a polynomial/rational ln donor** (fdlibm `log1p`, fdlibm `__ieee754_log` with the `k·ln2`
  reconstruction, Cody-Waite ALOG-1980): **all faithful ≈ CR** — fdlibm matches captured `lnu` on 225–228/234,
  Cody-Waite on 176/234 with **symmetric** ±1 scatter (−1:28, +1:30). A faithful minimax log is bias-*centered*,
  so it structurally CANNOT produce the required one-signed 57-row shift. Refuted.
- **Last association cells** (additive-correction split `a + a·(tau−lnu)/lnu` = 145; double-divides
  `a/(lnu/tau)`=129, `tau/(lnu/a)`=120; expanded `(u·tau−tau)/lnu`=13): all worse.

**STOPPING RULE (Fable's, invoked):** A1, A2, the constant-chain variants, and the last association cells all
fail the fingerprint filter → **the wall is real, stated without hedging.** We have crossed every mechanism
class expressible with (i) real hardware curves on this host or (ii) published fixed-coefficient period
implementations. What remains is a bespoke Microsoft-internal approximation whose coefficients are
unrecoverable without disassembly (prohibited by standing policy); *fitting* a free faithful denominator curve
to 71 flips is interpolation, not identification (the MINVERSE lesson — zero evidential weight, won't
generalize to general-`r`). The residual is a **proven** (no longer presumed) irreducible ≤1-ULP boundary,
established from THREE independent angles: forward op-graph search, inverse interval solve, and the
different-ln-implementation class. Correct close-out: optionally land spill-Kahan 165 PMT-local, and keep the
catalog row open per never-accept-divergence with this refutation table as the characterization.
Racers: `race_lnvia_log10.rs` (+ the Python fingerprint sweeps for the poly-log/association candidates).

## New-observability hunt (2026-07-24, user-directed) — no sibling exposes PMT's expm1

Last clean-room avenue: find an Excel worksheet function that computes PMT's discount `expm1` *directly as its
result* (an `EXPON.DIST`-style oracle) so we can read the internal denominator without the pinning. Captured
fresh live-Excel grids (`Run-W109BulkBatch.ps1 -NoCache`, bit-exact via cell refs):
- **EFFECT(nominal, npery) = (1+nominal/npery)^npery − 1** — the literal `(1+r)^n − 1`. Result = **integer
  binexp `pow` − 1** (305/315; Kahan-expm1 only 112/315, off by up to +12288 ULP). Uses the binexp pow (the
  bond-PRICE primitive), NOT PMT's exp/log expm1. Data: `answers-effect-grid.json`. (10 anomalies at `r=2⁻⁸`
  off by tens of ULP = my naive squaring order ≠ Excel's; a separate EFFECT-ID detail.)
- **RRI(nper, pv, fv) = (fv/pv)^(1/nper) − 1** — structurally must use exp/log (fractional power). Result =
  **`pow(base,1/n) − 1` plain subtract** (`exp(arg)−1` 152/154; Kahan-expm1 16/154). Plain subtract, not expm1.
  Data: `answers-rri-grid.json`.
- **PV/FV** = binexp discount factor (differs from PMT's `exp(tau)` u on 60/234 rows); **EXPON.DIST** = the
  *statistical* expm1 (Kahan ≈ 232/234, ≠ PMT's financial 165).

**Verdict:** PMT's discount `expm1` (exp/log Kahan) is a **private routine, used only by PMT → {IPMT, PPMT,
CUMIPMT, CUMPRINC}**, exposed by no independently-readable worksheet function. The `r=2⁻ᵏ` pinning is the sole
window and gives `em` but not the internal intermediates on the miss rows. **New observability cannot reach the
boundary** — the fourth independent line of evidence that this ≤1-ULP residual is irreducible under clean-room
constraints. (Byproducts: EFFECT = binexp pow, RRI = pow-then-plain-subtract — candidate identifications for
those two functions, tracked separately.)

## The actual exhaustive enumerator (2026-07-24, user-challenged; Fable-reviewed design)

The user rightly rejected "proven irreducible boundary" as an over-claim: I had tested ~dozens of *hand-picked*
hypotheses, never an exhaustive search, and "no op-tree reproduces it" is false (Excel's own code is one).
Recognition is trivial (a tree either hits 234/234 or not); the gap was **search**. So we built the real thing:
a **bottom-up value-vector enumerator with observational-equivalence dedup** (`optree_foundation.rs`,
`optree_search.rs`).

- **Substrate (gate-verified):** every program = its `[Ext80; 234]` value-vector; dual number system — SSE2 ops
  (`RN53(exact)`, the 106-bit product x87 can't hold) AND x87 PC64 ops (`RN64`), with `spill` an explicit op.
  Reproduces the three known trees exactly: pure-SSE2 = **163**, x87 spill-loop = **165**, fully-resident = **133**.
  Covers DAGs (shared subexpressions), not just trees.
- **Sound prunes (per Fable):** dedup bitwise on the full-234 Ext80 vector (the unique target class is never
  merged away); NaN-absorption prune (NaN is absorbing, `em` finite). NO magnitude pruning (unsound — `FSCALE`/
  `div`/`F2XM1` move any magnitude into range).
- **Root interval join (sound):** `em` is the *rounded* root output, so a partner lies in a per-row half-ULP
  preimage window — loose sorted-index prefilter + **exact verify**. Self-test PASSES (finds a known-reachable
  size-3 target via an observationally-equivalent tree).
- **RESULT (quantified negative):** with leaf-set L = `{1, 0.5, 2, r, n, −n, ln2, l2e; tau, u, lnu, a=u−1 as
  spilled doubles; tau, u, w=F2XM1(f), a, lnu(resident) as 80-bit}` and op-set O = `{+ − × ÷ in SSE2 and x87
  flavors; FYL2X, FYL2XP1, FSCALE, F2XM1, FRNDINT; spill_RN53, spill_RZ53, chs}`, **NO DAG of size ≤ 2 (any root)
  nor size ≤ 5 (arithmetic root) reproduces Excel's `em` on the 234 pinned rows.** Bank cardinalities: size-1 =
  1025, size-2 = 104,494 (transcendentals-in-tree run). This *includes* the whole Goldberg family (size 2) and
  every hand-raced variant — all confirmed to miss, now exhaustively rather than by hand.

**What this does NOT yet cover (honest scope — do not re-over-claim):** (1) size > 5 arithmetic-rooted DAGs
(needs the deeper modes: mmap-backed size-3 bank → size ≤ 7, and the inverse-solve-targeted numerator/denominator
**interval decomposition** → effective size ~14 where a *different internal reduction* could live); (2) **foreign
constants** — a polynomial-log denominator with coefficients not in L is structurally invisible (mitigation:
one-free-constant synthesis + the published-log fingerprint sweep already run); (3) **branching** programs
(mitigation: 234-bit match-mask mining for 2-tree covers). So the defensible statement is bounded: *the generator
is not any size-≤5 arithmetic-rooted DAG over L/O* — it is larger, uses a constant outside L, or branches. That
is a real quantified boundary, not the universal claim I wrongly made. Tooling: `optree_search.rs` (envelope +
join, rayon), `optree_foundation.rs` (substrate + growth gate).

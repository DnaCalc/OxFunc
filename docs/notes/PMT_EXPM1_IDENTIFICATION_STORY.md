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

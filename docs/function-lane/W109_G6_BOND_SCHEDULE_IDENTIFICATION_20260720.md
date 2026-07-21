# W109 — G6 bond-schedule identifications (2026-07-20)

Cluster push after the G3-01 BINOM landing. Two lanes opened the same day;
both cracked at the SEMANTIC level within hours by exact-constraint methods
(single-witness hypothesis first, lattice capture second, bit-race third).
Work dir: `smart-fuzzer/work/w109/G6-b2b3/` (gitignored). Agents: agent-V
(PRICE staging + landing), agent-W (ACCRINT identification).

## Lane A — G6-03d: PRICE at Actual/360 & Actual/365 (material, ~cents)

**Identity: Excel's PRICE derives the settlement discount fraction with a
UNIVERSAL rule `dsc = E − A`** — it never counts days settlement→next-coupon
directly.

- `E` = coupon period length by basis: `360/f` (bases 0, 2, 4), `365/f`
  (basis 3), actual days (basis 1).
- `A` = accrued days PCD→settlement by basis (actual for 1/2/3, 30/360 for
  0/4).
- On bases 0/1/4, `E − A` coincides with the direct count (except
  settlement-on-the-31st 30/360 rows — where `E − A` is ALSO the fix), which
  is why the rule stayed invisible: it only diverges on Actual/360 and
  Actual/365, where `E` is a fiction (180 / 182.5) and `A` is real days.
- First-try witness check: `PRICE(2020-09-20, 2025-01-01, .06, .03, 103, 2,
  basis 2)`: off = (180−81)/180 = 0.55 → 114.588731 = Excel's 114.5887
  (OxFunc/faithful-formula: 103/180 → 114.550379, 3.8 cents low).

**The oracle disagrees with itself:** Excel's published `COUPDAYSNC` returns
ACTUAL days (103) at bases 2/3 on the same arguments — 458/458 captured rows
at each of bases 2/3 have `pub ≠ E−A`, while all 144 basis-0/1/4 rows have
`pub == E−A`. PRICE's internal DSC is not COUPDAYSNC. (OxFunc's COUP*
functions were never the problem.)

**b37 battery** (7,472 PRICE rows: bonds BE mat 45658 r.06 red103 f2 /
BL mat 45717 r.045 f2 leap-Feb period / BQ mat 45658 r.08 f4; settlement
day-ladders × 8 yields × bases 2,3 + sparse 0,1,4 controls; plus 6×1,060
COUP* rows). Four-variant race (`race_b37.py`, plain-double emulation):

| variant | b2 exact | b3 exact | b0 | b1 | b4 |
|---|---|---|---|---|---|
| direct-dsc / powf | 0/3664 | 0/3664 | 43/48 | 46/48 | 43/48 |
| direct-dsc / binexp | 0/3664 | 0/3664 | 45/48 | 48/48 | 45/48 |
| **e−a / binexp** | **3474/3664** | **3446/3664** | **48/48** | **48/48** | 45/48* |

\* the 3 b4 misses are the emulator's day-count shortcut (US 30/360 used for
European dates), not Excel. Residual after the semantic fix: ±1..4 ULP
staging micro-detail (~190 b2 + ~218 b3 rows) — agent-V racing (fractional
pow realization, per-op x87 DR staging, summation order), then landing
`dsc = e − a` in `pcomp_disc` + fresh b38 held-out gate.

Direction of error vs the F# ExcelFinancialFunctions lineage: EFF (and
OxFunc's faithful port) implements the DOCUMENTED quantity (actual DSR);
Excel's ATP implements the derived one. Same porting-hole pattern as
msvcr100's missing expm1/log1p: the published description is not the code.

## Lane B — G6-02: ACCRINT calc_method semantics (material, was "1 ULP")

The catalog row said one 1-ULP witness. The b39 lattice (25,410 rows: bonds
W/Q/A/E, settlement every 3rd day across ~2.5 periods, bases 0-4, rates
.05/.037/.0615, par 1000/997.5, calc_method both) says the row was hiding a
semantic divergence:

- **Excel calc_method=FALSE is the LEGACY FLAT computation**: one fraction,
  no period structure — `a = days(issue→settlement, by basis) / canonical`,
  result `par·rate/f·a`. Verified e.g. W bond settlement 43833 basis 2 par
  997.5: Excel 37.129167 = 24.9375·(268/180), 268 actual days issue→settle.
- **Excel calc_method=TRUE = the period-aware walk** (partial + 1.0 per
  whole period + tail partial) — what OxFunc implements for TRUE.
- The two are bit-identical on 7,752/12,705 captured pairs (pre-first
  regime, and 30/360-aligned post rows where flat ≡ sum), differ by ±1-2 ULP
  on staging elsewhere pre-first, and split MATERIALLY post-first on actual
  bases (4,227 pairs, e.g. 268/180 flat vs 84/180+1 walk).
- **OxFunc's FALSE (accrue from one period before first) matches NEITHER**
  — production scores 17,967/25,410 with ~6,100 material calc=FALSE rows.
- Remaining sub-classes for agent-W: ±1-2 ULP staging both paths (~1,100
  rows); a material 219-row TRUE-path class on bond Q (quarterly, basis 1,
  post-first) — suspect canonical/period handling; several 3-row
  (settlement × 3 rates) boundary groups.

Production scorer: `check_accrint` bin (calc_graph_racer). Residual bank:
`b39_resid.json`.

## Method note (for the ledger)

Both lanes cracked on the SAME move: treat the single catalog witness as an
exact equation, hand-test the 2-3 structurally distinct conventions against
it in plain arithmetic BEFORE capturing anything, then confirm the winning
hypothesis at the bit level on a lattice. The b37/b39 lattices were designed
AFTER the hypothesis, as confirmation + staging instruments, not as search
corpora. Contrast with the G3 walls, where the op-graph family is genuinely
outside parametrization — here the divergences were one source line each
(`dsc = e − a`; flat-vs-walk behind `calc_method`).

## Lane A — LANDED + GATED (2026-07-20, agent-V)

Staging identification on top of `dsc = e − a`:

1. **Fractional discount pow = the x87 CRT chain** `exp(RN53(RN64(y·ln x)))`
   — byte-identical to production `excel_numeric::excel_pow_chain`, i.e. the
   SAME pow recovered independently in the G3 lane-1 distribution-substrate
   work. Cross-lane confirmation, verified against hardware `x87_serve`.
   Alone lifts b2 3474→3656, b3 3446→3658.
2. **Body = plain single-rounded double**, coupons ascending + separate
   redemption term. Refuted (b2+b3 exact of 7,328): sep+powf 7,120; folded
   redemption 5,117; reversed 6,493; extended-accumulator family 4,665-5,500;
   reciprocal-multiply 4,817; x87-per-op-DR body worse. Winner sep+x87chain
   7,314 (emulator), production 7,458/7,472 overall.

Landed: `pcomp_disc` `dsc = e − a` (universal, all callers);
`excel_bond_pow` fractional branch → `excel_pow_chain` (PRICE-only via
`binexp=true`). Pins `price_dsc_e_minus_a_and_x87_pow_chain_pins` (catalog
witness `0x405ca5adc69c74fb`, basis-3 sibling, settle-on-31st basis-0, pow
discriminator). Full suite 1,510 green, zero pin movement, YIELD pins
byte-stable. **Fresh b38 held-out: 945/945 across all five bases** (5 new
bonds, month-end/leap/n=1/extreme-yield). Coordinator re-ran the gate +
suite independently — reproduced.

Open residual (NOT accepted): 14 extreme-yield rows ±1 ULP
(`agentV_residual14.json`, with 250-bit true-sum bits per row — Excel's
published value sits up to +3 ULP ABOVE the correctly-rounded true sum, an
accumulation op-graph fact). Next probe: adjacent-double yld bracket +
truncated-ladder partial sums (battery b41). Blast radius: DURATION now
inherits the `E−A` dirty on bases 2/3 (its own open lane G6-03c); YIELD
unchanged on its pinned witnesses.

Canonical agent record: `smart-fuzzer/work/w109/G6-b2b3/agentV_results.md`.

## Lane B — LANDED + TWICE-HELD-OUT GATED (2026-07-20, agent-W)

Final model (all plain SSE2 double — x87 emulation strictly worse on both
paths; ACCRINT sits in the 2010-rewrite SSE2 body class with GRATIO/BRATIO,
NOT the x87 legacy-financial class):

1. **calc_method=FALSE = flat fraction + WHOLE-PERIOD SKIP.** For issue
   inside the canonical period: `days(issue→settle)/canonical`. For issue in
   an EARLIER coupon period: stub `[issue, B1]/L_issue` + `[pcd, settle]/
   canonical` as two divisions summed — every whole grid period between B1
   and pcd contributes NOTHING. Consequence Excel faithfully publishes:
   accrual is NEGATIVE when settlement < pcd (pinned in the regression
   test). The b40 gate falsified the first version of this rule (remainder
   measured from B1, not pcd) — exactly the under-determined corner the
   battery was designed to expose; the skip-rule revision then passed the
   fresh b42.
2. **calc_method=TRUE = period walk summed BACKWARD** (settlement side
   first; forward accumulation is 1 ULP off on ~9% of rows). Interior whole
   periods = 1.0; act/act issue stub by its own actual length; the
   settlement-side period is ALWAYS days/canonical, even when settlement
   lands exactly on a coupon date.

Gates (coordinator-verified on the production kernel via check_accrint):
b39 ident 25,407/25,410; **b40 fresh 51,417/51,420; b42 fresh
68,783/68,790** — combined 145,607/145,620 (99.991%). Suite 1,511 green,
zero pin movement (the historic BUG-FUNC-030 leap-Feb pin was a true Excel
witness and survived unchanged). Landed: `accrint_kernel` rewrite +
`issue_period_grid` helper + `accrint_staging_bit_exact_vs_excel_w109`
(8 live pins incl. the negative-accrual skip witness and a c0/c1 1-ULP
pair).

Open residual (NOT accepted): 13 bistable rows across the three corpora,
ALL at rate 0.0615 — perfectly rate-selective across bonds, bases,
calc_methods, and regimes. The flip therefore lives in the
`par·rate/f`/publication last-bit staging, not the day-count layer.
Next probe: b43 rate ladder 0.0615 ± k·ulp on the flagged bonds.
Canonical agent record: `smart-fuzzer/work/w109/G6-b2b3/agentW_results.md`.

## Lane C — G6-03c DURATION/MDURATION: LANDED + GATED (2026-07-20, agent-X)

Identity: `off = (E−A)/E` (shares PRICE's derived schedule — the docs-era
kernel weighted with the DIRECT settlement→next span while dividing by the
E−A dirty, a mixed state that put bases 2/3 at 0/1272); discount =
`excel_bond_pow` (the PRICE substrate); Macaulay body = plain-f64 SSE2 with
redemption separate, weights grouped `(diff·cash)/disc`, publication
`num/den/f` over its OWN sums. Accrued span A = CoupDaysBS
`diff360_us(prev, settle, ModifyStartDate)`, NOT plain `us_30_360`: the two
differ only on 31st/month-end settlements by adjustment ORDER (a 1-integer A
difference ≈ 2.5e13 ULP of duration).

The gates earned their keep AGAIN: the b45 pre-registered gate (456/540)
exposed the month-end A-convention break that b44 could not see; the fix is
one line; the fresh month-end-majority b46 then gated clean — DURATION
641/720 exact / all misses ±2 / zero material, MDURATION 644/720 mirroring
exactly (the `/(1+yld/f)` staging is validated). b44 final 6217/6360.
Suite 1,609 green, 8 pins added (both former catalog witnesses now exact).

Open residual (NOT accepted): the off-coupon ±1-2 ULP class
(`agentX_b44_residual143.json`, symmetric {−2:14, −1:59, +1:56, +2:14}) —
the SAME shared fractional pow-chain wall PRICE left open; one cross-lane
probe (b41/b43 family) covers both.

Three same-day landings from one cluster: PRICE (b38 945/945), ACCRINT
(b40+b42 145,607/145,620), DURATION/MDURATION (b46 641+644/720 ±2). Every
lane followed hypothesis-first → lattice → pre-registered held-out gate,
and TWO of the three had a wrong sub-rule caught only by the gate.

## b43 probe — the 0.0615 bistables SHARPENED (2026-07-20, coordinator)

Rate-ULP ladders (0.0615 ± 32 ulps × the 12 unique bistable rows, 780 live
rows): **the flip is ISOLATED at exactly the double nearest 0.0615 — all 64
neighbors bit-exact on all 12 ladders.** A staging difference would flip in
bands or scattered points; an isolated single-input flip recurring across
different bonds/dates/calc-paths is a TIE/publication phenomenon. Second
selector: all 13 rows are basis 3 (canonical 182.5) or basis 1 (actual) —
never 0/2/4 — so the fractional/irregular divisor is a co-condition.
Refuted on-row: 15-significant-digit publication snap (4/12, and the
failures go the WRONG WAY — ours is the short decimal, Excel's the
neighbor); naive DR-final-multiply and DR-coup-chain (0/12, but the probe
reconstructed `a` circularly from our own result — REDO with the model's
exact day-count quantities before trusting this refutation).
NEXT PROBE (banked, offline-ready): recompute each row's exact `a` from
agentW_model2.py quantities, race {SSE2, DR-final, DR-all, extended-a}
variants at 250-bit reference to find which op hits a 53-bit tie at exactly
the 0.0615 double; then a par-ladder (997.5 ± k·ulp) to separate
rate-tie from product-tie. Corpus: batch/answers-b43-accrint.json.

---

# Lane D — PMT / annuity ring (G6-01), 2026-07-21 (coordinator, Opus)

The financial-time-value ring (PMT/PV/FV/IPMT/PPMT/CUMPRINC) attacked with the
**metamorphic-sibling method** on the disjoint 875-row held-out corpus
(`answers-pmt-heldout.json`) + the 48-row r0. New tooling:
`tools/calc_graph_racer/src/bin/race_pmt_substrate.rs`, `race_pmt_x87stable.rs`;
`work/w109/G6-solvers/pmt_combine_search.py`, `gen_pmt_meta.py`, `pmt_meta_test.py`.

## Finding 1 — FV/PV and PMT use DIFFERENT algorithms (metamorphic proof)

`annuity_family_race.py` (re-run): **FV var0 = 149/149 and PV var0/var2 = 48/48
bit-exact** with the *naive FORWARD* binexp kernel in **plain SSE2 double**:
`P = binexp(1+r, n)` (LSB-first square-and-multiply), `q = (P-1)/r`,
`tf = 1 + r·type`, `fv = -(pv·P + pmt·(tf·q))`, `pv = -(fv + pmt·(tf·q))/P`.
So Excel's forward annuity factor is PINNED bit-exact.

PMT reuses none of it. Metamorphic harvest (`gen_pmt_meta.py` → 621 FV probes,
`answers-pmt-meta-fv.json`): `P = -FV(r,n,0,1,0)` and `tf·q = -FV(r,n,1,0,ty)`
give Excel's OWN internal factors. Feeding them into `-(pv·P+fv)/(tf·q)`
(`pmt_meta_test.py`) scores **242/923 and is 0/109 on every small/tiny/negative
rate**. If PMT shared FV's forward factor this would close. It does not →
**PMT is a numerically-STABLE (discount) form; FV/PV are the naive forward form.
The siblings do not share the annuity factor.** (Human-written code reused the
helper for FV/PV but PMT's author chose a cancellation-safe path.)

## Finding 2 — the residual is the transcendental primitive, nothing else

Discount identity: `em = expm1(-n·log1p r)`, `v = 1+em = (1+r)^-n`,
`pmt = (pv + fv·v)·r / (tf·em)`.

Held-out ceilings (ranked by held-out, overfit-guarded):
- `race_pmt_substrate` (SSE2 body, x87/CRT transcendentals): champion
  **482/875** at `L=log1pCR E=expm1_internal arr=num/den·r`. Prior forward-pow
  zoo (`fit_pmt_stores`) ceiling was ~57% — same ballpark, different family.
- `race_pmt_x87stable` (WHOLE body per-op x87 double-rounded + x87 log1p/expm1):
  **460/875**, and **completely invariant to the 8-bit store-mask** (every mask
  ties). So body precision (SSE2 ≈ x87-DR) and spill-staging are NOT the gap.
- `pmt_combine_search` (plain-double forward final-combine, 23 arrangements):
  ≤262/875, small-rate-catastrophic — forward is wrong for PMT (confirms F1).

**n=1 isolation lane** (125 rows, pure `expm1(-log1p r)` test): hard ceiling
**62-65/125** identical across the plain closed form `-(pv·(1+r)+fv)/tf`, every
discount provider, forward and x87-DR. The misses **sign-flip with rate
magnitude** (Excel more-negative than the naive form at small rate, less-negative
above ~0.8%). That sawtooth = the exact rounding point of Excel's `log1p`/`expm1`,
which is **none of**: portable-CR `log1p`/`expm1`, the identified internal
Kahan-corrected F2XM1 `expm1_internal`, the raw base-2 hardware `fyl2xp1`/`f2xm1`,
or `ln(1+r)` (forming 1+r first). Ruled out by elimination: form, body precision,
store-mask, and all four known primitives.

## Status + NEXT PROBE (banked)

PMT = stable discount form; residual localized to Excel's **exact annuity
`log1p`/`expm1`** (a bespoke or CRT routine we have not reproduced). This is a
primitive-ID problem of the same class as the G3 internal-exp/expm1 lanes
(task #16, agent-solved). NEXT: isolate Excel's `em` bit-exactly via the
fv=0/ty=0 rows (single division `pmt = pv·r/(tf·em)` → em recoverable to ~1 ULP
by the exact-interval instrument, cf. b34), tabulate em(r,n) vs candidate
routines, and micro-stage search {raw-F2XM1-no-Kahan, RZ vs RN final store,
alt reduction, MSVC/Cephes expm1 forms} at 250-bit reference. CUMPRINC/PPMT/IPMT
(G6-07) inherit PMT's residual and close with it. Corpora ready:
`answers-pmt-heldout.json`, `answers-pmt-meta-fv.json`.

## Lane D continued — em-isolation capture (2026-07-21, Fable-designed probe)

Fable consult reframe: the PMT residual has TWO sources — (a) the small-rate
one-signed lower branch = the em primitive (a fixed-sign absolute error amplified
by 1/|em|), (b) the +branch = body/arrangement. Decisive probe (`gen_pmt_em_probe.py`
→ 13,752 live rows `answers-pmt-em.json`): **fv=0, type=0, pv∈{1,1.5}** collapses
the arrangement to `pmt = RN(pv·r/em)`, so `fl(pv·r/em_candidate)==pmt_excel`
directly scores each em routine. r log-spaced [1e-6,0.05] sign-mirrored ×
n∈{1..480 mixed-popcount}. **This grid IS Excel's em oracle** (recover em = r/pmt).

Racer `race_pmt_em.rs` (REAL x87 F2XM1/FYL2XP1, mpmath can't emulate the
instruction chain). Scores on the pv=1 collapsed grid (6876 rows):
- **internal-Kahan expm1 (on log1p-CR): 4050/6876 = 59% — BEST**, and UNIFORM
  across every |t|=|n·log1p r| regime (59% at |t|<0.25, 0.25-1, 1-4, 4+).
- CR expm1 48%; raw F2XM1-fold RN 48%, RZ/chop 37%; exp(t)-1 12%; no-fold 12%.
- Pure x87 SPILL (em EXTENDED, single final store, no double-store of em): 45% —
  WORSE than storing em to double first. So em IS materialized to double before
  the divide; the body is not a single-store extended spill.
- `v = pow_chain(1+r,-n)` (forming 1+r, Excel's POWER pow): **refuted, <1%
  everywhere** — the annuity does NOT form 1+r; it uses a genuine log1p.
- Regime-split (expfold small-|t| / powchain large-|t|): max 792/6876 — worse
  than uniform internal. NOT regime-split.

**Localization (final for this cycle):** PMT = stable discount form, x87
substrate, em = internal-Kahan-style F2XM1 expm1 on a natural log1p; the residual
is a UNIFORM ~1 ULP difference between `excel_expm1_internal` and Excel's exact
annuity expm1 on ~41% of the isolated grid — a sub-op rounding in the shared chain,
not a wrong family/regime/form. RULED OUT this cycle: forward decomposition,
Gnumeric forward-stable, body precision (SSE2≈x87-DR), store-mask, extended-fold
spill, chop store, exp-1, pow_chain(1+r,-n), ln(1+r), regime-split. **NEXT** (clean
handoff): micro-stage which sub-op of the internal-Kahan chain (the fFEXP final
store RN vs the Kahan `(u-1)·t/ln u` correction ops, or the log1p delivery) rounds
differently, scored against the 13,752-row em oracle; recover em=r/pmt to bits via
the exact-interval instrument and fit the per-op rounding. CUMPRINC/PPMT/IPMT
inherit + close with it.

---

# Lane E — RATE (G6-05), 2026-07-21 (agent-R)

RATE solver identified via the metamorphic lever (RATE finds r where FV(r,…)=fv;
FV is the closed forward-binexp balance). SETTLED: **forward-difference Newton in
r-space** (IRR's v-space sibling), FD step h=1e-6·x, stop |f|<1e-7 publishing the
stepped iterate, cap ~100, #NUM! on cap/domain/non-finite. Balance = pv·(1+r)^nper
+ pmt·(1/r+type)·((1+r)^nper−1) + fv. **POWER = x87 87tran exp·ln** (confirmed by
#NUM! basin: x87chain 15/15 vs binexp/powf 10/15). **Basin 116/116** all corpora.
Secant refuted (false-converges on #NUM! rows). OPEN: exact bits 0-2/101 — the
balance catastrophic-cancellation op-graph (~1 ULP-of-f amplified 5-60× near root),
not closed by {dbl, Ext80, per-op-DR}×{5 pow}×{4 deriv}×{2 arr}. Racer race_rate.rs.
Next lever: r1/r2 near-root one-step ladder (single Newton step, no trajectory
chaos) to pin the balance spill + FD-h form.

# Lane D — PMT combine (G6-01), 2026-07-21 update (coordinator + agent-Q)

The combine (given em) is NOT a single-shared-quantity form: control-clean tests
(agent-Q) show single-em all-8-pv REAL 7.1% vs mismatched-CONTROL 0.0% (genuine but
only 7%); forward-(P,q) "82%" was pure over-fit (control 80%). Coordinator's 256-
CONSECUTIVE-pv sweep (answers-pmt-combsweep.json, over-fit-proof) independently
refutes every 1-/2-constant form (shared-em ≤214/256, forward-(P,q) no fit even
P±500ulp, fused/recip/x87-div all ≤205/256). ROBUST facts: pmt(pv)/pmt(1) within ≤1
ulp of pv (pv scales a base); the residual vs RN(pv·base) is ONE-SIDED (Excel
systematically MORE negative: {0:112,+1:126,+2:13} at (0.05,12)) → an extended
accumulator / truncation biasing high; pv·r is NOT pre-rounded (exact-num divide 62%
> pre-rounded 55.6%). CONCLUSION: the combine carries a SECOND independently-rounded
(r,n)-quantity (the forward P,q pair or v alongside em), with an x87 extended
intermediate. Path: pin the 2nd quantity (v) model-free from the fv≠0 rows
(answers-pmt-fvty.json) + gold em, over-fit-controlled; validate the final op-graph
on the 256-consecutive-pv sweep. Fable consult running on the exact x87 spill.

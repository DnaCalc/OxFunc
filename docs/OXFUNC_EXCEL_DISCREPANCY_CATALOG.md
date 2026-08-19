# OxFunc ↔ Excel Discrepancy Catalog

Status: `active_canonical_tracker`
Last reconciled: `2026-08-19` (Poisson CDF identified as the even-df chi
right-tail: `POISSON.DIST(k,μ,TRUE)` = `CHIDIST(2μ,2(k+1))` 70/70. A
published-PMF fold is not the graph (45/70). CDF now dispatches `k=0` through
`EXP(-μ)` and `k≥1` through GRATIO `Q(k+1,μ)`. PMF path unchanged. G3-01
still open for the Poisson PMF body, remaining GRATIO, and ERF.)
Previous reconcile: `2026-08-09` (current-build G6-06 IRR objective-graph
decomposition: a 300-row discovery plus a 900-point worksheet-NPV companion
separates worksheet evaluator publication from IRR's private objective. The
worksheet evaluator snaps 18 nonzero near-cancellations to +0 under a
scale-relative threshold bracket, including through a referenced raw-NPV cell;
IRR inherits none of those 18 snap decisions. Raw worksheet NPV remains
non-exact (best `636/900`), and the best frozen no-snap IRR graph is only
`44/72` on the guaranteed two-step subset. The 180-row heldout remains sealed;
G6-06 stays open and explicitly partial.)
Previous reconcile: `2026-08-09` (current-build G6-03d PRICE and G6-03c DURATION
residual-graph hardening: 528 PRICE, 264 DURATION, and a frozen adaptive
72-row PRICE companion were captured under build 20228/CV2/Value2/NoCache.
There is no exact survivor in the fixed, retained-PC64, factorized-coupon, or
association families. PRICE's leader is `571/600`, with all 29 misses exactly
-1 ULP; DURATION's leader is `237/264`. The coarse Chain-pow, forward-fold,
separate-redemption structure remains supported, but the former attribution to
a shared fractional-pow wall is withdrawn: retained pow lifetimes do not help,
and an accumulator/publication axis remains missing. Both heldouts remain
sealed and both catalog rows stay open and explicitly partial.)
Previous reconcile: `2026-08-09` (current-build G6-05 RATE discovery hardening:
the former schedule/substrate wording is demoted from identification to an
open graph. A frozen 256-row cancellation-tuned discovery set leaves zero
survivors across 13,824 balance/FD/update graphs and 7,864,320 inline-helper
variants; the best exact score is `2/256`. A paired 512-row public-FV capture
narrows that helper to `502/512` but exposes a separate cancellation/small-rate
sub-lane. Unstepped-current publication is refuted on all 256 rows; first-step
and stop-and-publish-next schedules remain observationally entangled until the
internal objective is exact. The disjoint heldout remains sealed and RATE stays
open and explicitly partial.)
Previous reconcile: `2026-08-09` (current-build G3-04 GROWTH/LOGEST and G6-07
CUMPRINC discovery checkpoints are recorded without promotion. The old
two-control GROWTH prediction-exact claim is retracted; no single-predictor
survivor exists. CUMPRINC is decomposed into payment, hidden principal, and
range-fold layers, but its best oracle-blind candidate is only `190/540`.
Both rows remain open and explicitly partial.)
Previous reconcile: `2026-08-09` (G6-01 PMT current-authority hardening: hidden
low-word and tested smooth correction families are bounded-negative; the
`832/832` power-of-two timing identity is explicitly local; a frozen
general-rate gate remains non-exact at best `378/480`. The July irreducibility
framing remains retracted, EXT6 is unfinished through logged shard `191/400`,
and BUG-FUNC-015 stays open and actionable.)
Previous reconcile: `2026-08-09` (the COMBIN and COMBINA current-reference
sublanes of mixed G4-04 are `closed_signed_off`. COMBIN's cyclic stored-x87
body remains exact and now includes shared DAZ plus the exact truncated-n
admission ceiling; COMBINA uses DAZ, separate truncation, its zero-pool/
asymmetric-guard order, and the transformed COMBIN route. Original/new typed
production replay is `64767/64767`; the central COMBINA gate is `2048/2048`
and the genuinely fresh paired admission gate is `220/220` without refinement.
Beads `oxf-jwh5.9` and `oxf-jwh5.11` are closed. G4-04 remains one open mixed
row only for ERF/ERFC.PRECISE, so the catalog's mechanically reconciled open
count remains 16; the implementation landed in `3f31f44`, and the wider W109
campaign remains partial.)
Previous reconcile: `2026-08-09` (the exact current-reference CONVERT graph landed
in `8ef5cac`: every linear product, quotient, and separate decimal-prefix
multiply publishes through x87 PC64-to-binary64 double rounding; exact unit
tables and direct temperature routes are pinned. The frozen prior-disjoint
publication gate is `10418/10418` and compiled production replay is
`34189/34189`; bead `oxf-jwh5.8` is closed signed off, so G4-05 was retired.
The wider W109 campaign remains partial.)
Previous reconcile: `2026-08-09` (the ACOTH graph landed in `7f7eac9` and
replays `268769/268769`: native binary64 ratio add/sub plus stored-x87 divide
below exact threshold `0x400d92b14ec204f3`, stored-x87 direct inverse
odd-power series above it, and positive-zero reciprocal flush. The frozen
post-selection held-out is `66552/66552`; BUG-FUNC-027 CLASS-C5 and bead
`oxf-jwh5.7` are closed signed off, and G4-03 was retired. Other BUG-FUNC-027
subclasses and the wider W109 campaign remain partial.)
Previous reconcile: `2026-08-09` (`bce3558` landed the exact current-reference
MINVERSE graph: right-looking Doolittle LU with all eight arithmetic sites
published through x87 PC64-to-PC53 double rounding plus completed-output +0
normalization. Production replays `1599/1599`; BUG-FUNC-025 and bead
`oxf-dzfk` are closed signed off, and G5-01 was retired. The separate 1x1
final-cell publication seam and the wider W109 campaign remain partial.)
Previous reconcile: `2026-08-09` (`a03a75f` landed the exact current-reference
ATANH three-regime graph. Dense discovery, exact boundary bisection, a retired
refinement set, and a fresh post-selection held-out replay `20780/20780` typed
bit outcomes; BUG-FUNC-027 CLASS-C4 is `closed_signed_off`, bead `oxf-jwh5.6`
is closed, and G4-02 was retired. Other BUG-FUNC-027 subclasses and the wider
W109 campaign remain partial.)
Previous reconcile: `2026-08-09` (`ed9f222` landed the corrected worksheet-COS
odd-quadrant tangent-square publication graph and the dependent BESSELJ
composition. COS replays `2561/2561`, production BESSELJ replays `794/794`,
BUG-FUNC-046/047 are `closed_signed_off`, and G4-06/G4-07 were retired. The
wider W109 campaign remains partial.)
Previous reconcile: `2026-08-09` (`cd1f9fe` landed ACCRINT's exact final
publication graph; BUG-FUNC-030 is `closed_signed_off`, so G6-02 was retired
after `146850/146850` exact replay. The wider bond/financial family and W109
campaign remain partial.)
Previous reconcile: `2026-08-09` (`876635e` landed the exact current-reference
`EFFECT`, `RRI`, and `NOMINAL` publication graphs and their tests/evidence;
BUG-FUNC-043/044/045 are `closed_signed_off`, so G6-12/G6-13/G6-14 were retired
from this open-only catalog. The wider W109 financial family and campaign remain
partial.)
Previous reconcile: `2026-08-09` (W109 campaign-state intake: registered the
confirmed `EFFECT`, `RRI`, and adjacent-family `NOMINAL` discrepancies as
G6-12/G6-13/G6-14; replaced the stale duplicate G6-03b PRICE membership with
the confirmed G4-06 BESSELJ internal-cosine residual; reopened the shared
worksheet-COS substrate as G4-07 after fresh phase witnesses refuted the former
universal G4-01 sign-off; corrected the G6-01 PMT boundary wording to the
superseding 2026-07-25 bounded-search result and retained coefficient
recovery/larger-graph search as active clean-room lanes)
Previous reconcile: `2026-07-18` (G3-01 lane-1: distribution pow pinned as POWER's chain
without the 0.5→sqrt shortcut (`excel_pow_chain` landed); WEIBULL.DIST + EXPON.DIST
bodies identified as legacy x87 per-op-double-rounded units; held-out evidence
5,999/6,000 and 4,000/4,000 respectively, so WEIBULL remains partial)
Previous reconcile: `2026-07-17` (G3-01: chopped series-exp identified + landed with the
a==1 dispatch — CHIDIST 152/195, GAMMA.DIST 159/268; *INV lattice inverter +
published-surface stagings landed for CHIINV/FINV/TINV; see the G3-01 row)
Previous reconcile: `2026-07-16` (G3-01 gamma-side substrate identified as plain-double
DCDFLIB GRATIO — the prior x87-extended-CF verdict corrected; see the G3-01 row)
Previous reconcile: `2026-07-11` (live Excel build `20131`; trig `G4-01`
identified and signed off `5425/5425` across all six functions; YIELDMAT `G6-09`
identified and signed off `1250/1250`; NPER `G6-08` identified
and signed off `1286/1286` + `7/7` error rows; XNPV `G6-11` identified
by the W109 calculation-graph search, repaired, and signed off `1530/1530`
numeric + `175/175` error rows; previous reconcile 2026-07-10: stale YIELDDISC
row removed; MMULT/MINVERSE `1x1` publication observations moved to Category 1;
TBILLYIELD association repaired and signed off `2156/2156`)
Last history cleanup: `2026-06-26`

## Purpose

OxFunc targets bit-exact emulation of Excel for every in-scope function and
operator. This file is the single live worklist of every open OxFunc-vs-Excel
discrepancy that OxFunc can evaluate locally: Category 2, context-free cases
under [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md).

Context-sensitive Category 1 discrepancies do not live here. They live in
`smart-fuzzer/corpus/context_sensitive_catalog/` and are evaluated downstream
through OxCalc -> OxFml -> OxFunc.

The durable method for identifying the calculation graph behind an open row is
[SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md](SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md).

Cases where OxFunc already agrees with Excel but Excel itself deviates from the
mathematically-most-accurate result live in
[EXCEL_MATH_DEVIATION_CATALOG.md](EXCEL_MATH_DEVIATION_CATALOG.md), not here.

## Maintenance Rules

1. This is the only open-status tracker for Category 2 discrepancies.
2. Fixed, signed-off, stale, or resolved items are removed from this file.
3. Detailed history, root cause, and evidence stay in `docs/bugs/streams/`,
   `docs/bugs/BUG_STREAM_REGISTER.csv`, git history, and run artifacts.
4. A function may appear more than once when it has distinct discrepancy types.
5. Newly found local discrepancies get a row here immediately, even before a full
   bug stream exists.

## Legend

Severity:
- `STR`: structural mismatch: wrong kind, error code, shape, array behavior, or admission.
- `NUM-L`: numeric, large: materially wrong number or `> ~2` ULP drift.
- `NUM-S`: numeric, small: `≤ ~2` ULP drift.

Maturity:
- `M0 noted`: witnessed, not minimized, no repair.
- `M1 tested`: minimized reproducers or focused tests exist.
- `M2 repair-tried`: repair attempted or repair direction proven, not landed.
- `M3 fixed-unsigned`: fix landed locally and locally green, awaiting live-Excel sign-off.
- `HO downstream`: OxFunc-side handled, blocked on downstream or seam acknowledgement.

## Current Summary

Open Category-2 rows: `16`

| Group | Current rows |
|-------|--------------|
| G1 error-code/domain guards | 0 |
| G2 structural kind/shape/admission | 0 |
| G3 special/statistical numeric exactness | 7 |
| G4 elementary/trig numeric exactness | 1 |
| G5 matrix numeric/shape | 0 |
| G6 financial exactness/solver | 8 |
| G7 comparison/misc semantics | 0 |
| G8 untriaged inbox | 0 |

W108 resolved (bit-exact via the x87 backend, removed from tracking): `EXP`, `LN`, `LOG10`,
`LOG(x, base)`, and `POWER` — 64-bit Excel computes these with the legacy x87 CRT
transcendental chain (`87tran.asm`, CW `0x133F`), reproduced bit-for-bit by
[`crate::excel_numeric::x87`] on the reference x86-64 host. `POWER` (BUG-FUNC-042, signed
off) is the fractional-path `exp(y·ln x)` with the `y<0` reciprocal staging and the
`|y|==0.5→sqrt` special case (715/715 live rows). `EXP`/`LN`/`LOG10`/`LOG` were never catalog
rows (W108-A research findings). Many small-ULP G3/G4 residuals whose kernels call `exp`/`ln`
internally may now be closable by routing those calls through the x87 backend.

## Bounded Reconnaissance Evidence — 2026-07-10

Every open row below now has a stable `G*-NN` reconnaissance id, two exact-input
test cases, live Excel 16.0 build 20131 result bits, and a bounded calculation-path
search map:

- [reconnaissance report](function-lane/DISCREPANCY_RECONNAISSANCE_PASS_20260710.md)
- [48-case corpus](../smart-fuzzer/corpus/discrepancy-recon/catalog-row-recon-v0.json)
- [exact result ledger](function-lane/DISCREPANCY_RECON_RESULTS_20260710.csv)
- [calculation-path map](function-lane/DISCREPANCY_CALCULATION_MAP.csv)

The path entries are black-box hypotheses, not implementation claims. They
explicitly enumerate strict-f64, x87 extended/store-boundary, association,
accumulation, table-constant, and solver-schedule alternatives for future search.

## G1 — Error-Code And Argument-Domain Guards

No current open rows.

## G2 — Structural Kind, Shape, And Admission

No current open rows.

## G3 — Numeric Exactness: Special And Statistical Functions

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G3-01 — BETAINV, CHIDIST, CHIINV, FDIST, FINV, GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, TDIST, TINV (+ CONFIDENCE.T, Z.TEST unprobed) | Distribution scalar numeric drift, `1`-`28` ULP. **W109 (2026-07-16): gamma-side substrate IDENTIFIED — the DCDFLIB/NSWC GRATIO branch structure (TOMS 654, DiDonato–Morris), plain SSE2 DOUBLE** — the 2026-07-14 "x87 80-bit extended CF" verdict is WRONG (extended only proved convergence; the true-x87 `check_igamma` race killed the extended family at stage A). Evidence (692-row multi-view corpus, 12 surfaces collapsed: legacy≡modern bit-for-bit, β-scaling transparent, one internal `P(a,x)`): per-branch differential match of a faithful GRATIO transcription — **closed-int `91%`**, asymp 4/6, Temme(a≥15) 5/5, Taylor 48%, erf-routes fail → **`a==1` wrapper dispatch to the exponential CDF (−expm1(−x), proven via `a=1+2⁻²⁰` NOT clean)**; `a==0.5`/half-integer paths use **Excel's OWN near-CR erf/erfc** (NSWC + Cody CALERF ruled out; 352-pt ERF.PRECISE/ERFC.PRECISE ladders captured → same sub-lane closes ERF.PRECISE/GAUSS G4-04/G3-07); fractional-a normalizer = **internal Γ (the G3-02 wall, now MEASURABLE per-a through this window)**; Taylor micro-staging open (a=2 slice, Γ exact, is the clean enumeration target). Beta side **CONFIRMED = BRATIO (TOMS 708)** (2026-07-17 agent sweep: bpser-in-plain-double BEATS correctly-rounded on FDIST/TDIST = literal code identity; accurate-complement argument stagings pinned; one-tail=0.5·two-tail bit-exact; **bgrat is the Excel-custom sub-kernel**, branch battery cached). Gamma-side staging CORRECTED (forward-summed series, 1/a outer; normalizer = CR-Γ±1 ≡ exp(internal lgamma), NOT NSWC gamma — a G3-02 measurement window); a=2 residual = one-sided x87-exp signature, next lever = real fFEXP chain. ***INV = fully-converged near-CR roots of Excel's own forward** (gaminv schedule RULED OUT; κ-correlation 0.97-0.995) — no solver-VM needed, just the forward + converge-to-last-bit. **GRATIO KERNEL PORTED+LANDED** (c71cde5/fa275e0): CHIDIST 12→144/195 exact (catastrophics eliminated), GAMMA.DIST 64→137/268 max 21 ULP, 1507 tests green. **W109 session 5 (2026-07-17): the series exp is CHOPPED — Excel's gser `r = exp(t1)/Γ` publishes a TRUNCATED (round-toward-zero) exp** (floor-of-true scores `38/45` on the implied-exp corpus vs CR 25, fdlibm 28; every real 2010-era MSVC CRT exp refuted by direct 32-bit binary probes incl. msvcr90 9.0.30729 via SxS manifest — the CRT SSE2 exp rounds one-sided HIGH, the mirror of Excel). Chop is CALL-SITE-LOCAL: a==1 wrapper (nearest exp/expm1, now dispatched inside gratio), CF and a<1 paths are NOT chopped. Landed as `exp_rd` (double-double, validated 0/25k vs floor-exp) → **CHIDIST 152/195, GAMMA.DIST 159/268** (b20 held-out gate: +3/111, fresh a-slices). ***INV inverter LANDED**: float-lattice bisection to adjacent doubles (early-stop bisection had `+880k`/`+1.9M` ULP catastrophes at small roots — b14: GAMMA.INV 8→18/60 worst −16, BETAINV 2→4/30 worst +13) + **invert-the-published-surface staging** (CHIINV roots Q directly, held-out-confirmed b19 15/40 vs 6/40; FINV roots the FDIST complement form 0→3/32 with small-p bias collapsed; TINV roots the two-tail surface, residuals −238→±7). erf "fine comb" REFUTED as grid aliasing (b18 matched-resolution scans, 242k rows): the erf last-op fingerprint is now the per-binade phase-gradient; the hunt merges with the internal exp/log identity. Beta tail (b21 discriminator, 127 rows): **family PROVEN = DiDonato–Morris TOMS-708 Eq-9 bgrat** (at k=2 Excel sits +41..+63 ULP from truth yet within ±7 of every Eq-9 realization across 25 rows — intrinsic asymptotic method error), realization exact-matched by NONE (NSWC grat1, GRATIO-sub, Boost 1.35–1.42, Cephes, AS63, NR all fail; chopped-exp inert here). **BRATIO PORTED TO PRODUCTION (2026-07-17 session 6)**: op-for-op from the validated transcription (bit-identity 20,008/20,008 vs the spec), NR continued fraction deleted, accurate-complement wrapper stagings landed (FDIST/TDIST/TTEST + all F/T inverter closures). Held-out b22 (671 fresh rows): 167→**285**/655 exact, worst ±145→126; b21 deep tail: 0/127 worst **8,848** → 4/127 worst 56 (catastrophic tail class eliminated); 422 improved/60 regressed, regressions confined to the bgrat wall. Open: bgrat-tail realization (Eq-9 family, arithmetic unmatched), integer-shape fast-path routing, A/B-bounds staging. **Lane-1 (2026-07-18): distribution pow CLOSED — `exp(RN53(RN64(y·ln x)))`, POWER's chain WITHOUT the 0.5→sqrt shortcut (b24 re-race with the real chain 33,145/33,145; b27D product-staging discriminator 113/113; landed as `excel_pow_chain`). WEIBULL.DIST + EXPON.DIST bodies IDENTIFIED as legacy x87 per-op-double-rounded compilation units (WEIBULL pdf = division-first left-to-right C expression, tree×spill-mask race 1,600/1,600) and SIGNED OFF: b28 held-out 5,999/6,000 (99.983%), b28c 4,000/4,000 (100.000%). Two body classes (plain-SSE2 GRATIO/BRATIO vs x87-DR closed-form) coexist behind the 2010 stats surface.** **Lane-2 (2026-07-18): POISSON pmf = TWO routes (k=1 extended-composed direct; k≥2 = Loader saddle-point dpois BIT-EXACT at λ≳14 — the "direct product proven" claim withdrawn, k=0 window is route-blind); BINOM = Loader dbinom control flow PROVEN (b29b p<0.1 branch-flip 383/400), general-k sub-staging enumeration bounded (implied-argument decode instrument, b29 banked). Lane-3 (2026-07-18): BOTH OxFunc-side integer-shape fast paths REMOVED (gamma one catastrophic ±4,400 ULP, silently overriding the landed GRATIO; b30 proves Excel has NO integer beta path; A/B-bounds staging broadly confirmed); production ≡ identified substrate bit-for-bit — GAMMA.DIST 337/446, b26 1,615/4,100 (worst −10), b22 293/671. Lane-8 (2026-07-18, agent-T/U): BINOM.DIST pmf IDENTIFIED as R dbinom_raw with THREE recovered source details (log1p form; log1p REALIZED AS TWO SEPARATE LNS - msvcr100 has no log1p, the expm1 porting hole again; the lc grouping ((s1-s2)-(s3+bd0_1))-bd0_2) + extended-entry fFEXP chain, and LANDED (85e91e4): b29 8.76%->49.81%, fresh b36 gate 49.59% overall (k=0 81.6%, k=n 96.4%, general-k 37.9% with quantified n-dependence). Row OPEN: second lc-flip source + small-operand bd0 bodies (exact residuals in agentT/U_results.md); NEGBINOM measured NOT to inherit (10.4%) - separate route. Lane-3b (b31): GAMMA.DIST pdf ≠ log-composed ≠ R dgamma; consolidated "closed-form pdf extended-composition body class" wall named (with POISSON small-λ/k=1).** See [W109_G3-01_GRATIO_IDENTIFICATION_20260716.md](function-lane/W109_G3-01_GRATIO_IDENTIFICATION_20260716.md) + [W109_WALL_CLUES_LEDGER.md](function-lane/W109_WALL_CLUES_LEDGER.md). | NUM-L | M2 | BUG-FUNC-021 / KED-STAT-001 / W109 GRATIO identification |
| G3-02 — GAMMA (+ GAMMALN substrate) | W109 re-scoping (2026-07-11): the row was under-scoped — a fresh 156-row live sweep shows the POSITIVE side is `0/79` exact (up to `1370` ULP at large x) and negatives reach `810k` ULP; the recon corpus had only probed two negative points. Ruled out: current Lanczos log-domain kernel; `x87-EXP(published GAMMALN)` composition (6/79, errors grow with |lnΓ| — Excel uses a HIGHER-precision internal lgamma than published GAMMALN). The former internal-extended-lgamma implication for COMBIN is superseded by the exact cyclic stored-x87 graph signed off in `c879f3f`; GAMMA/GAMMALN remain independent. **W109 run-2 identification (2026-07-11, supersedes the Cephes-small claim): GAMMALN `x>=11` IS the plain-double Cephes Stirling tail + UCRT-class log** — `136/139` bit-exact on a 361-row corpus incl. dense grids; the 3 residual rows are sub-ULP internal-CRT-log deltas; the boundary sits in `(10.25, 11.0]`. The `(0,11)` core is a **custom Microsoft rational** (accuracy <=3.5 ULP vs true, exact zeros at 1 and 2, real arithmetic at integer args, `-log(x)+poly` form below 0.5): Cephes-small, fdlibm, UCRT, R/SLATEC, Cody, DCDFLIB, AS245, GSL, NR and Boost are ALL ruled out bit-exactly under every staging (see W109_GAMMALN_IDENTIFICATION_20260711.md). **W109 session 6 (2026-07-18): the (0,11) core is STRUCTURALLY IDENTIFIED — the Cody & Hillstrom SPECFUN DLGAMA skeleton with Microsoft-RE-FIT coefficients** (zero-capture proof from existing data: downward-recurrence identity `excel(x)==double(excel(x+1)−log(x))` bit-exact 32/32 below 0.62 = Cody's PNT68 control flow; `GAMMALN(4)=CR(ln 6)` published verbatim = Cody's band-4 D4-anchor form; full band skeleton mapped, no other internal edges). Published Cody coefficients score b4 231/385 worst 3; ±1–3-ULP hill-climb stalls → decimal-conversion-error hypothesis FALSIFIED — a genuine re-fit on a flat GN manifold (b2 450/1247, b4 238/385 worst 2). b4 miss profile suggests re-fit interval [4,11] (Stirling switch moved from Cody's 12). Best composite forward model 1096/2850 (38.5%) worst 6 ULP. Batteries designed+queued: boundary pinning (1,793) + core recovery (9,642: peel ladders, adjacent-double clusters, recurrence partners, held-out sweep). 1967 tables obtained (user) + validated: verbatim RULED OUT — Excel = the 1967 FORM SET with retuned thresholds and a third Microsoft-refit n=7-class coefficient set. **Round-3 (2026-07-18): structure CLOSED — literal-0.7 threshold pinned to the double, NO Stirling switch at 12 (x≥8 = ONE formula: (x−0.5)log(x)−x+LS2PI + z·w with fdlibm's w1..w6 vector in a non-fdlibm staging, 99.83% worst 2), seam at 8.0 pinned, [4,8) = published SPECFUN P4/Q4 under x87-continuous (worst 1). KERNEL LANDED (commit 223cfa5, agent port bit-identical to reference on 17,003 rows): GAMMALN/GAMMALN.PRECISE 0/79 (worst 1,370) → held-out 316/400 = 79.0% (worst 5).** Open: b1 [0.7,1.5) + b2 [1.5,4) exact coefficients (provisional: 1967-n7/d and gn2/x87-spill per held-out head-to-head; b1 weakest ~31%), the internal-CRT-log ±1 class (7 of 10 x≥8 residuals), [4,8) exact spill pattern; then GAMMA = exp composition (+ sin reflection) and G3-01 fractional-a re-race. **Lane-4 (2026-07-18, agent-S + fresh b32 gate): B2 RE-IDENTIFIED as fully-continuous x87 (same class as B4; noise floor 1.077 vs spill 1.113) and RE-LANDED with LM-refit coefficients — fresh never-probed b32: 549 vs 518/1,200 (a formally-gate-passing but held-out-tainted spill candidate LOST the fresh gate at 505 — the MINVERSE rule vindicated); aggregate all b2 rows +25. B1 CONFIRMED an op-graph wall: published-1967 plain-double is the held-out optimum of the entire (d,s,e,fma,recip)×association family, misses to −3.4 ULP pre-round; next probes = two-step argument reductions / re-weighted-Remez family / outer 2-op mask (agentS_results.md §7). gn2's original fit was mildly held2-contaminated (excluded only `held-`).** | NUM-L | M2 | BUG-FUNC-027 C1 / W109_GAMMALN_IDENTIFICATION_20260711.md / smart-fuzzer/work/w109/G3-02-gamma |
| G3-03 — TREND, LINEST, LOGEST (FORECAST closed) | **W109 sweep (2026-07-12): FORECAST/FORECAST.LINEAR IDENTIFIED and PROMOTED** — Excel computes them via the simple centered kernel (forward sums -> means; fused `Σdx·dy`/`Σdx²` loop; publish `a + b·x` intercept form), NOT the LINEST pipeline; `65/65` bit-exact incl. adversarial (1e12 offsets, n=2, near-constant x); OxFunc rerouted off `trend_kernel` with pinned witnesses. SLOPE/INTERCEPT confirmed already bit-exact (share the kernel, `4/4` each). TREND stays on the least-squares pipeline and publishes different bits. **The July `GROWTH(x*)=b·m^x*` prediction-exact claim is retracted**: its two integer controls admit 240/23,328 tested graphs. Current build-20228/CV2 discovery finds no exact single-predictor `LOGEST` coefficient graph (`270/358` numeric paired cells plus `2/2` structural for the leader). For LINEST, the known slope bias and non-discriminating Batch-D corpus still require design-for-divergence capture (non-integer Scxx, cancellation-heavy Scxy, minimal 2/3-point sets, and ancillary SS/SE outputs). | NUM-L | M2 | G8 probe / `W109_GROWTH_LOGEST_SINGLE_PREDICTOR_DISCOVERY_20260809.md` |
| G3-04 — GROWTH | **Current-reference single-predictor discovery remains open.** Two serialized build-20228/CV2 NoCache rounds cover 1,260 prediction cells plus paired LOGEST outputs. The best tested reconstruction from observed LOGEST cells—raw worksheet-x87 power followed by stored-x87 multiply—is only `666/1240` numeric exact; direct `EXP(a+b*x)` is `610/1240` and misses 18/20 structural `#NUM!` outcomes, while coefficient-publication then POWER/product candidates reproduce 20/20 structural outcomes. `GROWTH(0)` equals the published LOGEST base in 80/83 occurrences, with a separate subnormal +0/#NUM publication seam. No held-out or full-function survivor exists; multivariate, const=false, omitted/default, orientation/shape, coercion/error, coefficient-schedule, and publication lanes remain open. | NUM-L | M2 | `W109_GROWTH_LOGEST_SINGLE_PREDICTOR_DISCOVERY_20260809.md` / G3-04 remains open |
| G3-05 — CHISQ.TEST, CHITEST | **W109 sweep (2026-07-12): decomposition unblock proven** — `CHISQ.TEST(o,e) == CHIDIST(S, df)` BIT-EXACTLY for a specific stored double S (tail cancels in the comparison), so the internal statistic is directly measurable without the gamma substrate: the internal statistic is IDENTIFIED as the plain-double ROW-MAJOR `Σ(o-e)²/e` (offset 0 on the two injective-tail tables of a 4-table live set); the CHIDIST tail half remains on the G3-01 substrate, so ALL CHISQ.TEST drift is inherited. **2026-08-18 inverse identity:** on df=1 the CHIDIST tail is the published ERFC graph `ERFC.PRECISE(SQRT(x/2))` (154/154 live build 20228; `SQRT(x)/SQRT(2)` refuted). That df=1 route is now dispatched through the ERFC kernel; remaining CHISQ.TEST drift is still the ERFC body plus the non-df=1 GRATIO tail. | NUM-L | M2 | recon G3-05 / W109 sweep / `W109_CHIDIST_DF1_ERFC_IDENTITY_20260818.md` |
| G3-06 — F.TEST, FTEST | **Current-build correction (2026-08-09): the July universal external-FDIST decomposition is retracted.** The original 3/3 relation was a bounded, non-discriminating observation. On build-20228/x64/CV2 NoCache discovery, 48 F.TEST rows against 350 live FDIST groups and 96 VAR.S companions admit an external-tail-equivalent group on only `33/48`; direct live F CDF composition leaves that score unchanged. Forward stored-ratio variance candidates score `32/48`, public VAR.S ratio `28/48`. A retired exact-variance boundary set is only `15/24` under direct CDF/right-tail composition, and a 3,975-row correctly oriented inverse-neighborhood refinement finds an external-FDIST preimage on only `4/15` former no-hit rows; the other 11 locally skip the F.TEST target. Both the private F.TEST tail/publication graph and remaining variance schedule stay open. | NUM-S | M1 | `FTEST_VARIANCE_DISCOVERY_CHECKPOINT_20260809.md` / G3-06 remains open |
| G3-07 — GAUSS | Standard-normal `Phi(z)-0.5` drift, `2` ULP on the stable witness; needs the erf/CDF substrate (Phase-5 adjacent). PHI is resolved out of this row (W109 2026-07-11: `RN53(RN64(x·x))` -> x87 EXP -> `RN53(RN64(e·RN(1/sqrt(2π))))` with a live-pinned subnormal publication flush; `764/764` answered rows, see the ruled-out ledger and `smart-fuzzer/work/w109/G3-07-phi`). | NUM-S | M1 | recon G3-07 / W109 PHI closure |

## G4 — Numeric Exactness: Elementary And Trig

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G4-04 — COMBIN, COMBINA (closed); FACTDOUBLE control; ERF.PRECISE, ERFC.PRECISE (open) | **COMBIN and COMBINA current-reference sublanes closed_signed_off (2026-08-09):** COMBIN applies DAZ, rejects remaining raw negatives, truncates, admits `n<=2147483646`, reduces `k=min(k,n-k)`, then uses the cyclic stored-x87 quotient/product graph with `n` multiplied last. COMBINA applies DAZ and separate truncation, publishes one from the zero/zero pool before its asymmetric truncated-n/raw-DAZ-k guard, and delegates `tn+tk-1,tk` to COMBIN. The original COMBIN body is `22242/22242`; new COMBIN admission controls are `2195/2195`; COMBINA is `40330/40330`; combined replay is `64767/64767`. Candidate-frozen gates pass COMBINA `2048/2048` centrally and fresh paired admission `220/220` without refinement. The July GAMMALN/product-impossibility claim is retracted. FACTDOUBLE remains an exact control. **This mixed row remains open only for ERF/ERFC.PRECISE** on the incomplete-gamma staging lane described under G3-01. **2026-08-18:** live Excel publishes `CHIDIST(x,1)` as `ERFC.PRECISE(SQRT(x/2))` and `GAMMA.DIST(x,0.5,2,TRUE)` as `ERF.PRECISE(SQRT(x/2))` (154/154), so those surfaces are extra observations of this same open body. | NUM-S | M2 | BUG-FUNC-027 combinatorial group / W109_COMBIN_IDENTIFICATION_20260809.md / W109_COMBINA_IDENTIFICATION_20260809.md / W109_CHIDIST_DF1_ERFC_IDENTITY_20260818.md |

The former `G4-05` CONVERT row was signed off and removed on 2026-08-09 after
`8ef5cac` landed the exact current-reference table, dispatch, and three-store
publication graph. A deterministic, prior-disjoint `10418`-row frozen
publication gate passed `10418/10418` without refinement; direct replay of
discovery, both explicitly retired refinement attempts, the v3 refinement
battery, publication gate, and Value2 readback control is `34189/34189`.
History, candidate corrections, exact pins, provenance, hashes, and the scoped
Sections 12/14 audit remain in
[`W109_CONVERT_IDENTIFICATION_20260809.md`](function-lane/W109_CONVERT_IDENTIFICATION_20260809.md).

The former `G4-03` ACOTH row was signed off and removed on 2026-08-09 after
the exact graph landed in `7f7eac9`,
the candidate-frozen, prior-disjoint held-out passed `66552/66552` and the
actual production kernel replayed `268769/268769` distinct signed inputs with
zero anomalies. The exact current-reference graph switches at
`0x400d92b14ec204f3` from native-add/sub plus stored-x87 ratio division and
worksheet LN to a direct inverse odd-power series whose reciprocal,
multiplications, divisions, and accumulator additions are each stored through
x87 PC64; a subnormal reciprocal publishes positive zero for both signs.
History, artifact hashes, exact adjacent endpoints, regression gates, and the
scoped audit remain in
[`W109_ACOTH_IDENTIFICATION_20260809.md`](function-lane/W109_ACOTH_IDENTIFICATION_20260809.md).

The former `G4-01` trig row was signed off and removed on 2026-07-11 after a
`5425/5425` corpus. Its evidence remains valid, but fresh build-20228/CV2 phase
witnesses later refuted only the universal raw-FSIN odd-quadrant COS inference.
The correction landed in `ed9f222`: COS retains the exact-one tiny guard and
`FPREM1(|x|,FLDPI/2)` reduction, uses FCOS on even quadrants, and reconstructs
odd-quadrant sine magnitude as signed
`FSQRT(FPTAN(r)^2/(1+FPTAN(r)^2))` continuously in x87 PC64/RN. The selected
graph is `2561/2561` across the original 1020 validation rows, a 1027-row
adjacent/random discovery battery, and a frozen 514-row oracle-blind hold-out;
the original 24-row threshold ladder remains separate guard evidence. SEC
continues to consume the published COS through its identified double-rounded
reciprocal. BESSELJ now consumes corrected COS at both J0/J1 asymptotic sites
and stages only J0 `cosine*P`, producing `794/794`. G4-06/G4-07 are retired;
their history remains in BUG-FUNC-046/047, the calculation map, and the
ruled-out ledger. See
[`W109_TRIG_IDENTIFICATION_20260711.md`](function-lane/W109_TRIG_IDENTIFICATION_20260711.md).

## G5 — Matrix Numeric And Shape

No open Category-2 matrix numeric row remains on the current reference profile.
G5-01 MINVERSE was retired on 2026-08-09 after `bce3558` landed the
per-operation x87-double-rounded Doolittle graph and positive-zero publication
rule. Direct production replay is `607/607` banked + `576/576` retired
refinement + `416/416` frozen disjoint publication = `1599/1599`. History,
candidate corrections, hashes, and checklists remain in BUG-FUNC-025 and
[`W109_MINVERSE_IDENTIFICATION_20260809.md`](function-lane/W109_MINVERSE_IDENTIFICATION_20260809.md).

`MINVERSE(5)` and `MMULT(5,2)` are deliberately absent here. Nested `TYPE`
evidence proves that their function results remain `1x1` arrays; Excel's final-cell
scalar appearance belongs to the Category-1 worksheet publication/comparator seam.
They are now explicit `publication_shape` rows `CSC-0024`/`CSC-0025` under
`smart-fuzzer/corpus/context_sensitive_catalog/`, with downstream handoff `HO-FN-010`.

## G6 — Financial Exactness, Computation, And Solver

Current-authority correction for G6-01 (2026-08-09): the 2026-07-24 phrases
“not black-box-closable”, “needs provenance”, and “boundary proven” are
retracted by
[`W109_G6_PMT_TAKEOVER_BRIEF.md`](function-lane/W109_G6_PMT_TAKEOVER_BRIEF.md)
(2026-07-25). The evidence proves only bounded-negative results over the stated
leaf/op/size families. EXT6 is unfinished (191/400 shards), larger graphs remain
valid, and exact linear-interval coefficient recovery is an active clean-room
lane. The 2026-08-09 intermediate/timing audit hard-refutes direct low-word
delivery, exactly certifies infeasibility for `51/60` tested smooth interval
systems while leaving nine numerical-only, and limits the `832/832` reciprocal
identity to power-of-two timing pairs; its general-rate gate remains non-exact.
See
[`W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md`](function-lane/W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md).
G6-01 is therefore an actionable open discrepancy, not a stopping-rule boundary.

| Function(s) | Discrepancy | Sev | Mat | Evidence |
|-------------|-------------|-----|-----|----------|
| G6-01 — PMT, PPMT (IPMT, CUMPRINC/CUMIPMT adjacent) | **Open current-reference annuity graph.** The landed quotient-first discount structure remains the best supported high-level PMT composition, but Excel’s private `|tau|<1` helper and timing-1 association/publication path are not identified. Current-build hardening (`2026-08-09`) scores the x87-spill helper representative `226/324`; exact-product status is a useful stratifier but not a proved branch; direct hidden-low-word and tested smooth degree-`0..8` correction families are bounded-negative. A power-of-two `fv=0` type-pair metamer selects stored reciprocal multiplication `832/832` versus division `773/832`, but this identity is local: the frozen `480`-call general-rate discriminator has no exact tail graph (best subtractive `r/tf` family `378/480`, only `1/15` contexts exact). July “not black-box-closable / needs provenance / boundary proven” claims are retracted. EXT6 is unfinished (durable log through shard `191/400`); larger graphs, helper-association probes, and PPMT/IPMT/CUM recurrence/publication lanes remain actionable. | NUM-L→NUM-S | M2 | BUG-FUNC-015 / `W109_PMT_INTERMEDIATE_AND_TIMING_DISCRIMINATION_20260809.md` / W109 G6 takeover |
| G6-03 — YIELD | W109 (2026-07-13): the dive-plan premise "forward kernel already bit-exact, YIELD is 100% schedule" is FALSE. The **forward PRICE kernel is not bit-exact**: `pcomp` computes `base^(off+k)` via `base.powf` (Rust powf = `exp·ln` even for integer exponents), but Excel uses the C-runtime `pow` integer special case = **binary exponentiation**. IDENTIFIED + held-out validated (25 live-Excel PRICE points / 5 bonds: **15/15 ident + 10/10 held-out** for binexp-integer + powf-fractional + coupons-first). Ruled out: naive repeated-multiply (5/10, breaks exp≥4), powf/x87·log integer (6/10), getPrice_ redemption-first order (2/5). Residual after the forward fix = **plateau publication**: corrected PRICE is flat at exactly 95 across a ~20-ULP yield band; Excel publishes a specific point (catalog `0x…9983`, par exactly `0.05`). COUPLED: fixing pcomp alone + current bisection regresses par 6→~40 ULP — land the forward fix AND the schedule together. Writeup: `work/w109/G6-solvers/YIELD_PRICE_FORWARD_KERNEL.md`. | NUM-L | M2 | BUG-FUNC-031 / recon G6-03 |
| G6-04 — ODDFYIELD | W109 (2026-07-14): **ODDFPRICE forward kernel FIXED** — it shared the `base.powf` integer-exponent bug; `excel_bond_pow` (binexp) added to `oddfprice_kernel` → ODDFPRICE now **bit-exact 5/5** on the live US-30/360 integer-exponent ladder (`oddfprice_us30360_integer_exponents_bit_exact_vs_excel`), 1505/1505 green, no regression. So ODDFYIELD's ~`3e5` ULP drift is now confirmed **pure schedule/publication** — shares the YIELD solver-VM (G6-03), iterate-publication wall. | NUM-L | M1 | BUG-FUNC-032 / recon G6-04 |
| G6-03d — PRICE (Actual/360 & Actual/365) | **Main schedule repair landed; exact residual graph remains open.** The build-20131 work correctly established universal `dsc=E−A`, `excel_pow_chain` for fractional discount powers, forward coupons, and separate redemption, eliminating the material basis-2/3 error. Current build-20228/CV2 discovery now covers 528 fresh rows plus a frozen 72-row adaptive companion. Production is `564/600`; the best coherent Chain/forward/separate-redemption graph is `571/600`, and all 29 residuals are exactly -1 ULP. There are zero exact survivors among 1,152 fixed graphs, 288 retained-PC64 variants, 80 factorized-coupon variants, or 48 association families. Retaining raw/PC64 pow lifetimes does not improve the leader, so the former claim that the residual is simply the shared fractional-pow wall is withdrawn; a missing accumulator/publication detail is likelier. The disjoint PRICE heldout remains sealed and uncaptured. | NUM-S | M2 | `PRICE_DURATION_RESIDUAL_GRAPH_SCOPE_PARTIAL_20260809.md` / G6-03d remains open |
| G6-03c — DURATION, MDURATION | **Main schedule repair landed; exact residual graph remains open.** The build-20131 work correctly established `off=(E−A)/E`, the PRICE discount substrate, CoupDaysBS accrued span, `(diff*cash)/disc`, separate redemption, and `num/den/f`, eliminating the material month-end/basis failures. On 264 fresh build-20228/CV2 rows, production and the best fixed graph are only `237/264` (max 3 ULP, sum 45); no member of 2,592 fixed or 288 retained-PC64 families is exact, and a 72-member factorized-coupon race keeps the same exact count (`237/264`, max 2, sum 42). The remaining numerator/denominator accumulator graph is not identified, so the earlier broad “identified + gated” wording is narrowed to the landed main schedule only. The disjoint DURATION heldout remains sealed and uncaptured. | NUM-S | M2 | `PRICE_DURATION_RESIDUAL_GRAPH_SCOPE_PARTIAL_20260809.md` / G6-03c remains open |
| G6-05 — RATE | **Current-reference exact graph remains open.** A frozen 256-row build-20228/CV2 NoCache discovery set makes every tested first residual smaller than `1e-7`, then races the balance, finite-difference, update, and publication graph directly. There are zero exact survivors among `13,824` frozen balance/FD/update graphs (best `2/256`) and `7,864,320` raw-power inline-helper plus outer-spill graphs (best `2/256`). A paired 512-call FV companion isolates the public helper to `502/512`; its ten misses are cancellation/small-rate controls shared by all tested associations, so worksheet FV is not an exact proxy for RATE's internal objective. Schedule replay refutes any pre-step residual/delta rule that publishes the unstepped current guess (`0/256`), but first-step and stop-and-publish-next families remain observationally tied until the internal objective is exact. The inherited x87-power/basin evidence remains useful historical structure, not a complete exact-graph identification. The disjoint 256-row heldout is sealed and uncaptured. | NUM-L | M2 | BUG-FUNC-009 / `RATE_EXACT_GRAPH_PARTIAL_REPORT_20260809.md` / G6-05 remains open |
| G6-06 — IRR | **Current-reference objective and solver graph remain open.** A frozen 300-row build-20228/CV2 discovery has 270 numeric and 30 scale-sensitive `#NUM!` results. A separate 900-point worksheet-NPV companion proves that worksheet formula evaluation applies a scale-relative cancellation-to-+0 correction even through a referenced raw-NPV result cell, but IRR does not inherit any of the 18 nontrivial snap decisions. Raw worksheet NPV itself has no exact graph in the enumerated family (reverse-Horner leader `636/900`, max 4 ULP), and adding the worksheet snap to the IRR objective worsens the guaranteed two-step subset from `40/72` to `37/72`. The best frozen no-snap objective/schedule candidate is only `44/72`; the public VB Financial.IRR candidate is `2/300`. The 180-row heldout remains sealed and uncaptured until an exact discovery survivor exists. | NUM-L | M2 | BUG-FUNC-048 / bead `oxf-jwh5.10` / `W109_IRR_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md` |
| G6-07 — CUMPRINC | **Current-reference discovery remains open.** A frozen 60-call PMT companion plus 540-call CUMPRINC discriminator (build 20228/CV2, Value2, NoCache) separates payment, hidden per-period principal, and range-fold publication. Shipping scores `90/540`; the best oracle-blind published-PMT discount/geometric family is only `190/540` (worst 8 ULP), and a broad public `loan.fs` family is `172/540`. Exact power-of-two homogeneity holds `180/180`, but rounded range partitions do not recombine exactly and a hidden first-principal low word cannot explain all ranges. The apparent `498/540` Ext80 result is explicitly rejected as 90-parameter per-query interpolation with no context transfer. No held-out or production survivor exists. | NUM-S | M2 | `calc_graph_racer/CUMPRINC_EXACT_PARTIAL_REPORT_20260809.md` / G6-07 remains open |

The former `G6-09` YIELDMAT row is signed off and removed (2026-07-11). The
W109 search identified x87 spill-loop arithmetic with the PUBLISHED formula's
association — `term1 = (1 + DIM/B·rate) - term2` with `term2` reused — not the
F#-style left chain OxFunc had ported. Validated `1250/1250` bit-exact on live
build `20131` (bases 2/3 sweep incl. held-out) and both former catalog
witnesses (bases 1/0) through the production day-count logic. See
[`W109_YIELDMAT_IDENTIFICATION_20260711.md`](function-lane/W109_YIELDMAT_IDENTIFICATION_20260711.md).

The former `G6-08` NPER row is signed off and removed (2026-07-11). The W109
search identified the same legacy x87 spill-loop signature as XNPV — every
assignment double-rounded, both logs the x87 worksheet `ln`, denominator on a
double-rounded `1+rate` — plus three newly pinned lanes: no epsilon small-rate
branch (tiny rates take the main path, `#DIV/0!` once `1+rate == 1`),
`NPER(0,0,..)` is `#DIV/0!`, and the zero-rate linear branch is double-rounded.
Validated `1286/1286` numeric + `7/7` error rows on live build `20131`. See
[`W109_NPER_IDENTIFICATION_20260711.md`](function-lane/W109_NPER_IDENTIFICATION_20260711.md).

The former `G6-11` XNPV row is signed off and removed (2026-07-11). The W109
calculation-graph search identified the full staging — `RN53(RN64(1+rate))`
base, the full worksheet POWER kernel per term (integer binexp dispatch
included), `RN53(RN64(value/pow))` term, forward per-step-stored x87
accumulation — plus a previously unknown guard (`rate <= 0`, including `-0.0`,
publishes `#NUM!`; OxFunc formerly accepted `(-1, 0]`). Validated
`1530/1530` numeric + `175/175` error rows on live build `20131`, including
held-out and metamorphic sweeps. See
[`W109_XNPV_IDENTIFICATION_20260711.md`](function-lane/W109_XNPV_IDENTIFICATION_20260711.md).

The former `G6-10` TBILLYIELD row is signed off and removed. A `2,156`-case
settlement × duration × price sweep first reproduced `308` one-ULP failures,
then reached `2156/2156` exact after changing the expression association from
`((100-pr)/pr*360)/days` to `((100-pr)/pr)*(360/days)`. See
[`CANDIDATE_CLOSURE_SWEEP_20260710.md`](function-lane/CANDIDATE_CLOSURE_SWEEP_20260710.md).

The former `YIELDDISC(44013,44562,95,100,0)` row was stale: the rate-first
formula repair already landed, the in-crate bit target is pinned, and a fresh
three-way replay on Excel 16.0 build 20131 is `all_bit_exact` (OxFunc = F# =
Excel). The row and bead `oxf-pzav` were retired on 2026-07-10.

## G7 — Comparison And Misc Semantics

No current open rows.

## G8 — Untriaged Inbox

No current open rows.

New smart-fuzzer `mixed_or_open` findings land here first, then move to G1-G7
or the context-sensitive catalog after triage.

## Pointers

- Category boundary and policy: [ODR-FN-002](decisions/ODR-FN-002-invocation-test-category-split.md)
- Context-sensitive Category 1 catalog: `smart-fuzzer/corpus/context_sensitive_catalog/`
- Severity vocabulary and comparison policy: `CHARTER.md` §4.1 and smart-fuzzer `Get-StandardSeverityClass`
- Transferable lessons: [OXFUNC_FIX_LEARNING_LOG.md](OXFUNC_FIX_LEARNING_LOG.md)
- Detailed bug evidence and history: `docs/bugs/streams/BUG-FUNC-*.md`
- Bit-exact Excel comparison plumbing: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`
- How to reproduce and repair a row (fixer quick-start): [OXFUNC_DEVIATION_FIXER_QUICKSTART.md](OXFUNC_DEVIATION_FIXER_QUICKSTART.md)

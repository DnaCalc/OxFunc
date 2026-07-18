# W109 primitive-recovery campaign — RESUME DOC (2026-07-18)

Cold-restart state for the G3-01/G3-02 campaign. Full history: the two
identification notes (W109_G3-01_GRATIO_IDENTIFICATION_20260716.md,
W109_GAMMALN_IDENTIFICATION_20260711.md), the catalog rows, and the field
diary (docs/notes/CHOPPED_EXP_IDENTIFICATION_STORY.md). Work dirs
(gitignored): smart-fuzzer/work/w109/G3-01-dist/ and G3-02-gamma/ (agent
files prefixed agentA..R by lane; batteries batch-*/answers-* b1..b26).
Oracle: Run-W109BulkBatch.ps1 (~8,800 probes/s, one Excel instance —
serialize captures). STRICT BLACK-BOX policy (AGENTS.md Clean-room Rule):
no binary inspection, ever; oracle behavior + published sources only.

## IDENTIFIED + LANDED (production, committed through 6f01b96)

| What | Identity | Landed as | Evidence grade |
|---|---|---|---|
| Internal ln | fyl2x chain RN53 == CR everywhere probed | (callers use CR-equal std/chain) | proven 2 ways (LOGNORM RN-exact; cross-view solve) |
| Internal exp | x87 F2XM1 chain (naive y−round(y) reduction, PC=64), SITE-DEPENDENT publication: RN53 wrapper/pdf, RZ53 (chop) series r-site, extended delivery on erf path | excel_exp / excel_exp_rz (excel_numeric); all 49 substrate sites | **34,000/34,000 held-out (POISSON) — sign-off grade** |
| Internal expm1 | Kahan correction in DOUBLE: u=chain-exp(t); t if u==1; (u−1)·t/ln(u) if |t|<1; else u−1 (msvcr100 exports no expm1) | excel_expm1_internal; EXPON/WEIBULL cdf + a==1 wrapper | 17,996/18,000 (99.978%) |
| Gamma substrate | GRATIO (TOMS 654) plain-double SSE2; a==1 dispatch (−expm1(−x), exp(−x)); Cephes-form forward gser + (r/a)·ans + exact-factorial normalizer + chop-exp r | regularized_gamma_p/q | family EXHAUSTED, held-out-confirmed; ceiling = 2 microdetail walls (below) |
| Beta substrate | BRATIO (TOMS 708); accurate-complement wrapper stagings (FDIST x=d2/den y=d1F/den; T-family x=df/den y=t²/den) | bratio + wrappers + TTEST | bit-identity to spec 20,008/20,008; b22 held-out 285/655 |
| *INV inverters | fully-converged float-lattice bisection publish-hi, rooting the PUBLISHED surface (CHIINV→Q, FINV→complement form, TINV→2t) | bisect_inverse + kernels | b14+b19 held-out validated |
| Published GAMMALN | Cody-1967 FORM SET, thresholds retuned: x<0.7(exact double) composed −log(x)+core(x+1); B1 [0.7,1.5); B2 [1.5,4) x87-spill; B4=[4,8) published SPECFUN P4/Q4 x87-continuous (worst 1); x≥8 ONE Stirling formula (q=(x−0.5)log x−x+LS2PI; fdlibm w1..w6 vector, plain double) 99.83% worst 2 | excel_numeric/gammaln.rs → GAMMALN/GAMMALN.PRECISE | port bit-identical to ref on 17,003 rows; held-out 79.0% worst 5 |
| Distribution pow | exp(y·ln x) composition (NOT bond-lane binexp) | (not yet landed — powf retained) | b24 route proven |
| Internal lgamma | EXTENDED precision, sub-ULP from CR, E_g nearest; distinct from published GAMMALN | (normalizer via exact factorial at integer a) | b24 GAMMA-window reads |

Corpus scores after all landings: CHIDIST 152/195, GAMMA.DIST 159/268,
beta b22 285/655, GAMMALN held-out 79.0%, BETAINV 12/30 worst +5,
POISSON k=0 exact. Suite 1,606 green.

## OPEN WALLS (each with banked per-row residuals + designed next probe)

1. **Chain-microdetail ±1 on series r** (the 7/45 class; some rows near tiny
   F2XM1 fractions): idealized == hardware == 38/45; inconsistent ±1 needed.
   Probe: dissect the 7 rows' chain internals op-by-op vs implied-r bits
   (agentQ_diag7.py started). Same class: 3/30k POISSON + 4/18k expm1 rows.
2. **ln-amplification wall at a≥3** (b26: worst grows 4→7→10 with a): a·L
   staging delivers CR-L; Excel's L may differ sub-ULP (extended L into a·L?
   — but extended-t1 REFUTED at a=2 with ±17; test a-dependent delivery).
3. **erf 190-path** (parked, resting state agentJ_resting_state.json): C10r
   composite 67.65%; the 1.01/0.505-pub-ULP equidistributed component with
   grid ∝ 2^Ez has ALL raced sources refuted. Probes: (i) find the 2^Ez-grid
   source, (ii) repair j-pipeline park phases at e=−15/−20 vs banked
   residuals, (iii) parked-vs-register-continuous chain floor.
   **b9heldout (256 rows) NEVER RACED — the promotion gate.**
4. **bgrat body op-graph** (parked; agentM: 53-bit, per-op-DR AND
   register-resident families ALL falsified by shared-z group-intersection;
   normalizer algdiv-class proven; GRATIO-sub q > grat1 in all combos).
   Probes: differential z-pairs (±1-2 ULP), flip-bracketing at bimodal
   a-positions, b→1⁻ sweep.
5. **GAMMALN b1/b2 coefficients** (Microsoft third refit; provisional:
   1967-n7/d for b1 ~31%, gn2/x87-spill for b2 ~47.5%; GN flat manifold;
   anchors D1=−γ, D2=1−γ CR pinned; b4 r0 fingerprint −3.34e-15 vs Cody).
   Probe: exact-arithmetic peel probes + lattice with the round-3 corpora.
6. **POISSON k≥1 product staging** (direct-product route proven; ~21%
   unexplained at k=1). Also BINOM/NEGBINOM route rewrites (log-composed in
   production; Excel = exp(n·ln q) class at k=0 — general k unproven).
7. **Distribution pow staging** (exp(y·ln x) — race excel_pow_positive vs
   powf vs chain compositions on the b24 weibull/binom corpora; then land).
8. **BETA.DIST integer-shape fast path + A/B-bounds staging** (small probes,
   unmeasured).

## METHOD RULES (hard-won, do not relearn)

- NEVER call residuals "noise" — deterministic per-argument signal;
  distributions are class constraints; bank per-row values.
- Race DIRECTED-ROUNDING publications, not just nearest (the chop lesson).
- A period from a dense scan is real only if reproduced at a 10× finer grid
  (the b18 aliasing lesson); phase-gradients are the alias-free fingerprint.
- Tiny-y/high-a gamma rows are ln-amplification tests, not series tests.
- Held-out gates before promotion, always (MINVERSE lesson). Generate
  constant tables programmatically (the hand-conversion lesson).
- Captures serialized through one Run-W109BulkBatch.ps1 at a time.
- x87 emulation: mpmath workprec(64) RN per op then float(); real hardware
  via excel_numeric ext ops (parking through tbyte is transparent).

## KEY TOOLING

- Racers (calc_graph_racer bins): check_x87exp(_ext), check_gratio_prod,
  check_bratio, check_inv, check_exp_rd, check_gammaln_port, check_erf190.
- Scorers in work dirs: score_b22.py, agentQ_lib.py, agentL_composite3.py,
  agentJ_transfer.py (build/gate/hyp/invert), rdexp_race.py.
- Oracle cache shared; b9heldout + b26X reserved unraced.

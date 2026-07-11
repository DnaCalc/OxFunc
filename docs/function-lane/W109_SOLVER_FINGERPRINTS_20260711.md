# G6 solver phase — guess-sweep fingerprints (2026-07-11, build 20131)

Corpora: answers-irr-r0.json (49), answers-rate-r0.json (48).

## IRR (cashflow A = [-1000,300,420,680]; B = [-1000,1210])

- Case A: 24 guesses -> 18 distinct published values spanning ~7.8e-12
  absolute (~7800 ULP at 0.1634). Publication is tolerance-gated, NOT
  converged-to-fixpoint; the final iterate depends on the guess.
- Case B: every guess (incl. -0.5, 1.5) lands within +-8 ULP of the exact
  double root 1210/1000-1 = 0x3fcae147ae147ae0, and the outputs take only
  THREE values: root, root+8 ULP, root-8 ULP. guess==root publishes the
  root bits unchanged -> NO mandatory seed perturbation for IRR (unlike
  the RATE root==guess 72-ULP recon witness).
  The 8-ULP quantization of the last step is a strong schedule
  discriminator (last update lands on a coarse grid -> the update term is
  computed at ~1e-16 absolute from a tolerance-scaled quantity).

## RATE (A = (10,-120,1000,0,0); B = (360,-599.55,100000,0,0))

- Case A: 22 distinct outputs over 24 guesses (same loose-tolerance
  publication shape as IRR-A); guesses -0.5/-0.3 -> #NUM!.
- Case B (root 0.0049999932): #NUM! from guesses {-0.5..0.0, 0.5..5.0} —
  divergence from guesses as benign as 0.5 despite a well-behaved f.
  Plain Newton would converge from 0.5 in <10 iterations -> the update is
  NOT plain Newton (secant with the documented 20-iteration / 1e-7 cap is
  the leading hypothesis; docs: "if results do not converge to within
  0.0000001 after 20 iterations, RATE returns #NUM!").

## Next

1. Feed published outputs back as guesses (fixpoint probe): if returned
   unchanged, the stop test is on the INPUT residual/step, pinning the
   tolerance form.
2. Dense guess ladders around plateau boundaries (e.g. IRR-A between
   0.09 and 0.095) -> exact tolerance constants via threshold bisection.
3. Solver-VM grid in calc_graph_racer (iterate-node): method {newton,
   secant, damped} x seed rule x tol type {|f|, |dx|, |dx| relative} x
   tol value {1e-7 ladder} x cap {20, 100} x publication {last, previous,
   first-satisfying}; score jointly on IRR-A/B + RATE-A/B sweeps.
4. Prerequisite kernels: IRR f = NPV polynomial (race spill vs plain on
   the SAME published data via f(published) residual fingerprints in MPFR);
   RATE f = annuity equation (W108 Phase C characterization exists).
5. YIELD/ODDFYIELD after RATE/IRR (100-iteration price inversion; forward
   PRICE kernel already aligned).

---

# Round 2 (same day): IRR update rule IDENTIFIED from the one-step error map

Probes: bit-level guess ladders at anchor+-2^j ULP, j=24..40, both cases
(answers-irr-r1.json, answers-irr-r2.json; 128 more live rows).

## The measurement

For case A (4 cashflows) the one-step error map is PURELY LINEAR and
sign-preserving over three decades: err_out/err_in = +7.40e-7 constant
for err_in in [1e-9, 3e-5] (plateau 7.38-7.41e-7, both sides). For case B
(2 cashflows) the map is EXACTLY ZERO everywhere (every ladder guess ->
exact root bits).

## The identification

The unique model consistent with both:

- Excel iterates in v = 1/(1+r) (discount-factor space). Case B's NPV is
  LINEAR in v -> any chord/FD slope is exact -> lambda = 0. Bit-confirmed.
- The update is forward-difference Newton with RELATIVE step h = 1e-6*v:
  predicted lambda = (f''_v / 2 f'_v) * 1e-6 * v* = 7.3865e-7 for case A
  vs measured 7.40e-7 +- 0.02. (Analytic Newton = quadratic map: ruled
  out; fixed-h FD, secant seed pairs, r-space FD: all ruled out by the
  case-B zero and/or the sign/linearity of the map.)
- Publication is r = 1/v - 1 from the final v: predicts output
  quantization of ~5.86 r-ULP per v-ULP; observed case-B outputs take
  only {root, root+-8 ULP} from far guesses, and the whole ladder basin
  collapses to exact root bits.
- Stop: guess passthrough when computed f rounds to zero (observed at
  anchor+8 exactly); otherwise |dv|-tolerance around 1e-5 with the final
  sub-tolerance step still applied (apply-last), then iterate-to-fixpoint
  behavior for far guesses. Exact tol/cap/order still open.

Python plain-double simulation of this structure already reproduces
104/177 rows (all fixpoint-attractor rows) with NPV in sequential
v-accumulation form; the residual far-guess scatter (~1e2 r-ULP) is
f-evaluation staging noise -> the remaining work is the joint (small)
race of NPV staging {plain, x87-spill; seq/Horner/pow} x {h sign, tol,
cap, check order} in the Rust racer with x87 ops. Work-dir scripts:
irr_schedule_race.py, irr_vspace_race.py.

RATE contrast: no passthrough anywhere (published anchors move by
50-160 ULP when fed back) -> RATE has a different schedule (mandatory
second seed / always-step), consistent with the recon root==guess 72-ULP
witness and the #NUM! divergences.

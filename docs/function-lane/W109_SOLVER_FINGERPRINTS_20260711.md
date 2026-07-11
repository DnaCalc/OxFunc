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

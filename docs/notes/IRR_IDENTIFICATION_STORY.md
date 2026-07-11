# How we cracked Excel's IRR (a field diary)

*W109 Phase-4, 2026-07-11. Live oracle: Excel 16.0 build 20131. Kept as a
running narrative — the dead ends matter as much as the hits.*

## The setup

Catalog row G6-06: OxFunc's IRR agreed with Excel to ~1e-9 but never to
the bit. The charter's rule: **never search the iteration schedule and
the function kernel at the same time** — you can't fit twenty coupled
unknowns to one number. So the plan was to make Excel *draw us a
picture* of its solver, one probe batch at a time.

## Act I — the guess sweep (what does the knob do?)

First 97 probes: same cashflows, 24 different `guess` values.

- An irregular 4-flow case published **18 different results for 24
  guesses**, scattered across ~7,800 ULP around the root. So Excel does
  NOT converge to a fixpoint and publish it; it stops somewhere
  tolerance-gated, and the parking spot depends on where you started.
  The solver's fingerprints were all over the answer bits.
- A 2-flow case (`[-1000, 1210]`, root analytically 1210/1000−1) was
  eerily different: every guess landed on exactly one of THREE values —
  the exact root double, or the root ±8 ULP. And feeding the root in as
  the guess returned it **bit-unchanged** (so IRR, unlike RATE, has no
  mandatory seed perturbation).
- RATE, probed the same way, #NUM!'d from guesses as harmless as 0.5 on
  a plain mortgage — whatever RATE is, it is not a plain Newton, and it
  is not IRR's schedule.

The ±8-ULP quantization was the first real clue, though we didn't know
it yet.

## Act II — the wrong race (fit first, think later)

We raced the obvious zoo in r-space — analytic Newton, secant with
various second seeds, finite differences with fixed h, every stop rule
and tolerance from the docs — against 109 answers. Best score: **14/109**,
barely above nothing. The lesson: our simulated fixpoints didn't even
land on Excel's attractor bits, so the whole zoo was disqualified at the
starting line. Structure was needed, not enumeration.

## Act III — ladders (make the solver take exactly one step)

Next batch: guesses placed at the anchor ±1, ±2, ±3 … ±2³⁵ ULP. If the
guess is that close, the solver takes one step and stops — so the output
IS the update rule, photographed mid-stride.

- Passthrough at exactly anchor+8: computed f rounds to 0.0 there, and
  Excel hands back the guess untouched. That pinned the stop test's
  zero-check *before* the first step.
- A huge basin (±0.001!) collapsed to the exact anchor bits.
- And the crucial anomaly: at ±2²⁶ ULP the output was ±48 ULP; at ±2³⁵
  it was ±25,400. Same ratio. Twice.

## Act IV — the error map (the solver's portrait)

So we ran the full ladder, j = 24…40, both sides, both cashflow cases —
and plotted err_out/err_in:

```
case A:  +7.15e-7, 7.15, 7.40, 7.40, 7.40, 7.40, 7.41, 7.43, 7.48 ...
case B:   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0  (both sides)
```

A **constant, sign-preserving, linear contraction of 7.40e-7 across
three decades** for the 4-flow case — and **exactly zero** for the
2-flow case. That pair of facts is brutally restrictive:

- Linear and constant ⇒ the slope Excel divides by is off from the true
  derivative by a *fixed relative amount* — so not analytic Newton
  (quadratic map), not converging secant (superlinear).
- Exactly zero for `[-1000, 1210]` ⇒ the slope is *exact* when NPV is
  **linear in the iteration variable**. NPV is linear in v = 1/(1+r) for
  two flows — not in r.

Conclusion: **Excel iterates in discount-factor space.** The update is a
finite-difference Newton in v, and the FD step h must satisfy
(f″ᵥ/2f′ᵥ)·h = 7.40e-7. Computing the curvature at the case-A root:
f″ᵥ/2f′ᵥ = 0.8593, so h ≈ 8.6e-7 ≈ **1e-6 · v*** (v* = 0.8595). Predicted
λ = 7.3865e-7. Measured: 7.40e-7. Three digits.

Even the ±8-ULP mystery dissolved: publishing r = 1/v − 1 from a
discrete v grid spaces the representable answers 5.86 r-ULPs apart.

## Act V — the confirmation races

- Python, plain double, v-space FD-Newton: **104/177** immediately (every
  fixpoint-attractor row).
- Rust with x87 double-rounded staging (the legacy-financial-body prior
  from XNPV): **x87-spill Horner NPV, h = 1e-6·v, tol = 1e-7 apply-last →
  112/177**, beating every plain-double variant.

65 rows remain — all staging detail (the dv expression, the v update,
the 1/v−1 publication chain), a bounded enumeration now that the
structure is fixed.

## What made it work

1. **Geometry before bits.** The schedule was identified from robust
   *shapes* — linearity, sign, a zero — not from bit-fitting. The bit
   race only started once there was one structure left to race.
2. **The linear probe.** A two-cashflow IRR is the solver's unit test:
   any slope method is exact on it. Its all-zero ladder was worth more
   than a thousand random witnesses.
3. **Passthrough probes** read the stop rule; **quantization** read the
   publication rule; **the ladder ratio** read the update rule. Each
   observable decoded one component, independently.
4. **The failed race was load-bearing.** 14/109 in r-space is what said
   "wrong space" loudly enough to make us look for v.

*Continued in W109_SOLVER_FINGERPRINTS_20260711.md (data) and
check_irr_schedule.rs (the racer). Next: close the 65 staging rows,
then aim the same ladder camera at RATE.*

## Act VI — the last mile is a store mask (in progress)

With the schedule skeleton fixed, we tried to pin the NPV kernel from
the one-step ladder rows alone (each is a single photographed FD step).
Five NPV forms x four h-forms x two stagings all plateau at 25/54 —
identical to the full simulator on those rows, proving the rows really
are single steps and the ±8-ULP misses are pure f-evaluation noise:
`den = f1 − f0` is Sterbenz-exact, so every last bit of the update comes
from the rounding pattern inside the two NPV evaluations. Also learned:
the stop rule is NOT a plain |dv| threshold (no constant explains
one-step at 9.5e-7 and multi-step far sweeps simultaneously).

Next tool: the same exhaustive store-mask fit that cracked the GAMMALN
Stirling staging, applied to the NPV+step chain, scored on the ladder
rows. Fifty-four single-step photographs against a few thousand masks.

## Act VII — the model eats its own tail (and the doctrine wins)

Confession first: Act IV's "one-step FD, h = 1e-6·v, three-digit match"
was itself an over-fit — of the *small-error plateau*. A single FD step's
error ratio must grow ~25× across the ladder (the quadratic term never
sleeps); the data's ratio moves 4.5%. The only composition that kills a
quadratic is another step: the rungs take **two** steps, and the plateau
is λ'² — which pins λ' = 8.59e-4 per step, i.e. **h = 0.001, absolute**.
(0.8593·10⁻³)² = 7.39e-7. The plateau again, to three digits — same
number, new meaning. Lesson: a perfect fit on a plateau identifies the
plateau, not the mechanism; only the *curvature* of the error map tells
one step from two.

Then case B paid out a second time. Its mid-rung ±8s scale exactly like
f's own cancellation noise — and *no* staging of `1210·v − 1000` could
reproduce Excel's noise realization. The kernel is not a v-polynomial.
At which point we finally did what the charter said on day one: identify
f first. Excel documents IRR's f as `cf0 + NPV(rate, …)` — and worksheet
NPV answers probes directly, no solver in the way. Ninety-four probes
later: every single-flow row matches a division chain exactly, and the
3-flow leader is a **reverse Horner division chain** — `s = (s+c)/w`
from the last cashflow — at 79/94, fifteen ±1-ULP rows from closure.

The irony is not lost on us: the tool that cracked GAMMALN's over-fit
(dense probes killing a free-parameter fit) is the same tool that
corrected our own Act IV. The oracle doesn't care whose hypothesis it
is.

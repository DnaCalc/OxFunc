# Complex arithmetic kernel evidence

Status: `execution_state=in_progress`, `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: shared coefficient formatter midpoint discrepancies; broader decimal-string grammar/context validation;
other complex transcendental kernels and full
coercion/reference/array coverage. This packet
does not establish whole-function parity.

All oracle captures use Excel 16.0 build 20430, workbook Compatibility Version 2.
The numeric files include exact binary64 inputs and a Value2 capture profile.
The typed files retain the full input/formula/outcome records and the generic
runner manifest. The campaign owner ran the live oracle; research code only
consumes the retained results and public OxFunc implementations.

## Division

The original complex division multiplied the denominator by its conjugate and
divided by its squared magnitude. That graph caused numerical drift for real
inputs and lost very large or tiny complex denominators. The replacement scales
by the larger denominator component first:

- If `abs(c) >= abs(d)`, set `r=d/c`, `q=c+d*r`, and return
  `((a+b*r)/q, (b-a*r)/q)`.
- Otherwise set `r=c/d`, `q=d+c*r`, and return
  `((a*r+b)/q, (b*r-a)/q)`.

A zero denominator returns `#NUM!`. Subnormal resulting coefficients become
zero before component assembly. This differs from formatting a normal input
whose rounded decimal becomes signed text zero. The algorithm uses general
arithmetic operations and contains no observed-value corrections. In particular,
it reproduces the captured tiny imaginary residual of
`(3e150+4e150i)/(3e150+4e150i)` instead of forcing identical operands to `1`.

| Capture | Matching admitted rows after change |
| --- | ---: |
| `extended-imdiv.json` | 1,216/1,216 |
| `extremes-imdiv.json` | 846/846 |
| `typed-cases.json` + `typed-excel.jsonl`, IMDIV rows | 400/400 |
| `imdiv-heldout-cases.json` + `imdiv-heldout-excel.jsonl` | 512/512 |

The extreme capture contains 852 total rows, including six source-subnormal
inputs that Value2 changes to zero. Those six rows remain retained and are
explicitly excluded from the exact-input replay; the independent ingress
evidence is `../numeric-ingress.json`. Five happened to match and one did not;
accidental matches are not counted as evidence for the source inputs.

The shared division helper also serves reciprocal complex functions. On the
unchanged 1,209-row numeric discovery captures, exact matches improve from
1,198 to 1,206 for IMCSC, 1,179 to 1,200 for IMCSCH, and 1,180 to 1,204 for
IMSECH. No previously matching row becomes a mismatch. Remaining failures are
retained in `division-adjacent-replay.jsonl`; the campaign owner retains the
bulk extended discovery corpus. This does not establish complex-input parity
for those neighboring functions.

`../complex/imdiv-candidate-freeze.json` records the source hash before the
independent 512-case complex-text holdout was generated with seed 2026092907.
All 512 rows match without altering the frozen division graph.

## Square-root investigation

The old IMSQRT uses a scaled `hypot` magnitude and platform sine/cosine.
Replacing only the trigonometric calls with the existing characterized SIN/COS
kernels matches 1,208/1,209 real discovery rows; the remaining tiny input shows
that the magnitude calculation is observably different from `hypot`.

The revised graph squares each component separately, flushes subnormal squared
terms, adds them, then takes two separate square roots. Overflow in the sum of
two finite terms publishes zero; an individually overflowing square remains
nonfinite and produces `#NUM!`. The argument uses `atan(im/re)` with explicit
quadrant adjustment after signed-zero normalization. These changes match 1,209
real discovery rows, 821 admitted numeric extreme rows, 93 initial complex-text
rows and 523 independent squared-term/sum boundary rows: 2,646/2,646.

The frozen candidate (`imsqrt-candidate-freeze.json`, source copy retained) then
matched 952/960 independent rows. Six failures had finite components but an
overflowing `im/re`; 64 new ratio controls distinguish a zero-angle fallback from
atan2 or an infinite-ratio atan. That observed rule now matches all 64 and brings
the earlier holdout to 958/960. The two remaining failures were ordinary-quadrant
last-rendered-digit differences and remain retained in the earlier candidate reports.
`imsqrt-heldout-research-candidates.jsonl` preserves the original eight failures;
`imsqrt-heldout-research-current.jsonl` preserves the two residuals before the
subsequent arctangent refinement described below.

`imsqrt-output-format-complex.json` probes the two computed coefficient pairs and
their independent ±2 ULP neighbors. All 50 COMPLEX results match; this isolates
the remaining IMSQRT differences upstream of coefficient rendering. IMARGUMENT
now exposes the same ratio/quadrant primitive with `#DIV/0!` for zero and ratio
overflow, while IMSQRT maps these errors to angle zero. All 64 sibling controls
match exact numeric bits/error types. An independent 476-row quadrant/axis/scale
holdout is generated after the retained `imargument-frozen-complex-family.rs`;
that frozen candidate matched 468/476. Six ordinary angle rows differed by one
ULP and two subnormal angles should publish +0. All eight are preserved in
`imargument-heldout-first-candidate-judge.jsonl`.

General RN64 product, quotient and square-root staging variants did not resolve
the two IMSQRT residuals. Replacing the platform arctangent with the public CPU
FPATAN operation, rounded to binary64 before quadrant adjustment, matches all
476 IMARGUMENT rows and all 960 IMSQRT rows. Subnormal argument results publish
+0. This graph is selected solely by reproducible black-box observations and
public ISA arithmetic, with no Excel binary inspection or inferred binary identity.
The primitive is isolated in `excel_numeric/x87.rs` using the existing saved/restored
control-word convention; at this initial freeze public ATAN still used its earlier
kernel. Lean exposes
an explicit arctangent binding parameter. `fpatan-candidate-freeze.json` records
the retained source hashes for fresh independent IMARGUMENT476 and IMSQRT960
captures. Those fresh captures match 476/476 IMARGUMENT and 960/960 IMSQRT
observations without changing any of the three frozen arithmetic source files.
The shared decimal parser was refined independently during this interval; its
new staged conversion also passes the prior retained kernel captures. Alternate
CPU/platform behavior remains a qualification of the x86-64 arithmetic substrate.

Subsequent shared ATAN research characterizes a reciprocal-reduced FPATAN graph
for arguments outside [-1,1], retaining the extended complementary angle until
final publication. Both IMARGUMENT and IMSQRT now bind that refined helper; all
11,049 retained kernel/parser rows remain exact. `reduced-candidate-freeze.json`
precedes fresh IMARGUMENT476 and IMSQRT960 captures with independent seeds
2026092920 and 2026092921. Those additional captures match 476/476 and 960/960, with both callers and
the two shared arithmetic source files unchanged from the freeze. The retained
complex kernel/parser total is now 12,485/12,485 exact.

The wide numeric captures also distinguish the ordinary IMCOS/IMSIN kernels
from coefficient rendering. Their real-axis candidates using the existing
characterized COS/SIN kernels and circular-trig bound match 1,209/1,209 each.
The 200-row complex-axis capture for each function additionally distinguishes
exponential half-sum/half-difference factors from platform sinh/cosh. The general
first candidate used characterized SIN/COS parts and `(exp(b) ± exp(-b))/2`, retaining
cancellation for tiny imaginary inputs. Complex coefficient text truncates after
15 significant lexical digits, separately corroborated by IMREAL/IMAGINARY
probes; the pure decimal converter is shared with numeric coercion, while their
syntax policies remain separate. Under this parser rule both discovery sets
match 200/200. The source is frozen in `trig-frozen-complex-family.rs` before the
independent 768-row complex-axis holdout. That first frozen candidate matched
369/384 IMCOS and 363/384 IMSIN rows. All 36 failures remain in
`trig-heldout-first-candidate-judge.jsonl`. The holdout distinguishes the alternate
reciprocal expression: compute `p = EXP(b)`, `q = RN53(RN64(1/p))`, then use
`(p+q)/2` and `(p-q)/2`. This general reciprocal graph matches 384/384 for each
function, including the asymmetric near-one rounding that separates binary64
reciprocal from the intermediate 64-bit-significand rounding. No sign-dependent
selection or observed-value correction is present. The existing
`excel_x87_recip` primitive supplies that operation on the current x86-64 host;
its non-x86 fallback is a remaining platform qualification. The refined source
is frozen in `trig-reciprocal-frozen-complex-family.rs` before a second 768-row
capture (640 new random rows and 128 repeated boundary controls).
That second capture matches 384/384 for each function without further changes.
The trigonometric graph therefore matches 1,936 retained typed rows in total;
this remains conditional on the still-partial parser and coefficient formatter.

IMTAN/IMCOT general expression research is retained in
`tangent-general-research-candidates.jsonl`. Double-angle, tangent/hyperbolic,
complex-division and squared-denominator identities do not match the 200-row
complex-input captures. The best explored candidates reach 173/200 for IMTAN
and 154/200 for IMCOT; these are rejected as parity models. Their production
kernels remain unchanged, and no real-axis-only substitution hides the open
complex-input lanes.

## Complex coefficient text parsing

`parser-cases.json` binds 168 IMREAL/IMAGINARY coefficient-string observations.
`parser-edge-cases.json` adds 275 exponent, equivalent-decimal-scaling and
whitespace controls. All 443 match exact numeric bits/error types. Only ASCII
space is trimmed around components and their leading signs; other whitespace
remains invalid. The shared unsigned decimal converter truncates to 15 significant
lexical digits and admits normalized decimal point positions from -308 through
308 inclusive, before conversion; resulting subnormals publish +0. The position
is `lexical exponent - fractional digit count + significant digit count`, with
the full pre-truncation count and zero significant digits for all-zero mantissas.
The paired ABS/IMREAL packet `shared-parser-edge-*` independently confirms
all-zero forms and equivalent scales. It also exposed a shared ordinary decimal
conversion rounding residual. The general staged-power conversion characterized
in `../numeric-text/` now resolves that residual and matches its independent
3,760-row capture. No per-value correction has been added; broader text context
and locale grammars remain separate validation lanes.

## Replay and formal alignment

`crates/oxfunc_core/tests/w111_complex_kernel_live_replay.rs` checks all 2,974
admitted IMDIV rows, 5,590 IMSQRT rows, 50 coefficient isolation rows, 1,936
trigonometric rows, 1,492 argument controls, and 443 coefficient parser rows through
public dispatch: 12,485 admitted rows (8 tests) with exact typed comparison. Earlier failed
candidate outcomes are retained separately; the current arctangent refinement
matches those rows and both fresh independent captures. The
input-transport exclusion count is asserted explicitly; the typed replay binds
case IDs, function IDs and formula text to the captured outcomes.

The Lean ComplexFamily module records the scaled division graph over both
rational and binary64 carriers, including the exact self-division residual,
the square-root term/sum/ratio boundaries, and the trigonometric expression graph
parameterized by the characterized scalar backends. Its executable examples
bind the observed boundaries without claiming a universal parity proof. Focused
`lake build OxFunc.Functions.ComplexFamily` passes.

Reproduction tools under `smart-fuzzer/tools/w111/` are
`gen_complex_kernel_probes.py`, `gen_imsqrt_boundary_probes.py`,
`gen_imdiv_holdout.py`, `gen_imsqrt_holdout.py`, `gen_complex_trig_probes.py`,
`gen_complex_trig_holdout.py`, `gen_complex_parser_probes.py`,
`gen_complex_parser_edges.py`, `gen_decimal_parser_shared_edges.py`,
`gen_imargument_holdout.py`, `gen_imsqrt_output_probes.py`, and the standalone `complex_kernel_probe` Rust research
program. The production formatter's unresolved midpoint outcomes remain in
`../complex/`; no kernel repair hides or special-cases them.

# Generic Number-to-Text rendering

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: midpoint arithmetic precision, unexercised consumers, locale/context and
upstream acknowledgement. No whole-function identity claim follows.

The current Excel 16.0 build 20430 / workbook CV2 discovery captured 7,048 typed
cases with exact admitted zero/normal numeric ingress. There are 1,465 distinct
number inputs: fresh random normal bits, decimal midpoint and notation boundaries,
and earlier specialized-formatter discriminators. LEFT with count 32767 exposes
the entire generic text. LEN, EXACT, RIGHT, MID, TRIM, CONCAT, ENCODEURL, LOWER,
UPPER, CLEAN, REPT, SUBSTITUTE and TEXTJOIN agree with it in every captured
cross-function control. `discovery-analysis.json` records that comparison.

All 1,465 LEFT strings also match the already evidenced ADDRESS numeric sheet
formatter. The runtime factors that existing mechanism into the crate-private
`functions/numeric_text.rs`; the Number branch of `coerce_prepared_to_text`
and ADDRESS's numeric sheet branch call it. COMPLEX remains separate. The 87
COMPLEX controls remain in `discovery.json` as distinct-policy observations,
not generic formatter expectations. The remaining 6,961 rows pass production
surface-dispatch replay in `w111_numeric_to_text_live_replay.rs`.

The current finite normal/zero candidate mechanism is:

1. Render the absolute binary64 value to 31 significant decimal digits with
   nearest-even rounding, then round that decimal value to 15 significant digits,
   taking an exact midpoint toward zero, and retain an integer significand and scale.
2. Remove trailing significand zeroes and form its unsigned fixed string.
3. Use fixed notation if that string has at most 20 characters. Otherwise use
   scientific notation, with `E`, an explicit exponent sign and at least two
   exponent digits.
4. On the scientific path, if the exponent magnitude before the second rounding
   is at least 99, reduce to 14 significant digits, half away from zero. Strip
   trailing zeroes again and recompute the exponent after any carry.
5. Prefix a minus sign for a negative nonzero input. Zero renders as `0`.

This rule explains why `100000000000000.5` renders as `100000000000000`, while
the largest finite binary64 value renders as `1.7976931348623E+308`.
The separate public `VarBstrFromR8` API is a rejected formatter hypothesis:
it matches only 879 of 1,465 strings. Its documented API capture is retained
solely as reproducible non-authoritative research, with no production dependency.

`NumericTextRendering.lean` executes the current 31-digit intermediate using
exact rational arithmetic and the decimal-pair rendering substrate. It separately
retains the exact-15 alternative as a research hypothesis, not an Excel rule.
Theorems exercise midpoint direction, the two initial rules' difference, unsigned
width, exponent transition, second-round carry and maximum-normal text. ADDRESS
and the prepared text-broadcast substrate import it. This is a substrate binding,
not a claim that every downstream consumer has a duplicated Lean function model.

Nonfinite and subnormal direct carriers cannot be admitted by this Value2
capture. The shared adapter retains its previous rendering for them to avoid a
new panic or an unsupported domain inference. Text, logical, blank, error,
reference preparation and array lifting are unchanged by this helper extraction.
`IMPACT_ASSESSMENT.md` lists all 28 helper-dependent modules and the evaluator
boundary implications. The parent campaign owns the upstream handoff.

The frozen fresh packet has 8,208 rows over 2,312 number inputs, excludes every
discovery input bit pattern, and adds reference/array origin, notation carries,
fresh midpoint neighbors and additional text-function consumers. Its input,
source hashes and seed are in
`smart-fuzzer/runs/w111-numeric-to-text-heldout-20260929/design.json`.
All 8,208 heldout rows match the frozen candidate exactly with zero exclusions.
The two production dispatch tests now replay 15,169 admitted generic/ADDRESS
formatter observations. The 248 text-filtered library tests and the pre-existing
ADDRESS/text-slice replay banks also pass after the shared helper change.

An additional rational-approximant search found 414 of 103,139 constructed
normal inputs where the current helper's 30-decimal-place scientific intermediate
turns a value slightly above a 15-digit midpoint into an apparent exact tie.
The live 3,204-row packet now exposes 48 admitted discrepancies: 6 LEFT, 6 ADDRESS
and 36 ROUND rows. These represent three positive generic-rendering centers and
18 positive ROUND centers, plus both signs. The other 3,156 rows match. The runner
verifies exact numeric Value2 readback before calculation; current production
dispatch independently reproduces the same 48 differences. `precision.json`,
`precision-candidate-differences.json` and `precision-replay.log` retain this
unresolved counterevidence. The corresponding regression is explicitly ignored
with its open reason, not changed to fit local output.

Exact-15 rounding is also disproved as a universal replacement: it differs on
397 of 534 positive ROUND rows. General finite-precision scaling models remain
unsuccessful (`precision-models.json`). The evidenced decimal-parser nibble graph
predicts one final-assembly ULP difference but does not resolve the 17 remaining
initial-rounding centers. No new runtime rounding policy was selected from these
observations. The shared formatter, ADDRESS and ROUND therefore retain a known
function-semantic gap, beyond the separate contextual integration dependency.

The explicit serializer follow-up matches generic LEFT text in all 560
VALUETOTEXT/ARRAYTOTEXT rows across both modes and mixed arrays. Their Number
branches now use the same prepared helper; TEXT General remains separate and
has distinct captured output. All 1,420 rows of the frozen independent serializer
packet match exactly with zero exclusions. The four regular replay tests now
cover 17,149 observations, and all 22 serializer-filtered unit tests pass.
This source extension does not change the formatter arithmetic or
resolve its precision residuals.

The public documented `_ecvt_s` calls in `documented-ecvt.json` are also rejected
as a universal replacement: both tested CRT API providers differ on 397 of the
534 positive ROUND precision rows. This is documented-API behavior only; no
binary internals were inspected and no CRT formatting dependency was added.

Ignoring notation differences does not rescue the public Automation formatter:
`documented-api-precision.json` compares VarBstrFromR8's decimal digits against
the same 534 positive precision controls and also finds 397 differences. Its
documented public API therefore supplies no replacement for the unresolved
initial rounding rule.

The further offline staged-power search compares 384 uniform graphs using
binary, nibble, decimal and normalization stages at 53, 64, 80 or 96 binary
significand bits. Its best graph still differs on 19/534 positive controls, more
than the existing 18-row decimal-intermediate residual (17 when paired with the
separately evidenced decimal-parser assembly). `staged-precision-models.json`
retains the full counts and best residual identities. No per-input selection or
new production arithmetic follows from these rejected hypotheses.

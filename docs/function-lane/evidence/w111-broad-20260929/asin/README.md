# ASIN arithmetic exploration

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: numeric arithmetic graph,
broader input/context behavior, and
alternate arithmetic platforms. ASIN and ACOS now bind the observed staged
graphs; no whole-function parity claim is made.

The retained 1,224-row discovery uses Excel 16.0 build 20430 and workbook
Compatibility Version 2. The discovery includes 424 inputs inside the domain and 800 domain-error
controls. The platform asin kernel matches 1,166 rows. A general
factor/product expression matches 1,223 rows:
`atan(x / sqrt((1-x)*(1+x)))`, with RN64 intermediate operations each published
to binary64. Its sole discovery residual is input `0xbfe52a3a8df2eadb`:
the candidate returns `0xbfe720476e0c1ac2`, Excel `0xbfe720476e0c1ac3`.
This expression is therefore a rejected parity model, not a production repair.

The 14 signed neighbor controls in `isolation-asin.json` show non-oddness at
three of seven magnitudes. Excel's negative results differ from sign-restored
positive results by -1, +1, and -1 ULP for magnitude bits ending `ead9`, `eadb`,
and `eadd`. The parent campaign independently read back all source binary64
values and formulas; `asin-exact-readback.json` preserves that verification.
Public SQRT of each candidate product agrees with its proposed denominator;
public ATAN of the proposed quotient agrees with all seven positive ASIN rows
but differs on those three negative rows. Those controls isolate the uncertainty
to ASIN's expression rather than establish an arbitrary sign adjustment.

The research program compares ordinary difference/product/half-angle identities,
binary64 versus retained or stored RN64 operations, direct two-operand FPATAN
versus a published quotient, all independent factor/product/root/quotient store
barriers, per-operation 53/64-bit precision choices, complementary-angle
identities, separate square-root factors and sequential division orders.
These earlier graphs did not explain every isolated observation. Complete differences
for every alternative are retained in the `*-graph-comparison.jsonl` files.
No input-specific rule or fitted constant has been introduced.

`public-runtime-comparison.json` additionally records calls only to the
[documented C asin API](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/asin-asinf-asinl?view=msvc-170)
in the locally available runtimes. Neither runtime supplies universal parity:
msvcrt matches 1,161/1,224 discovery rows; ucrtbase 1,166/1,224. Both match 9/14
isolation rows. No binary content was inspected and no runtime dependency was
added to production.

`gen_asin_signed_probes.py` prepares 6,180 paired-sign cases, including fresh
uniform/scale draws, mathematical boundary neighbors, and separately labelled
neighbors of the prior asymmetric input. That capture finds 687 non-odd pairs out of 3,090, including 485 of 1,600
fresh uniform pairs. The original product expression matches 5,489/6,180; its
positive inputs have only four discrepancies, while negative inputs account for
687. Broad discovery inputs had often been quantized by a wider uniform interval,
which hid many low-bit distinctions. `signed-oddness-analysis.json` retains the
full paired observations.

A general reused-factor identity matches all 7,418 discovery observations:

1. `t = RN53(RN64(1 - x))`;
2. `u = RN53(RN64(2 - t))`;
3. `p = RN53(RN64(t * u))`;
4. `s = RN53(RN64(sqrt(p)))`;
5. `q = RN53(RN64(x / s))`;
6. apply the arctangent primitive and publish subnormal results as +0.

The reused rounded `t` explains the sign asymmetry without a sign-specific
selection. Single binary64 arithmetic matches only 6,141/6,180 signed rows;
continuous RN64 arithmetic matches 4,684/6,180. Both direct FPATAN and the
shared reduced arctangent match those discovery observations. The research source is frozen in `reused-factor-freeze.json`
before a fresh 6,722-row packet. That packet includes 31 direct/reduced angle
disagreement centers selected from 300,000 fresh mathematical draws, without
Excel answers, plus fresh domain-scale/uniform pairs and boundaries. The direct
FPATAN binding matches 6,674/6,722; the reduced binding matches 6,722/6,722.
Every one of the 48 former differences remains in `heldout-graph-comparison.jsonl`.
The retained ASIN total under the selected reduced binding is 14,140/14,140.

The executable Lean `asinWithKernels` graph exposes each staged primitive and
the final arctangent dependency. Its domain, zero, half and underflow bindings
build successfully alongside the ACOS model (8 jobs). This is graph alignment,
not a universal equivalence proof.

ACOS calls `asin_kernel`; its separate 6,180-row impact capture first matched
6,156/6,180 with the new ASIN and a plain binary64 subtraction from binary64
PI/2. All 24 differences occur just above half an output ULP, for both signs.
Using RN53(RN64(PI/2 - ASIN(x))) matches 6,180/6,180, preserving the original
24 differences in `acos-caller-graph-comparison.jsonl`. Production uses the
existing staged subtraction helper, not a boundary-specific adjustment. The
ACOS Lean graph exposes this subtraction and ASIN dependency explicitly.

`w111_asin_acos_live_replay.rs` passes 4 public-dispatch tests covering all
20,320 retained numeric observations. Domain error tags and result bits are
compared exactly. `production-candidate-freeze.json` records caller and shared
arithmetic sources before a second independent combined capture of 8,052 rows
per function. It includes new paired uniform/scaled inputs, fresh arctangent
disagreement centers, and 32 subtraction midpoint neighborhoods with extended
half-ULP offsets. That fresh capture matches 8,052/8,052 per function without source changes.
The retained totals are 22,192 ASIN and 14,232 ACOS observations: 36,424
exact results in 5 public-dispatch tests. All four frozen source hashes are unchanged.

Research source: `smart-fuzzer/tools/w111/complex_kernel_probe/src/bin/asin_probe.rs`.

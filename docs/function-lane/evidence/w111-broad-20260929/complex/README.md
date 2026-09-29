# Complex coefficient rendering evidence

This packet concerns numeric coefficient rendering in COMPLEX and identity
controls for IMCONJUGATE, IMSUM and IMPRODUCT. It supplies evidence for
BUG-FUNC-053 (`oxf-mwue.28.5`); it does not establish whole-function parity.

Status: `execution_state=in_progress`, `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: 40 unresolved midpoint results in the fresh 944-case scaling search; text,
suffix, coercion, reference and array behavior outside this numeric capture;
arithmetic accuracy of the other IM functions; canonical reconciliation by the
campaign owner. Locale and alternate Excel-version validation are separate.

## Capture scope and provenance

The retained oracle files contain exact input binary64 encodings and exact
text/error outcomes. Each includes a capture profile: Excel 16.0 build 20430,
64-bit Windows, workbook Compatibility Version 2, `cell_value2_bulk`, and
disabled oracle cache. The campaign owner captured them through
`smart-fuzzer/tools/Run-W109BulkBatch.ps1`. No proprietary binary or internal
Excel implementation source was consulted.

| Capture | Rows | Purpose |
| --- | ---: | --- |
| `format-complex.json` | 2,382 | Decimal precision, notation, signs and both components |
| `format-imconjugate.json` | 794 | Adjacent function rendering control |
| `format-imsum.json` | 794 | Adjacent function rendering control |
| `format-improduct.json` | 794 | Adjacent function rendering control |
| `extremes-complex.json` | 528 | Minimum normal and maximum finite machine neighbors |
| `ties-complex.json` | 252 | Decimal midpoint and nearby coefficient cases |
| `heldout-complex.json` | 2,500 | First independent random/boundary sweep |
| `heldout-imconjugate.json` | 625 | First independent identity control |
| `heldout-imsum.json` | 625 | First independent identity control |
| `heldout-improduct.json` | 625 | First independent identity control |
| `midpoints-complex.json` | 280 | Competing rounding hypotheses and machine neighbors |
| `heldout2-complex.json` | 2,500 | Second independent ordinary sweep |
| `heldout2-imconjugate.json` | 625 | Second independent identity control |
| `heldout2-imsum.json` | 625 | Second independent identity control |
| `heldout2-improduct.json` | 625 | Second independent identity control |
| `scaling-heldout-complex.json` | 944 | Fresh arithmetic discriminators and extreme controls |

The first formatting generator reused four `normal-limit` IDs. No rows were
dropped or indexed solely by ID: replay iterates every row, and research
comparisons check IDs together with input encodings. The generator now includes
the input bits in those labels. The retained captures are unchanged.

These inputs are zero or normal binary64. Separate Value2 readback evidence in
`../numeric-ingress.json` shows that Excel changes source subnormal values and
negative zero to positive zero. The original broad COMPLEX capture therefore
has eight source-subnormal mismatches that are input-transport limited; this
packet does not add per-function input erasure to imitate the harness.

## Observations and rejected hypotheses

Coefficient text uses 15 significant decimal digits. Fixed notation is used
when its unsigned rendered length is at most 21; otherwise scientific notation
uses `E`, an exponent sign, and at least two exponent digits. There is no
absolute near-integer snapping. Tiny normal coefficients remain present.
Component suppression and the unit imaginary coefficient use the original
values, before formatting. A rounded coefficient below the normal binary64
range is rendered as signed text zero without suppressing its component.

The exact maximum finite magnitude is rejected with `#NUM!`, but its immediate
lower neighbors are admitted. Some admitted coefficients render as
`1.79769313486232E+308`, whose decimal value exceeds the largest binary64 value.
This is an observed admission boundary, not an inferred Excel mechanism.

The initial research predictor used ties-even and rejected every coefficient
whose rounded decimal overflows binary64. The ties and extremes probes rejected
those parts. The first production candidate used exact decimal half-away
rounding and the observed strict maximum boundary. It matched the first 5,544
rows, then matched only 4,371/4,375 rows in the first independent holdout.
All four failures share magnitude bits `0x38cf805c3cd702b2`: the exact value is
slightly below the decimal midpoint, but Excel rounds upward to
`4.73980498978759E-35`. The first candidate's hash and rules remain in
`candidate-freeze.json`; its manifest was written after capture, although the
same source hash had been communicated before the oracle run. The capture
manifest and all four failures remain in `heldout-capture-manifest.json` and
`heldout-first-candidate-judge.json`.

The 280-case search compared exact decimal rounding (268 matches), rounding
the decimal input to 19 digits first (236), and multiplying by a nearest-even
64-bit-significand power of ten (276). The frozen predictions are retained in
`midpoints-hypotheses.json`. The multiplication model's four failures are one
positive-exponent value in both signs and component positions. Division by
the rounded positive power, instead of multiplication by its rounded
reciprocal, distinguishes those cases.

The revised arithmetic model is:

1. Let `k = 14 - floor(log10(abs(n)))`, and `p = RN64(10^abs(k))`.
2. Compute `RN64(abs(n) * p)` for nonnegative `k`, otherwise
   `RN64(abs(n) / p)`.
3. Round that positive scaled value to an integer with ties away, and retain
   the decimal pair `(integer, -k)` for rendering.

Here `RN64` means nearest-even rounding to **64 binary significand bits** with
an unbounded exponent. It does not mean binary64 storage. This is a portable
arithmetic hypothesis derived from public-interface observations and is not a
claim about Excel internals. It matches all 10,199 prior retained rows. Those
rows now count as research/regression evidence, not an independent test of the
revised model. `candidate-scaling-freeze.json` records the revised source hashes
before generation and capture of the second independent cohorts.

The second ordinary holdout matches **4,375/4,375**. The independent arithmetic
search matches only **904/944**: ten positive coefficient centers fail in both
signs and component positions. All 40 failures are retained, without truncation,
in `scaling-heldout-candidate-judge.json`; the complete input predictions and
capture manifest are retained alongside it. Thus the arithmetic model remains
partial and the midpoint lane remains open. No per-value exceptions or fitted
corrections were added. `scaling-candidate-complex-family.rs` and
`scaling-candidate-complex-decimal.rs` preserve the frozen formatter source before
subsequent IM kernel work and explicit comments noting the unresolved outcomes.

## Implementation and formal alignment

`complex_family.rs` performs presentation and the observed coefficient
admission checks. `complex_decimal.rs` implements the rounding model using
portable `u128` arithmetic. `complex_decimal_powers.rs` contains correctly
rounded mathematical powers of ten, generated entirely from exact rational
arithmetic by `gen_complex_decimal_powers.py`. No oracle outputs, per-input
corrections or fitted tolerances are in that table or rounding code. The shared
ROUND helper is unchanged.

`formal/lean/OxFunc/Functions/ComplexFamily.lean` has an executable rational
RN64/scaling model, midpoint alignment examples with exact binary inputs,
decimal presentation, and component assembly. It does not claim a universal
proof equating the Rust implementation with Excel or prove unprobed complex
arithmetic. The existing metadata layer remains intact.

## Reproduction and validation

Generators are under `smart-fuzzer/tools/w111/`:

- `gen_complex_format_20260929.py`: first notation and identity probes.
- `gen_complex_extremes_20260929.py`: extreme neighbors; `--ties` emits ties.
- `gen_complex_holdout_20260929.py`: ordinary independent sweeps, default seed
  2026092902; explicit second argument 2026092904 selects the second sweep.
- `gen_complex_midpoint_search_20260929.py`: first 300,000-draw discriminator
  search, seed 2026092903; it preserves the rejected hypothesis definitions.
- `gen_complex_scaling_holdout.py`: fresh 600,000-draw discriminator search,
  seed 2026092905, with independent minimum/maximum controls.
- `complex_format_candidate_20260929.py`: original rejected research predictor.
- `complex_binary_scaling_candidate.py`: revised exact-rational research model.

`cargo test -p oxfunc_core --offline --test w111_complex_live_replay` replays
four tests and 14,574 matching retained rows through public dispatch with
exact text/error comparison. It does not use a numeric tolerance. Focused
`complex_family` kernel tests passed six cases after the scaling refinement;
the later full campaign run owns overall integration status. `lake build
OxFunc.Functions.ComplexFamily` passes the executable model and examples.

No function metadata, argument-admission declaration or FEC/F3E interface is
changed. No XLL host path is exercised by this COM and prepared-value packet.

Subsequent staged-power research is retained in `staged-scaling-research.json`,
reproduced by `complex_staged_scaling_research.py`. Thirty-two general ascending or
descending nibble/binary-power graphs and eight normalize-then-scale graphs were
compared with all 15,518 retained observations. None matches all rows; the best
staged graph leaves 16 failures, compared with the production candidate's 40.
These are rejected as parity models and have not changed production behavior.

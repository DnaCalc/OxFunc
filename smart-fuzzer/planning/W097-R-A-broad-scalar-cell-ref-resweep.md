# W097 R-A — BUG-FUNC-027 CLASS-C* cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.1`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

This is the per-tranche aggregate for the W097 R-A cell-ref re-sweep of
the BUG-FUNC-027 broad-scalar CLASS-C* surface. The seven literal-text
cycles `broad-scalar-cycle-003 .. broad-scalar-cycle-009` and the single
cell-ref reference cycle `broad-scalar-cycle-010-cellref` were extended
with five additional cell-ref cycles (`011..015`) using fresh seeds so
that each CLASS-C* subclass has an independent multi-seed measurement
under the corrected plumbing.

## 2. Cycles in this tranche

All cycles use `Run-BroadScalarExploration.ps1` with cell-ref plumbing
(`excel_input_plumbing = "cell_value2"`), `1,000,000` local cases per
cycle, `800` Excel-sampled candidates per cycle.

| Cycle                            | Seed | Excel sampled | Matches | Unexpected |
| -------------------------------- | ---: | ------------: | ------: | ---------: |
| `broad-scalar-cycle-010-cellref` | `17` |         `600` |   `468` |      `132` |
| `broad-scalar-cycle-011-cellref` | `23` |         `800` |   `593` |      `207` |
| `broad-scalar-cycle-012-cellref` | `31` |         `800` |   `614` |      `186` |
| `broad-scalar-cycle-013-cellref` | `41` |         `800` |   `602` |      `198` |
| `broad-scalar-cycle-014-cellref` | `53` |         `800` |   `603` |      `197` |
| `broad-scalar-cycle-015-cellref` | `61` |         `800` |   `601` |      `199` |

Cell-ref `match-rate` is stable around `~75%` (vs `~57%` under literal-
text plumbing where the encoding-drift bucket consumed the rest), and
the unexpected-mismatch fraction is stable around `~25%` across all
five fresh seeds — i.e. seed variance does not blur the underlying
class structure.

Excel environment for every cycle: `16.0` build `19929`, workbook
compatibility `2`.

## 3. CLASS-C* subclass characterisation under cell-ref plumbing

ULP magnitudes below are aggregated across the six cell-ref cycles
(`010..015`) for the function families originally covered by each
CLASS-C* subclass in BUG-FUNC-027. Magnitudes for negative-sign or NaN
operand pairs are reported as a separate "saturation/NaN" row because
the wrapper script's `Get-UlpDistance` returns a saturated bit-pattern
distance for `NaN` operand pairs.

### CLASS-C1 — GAMMA negative-non-integer numeric drift

- `39` non-match rows across the six cycles, all `GAMMA(x)` with `x`
  near a negative non-integer pole.
- ULP histogram (finite): `min=0`, `median=90`, `max=2,050`.
- Direction: **shrinks materially**. The original BUG-FUNC-027 witness
  `=GAMMA(-1.00012)` was reported at `237,441` ULP under literal-text
  plumbing; under cell-ref plumbing the matched re-replay
  `=GAMMA(-1.00011965486703613)` is at `1,290` ULP. The other near-pole
  witness `=GAMMA(-1.00061)` was reported at `110,592` ULP and the
  re-replay `=GAMMA(-1.00034887864272481)` is at `1,540` ULP.
- The "237441 ULP / 110592 ULP" magnitudes recorded in
  `BUG-FUNC-027` Section CLASS-C1 were largely an input-side encoding
  artefact — the kernel drift in this band is two orders of magnitude
  smaller. Repair direction (reflection-formula precision) is unchanged
  but the urgency is lower.

### CLASS-C2 — MOD numeric drift

- `31` non-match rows across the six cycles, all `MOD(n, divisor)` with
  `|n| ≫ |divisor|`.
- ULP histogram (finite): `min=0`, `median=2.95E5`, `max=9.51E10`.
- Direction: **persists / confirmed**. Top witness
  `=MOD(9.654E9, -0.374) = 9.51E10` ULP. The original
  `=MOD(-9.26E9, 1.86) → 9.84E9` ULP magnitude is bracketed by the
  re-replay; this remains a clear kernel-side issue with intermediate-
  truncation behaviour in OxFunc's MOD substrate.

### CLASS-C3 — trig / hyperbolic family precision drift in moderate-large argument band

- `147` non-match rows aggregated across `TAN/SIN/COS/COT/SEC/CSC/
  TANH/COTH/SECH/CSCH`.
- Per-function histograms (finite ULP, top six functions):

| Function | rows | min | max         |
| -------- | ---: | --: | ----------: |
| `COT`    | `27` | `0` | `3.34E+12`  |
| `TAN`    | `23` | `0` | `3.34E+12`  |
| `SIN`    | `21` | `0` | `2.03E+12`  |
| `CSC`    | `18` | `0` | `3.34E+12`  |
| `COS`    | `14` | `0` | `2.03E+12`  |
| `COTH`   | `13` | `0` | `1.38E+19`† |
| `TANH`   | `12` | `0` | `2.00E+0`   |
| `SEC`    | `10` | `0` | `3.34E+12`  |

  `†` saturated by NaN-vs-finite distance. See "New finding" below.

- Direction: **grows**. The original BUG-FUNC-027 witness
  `=TAN(797601.58)` at `1.31E7` ULP is in the lower half of the new
  distribution; the maximum trig drift now reaches `3.34E12` ULP for
  `COT/TAN/SEC/CSC` arguments in the `~10^5..10^6` band — this is
  the classical Cody-Waite-vs-double-precision-π argument-reduction
  delta scaling all the way up to one full radian-band of phase loss.
- **New finding (CLASS-C3.h)**: `COTH(x)` with `|x| >> 700` returns
  the IEEE NaN bit pattern `0xfff8000000000000` locally (because both
  `cosh(x)` and `sinh(x)` overflow to `±Inf` and `Inf/Inf = NaN`),
  while Excel saturates to `±1`. Witnesses include
  `=COTH(-145711.069...)`, `=COTH(-682355.955...)`,
  `=COTH(-911970.711...)`, `=COTH(-987452.049...)`. This is
  *type-correct* in OxFunc (a number is returned) but *value-wrong*;
  the `COTH` substrate needs a saturation guard analogous to the
  ones FISHERINV needs (see CLASS-A6 of BUG-FUNC-027).

### CLASS-C4 — ATANH near `±1` precision

- `14` non-match rows; ULP histogram (finite): `min=0`, `median=1`,
  `max=1.48E13`.
- Direction: **stable / confirmed**. Top witness is exactly the
  literal-text-era witness:
  `=ATANH(-0.99999999999999900) → 1.48E13` ULP, both runs. Switching
  to the `0.5 * log1p(2x / (1-x))` formulation remains the right
  repair direction.

### CLASS-C5 — ACOTH and ACOSH near `1`

- `27` non-match rows: `ACOTH=20`, `ACOSH=7`.
- ULP histograms (finite):

| Function | rows | min | max         |
| -------- | ---: | --: | ----------: |
| `ACOTH`  | `20` | `1` | `1.20E+14`  |
| `ACOSH`  | `7`  | `1` | `1.23E+02`  |

- Direction: **broadens**. The literal-text-era witness
  `=ACOTH(1.001)` at `~11244` ULP is reproduced. In addition the new
  data exposes a *large-argument* ACOTH band where both sides return
  values near zero (e.g. `=ACOTH(881958364949856.125)` returns
  `1.110E-15` locally, `1.134E-15` in Excel — `1.20E14` ULP because
  both values are sub-normal-adjacent). Repair direction for ACOTH
  large-argument should be the standard
  `ACOTH(x) = ATANH(1/x) ≈ 1/x + (1/3)*(1/x)^3 + ...` series — under
  the no-tolerance policy this matters even though the absolute
  magnitudes are tiny.

## 4. Cross-class observations

- The literal-text plumbing was suppressing roughly half the
  CLASS-C signal. Under cell-ref plumbing the unexpected-mismatch
  count per cycle stabilises at `~190..210` rather than the
  `~50` of literal-text cycles.
- A new "OxFunc more accurate than Excel" classification is needed
  for integer-valued combinatorial kernels: `COMBIN(23,10) = 1144066`
  exact in OxFunc vs `1144066.0000000002` in Excel; same pattern for
  `COMBIN(9,6)`, `COMBIN(41,16)`, `COMBINA(9,6)` (`26` such rows in
  the six cycles). Under the no-tolerance comparison policy these
  rows should be classified `known_excel_imprecision_witness`, not
  `unexpected_mismatch`. This is W097-broad scope and warrants a
  successor classification helper in `Run-BroadScalarExploration.ps1`
  rather than per-row triage; recorded as a follow-up below.

## 5. Follow-ups opened

1. **CLASS-C3.h COTH NaN-vs-saturated**: needs a separate small bead
   under `BUG-FUNC-027` (CLASS-A subclass — saturation guard, not a
   precision drift). Witness already minimised:
   `=COTH(-145711.06975561508443207)`. Drop in same family as
   CLASS-A3 (`SINH/COSH overflow → +Inf`).
2. **OxFunc-more-accurate-than-Excel rows**: extend the comparator
   to map rows where the `local` outcome is the exact integer and the
   `excel` outcome is the same integer ±1 ULP into a new
   `known_excel_imprecision_witness` triage class, so they stop
   landing in `unexpected_mismatch`. Tracked as a follow-up of W097
   itself, not of any specific BUG-FUNC stream.
3. **CLASS-C1 GAMMA repair priority**: the kernel work direction
   (Lanczos-near-pole reformulation) does not change but the urgency
   should be lowered in BUG-FUNC-027's repair-priority rank since the
   actual drift is `~10^3` ULP, not `~10^5..6` ULP.
4. **CLASS-C5 ACOTH large-argument band**: BUG-FUNC-027 CLASS-C5
   should be widened to also cover `ACOTH(x)` for `|x| ≫ 1` — not
   only `|x| ≈ 1`. Repair direction is `ACOTH(x) = ATANH(1/x)`
   reformulation.

## 6. Doctrine

This tranche is a re-measurement only. No kernel repair lands in W097.
The corrected ULP magnitudes here are the input the BUG-FUNC-027
follow-up bead set (`oxf-h*.*`) should use to set repair priority.

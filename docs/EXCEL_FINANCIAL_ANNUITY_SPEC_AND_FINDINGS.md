# Excel annuity functions (PMT/FV/PV/IPMT/PPMT/CUM): W108 Phase-E findings

Status: `characterized_high_confidence_not_bit_exact`

Last reconciled: `2026-07-10`

Target baseline: Excel `16.0` build `20131`, 64-bit, Windows, AMD Zen2. Numeric
inputs were written as exact doubles through `Range.Value2`; results and input
round-trips were captured as binary64 bit patterns.

This record supersedes the Phase-C conclusion that Excel PMT evaluates the
OpenOffice/LibreOffice forward formula. The public OpenOffice/LibreOffice source
remains a useful clean-room comparison source, but the expanded black-box Excel
evidence proves that Excel uses a different, discount-form arrangement.

## 1. Current conclusion

For nonzero `rate`, the Excel PMT path is best described by:

```text
l   = log1p(rate)
t   = nper * l
em  = expm1(-t)
v   = 1 + em
pmt = (pv + fv*v) * rate / em          // type 0
pmt = pmt / (1 + rate)                 // additional type-1 step
```

The important points are:

1. Excel uses the discount arrangement. The current OxFunc PMT kernel therefore
   has the correct high-level arrangement.
2. Excel's `log1p`/`expm1` behavior is consistent with the historical
   Kahan/Goldberg compensation formulas built on the already identified x87
   `EXP`/`LN` substrate.
3. The exact final binary64 rounding of the composite
   `expm1(-nper*log1p(rate))` remains unresolved. This is an open OxFunc-vs-Excel
   discrepancy, not an accepted tolerance.

## 2. Why the forward-form conclusion was rejected

The public historical OpenOffice implementation uses a forward form based on
`exp(n*log1p(rate))` and `expm1(n*log1p(rate))`. Excel agrees with it over ordinary
inputs where both algebraic forms round to the same result, but the large-`t`
tail separates them decisively:

| Input | Excel observation | Consequence |
|---|---|---|
| `PMT(0.015625,4096,0,1,0)` | exactly `0` | requires discount-factor cancellation after `v = 1 + em` |
| `PMT(0.015625,2048,0,1,0)` | about `3.2e12` ULP from the accurate forward form | incompatible with the forward arrangement |
| `PMT(0.25,240,0,1,0)` | exactly `0` | again exposes `v = 1 + expm1(-t)` cancellation |

The forward expression remains nonzero on these rows. No last-bit choice inside
that expression can produce Excel's zero, so this is an arrangement discriminator,
not a library-rounding preference.

## 3. Primitive-substrate characterization

The best tested model uses the historical compensation identities:

```text
log1p(x): fp = 1+x
          fp == 1 ? x : LN(fp) * x / (fp-1)

expm1(x): fe = EXP(x)
          fe == 1 ? x : (fe-1) * x / LN(fe)
```

The `LN` and `EXP` operations are the x87 routines already reproduced by
`crate::excel_numeric::x87`. A 4,040-row PMT sweep over operation order and
SSE/x87-extended store placement produced this best candidate:

| Model | Exact | Within 1 ULP |
|---|---:|---:|
| `log1p` extended, `expm1` binary64 order A | `2285/4040` (`56.6%`) | `92.9%` |
| both helpers binary64 | `55.2%` | `92.2%` |
| UCRT C99 helpers | `47.8%` | `89.0%` |
| fdlibm helpers | `51.9%` | `91.9%` |
| naive `exp(x)-1` | `21.5%` | `41.8%` |

The clean `nper=1`, `pv=1`, `fv=0`, `type=0` isolation lane reduces PMT to
`rate/em`. Across 553 such rows the final division was exact whenever the
candidate `em` was exact (`div-only-wrong = 0`). That localizes the remaining PMT
last-bit problem to `em = expm1(-log1p(rate))`, not PMT's final division.

## 4. Adjacent-family observations

The expanded live corpus contains 5,319 rows across the financial family. The
public source recurrences, evaluated with the current best research model, score:

| Function | Current research-model result |
|---|---|
| `RRI` | `65/65` bit-exact |
| `PDURATION` | `17/17` bit-exact |
| `FV` | `79/90` exact; `94%` within 1 ULP |
| `PV` | `25/90` exact; `62%` within 1 ULP |
| `IPMT` | `55/180` exact; `41%` within 1 ULP |
| type-1 `IPMT` | `64/180` exact; `48%` within 1 ULP |
| `PPMT` | `17/180` exact; `31%` within 1 ULP |
| `CUMIPMT` / `CUMPRINC` | recurrence shape supported, exact op-order and accumulated rounding still open |

These figures characterize the research model, not the current Rust kernel.
They must not be reported as OxFunc pass rates. The current Rust implementation
has not yet been replayed over all 5,319 Phase-E rows through a repo-owned runner.

## 5. Current OxFunc decision

No Rust kernel change is promoted from Phase E yet.

The current OxFunc PMT implementation already uses the empirically correct
discount arrangement on the portable `excel_log1p`/`excel_expm1` core and closed
the canonical mortgage witness exactly. The Phase-E Kahan/x87 candidate does not
yet dominate that implementation across the established realistic and
adversarial corpora, and it still misses roughly 43% of the 4,040 adversarial PMT
rows by a last bit. Swapping models now would be a partial trade, not parity.

The next implementation gate is:

1. import the Phase-E probe set into a repo-owned replay format,
2. score current OxFunc and the Kahan/x87 candidate over the same exact inputs,
3. isolate the partial-extended rounding placement inside the two helpers,
4. promote only a non-regressing change with exact-bit regression pins.

## 6. Evidence and reproduction

Validated on `2026-07-10` from `C:\Temp\ExcelExpFunction`:

1. `python validate_reference.py research/data/ground_truth_all.json research/data/disc2_results.json`
   - x87 EXP/LN reference: `294/294` exact.
2. `python validate_power_reference.py`
   - POWER reference: `315/315` exact.
3. `python finlab/final_sweep.py`
   - `4040` PMT rows; best helper configuration `2285/4040` exact, `92.9%` within 1 ULP.
4. `python finlab/score_family.py`
   - `855` adjacent-family rows; `244/855` exact for the tested public-source recurrence model.

Primary Phase-E report SHA-256:
`14647C7B461D2198DE2817363D172CAC3A19B8CB1025B0E3BBE57AE566DEC955`.

The compact repo-owned witness seed is
[`W108_ANNUITY_PHASE_E_WITNESS_SEED.csv`](function-lane/W108_ANNUITY_PHASE_E_WITNESS_SEED.csv).

## 7. Claim boundaries

- `EXP`, `LN`, `LOG10`, `LOG`, and `POWER` are separate W108 results and remain
  signed off on the declared x86-64 reference baseline.
- The PMT/PPMT/IPMT/CUM family remains `scope_partial` for exact current-baseline
  parity.
- Alternate CPU, Excel channel, locale, and workbook Compatibility Version
  sweeps remain separate validation lanes.

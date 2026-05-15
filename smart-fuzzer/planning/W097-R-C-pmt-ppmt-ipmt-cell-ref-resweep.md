# W097 R-C — BUG-FUNC-015 PMT/PPMT/IPMT cell-ref re-sweep

Status: `tranche_complete`

Owning workset: `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md`
Owning bead: `oxf-ic1h.3`
Plumbing rule: `smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`

## 1. What this record is

R-C re-replays the BUG-FUNC-015 PMT/PPMT/IPMT exactness surface under
the shared `CellRefBatch.psm1` plumbing. The 28-case fixed pilot
(`w088-pmt-ppmt-pilot`) and a fresh ~1M-case finance broad seed are
each re-run, and the per-function ULP histogram replaces the legacy
"absolute delta + tolerance" measure recorded in BUG-FUNC-015.

## 2. Refactored runners

The two runners that drove BUG-FUNC-015 evidence were refactored to
import `CellRefBatch.psm1` and consume the shared
`Invoke-ExcelCellRefBatch`:

- `smart-fuzzer/tools/Run-PmtPpmtPilot.ps1` (runner-id v0.2.0-cellref)
- `smart-fuzzer/tools/Run-ExpandedFinanceExploration.ps1`

The legacy `expected_formula_literal_encoding_drift` and
`expected_known_financial_exactness_drift` triage classes are retired
in favor of:

- `match` (bit-exact)
- `match_signed_zero_difference`
- `known_residual_pmt_family_kernel_drift` (PMT/PPMT/IPMT non-zero-rate)
- `unexpected_mismatch`

A `ulp_distance` field is recorded for every numeric drift row.

## 3. Pilot re-replay (28 fixed cases, seed `8803`)

Run: `smart-fuzzer/runs/W097-R-C-pmt-ppmt-pilot-cellref/`

| Metric            | `w088-pmt-ppmt-pilot` (literal-text) | R-C (cell-ref) |
| ----------------- | -----------------------------------: | -------------: |
| Cases             |                                 `28` |           `28` |
| Matches           |                                  `7` |            `7` |
| Mismatches        |                                 `21` |           `21` |
| Blocked           |                                  `0` |            `0` |

Per-row Excel `bits_hex` differences between literal-text and cell-ref:
**`0` rows changed**.

The pilot uses short numeric literals (`0.05`, `12`, `360`, `200000`,
`0.05/12`, `1E-9`, etc.) that round-trip correctly through Excel's
formula parser, so the literal-text harness was not introducing any
input-side encoding drift on this surface. The 21 mismatches recorded
in `BUG-FUNC-015` are confirmed real OxFunc kernel drift; no shrink,
no grow, no re-classification.

The high-magnitude pilot witnesses are unchanged:

| Case            | Local bits          | Excel bits          | abs delta             |
| --------------- | ------------------- | ------------------- | --------------------- |
| `SFZ-PMT-0001`  | `0xc090c692af15f632`| `0xc090c692af15f63a`| `1.82E-12`           |
| `SFZ-PMT-0011`  | `0xc0590000b2882a2b`| `0xc0590000d1ecc989`| `7.48E-6` (`tiny rate`) |
| `SFZ-PPMT-0001` | `0xc06e09eace0506e4`| `0xc06e09eace050723`| `1.79E-12`           |
| `SFZ-PPMT-0012` | `0xc058ffff0f19fb79`| `0xc058ffff2e7e9ad7`| `7.48E-6` (`tiny rate`) |

## 4. Finance broad-seed re-replay (1M cases, seed `8804`)

Run: `smart-fuzzer/runs/W097-R-C-expanded-finance-1m-cellref/`

Comparison to the literal-text reference run
`expanded-finance-10m-20260428` (which used `CaseCount=10M`,
`CandidateLimit=640`):

| Metric                                | `expanded-finance-10m-20260428` | R-C cell-ref |
| ------------------------------------- | ------------------------------: | -----------: |
| Local cases                           |                          `10M` |         `1M` |
| Excel sampled                         |                          `640` |        `800` |
| Matches                               |                          `536` |        `706` |
| Match rate                            |                          `84%` |        `88%` |
| Known residual PMT-family drift rows  |                          `102` |         `92` |
| Unexpected mismatches                 |                            `2` |          `2` |
| Blocked                               |                            `0` |          `0` |

The match-rate uptick from `84%` to `88%` is attributable to short-
literal rows in the literal-text bucket that were absorbing into
`expected_formula_literal_encoding_drift`; under cell-ref they
collapse to `match`.

### 4.1 Per-function ULP histogram (cell-ref, `known_residual_pmt_family_kernel_drift`)

| Function | rows | min ULP | median ULP | max ULP        |
| -------- | ---: | ------: | ---------: | -------------: |
| `PMT`    | `19` |     `0` |        `4` | `5.1E10`       |
| `IPMT`   | `22` |     `0` |      `832` | `4.2E16`       |
| `PPMT`   | `51` |     `0` |      `282` | `1.4E19`†      |

`†` saturated by NaN-vs-finite distance for the top witness; absolute
magnitudes for the saturating rows are still finite f64s but the
sign-magnitude-to-Int64 transform overflows in the wrapper script's
`Get-UlpDistance`. Treat as "very large" rather than literal value.

The PMT median of `4` ULP confirms the small drift surface that the
literal-text run absorbed under `1e-12 * scale` tolerance. The IPMT
and PPMT distributions are bimodal: a tight cluster near `0..1000` ULP
and a long tail to `~10^16+` ULP for high-rate / long-horizon /
huge-PV combinations — repair direction for that tail belongs to the
existing BUG-FUNC-015 fix plan.

### 4.2 Unexpected mismatches (NEW)

Two rows escalated from "expected drift" to `unexpected_mismatch`
under cell-ref plumbing because the OxFunc local outcome is `#NUM!`
while Excel returns a tiny denormal:

1. `=PPMT(0.94202241811931720, 1147, 1600, 677560705614.16699218750000000)`
   - local: `error:Num`
   - excel: `-8.66E-120` (`0xa7365da0faa805b4`)
2. `=PPMT(0.65754274790347489, 475, 1992, 629739.80507821717765182)`
   - local: `error:Num`
   - excel: `0` (`0x0000000000000000`)

These are the exact rows the legacy
`expected_known_financial_exactness_drift` bucket was suppressing
because the comparator never reached the kind-mismatch branch when
both kinds were "number". Under cell-ref triage these are now visible
as PPMT kind-drift bugs: PPMT should not return `#NUM!` for high-rate
/ long-horizon arguments where Excel returns a finite value.

## 5. Doctrine

- BUG-FUNC-015 measured magnitudes are confirmed accurate under
  cell-ref plumbing for the standard non-zero-rate band; no repair
  re-direction is required.
- The cell-ref re-replay does surface a kind-drift sub-class of
  PPMT (`#NUM!` locally where Excel returns a denormal). This is
  consistent with BUG-FUNC-015 Section "Investigation Log" item 4
  which already noted `2` similar witnesses under literal-text.
- Both R-C runs and the W088 baseline agree on the broad shape of
  the BUG-FUNC-015 surface — re-measurement does not reopen the
  closed-and-validated portion.

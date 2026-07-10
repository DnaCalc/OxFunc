# G6 three-way checking (OxFunc / F# / live Excel)

Bit-exact cross-check of the open **G6 financial** discrepancy catalog rows against
two independent references: the public **ExcelFinancialFunctions** F# library
(Luca Bolognese) and **live Excel** via COM.

Every open G6 catalog row maps to a function the F# library implements — and *only*
G6 does (the F# library is purely financial; G3/G4/G5 have no F# counterpart). So
this harness is coextensive with the financial group.

## Why three-way

A two-way OxFunc-vs-Excel diff tells you *that* you differ. The third leg tells you
*why*, by partitioning every gap:

| Class | Meaning | What it tells us |
|-------|---------|------------------|
| `all_bit_exact` | ox == fs == excel | already matched — recheck/close the row |
| `fs_exact_ox_off` | F# == Excel, OxFunc differs | **F# source is a direct repair roadmap** |
| `both_off_ox_eq_fs` | OxFunc == F#, both differ from Excel | **Excel is idiosyncratic** (its solver/op-order); F# is *not* a roadmap — replicate Excel |
| `all_diverge` | all three differ | closed-form op-order or solver drift; F# also off, so neither reference is a turnkey fix |

`ulp_ox_excel` / `ulp_fs_excel` are IEEE-754 ULP distances (informational; equality
is hex-of-bits, not tolerance).

## Run

```powershell
# full pass (builds F# DLL if absent, runs cargo local-eval, drives live Excel COM)
.\tools\g6-threeway\run-g6-threeway.ps1

# smoke a subset
.\tools\g6-threeway\run-g6-threeway.ps1 -Only pmt.witness,irr.unit,oddfprice.witness

# partial engines
.\tools\g6-threeway\run-g6-threeway.ps1 -SkipExcel    # OxFunc + F# only
```

Ledger lands at `.tmp/g6-threeway-ledger.csv`; the committed snapshot is
`docs/function-lane/G6_THREEWAY_LEDGER.csv`.

## Files

- `g6-cases.ps1` — the witness set (single source of truth). Neutral typed args
  (`num`/`date`/`freq`/`basis`/`paydue`/`array`) render identically into all three
  engines. Add a case here; all three pick it up.
- `run-g6-threeway.ps1` — renders + runs + classifies.

## Bit-exactness guarantees

- The **same double** reaches all three engines: F# reconstructs numeric args from
  their `Int64` bits (`BitConverter.Int64BitsToDouble`), Excel receives them via
  `Range.Value2` cell-refs (never formula-literal text, which the parser re-rounds),
  OxFunc via round-trip `R`-format JSON numbers.
- Excel results are read by **.NET type** (`[double]` = number, int code = error),
  not `.Text` — `########` (column-width) and `$`/thousands formatting fool `.Text`.

## Reading the current ledger (see CSV for full digests)

- **`fs_exact_ox_off` is now empty.** The cases where F# matched Excel and OxFunc
  didn't — ODDFPRICE across all five bases (the actual-day bases were 10^10–10^12 ULP
  off) and YIELDDISC (5 ULP) — were repaired by porting the F# `oddFPrice`/`yieldDisc`
  forms and are now `all_bit_exact` (OxFunc == F# == Excel, 0 ULP). 2026-06-20.
- **Excel-solver-only** (F# agrees with OxFunc, both off Excel): YIELD (19 ULP),
  YIELDMAT (1 ULP). These need Excel's exact root-finder iteration, not F#.
- **Closed-form op-order** (all three differ, F# no help): PMT/PPMT, CUMPRINC.
- **Solver-dominated divergence** (all three differ): ODDFYIELD, RATE, IRR — the price
  kernel is now exact but the inversion solver isn't; shared financial-solver substrate.
- **Already bit-exact on the probed witness**: IPMT, ACCRINT (30/360 + the constructed
  act/act multi), and NPER. Their distinct drifting witnesses are now separately
  pinned in the discrepancy reconnaissance corpus.
- **TBILLYIELD signed off 2026-07-10**: expression association repaired from
  left-associative `*360/days` to `*(360/days)`; expanded live matrix `2156/2156`.

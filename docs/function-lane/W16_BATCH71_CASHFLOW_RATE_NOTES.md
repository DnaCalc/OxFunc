# W16 Batch 71 - Cashflow Rate Family

Status: `packet-evidenced-by-w24`
Workset: `W16`
Function lanes: `IRR`, `XNPV`, `XIRR`

## Admitted Local Slice
1. Numeric cashflow vectors only.
2. `IRR` supports a 1-D numeric cashflow vector plus optional scalar guess.
3. `XNPV` supports scalar discount rate, 1-D numeric cashflow vector, and 1-D numeric Excel-serial date vector.
4. `XIRR` supports 1-D numeric cashflow vector, 1-D numeric Excel-serial date vector, and optional scalar guess.
5. `XNPV` and `XIRR` interpret dates as truncated Excel serial numbers and discount by `(date_i - date_0) / 365` using the first supplied date as the anchor.
6. The iterative admitted slice uses a bounded hybrid secant/Newton solver with:
   - default guess `0.1`,
   - rate floor just above `-1`,
   - tolerance `1e-8`,
   - iteration cap `100`.

## Explicitly Out Of Slice
1. Native Excel replay parity across alternative iteration paths and difficult multi-root cases.
2. Text/logical/blank cashflow or date coercions beyond the strict numeric slice.
3. Multi-column matrix cashflow/date inputs.
4. Date vectors containing values earlier than the first date; this bounded slice rejects them as `#NUM!`.
5. Shared dispatch/export integration in this subtask; shared files were intentionally left untouched.

## Local Evidence
1. Rust unit tests cover metadata shape, exact two-cashflow `IRR`, one-year `XNPV`, one-year `XIRR`, sign-change rejection, length mismatch rejection, pre-anchor-date rejection, reference resolution, optional guess admission, and out-of-slice matrix rejection.
2. `W24` Batch 12 now adds native Excel replay evidence in `docs/function-lane/W24_BATCH12_CASHFLOW_RATE_SCENARIO_MANIFEST_SEED.csv` with output `.tmp/w24-batch12-cashflow-rate-results.csv`.

## Current Standing
1. The admitted current-baseline numeric vector/date-vector slice is now packet-evidenced by `W24`.
2. This note remains the original batch snapshot, not the current closure record.

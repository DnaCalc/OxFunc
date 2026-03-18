# Function Slice Contract (Preliminary) - Lookup / Prob / Frequency Family

Status: `provisional`
Workset: `W24`
Primary Functions: `LOOKUP`, `FREQUENCY`, `PROB`, `MODE.MULT`

## 1. Scope
1. close the admitted current-baseline slice for the lookup/probability/frequency family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the old incorrect `PROB` sum-mismatch result with the empirically pinned `#NUM!` rule.

## 2. Admitted Current-Baseline Slice
1. `LOOKUP`
   - numeric lookup value only,
   - ascending 1-D lookup vector only,
   - optional 1-D result vector of matching length,
   - array form admitted for numeric 2-D arrays using Excel's row-vs-column heuristic:
     - tall arrays search first column and return from last column,
     - wide arrays search first row and return from last row,
   - approximate last-`<=` selection only.
2. `FREQUENCY`
   - numeric 1-D data and bin vectors,
   - nonnumeric data/bin cells ignored in the admitted slice,
   - returns the raw vertical count array, witnessed here through `ARRAYTOTEXT(...,1)`.
3. `PROB`
   - strict numeric 1-D `x_range` and `prob_range`,
   - scalar numeric lower limit and optional scalar numeric upper limit,
   - probability vector must sum to `1` within tolerance or return `#NUM!`,
   - omitted upper limit means point probability.
4. `MODE.MULT`
   - numeric 1-D inputs only,
   - nonnumeric cells ignored in the admitted slice,
   - returns all surviving modes as a vertical array sorted ascending, witnessed here through `ARRAYTOTEXT(...,1)`.

## 3. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `refs_visible_in_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `ref_only`
8. surface_fec_dependency_profile: `ref_only`

## 4. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/lookup_prob_frequency_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/LookupProbFrequencyFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH08_LOOKUP_PROB_FREQUENCY_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch08-lookup-prob-frequency-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH08_LOOKUP_PROB_FREQUENCY_EXECUTION_RECORD.md`

## 5. Scope Boundary
1. The closure is bounded to the admitted current-baseline lookup/probability/frequency slice.
2. Broader mixed-type comparison/coercion breadth for `LOOKUP`, unsorted lookup vectors, unsorted frequency bins, and spill-host publication nuances remain outside this packet.

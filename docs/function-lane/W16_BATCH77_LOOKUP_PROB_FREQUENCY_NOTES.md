# W16 Batch 77 - Lookup / Prob / Frequency Family

Status: `packet-evidenced`
Workset: `W16`
Function lanes: `LOOKUP`, `FREQUENCY`, `PROB`, `MODE.MULT`

## Admitted Local Slice
1. `LOOKUP`:
   - numeric lookup value only,
   - ascending 1-D lookup vector only,
   - optional 1-D result vector of matching length,
   - array form admitted only for numeric 2-D arrays using Excel's row-vs-column heuristic:
     - tall arrays search first column and return from last column,
     - wide arrays search first row and return from last row,
   - approximate last-`<=` selection only.
2. `FREQUENCY`:
   - numeric 1-D data and bin vectors,
   - nonnumeric data/bin cells are ignored in the admitted slice,
   - bins must be ascending,
   - returns the raw vertical count array.
3. `PROB`:
   - strict numeric 1-D `x_range` and `prob_range`,
   - scalar numeric lower limit and optional scalar numeric upper limit,
   - probability vector must sum to `1` within a small tolerance,
   - omitted upper limit means point probability.
4. `MODE.MULT`:
   - numeric 1-D inputs only,
   - nonnumeric cells are ignored in the admitted slice,
   - returns all surviving modes as a vertical array sorted ascending.

## Explicitly Out Of Slice
1. Full Excel mixed-type comparison/coercion breadth for `LOOKUP`.
2. Unsorted lookup vectors and unsorted frequency bins; this bounded slice rejects them.
3. Broader matrix semantics beyond the admitted `LOOKUP` array-form heuristic.
4. Full dynamic-array publication nuances beyond the raw returned arrays for `FREQUENCY` and `MODE.MULT`.
5. `W24` Batch 08 now supplies the native worksheet packet and closes the old local-only state on the integrated surfaces.

## Local Evidence
1. Rust unit tests cover metadata shape, vector-form and array-form `LOOKUP`, text return from the result vector, vertical `FREQUENCY` counts, point/range `PROB`, vertical sorted `MODE.MULT`, no-mode `#N/A`, reference resolution, and explicit out-of-slice unsorted/matrix lanes.
2. `W24` Batch 08 adds the missing native Excel packet, including the corrected `PROB` non-unit-sum `#NUM!` lane.

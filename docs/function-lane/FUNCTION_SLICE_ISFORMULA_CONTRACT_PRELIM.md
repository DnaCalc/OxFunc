# Function Slice Contract (Prelim) - ISFORMULA()

## 1. Purpose
Pin the admitted current-baseline OxFunc slice for `ISFORMULA`.

## 2. Admitted Slice
1. one-argument reference form only,
2. operand must preserve reference identity into the adapter,
3. OxFunc owns argument admission and worksheet projection,
4. host side owns the truth of whether the referenced cell contains a formula.

## 3. OxFunc / Host Seam
1. `arg_preparation_profile`: `refs_visible_in_adapter`
2. required host callback:
   - `query_cell_info(CellInfoQuery::IsFormula, Some(reference))`
3. returned host fact is projected directly to worksheet logical `TRUE` / `FALSE`.

## 4. Current-Baseline Findings
1. formula cells return `TRUE`.
2. non-formula value cells return `FALSE`.
3. formulas returning text still count as formulas.
4. non-reference operands return worksheet-visible `#VALUE!`.

## 5. Status
1. runtime_status: `evidenced`
2. seam_status: `typed_host_query_pinned`
3. closure_reading: this slice is now closure-grade for the current reference baseline inside `W23`.

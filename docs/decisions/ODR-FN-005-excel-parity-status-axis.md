# ODR-FN-005: Excel Parity Status Axis on FunctionMeta

- **Status**: proposed
- **Date**: 2026-09-22
- **Context**: Parity evidence lives in prose comments, catalog rows, ledgers, a July
  status-map snapshot and the Handbook. Collating "what matches Excel" took four agents
  and produced a classification that mixed how strong the evidence is with how far
  off a known divergence is. We also cannot check every bit, and Excel changes, so no
  status for a function is ever final.
- **Decision**: add one declared axis, `excel_parity`, to `FunctionMeta`, with the
  shape below, backed by a per-function evidence ledger we keep adding to.
- **Consequences**: every function spec states its current parity picture where the
  compiler, the registry export and the Handbook can read it; the picture is
  provisional and revisable.
- **Cross-repo impact**: registry export gains three columns; OxFml/OxCalc may read
  them; Handbook ingest maps them onto `efh.compatibility`. The columns sit on
  `RegistryFunctionMeta` directly, outside `FunctionSpecAxesMetadata`, so a parity
  update never advances the `function_spec_axes_metadata` version key OxFml
  invalidates on.

## 1. Principles

1. **Provisional.** A status is what the evidence supports today, on the builds judged
   so far. Any function can be revisited, and a new miss demotes it.
2. **Two axes, never mixed.** Evidence strength says how much we have looked. Severity
   says how far the worst known divergence is. One known differing row makes the
   function `Divergent`, however many rows agree elsewhere.
3. **Evidence accumulates.** The ledger records what was judged, on which build, with
   what result, and where the detail lives. Partial-domain agreement is evidence to
   build on, not a tick.

## 2. The field

```rust
/// Current Excel-parity picture for this function. Provisional: says what the
/// evidence in `docs/function-lane/EXCEL_PARITY_LEDGER.csv` supports as of `as_of`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExcelParity {
    pub status: ExcelParityStatus,
    /// Date or snapshot key of the evidence this value was set from.
    pub as_of: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcelParityStatus {
    /// No oracle evidence for this surface. The default for every spec.
    Unverified,
    /// Every judged row agrees with Excel, but the rows are a fitted or local suite
    /// and there is no identified kernel story. "We have not yet seen a miss."
    Consistent,
    /// Every judged row agrees with Excel, the kernel (or exact-by-construction
    /// nature) is written down, and at least one fresh held-out sweep exists.
    Characterized,
    /// At least one known row differs from Excel. Confidence of exactness is zero.
    Divergent { severity: DivergenceSeverity },
    /// Outside the campaign by policy. Only `CUBE*`, `WEBSERVICE`, `STOCKHISTORY`.
    Deferred,
}

/// Size of the worst known divergence. A function carries its worst class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DivergenceSeverity {
    /// ≤ 4 ULP on every known miss, same sign, magnitude and type.
    LastBit,
    /// 5..=1024 ULP. Right algorithm, wrong op-graph or coefficients.
    Numeric,
    /// > 1024 ULP, wrong magnitude or sign. Wrong algorithm or solver.
    Gross,
    /// Wrong type, error code, shape, text, or error-vs-value.
    Structural,
}
```

Deliberately not in the enum: worst-ULP numbers, partial-domain notes, substrate
links, per-build detail. Those go in the ledger.

## 3. The ledger

`docs/function-lane/EXCEL_PARITY_LEDGER.csv`, one row per function, columns roughly:

```
function_id, status, severity, as_of, excel_builds_judged, rows_judged, rows_agree,
worst_ulp, exact_domain_note, kernel_story_ref, blocked_by, evidence_refs
```

`evidence_refs` is a list of pointers (catalog row, run id, wall-ledger line, Handbook
record, lane doc) and grows as we learn more. When a row's picture changes, update the
row; git keeps the history.

Two consistency checks are worth having from the start, both cheap: the spec value
equals the ledger row, and no `Consistent`/`Characterized` function appears in an
open row of the discrepancy catalog.

## 4. What counts as a held-out sweep

A sweep supports `Characterized` when the corpus was generated after the kernel's
last change, covers the argument-domain regions the kernel branches on plus the edge
rows (zero, negative zero, subnormal, ±max, midpoints, integer arguments, errors,
blank, text, logical, array shapes where lifted), is large enough for the kernel's
branch count (order of 1,000 numeric rows as a floor), records the live build and
Compatibility Version, and comes with a stated reason the bits agree.

## 5. Incompleteness scale (campaign ordering only)

| Rank | Class | Why first |
|---|---|---|
| 1 | `Unverified` with a reachable corpus | Cheapest way to learn the truth |
| 2 | `Divergent{Structural}` | A rule, not a kernel; widest wins |
| 3 | `Unverified` because harness-blocked | Unblocks whole families |
| 4 | `Divergent{Gross}` outside the six walls | Wrong algorithm; large payoff |
| 5 | `Divergent{Numeric}` outside the walls | Op-graph search with the racer tooling |
| 6 | `Divergent{LastBit}` not on a wall | Independent last-bit rows, timeboxed |
| 7 | `Consistent` → `Characterized` sweeps | Turns counts into evidence |
| 8 | The six walls | Search spaces exhausted; needs new ideas |

A function inheriting a wall (NORMSDIST on the ERFC body, IPMT on the PMT expm1
helper) is `Divergent` with the inherited severity and names the wall in
`blocked_by`. It is not worked directly.

## 6. Open

Whether population is by hand from the ledger or by a generator, how re-judging is
scheduled when a new Excel build arrives, and whether the ledger later needs
per-observation rows are all left open until the first population has been done.

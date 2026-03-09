# WORKSET - TUX1000 Value Universe and Extended Types (W3)

## 1. Purpose
Define the formal value universe for OxFunc/F3E value semantics with explicit boundary-specific sets.

Primary question:
1. what counts as a value at each boundary (cell content, formula eval, call args, references, interop surface).

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W3 in `W000_KICKOFF_PROGRAM_W001_W006.md`.

Dependencies:
1. depends on W2 floating-point characterization baseline.
2. consumes W7 string characterization feed where available (advisory during early W3 drafting; required before W3 validation closure).

Downstream consumers:
1. W4 coercion and resolver seam typing,
2. W5 and W6 function contract correctness.

## 3. Scope
In scope:
1. value-set decomposition (`CellContentValue`, `EvalValue`, `CallArgValue`, `ReferenceLike`, `ExtendedValue`).
2. disputed categories (`missing`, `empty`, `null`, lambda intermediates, 3D references) with evidence anchors.
3. error-family taxonomy and versioning notes (legacy/newer errors).
4. array and dynamic-spill participation in value algebra.

Out of scope:
1. full workbook scheduler semantics.
2. full localization closure beyond explicit bounded notes.

## 4. Required Axes per Claim
1. Excel app version/channel.
2. workbook Compatibility Version.
3. boundary context (`formula`, `materialized cell`, `reference reuse`, `interop`).

## 5. Deliverables
1. `docs/function-lane/VALUE_UNIVERSE_RESEARCH_AND_OPEN_QUESTIONS.md` (updated)
2. `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`
3. `docs/function-lane/VALUE_UNIVERSE_TAG_TABLE.csv`
4. Lean value-tag scaffold and invariants
5. Rust mirrored value-tag scaffold
6. conformance-row linkage updates (`FDEF-028` and affected rows)

## 6. Gate Model
### G1 - Taxonomy Closure
Pass when:
1. value sets are explicit, bounded, and non-overlapping by declared interpretation.

### G2 - Evidence Closure
Pass when:
1. disputed categories have source or empirical anchors with open points recorded.

### G3 - Formal/Runtime Closure
Pass when:
1. Lean and Rust scaffolds compile using shared tag vocabulary.

### G4 - Integration Closure
Pass when:
1. downstream worksets (W4/W5/W6) have explicit dependencies mapped to W3 outputs.

## 7. Status
Execution state:
1. `in_progress`.

Claim confidence:
1. `provisional` for baseline taxonomy/spec + Rust/Lean scaffolds.
2. `draft` for full validation pending expanded evidence beyond current W7 baseline scope.

Gate snapshot:
1. `G1` taxonomy closure: `closed-provisional`.
2. `G2` evidence closure: `closed-provisional` (open points explicitly tracked).
   - includes W7 baseline feed consumption (`W7-STR-BL-20260305`) for text-boundary constraints.
3. `G3` formal/runtime closure: `closed` (Rust + Lean compile passing).
4. `G4` integration closure: `open` (W4/W5/W6 consumption not yet executed).


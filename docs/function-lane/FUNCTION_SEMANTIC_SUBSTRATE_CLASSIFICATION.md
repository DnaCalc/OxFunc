# Function Semantic Substrate Classification

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Classify the functions covered so far by semantic substrate rather than by Excel UI category.

This note is intended to support:
1. Lean formalization planning
2. Rust substrate extraction decisions
3. contract/profile consistency checks

It is provisional and may change as more functions are characterized.

## 2. Classification Rule
This classification is a rough semantic partition, not a second profile system.

Rule:
1. each function gets exactly one primary semantic home
2. cross-cutting traits remain in the formal profile fields and contract notes
3. this note should not be used to restate the profile matrix in informal prose

Primary substrate means:
1. the most important reusable semantic machinery needed to model the function correctly
2. the substrate that would normally own the highest-value Lean executable model for that function

## 3. Current Substrate Vocabulary
Current working substrate classes for the covered function set:

1. `ConstScalar`
   - nullary deterministic constant production
2. `NumericUnary`
   - unary numeric kernel with scalar/array coercion/lift behavior
3. `NumericNary`
   - direct numeric arithmetic over one or more scalar inputs
4. `AggregateProvenance`
   - aggregate behavior sensitive to direct-vs-range/reference provenance
5. `LogicalFold`
   - variadic logical reduction with Excel-specific ignore/error/direct-vs-reference policy
6. `LogicalControlLazy`
   - branch or fallback laziness as a primary semantic feature
7. `LookupSelection`
   - exact/reverse/wildcard/approximate/binary lookup selection over comparable candidates
8. `ReferenceSelectionReturn`
   - reference-preserving selection/projection over cells/areas
9. `ReferenceConstruction`
   - construction or transformation of reference identities rather than plain values
10. `ReferenceTextInterpretation`
   - text-to-reference interpretation with caller/workbook sensitivity
11. `TextCoercionNormalization`
    - Excel-grade scalar-to-text, text comparison, or control-character normalization
12. `TypePredicateClassification`
    - classification/predicate behavior over already-admitted values
13. `ArrayShapeConstruction`
    - dynamic/spill/shape-producing or shape-transforming array behavior
14. `ProviderEffectMetadata`
    - provider-fed values and post-evaluation metadata such as format hints
15. `DateSerialArithmetic`
    - date serial normalization and calendar/1900-system behavior
16. `WorkbookInfoContext`
    - workbook/caller-context-dependent informational behavior

## 4. Covered Functions by Substrate

### 4.1 `ConstScalar`
Primary members:
1. `PI`

### 4.2 `NumericUnary`
Primary members:
1. `ABS`
2. `ROUND`

### 4.3 `NumericNary`
Primary members:
1. `OP_ADD`

### 4.4 `AggregateProvenance`
Primary members:
1. `SUM`
2. `AVERAGE`
3. `COUNT`
4. `COUNTA`

### 4.5 `LogicalFold`
Primary members:
1. `AND`

### 4.6 `LogicalControlLazy`
Primary members:
1. `IF`
2. `IFERROR`

### 4.7 `LookupSelection`
Primary members:
1. `XMATCH`
2. `MATCH`
3. `XLOOKUP`

### 4.8 `ReferenceSelectionReturn`
Primary members:
1. `INDEX`
2. `XLOOKUP`

### 4.9 `ReferenceConstruction`
Primary members:
1. `OFFSET`

### 4.10 `ReferenceTextInterpretation`
Primary members:
1. `INDIRECT`

### 4.11 `TextCoercionNormalization`
Primary members:
1. `TEXTJOIN`
2. `CLEAN`
3. `EXACT`

### 4.12 `TypePredicateClassification`
Primary members:
1. `ISNUMBER`

### 4.13 `ArrayShapeConstruction`
Primary members:
1. `SEQUENCE`
2. `HSTACK`

### 4.14 `ProviderEffectMetadata`
Primary members:
1. `NOW`
2. `TODAY`
3. `RAND`

Notes:
1. `NOW` and `TODAY` also participate in format-hint metadata.
2. `RAND` shares provider/effect shape, but not the time/date semantics.

### 4.15 `DateSerialArithmetic`
Primary members:
1. `DATE`

### 4.16 `WorkbookInfoContext`
Primary members:
1. `CELL`

## 5. Per-Function Primary Classification
Current primary substrate assignment for each covered function:

1. `PI` -> `ConstScalar`
2. `ABS` -> `NumericUnary`
3. `XMATCH` -> `LookupSelection`
4. `SUM` -> `AggregateProvenance`
5. `IF` -> `LogicalControlLazy`
6. `INDEX` -> `ReferenceSelectionReturn`
7. `MATCH` -> `LookupSelection`
8. `ISNUMBER` -> `TypePredicateClassification`
9. `NOW` -> `ProviderEffectMetadata`
10. `XLOOKUP` -> `LookupSelection`
11. `INDIRECT` -> `ReferenceTextInterpretation`
12. `SEQUENCE` -> `ArrayShapeConstruction`
13. `OP_ADD` -> `NumericNary`
14. `AVERAGE` -> `AggregateProvenance`
15. `COUNT` -> `AggregateProvenance`
16. `COUNTA` -> `AggregateProvenance`
17. `IFERROR` -> `LogicalControlLazy`
18. `ROUND` -> `NumericUnary`
19. `TEXTJOIN` -> `TextCoercionNormalization`
20. `TODAY` -> `ProviderEffectMetadata`
21. `RAND` -> `ProviderEffectMetadata`
22. `OFFSET` -> `ReferenceConstruction`
23. `CELL` -> `WorkbookInfoContext`
24. `AND` -> `LogicalFold`
25. `CLEAN` -> `TextCoercionNormalization`
26. `DATE` -> `DateSerialArithmetic`
27. `EXACT` -> `TextCoercionNormalization`
28. `HSTACK` -> `ArrayShapeConstruction`

## 6. High-Value Formalization Units
If we want reusable Lean substrate modules rather than one-file-per-function growth, the highest-value units from the current set are:

1. `LookupSelection`
   - covers `XMATCH`, `MATCH`, `XLOOKUP`
2. `AggregateProvenance`
   - covers `SUM`, `AVERAGE`, `COUNT`, `COUNTA`
3. `LogicalFold`
   - covers `AND`, and later likely `OR`
4. `ReferenceSelectionReturn`
   - covers `INDEX`, `XLOOKUP`
5. `ReferenceConstruction`
   - covers `OFFSET`, `INDIRECT`
6. `TextCoercionNormalization`
   - covers `TEXTJOIN`, `CLEAN`, `EXACT`, with lookup-family text comparison as a related consumer
7. `ProviderEffectMetadata`
   - covers `NOW`, `TODAY`, `RAND`
8. `ArrayShapeConstruction`
   - covers `SEQUENCE`, `HSTACK`, plus parts of `INDEX` and `XLOOKUP`
9. `DateSerialArithmetic`
   - covers `DATE`, and later supports parts of `TODAY`/`NOW`

## 7. Immediate Implications
1. We should not assume one Lean file per Excel function is the long-run formal structure.
2. The next useful refactor target is probably a reusable lookup substrate under `formal/lean/OxFunc/Semantics/*` or similar.
3. Aggregate formalization should wait for the OxFml provenance boundary to settle enough to avoid rework.
4. `XLOOKUP` should live in one place only for primary classification: the lookup family. Its reference-return behavior remains a cross-cutting note, not a second home.
5. `INDIRECT` should live in one place only for primary classification: reference-text interpretation. Its reference-return behavior remains a cross-cutting note, not a second home.
6. `OFFSET` and `CELL` should not be forced into the same substrate even though both touch references; `OFFSET` is reference construction, while `CELL` is workbook/context information over references.

## 8. Open Questions
1. Should `LogicalFold` remain explicit now with only `AND`, or should it become a stronger family once `OR` and related functions land
2. Should `INDEX` be split between `ReferenceSelectionReturn` and `ArrayShapeConstruction`, or is the current primary assignment stable enough
3. Should `NOW` and `TODAY` eventually move into a combined `ProviderEffectMetadata + DateSerialArithmetic` composite substrate
4. When more date/time functions land, do we need a broader `SerialTimeCalendar` substrate distinct from the provider layer

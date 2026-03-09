# WORKSET - TUX1000 String Characterization (W7)

## 1. Purpose
Characterize Excel string behavior deeply enough to support formal value/coercion/function contracts.

Primary objective:
1. establish a version-scoped, evidence-backed policy map for string comparison, normalization, limits, and boundary behavior.

Method requirement:
1. execute three coupled lanes:
   - spec corpus extraction,
   - wide-search evidence sweep,
   - reproducible empirical workbook runs.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W7 extension workset in `W000_KICKOFF_PROGRAM_W001_W006.md`.

Dependencies:
1. depends on W1 method template and artifact doctrine.

Downstream consumers:
1. W6 (`XMATCH`) comparison/coercion behavior and deterministic-quirk classification,
2. W3 value-universe string subtype and boundary distinctions,
3. W4 coercion primitives (string-related conversion rules).

Policy on ordering:
1. W7 is a hard prerequisite for final W6 closure.
2. W3 may proceed before W7 closure but must absorb W7 findings before W3 validation closure.

## 3. Scope
In scope:
1. string equality and ordering behavior across:
   - case,
   - accents/diacritics,
   - punctuation/symbols,
   - locale-sensitive formatting boundaries.
2. whitespace and non-printable handling:
   - control characters,
   - NBSP and similar separators,
   - `TRIM`/`CLEAN`/`SUBSTITUTE` interactions.
3. length and storage limits:
   - formula-produced text limits,
   - cell-content limits,
   - error/normalization behavior at boundaries.
4. Unicode handling:
   - `LEN`, `LEFT/RIGHT/MID`, `UNICODE`, `UNICHAR` edge behavior.
5. persistence and interchange:
   - save/reload,
   - CSV/text round-trip effects.
6. worksheet vs reference vs interop surfaces where relevant.

Out of scope:
1. full collation model proof across all locales in this initial phase.
2. external data-source text transformations outside worksheet-visible boundaries.

## 4. Required Axes per Observation Row
1. Excel app version/channel.
2. workbook Compatibility Version.
3. locale profile (`en-US` baseline in this phase).
4. lane (`STR-A/B/C/D/E`).
5. reproducibility metadata (runner/tool revision).
6. evidence source tag (`spec`, `wide-search`, `empirical`).

## 5. Deliverables
1. `docs/function-lane/STRING_BEHAVIOR_RESEARCH_NOTES.md`
2. `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
3. `docs/function-lane/STRING_EXECUTION_RECORD.md`
4. `docs/function-lane/STRING_NORMALIZATION_AND_COMPARISON_POLICY_MAP.md`
5. updates to `docs/function-lane/VALUE_UNIVERSE_RESEARCH_AND_OPEN_QUESTIONS.md` (W3 feed)
6. updates to `docs/worksets/W006_XMATCH_DETERMINISTIC_QUIRKS.md` (W6 feed)
7. conformance-row linkage updates (`FDEF-032` and affected rows)

## 6. Gate Model
### G1 - Source Map Closure
Pass when:
1. spec corpus references are captured with extraction notes.
2. wide-search findings and unresolved contradictions are logged.

### G2 - Scenario Closure
Pass when:
1. seed scenario matrix is explicit and replayable.

### G3 - Observation Closure
Pass when:
1. baseline empirical runs cover declared lanes for one build/channel + compatibility + locale baseline.

### G4 - Characterization Closure
Pass when:
1. policy map is explicit for comparison semantics, normalization, limits, and non-printable handling.

### G5 - Integration Closure
Pass when:
1. W3 and W6 dependency touchpoints are updated with explicit consumed findings and residual risks.

## 7. Status
Execution state:
1. `complete-provisional`.

Claim confidence:
1. `provisional` (single-build/channel + compatibility/locale baseline executed).

Gate snapshot:
1. `G1` source map closure: `closed-provisional`.
2. `G2` scenario closure: `closed`.
3. `G3` observation closure: `closed`.
4. `G4` characterization closure: `closed-provisional`.
5. `G5` integration closure: `closed-provisional`.

Primary evidence:
1. `W7-STR-BL-20260305`
2. `docs/function-lane/STRING_EXECUTION_RECORD.md`
3. `.tmp/string-results-excel.csv`

Follow-on execution packet:
1. `W008_STRING_W8_1_CHECKLIST.md`


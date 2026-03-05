# WORKSET - TUX1000 Kickoff Program (W1-W7)

## 1. Purpose
This document treats worksets W1-W7 as one coupled kickoff program.

Goal:
1. establish OxFunc execution method integrity,
2. close core semantic substrate (floating-point + string + value universe + coercion seam),
3. deliver first nontrivial function closures (`ABS`, `XMATCH`) under unified doctrine.

## 2. Included Worksets
1. `WORKSET_TUX1000_PI_END_TO_END_SLICE.md` (W1)
2. `WORKSET_TUX1000_FLOATING_POINT_CHARACTERIZATION.md` (W2)
3. `WORKSET_TUX1000_VALUE_UNIVERSE_AND_EXTENDED_TYPES.md` (W3)
4. `WORKSET_TUX1000_COERCION_AND_REFERENCE_RESOLUTION_PRIMITIVES.md` (W4)
5. `WORKSET_TUX1000_ABS_FULL_FORMALITY.md` (W5)
6. `WORKSET_TUX1000_XMATCH_DETERMINISTIC_QUIRKS.md` (W6)
7. `WORKSET_TUX1000_STRING_CHARACTERIZATION.md` (W7)

## 3. Dependency Graph
1. W1 -> W2
2. W1 -> W7
3. W2 -> W3
4. W3 -> W4
5. W2 + W3 + W4 -> W5
6. W3 + W4 + W7 (+ W2 numeric-edge feed) -> W6
7. W7 -> W3 (advisory feed; W3 may start before W7 closure)

Interpretation rule:
1. a downstream workset may start exploratory drafting before dependencies close,
2. but it cannot claim gate closure that depends on unfinished upstream obligations.

## 4. Shared Artifact Contract
Every workset in kickoff must define and maintain:
1. scope + non-goals,
2. dependency declaration,
3. deliverable list,
4. gate model (`G1..Gn`),
5. explicit status in execution-state vocabulary,
6. linked conformance row(s) and evidence placeholders.

Global required artifacts across kickoff:
1. function-lane conformance row alignment (`FDEF-026..FDEF-032` at minimum),
2. function-lane supporting specs/notes for each workset topic,
3. Lean/Rust scaffold or implementation updates where claimed,
4. correlation and evidence bindings for function slices.

## 5. Combined Kickoff Gates
1. KG1 Method Integrity:
   - W1 reusable slice method is complete and documented.
2. KG2 Numeric Edge Baseline:
   - W2 produces replayable floating-point behavior map.
3. KG3 Value Algebra Stability:
   - W3 produces explicit value-universe taxonomy and open-question evidence ledger.
4. KG4 Coercion/Resolver Seam Stability:
   - W4 selects a baseline seam contract with alternatives documented.
5. KG5 Nontrivial Closure Seed:
   - W5 (`ABS`) completes end-to-end artifact chain at declared status.
6. KG6 Classification Closure Seed:
   - W6 (`XMATCH`) records evidence-backed classification decision.
7. KG7 String Semantics Baseline:
   - W7 produces replayable, version-scoped string behavior characterization and policy map.

## 6. Status Board (Current Intent)
1. W1: `complete` (method seed complete; empirical depth expansion remains follow-on).
2. W2: `complete-provisional` (baseline lanes executed, Lean comparison captured, divergence ledger populated).
3. W3: `planned`.
4. W4: `planned`.
5. W5: `planned`.
6. W6: `planned`.
7. W7: `planned`.

## 7. Promotion and Maturity Notes
1. Workset completion does not automatically imply `validated` status for all linked claims.
2. Use `draft/provisional/validated` for claim confidence and `exercised/green-validated` for assurance maturity.
3. Foundation-level pack closure is required for program-level profile-green claims.

## 8. Handoff Rule to Foundation Editors
When a kickoff gate closes and impacts cross-program policy, emit a concise handoff packet containing:
1. scope and version bounds,
2. changed assumptions or policy proposals,
3. evidence and replay references,
4. explicit open questions requiring Foundation decision.

# Formalization Strategy - Executable Semantic Model

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Define how OxFunc should use Lean over time without creating an unmaintainable second production implementation.

This note treats Lean as an executable semantic model and proof substrate, not as a duplicate of every Rust/runtime detail.

## 2. Core Position
The best long-run shape is:
1. Rust is the production implementation.
2. Lean is the canonical executable semantic model for the semantics we understand well enough to state cleanly.
3. Shared contracts, manifests, and correlation rows keep the two aligned.
4. Host/XLL behavior is described as seam contracts and evidence, not reimplemented in full inside Lean.

## 3. What “Function Family” Should Mean
`Function family` should not mean Excel’s presentation categories such as “Lookup”, “Text”, or “Math & Trig” unless those happen to match a real semantic reuse boundary.

For OxFunc formalization work, the useful grouping is a rough semantic partition used for:
1. planning reusable Lean semantic modules
2. assigning each function a single primary semantic home
3. keeping discussions about reuse understandable at a higher level than the full profile matrix

It must not become an informal overlay that re-encodes the formal profile system.

The profile fields already capture orthogonal characteristics such as:
1. `arg_preparation_profile`
2. `coercion_lift_profile`
3. `kernel_signature_class`
4. adapter-level and surface-level FEC dependency profiles
5. volatility and host-interaction classes

Those remain the authoritative multi-axis description.

Function families should instead be used as a rough primary partition:
1. one primary family/home per function
2. chosen for semantic centrality, not for exhaustively encoding every trait
3. supported by cross-cutting notes where necessary, but without assigning the function to several equal-status families

Examples:
1. `MATCH`, `XMATCH`, and `XLOOKUP` belong to a lookup-selection substrate family.
2. `SUM`, `AVERAGE`, `COUNT`, and `COUNTA` belong to an aggregate argument-structure policy family.
3. `AND` belongs more naturally to a logical-fold substrate, even though some of its current Excel-observed lanes share aggregate-style direct-scalar versus array-like distinctions.
4. `INDEX` belongs to a reference-selection family.
5. `OFFSET` belongs to a reference-construction family.
6. `INDIRECT` belongs to a reference-text-interpretation family.
7. `TEXTJOIN`, `EXACT`, and `CLEAN` belong to a text coercion/text normalization family.
8. `NOW`, `TODAY`, and `RAND` belong to a provider/effect-metadata family, even though their value semantics differ.

## 4. Distinction From Profiles
Profiles answer:
1. how the function is prepared
2. what coercion/kernel shape it has
3. what host/FEC capabilities it needs
4. what volatility/host-interaction class it has

Families answer:
1. where the function primarily belongs in the rough semantic map
2. which reusable semantic module should usually own its Lean description
3. which other functions should be discussed with it first when extracting shared semantics

So:
1. profiles are formal, multi-axis, and authoritative
2. families are rough, single-home, and organizational

## 5. Better Term Than “Family”
When needed, prefer:
1. `semantic substrate`
2. `formalization unit`
3. `behavior class`

These are more precise than `family` when a function participates in several reusable semantic structures.

## 6. Layering Rule
Lean should primarily formalize layers `1` and `2`, describe layer `3`, and usually avoid duplicating layer `4`.

1. Pure semantic substrate
   - comparison
   - wildcard semantics
   - duplicate selection
   - approximate/binary selection
   - array/reference selection
   - blank/empty/error distinctions
2. Declared adapter policy
   - defaulting
   - coercion class
   - direct-scalar versus array-like behavior and other declared preparation distinctions
   - admitted preparation assumptions
3. FEC/F3E seam contract
   - what prepared arguments/results may contain
   - what metadata/effects may cross the boundary
4. Host realization
   - XLL bridge behavior
   - COM/entrypoint quirks
   - registration mechanics
   - test-seam limitations

## 7. Executable Semantic Model Rule
Lean should be executable enough to run representative semantic cases for the admitted slice.

That means:
1. a Lean module should compute outcomes for the slice it claims to model
2. empirical examples should be runnable as Lean equalities/examples
3. the model should be observationally aligned with Rust on the admitted semantic surface

This is better than a purely narrative formal note, and better than a second production engine.

## 8. What Not To Mirror
Lean should not try to mirror every Rust helper or infrastructure choice.

Avoid:
1. one-to-one duplication of production module decomposition
2. XLL bridge implementation detail duplication
3. Excel host plumbing recreation
4. reproducing optimization-oriented code paths as if they were semantics

Rust may change structure for engineering reasons. Lean should track semantics.

## 9. How Alignment Should Work
Alignment should be artifact-driven, not memory-driven.

Use these shared anchors:
1. function slice contracts
2. machine-readable correlation rows
3. shared empirical manifests
4. execution records
5. shared terminology for substrate classes and seam assumptions

Expected alignment chain:
1. Excel empirical behavior
2. contract statement
3. Rust behavior
4. Lean executable model

If any one of those moves, the correlation row and/or shared manifest should expose the drift.

### 9A. Replay-Bundle and Evidence-Correlation Binding
Replay appliance bundle projections are an additional alignment carrier, not a replacement semantic authority.

For OxFunc they should bind:
1. source manifest rows,
2. execution-record summaries,
3. evidence ids,
4. correlation-ledger refs,
5. function-contract refs,
6. formal artifact refs,
7. limitation refs,
8. invariant refs.

Replay rule:
1. normalized replay views may summarize or index these bindings,
2. but they must not sever the direct path back to the local contract/evidence/correlation artifacts that define OxFunc meaning.

### 9B. Capability-Level Evidence Path
Replay adapter capability claims and formal maturity claims are related but not identical.

Current OxFunc rollout rule:
1. `cap.C0.ingest_valid` through `cap.C3.explain_valid` may be claimed through bundle-valid packet import, replay, diff, and explain surfaces over manifest-driven empirical packets.
2. `cap.C4.distill_valid` requires a locally proven reduced witness that remains replay-valid under an explicit preservation predicate.
3. `cap.C5.pack_valid` requires pack-policy evidence and witness-lifecycle promotion evidence.

Formal consequence:
1. a replay capability claim does not by itself strengthen a semantic claim,
2. and a reduced witness cannot strengthen a formal or empirical claim unless its lifecycle state and replay-valid status are explicit.

### 9C. Witness Lifecycle Effect On Claims
Witness lifecycle state affects how replay artifacts may be used in OxFunc reasoning.

Rules:
1. `wit.explanatory_only` witnesses may support explanation but do not upgrade semantic closure claims.
2. `wit.quarantined` witnesses may remain indexable and analyzable but are not promotion-grade evidence.
3. `wit.superseded` witnesses remain traceable but should not silently replace the primary source evidence path.
4. only replay-valid retained witnesses may support a future `cap.C4.distill_valid` claim.

## 10. Recommended Formalization Shape
Recommended structure over time:

1. substrate modules
   - reusable semantics shared across multiple functions
   - examples: lookup selection, aggregate argument-structure policy, text coercion, date serial arithmetic, reference selection
2. function binding modules
   - function-specific admission/defaulting/result-adaptation binding into the substrate
3. seam contract modules
   - types and invariants for prepared arguments/results and effect metadata
4. proof layers
   - reusable invariants on the substrate
   - lighter per-function closure lemmas on top

## 11. Proof Strategy
Proof effort should concentrate on reusable invariants, not on reproducing every branch of Rust.

High-value proof targets:
1. determinism
2. duplicate-selection invariants
3. monotonicity or order-selection properties for approximate/binary lookup rules
4. preservation of reference identity under selection
5. separation of blank cell vs empty string vs omitted argument
6. provider/result metadata invariants

Lower-value proof targets:
1. host bridge details
2. registration/export mechanics
3. exact duplication of every production helper

## 12. Suggested Maturity Ladder
For a semantic substrate or function slice:

1. `descriptive`
   - Lean metadata and minimal examples exist
2. `executable`
   - Lean model computes admitted outcomes for representative cases
3. `aligned`
   - shared scenarios show Lean and Rust agree on the admitted slice
4. `proved`
   - reusable invariants are proved for the substrate

Not every function needs to reach `proved` before useful formalization value appears.

## 13. Practical Rule For New Work
For each nontrivial function or substrate:
1. characterize Excel empirically
2. update the contract
3. implement Rust
4. implement Lean at the semantic/adaptor layer
5. align both against shared cases
6. only then call the function `function-phase-complete`

This keeps Lean from lagging silently behind Rust.

## 13A. Completion Consequence
`Function-phase-complete` does not mean every function must have a full standalone duplicate implementation in Lean.

It does mean the formal work required by this strategy for the function's primary semantic substrate and admitted slice has been attended to and aligned.

That may mean:
1. a reusable substrate module exists and computes the relevant semantics
2. the function has the necessary Lean binding into that substrate
3. shared examples or alignment artifacts cover the admitted slice
4. any required invariants or seam-contract notes for that substrate are in place

If that required formal work is still missing or stale, the function is not `function-phase-complete` even when Rust and empirical replay look strong.

## 14. Current Implication For OxFunc
Current lookup-family work is a good example:
1. the useful formalization unit is not the user-facing “lookup category”
2. it is the lookup-selection substrate:
   - comparable formation
   - wildcard matching
   - exact/reverse/approximate/binary selection
   - blank-vs-empty policy
   - return selection/reference preservation
3. `XMATCH`, `MATCH`, and `XLOOKUP` then become function bindings whose primary home is still the lookup family
4. cross-cutting concerns like reference-return or text comparison remain notes on the family boundary, not separate equal-status homes for the same function

The same pattern should apply later to:
1. aggregate argument structure
2. reference-return selection
3. text coercion and text comparison
4. date/time serial handling
5. provider/result-metadata behavior

## 15. Operational Guidance
When deciding whether to introduce a new Lean module, ask:
1. is this a reusable semantic substrate, or only a Rust implementation detail
2. can at least two functions benefit from the abstraction
3. can it be aligned against shared cases
4. is there a stable contract vocabulary for it already

If the answer is mostly no, keep the Lean work local to the function binding for now.

## 16. Seed Rule
Seed rule for OxFunc formalization:

`Lean should model semantic substrates and declared adapter policies, while Rust remains the production implementation. Alignment must be maintained through shared contracts, manifests, and correlation artifacts rather than by attempting a full duplicate engine.`

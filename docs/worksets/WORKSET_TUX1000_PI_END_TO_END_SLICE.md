# WORKSET - TUX1000 PI End-to-End Slice

## 1. Why This Exists
This workset establishes the first cross-cutting OxFunc precedent:
1. one function slice,
2. full contract/formal/runtime/verification/correlation chain,
3. promotion-ready artifact discipline.

`PI()` is intentionally small and stable, making it ideal for method hardening.

## 2. Restated PI Proposal (Operations + TUX1000 Aligned)
For `PI()`, we enforce the full five-artifact chain:
1. Contract artifact:
   - exact arity (0),
   - determinism/volatility/host/FEC tags,
   - admission vs runtime boundary.
2. Formal artifact:
   - Lean model with arity admission theorem, deterministic theorem, and admitted-result theorem.
3. Runtime artifact:
   - Rust kernel and adapter with explicit arity admission handling.
4. Verification artifact:
   - Rust tests for arity behavior and deterministic output bits.
5. Correlation artifact:
   - machine-readable record linking contract id, theorem ids, test ids, and evidence/version scope.

Generalization objective:
1. reuse identical skeleton for `SIN`, `SUM`, and `ROW` with only function-specific contract deltas.

## 3. Scope
In scope:
1. OxFunc-local doctrine-compliant PI slice scaffolding.
2. Minimal Rust workspace and crate for function core.
3. Minimal Lean package for function contract semantics.
4. Function-lane documentation artifacts for PI contract and correlation ledger.

Out of scope:
1. final Excel-validated status for PI.
2. broad function catalog rollout.
3. full evaluator/coercion engine implementation.

## 4. Ordered Execution Steps
1. Add workset and documentation scaffolding.
2. Add PI contract row artifact and correlation ledger.
3. Implement Rust function-core skeleton and PI function.
4. Implement Lean model skeleton and PI function proofs.
5. Run local checks (`cargo test`; `lake build` if available).
6. Record gate status and remaining blockers.

## 5. Gate Model for This Workset
### G1 - Contract Completeness
Pass when:
1. PI contract file exists with mandatory fields.
2. conformance registry references PI seed row.

### G2 - Formal Artifact Completeness
Pass when:
1. Lean module compiles.
2. PI theorem set exists for admitted result, nonzero-arity rejection, and determinism.

### G3 - Runtime Artifact Completeness
Pass when:
1. Rust PI implementation compiles.
2. PI tests pass for arity and deterministic bits.

### G4 - Correlation Completeness
Pass when:
1. ledger row links contract id, Lean theorem ids, Rust test ids, and version scope placeholders.

## 6. Execution Record
Status transitions:
1. `planned -> in_progress -> complete`

Current status:
1. `complete` for scaffolding/proof-of-method.
2. PI validation status remains `provisional` pending empirical Excel evidence capture.

## 7. Output Artifacts
1. `docs/function-lane/FUNCTION_SLICE_PI_CONTRACT_PRELIM.md`
2. `docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv`
3. `crates/oxfunc_core/*`
4. `formal/lean/*`
5. updates in function-lane conformance registry.


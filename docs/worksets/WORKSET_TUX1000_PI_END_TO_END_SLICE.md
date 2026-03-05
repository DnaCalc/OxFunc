# WORKSET - TUX1000 PI End-to-End Slice (W1)

## 1. Purpose
Establish the first complete OxFunc slice method on a minimal deterministic function.

`PI()` is the method seed for the full artifact chain:
1. contract,
2. formal,
3. runtime,
4. verification,
5. evidence,
6. correlation.

## 2. Kickoff Position and Dependencies
Kickoff role:
1. W1 in `WORKSET_TUX1000_KICKOFF_PROGRAM_W1_W6.md`.

Dependencies:
1. none.

Downstream consumers:
1. W2..W6 use W1 as template for artifact/gate discipline.

## 3. Scope
In scope:
1. complete `PI()` contract-formal-runtime-verification-correlation chain.
2. explicit function tags (`deterministic`, `nonvolatile`, `host_interaction=none`, `fec_dependency_profile=none`).
3. linkage into function-lane conformance rows.
4. admission-boundary empirical baseline for `PI()` arity behavior across entry/evaluate/file-ingress mechanisms.

Out of scope:
1. broad catalog rollout.
2. final profile-green Foundation maturity claims.

## 4. Deliverables
1. `docs/function-lane/FUNCTION_SLICE_PI_CONTRACT_PRELIM.md`
2. `docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv` (`FUNC.PI` row)
3. Rust `PI()` implementation + tests
4. Lean `PI()` module + theorem inventory
5. conformance row linkage (`FDEF-026`)
6. `docs/function-lane/FORMULA_ADMISSION_BEHAVIOR_NOTES.md` (W1 admission-boundary supplement)
7. `tools/formula-admission-probe/*` baseline runner + schema

## 5. Gate Model
### G1 - Contract Closure
Pass when:
1. contract fields are explicit (arity/admission/result tags and version scope placeholders).

### G2 - Formal Closure
Pass when:
1. Lean module compiles,
2. theorem inventory includes totality, determinism, and arity behavior obligations.

### G3 - Runtime and Verification Closure
Pass when:
1. Rust implementation compiles,
2. required deterministic and arity tests pass.

### G4 - Correlation Closure
Pass when:
1. correlation record links contract/theorems/tests/evidence identifiers.
2. admission-boundary baseline evidence is linked for `PI()` arity rejection behavior.

## 6. Status
Execution state:
1. `complete`.

Claim confidence:
1. `provisional` for broader Excel-version closure until empirical replay depth is expanded.
2. `provisional` for direct C API (`xlfEvaluate`/`xlCoerce`) admission-boundary equivalence until dedicated lane is executed.

Assurance maturity:
1. `exercised`.

# Conditional prepared-value observations

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes are evaluator scheduling and
reference side effects, uncommon reference forms and host publication,
cross-repository acknowledgement/integration, and reference-returning selection.

The discovery capture has 430 IF, 143 IFERROR and 143 IFNA observations. The
original adapters differ on 324/716; `discovery-original-local.jsonl` preserves
those outcomes. `discovery-candidate-judge.json` records 716/716 exact matches
after the prepared-value repair. The additional uniform-selector packet matches
96/96 and proves that unused branch dimensions remain observable for arrays.
Every comparison uses exact typed values and array dimensions; no tolerance or
error-string normalization is used.

`IMPACT_ASSESSMENT.md` describes the contract corrections and evaluator seam.
The sources in `candidate-freeze.json` were retained before generating the
independent 660-case mixed-shape packet, seed 2026092916.
That first candidate matched 496 exact outcomes, differed on 24 direct 1x1 array
selectors, and encountered 140 invalid local inputs. The original generator
incorrectly allowed Missing cells inside prepared arrays; Excel's CHOOSE first
evaluates syntactic missing arguments, so the declared local input was not the
same prepared value. All cases and failed outcomes are retained unchanged.

The refined adapter preserves direct 1x1 selector arrays through output shape
selection, while one-cell references use ordinary scalar preparation. It matches
all 520 admitted first-heldout rows. `refined-candidate-freeze.json` records the
source before a corrected independent 660-case packet, seed 2026092917, which
excludes syntactic Missing inside direct arrays. The second capture matches
660/660 exact outcomes, 220 for each function, without further source changes.

`w111_conditional_live_replay.rs` exercises all 1,992 admitted observations through
public Rust dispatch with real reference fixtures, and binds case IDs, function
IDs and formulas to the captured outcomes, asserting the 140 input exclusions.
The four replay tests pass, as do all 11
IF/IFERROR/IFNA unit tests. The executable Lean `ConditionalSelection` model
defines strict logical text, scalar/array shape selection, singleton broadcasting,
per-cell errors, catchable padding and selected missing/blank publication. The
IF, IFERROR and IFNA modules bind that model; their focused build passes (11 jobs).

Reproduction tools in `smart-fuzzer/tools/w111/`:

- `gen_conditional_surface_probes.py`: 716 discovery cases.
- `gen_conditional_shape_controls.py`: 96 uniform-selector controls.
- `gen_conditional_holdout.py`: 660 independent mixed-shape cases.
- `judge_typed_capture.py`: compares every exact typed outcome and retains all
  failures, binding each to its original formula and function identity.

The Excel COM runner is owned by the campaign coordinator. This packet does not
exercise XLL or infer that prepared-value masking establishes expression laziness.

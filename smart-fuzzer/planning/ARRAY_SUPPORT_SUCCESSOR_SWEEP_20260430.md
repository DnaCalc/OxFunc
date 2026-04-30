# W090 Successor Array-Support Sweep - 2026-04-30

## Scope
This record covers the executable successor pass after W090 tranche A. It uses
the generated candidate inventory plus existing scenario manifests to explore
remaining array-valued scalar-parameter lanes. Passing rows remain compact
telemetry; full packets are retained only in ignored run artifacts for
non-pass classifications.

Comparison policy: exact typed equality, bit-exact numeric digests, no
tolerance.

## Generated Case Set
Builder:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ArraySupportExecutableTranches.ps1
```

Output:

```text
smart-fuzzer/cache/array-support-successor-executable-tranches-v0.json
```

Generated rows:

1. executable cases: `139`
2. category tranches: `8`
3. skipped rows: `243`
4. skipped classes:
   - blocked or known-deviation lanes: `112`
   - no parseable manifest seed: `131`

Risk-band coverage in executable cases:

1. high: `120`
2. medium: `13`
3. low: `6`

Tranche coverage:

| Tranche | Cases | Surfaces |
|---|---:|---:|
| `w090-successor-compatibility` | 68 | 27 |
| `w090-successor-engineering-functions` | 32 | 26 |
| `w090-successor-financial-functions` | 2 | 1 |
| `w090-successor-logical-functions` | 6 | 2 |
| `w090-successor-lookup-and-reference-functions` | 20 | 11 |
| `w090-successor-math-and-trigonometry-functions` | 4 | 2 |
| `w090-successor-statistical-functions` | 6 | 3 |
| `w090-successor-text-functions` | 1 | 1 |

## Final Runs

| Run id | Cases | Rollup |
|---|---:|---|
| `w090-successor-compatibility-final-001` | 68 | `unexpected_mismatch=68` |
| `w090-successor-engineering-functions-final-002` | 32 | `unexpected_mismatch=32` |
| `w090-successor-financial-functions-final-001` | 2 | `unexpected_mismatch=2` |
| `w090-successor-logical-functions-final-002` | 6 | `exact_typed_bit_match=2`, `unexpected_mismatch=4` |
| `w090-successor-lookup-and-reference-functions-final-003` | 20 | `exact_typed_bit_match=5`, `unexpected_mismatch=14`, `local_harness_blocked=1` |
| `w090-successor-math-and-trigonometry-functions-final-001` | 4 | `unexpected_mismatch=4` |
| `w090-successor-statistical-functions-final-001` | 6 | `unexpected_mismatch=6` |
| `w090-successor-text-functions-final-001` | 1 | `unexpected_mismatch=1` |

Aggregate:

1. total executed cases: `139`
2. exact typed bit matches: `7`
3. unexpected mismatches: `131`
4. local harness blockers: `1`
5. Excel harness blockers after fallback repair: `0`

## Deviation Classes

### BUG-FUNC-018
Broad scalar-parameter array-lift gap:

1. `127` local `#VALUE!` vs Excel array-result mismatches.
2. `69` unique surfaces.
3. Affected axes:
   - `one_array_arg:arg1`: `56`
   - `one_array_arg:arg2`: `44`
   - `one_array_arg:arg3`: `27`
4. Bead: `oxf-b39r`.

### BUG-FUNC-019
Complex aggregate array-literal gap:

1. `IMPRODUCT` and `IMSUM` accept array literals in Excel and return scalar
   complex text results.
2. Local OxFunc returns `#VALUE!`.
3. Bead: `oxf-bp23`.

### BUG-FUNC-020
`EXPAND` array-valued `pad_with` panic:

1. local smart-fuzzer status: `local_eval_panic`.
2. Excel result: `#VALUE!`.
3. Bead: `oxf-9pcl`.

## Infrastructure Changes
This pass added or hardened:

1. generated successor case-set builder,
2. case-set tranche filtering in the runner,
3. Excel scalar non-spill fallback when `SpillingToRange` is not a usable
   Range,
4. per-case panic capture in the local smart-fuzzer evaluator.

## Status Axes
1. `scope_completeness`: `scope_complete` for this successor sweep pass
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `partial`
4. `open_lanes`:
   - `BUG-FUNC-018`
   - `BUG-FUNC-019`
   - `BUG-FUNC-020`
   - unexecuted skipped candidate rows needing richer harness/reference/context
     handling

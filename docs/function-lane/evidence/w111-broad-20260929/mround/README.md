# MROUND arithmetic and fractional boundary observations

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

The current candidate agrees with 47,964 numerical and 1,514 typed observations,
with no retained mismatch on these admitted banks. Function status remains partial.

The numerical candidate agrees with all 29,212 discovery observations and all
18,752 fresh heldout observations: 47,964 observations. The first heldout has
15,529 new input pairs and 23 stable controls. The second holds 3,200 new
operation-staging controls selected without any Excel answers.
No whole-function parity claim follows from these finite numeric controls.

## Observed graph

For the captured normal finite and positive-zero inputs, zero arguments return
positive zero before the sign check; opposite nonzero signs return `Num`.
The candidate then:

1. Divides the absolute inputs, rounds to a 64-bit significand, and stores the
   quotient as binary64.
2. Takes its floor and increments it when the remaining fraction is at least
   binary64 `0x3fdfffffffffffa6` (`0.499999999999995`).
3. Multiplies that count by the signed multiple, with the same staged rounding.
4. Returns `Num` for a nonfinite quotient or product and normalizes zero to +0.

The fractional cutoff is an empirical semantic boundary, not an assertion about
Excel's internal algorithm. `fraction-cutoff-answers.json` directly brackets it:
`mround-fraction-cutoff-00183` supplies number bits `0x3fdfffffffffffa5` and
multiple 1, returning +0; `mround-fraction-cutoff-00181` supplies the next adjacent
number `0x3fdfffffffffffa6`, returning 1. The same uniform graph was then checked
across six scales, both signs, and additional integer parts. Rounding only the
fraction to 14 decimal places agrees on this bank, but is not adopted as a
separate identified decimal primitive.

The root's initial 64-way operation/store comparison is retained in
`initial-store-graphs.json` and `initial-store-probe.rs`; its best graph still
missed three broad cases. Subsequent comparisons reject ordinary half-away
rounding, whole-quotient decimal rounding, shifted numerator rounding,
reciprocal multiplication, and quotient-relative tolerances. This table gives
exact typed matches, including numerical bit patterns and worksheet errors:

| Graph | Broad (16,018) | Half neighborhoods (7,734) | Deep fraction controls (5,460) |
|---|---:|---:|---:|
| Staged divide, ordinary round, staged product | 16,015 | 6,682 | 4,028 |
| Whole-quotient 15-significant-digit round | 13,091 | 5,414 | 4,414 |
| Stored quotient plus 0.5, floor | 16,015 | 6,694 | 4,040 |
| Shift numerator by half the multiple | 15,295 | 6,660 | 4,044 |
| Measured fractional cutoff candidate | 16,018 | 7,734 | 5,460 |

The subsequent graph-profile reports preserve every mismatch as an ID and actual typed result;
the raw answer bank supplies its exact arguments and expected result. Identical
mismatch lists are shared between graph labels to avoid duplicating evidence.
The `w111-mround-graph-profiles-v1` schema links each model to a profile index.
The research Rust source regenerates the uncompressed comparisons.

## Provenance and validation

Captures use Excel 16.0 build 20430, 64-bit, workbook Compatibility Version 2,
through public COM Value2 inputs and outputs. Each retained answer bank contains
the capture time and runner metadata. Inputs in the three discovery banks are
finite normal numbers or positive zero; no negative-zero or subnormal source
claim is inferred through Value2. Output bits are compared without tolerance.

`candidate-freeze.json` records the production MROUND, Lean model, and shared
numeric source hashes before `heldout-manifest.json` was generated. The fresh
bank uses 192 seeded new scales, source-ULP neighbors at five quotient ranges,
both signs, 1,600 full-bit normal pairs, and sign/zero/overflow controls. The
earlier root aggregate heldout packet predates this freeze and is not used as
fresh validation of this candidate.

The second independent selector searched 1,949,014 seeded mathematical pairs
and retained 160 centers that distinguish staged division from binary64-only
division and 160 that distinguish staged multiplication. Five source-ULP
neighbors and both signs produce 3,200 observations, all matching. The selector,
manifest, exact inputs, and raw responses are retained as `staged-heldout-*`.

Validation:

- `cargo test -p oxfunc_core --offline --test w111_mround_live_replay`: four
  public-dispatch tests pass, covering all 47,964 numerical observations. All four
  frozen source hashes were checked unchanged after the first heldout capture.
  The later prepared-value change preserves the numerical body byte-for-byte;
  `prepared-candidate-freeze.json` records that check and the second freeze.
- `lake build OxFunc.Functions.Mround`, from `formal/lean`: seven jobs pass.
  The executable model takes the staged arithmetic primitives explicitly;
  it checks the adjacent cutoff, sign/zero, and overflow bindings. It does not
  substitute ordinary Float arithmetic as a proof of the staged primitives.
- A separate read-only arithmetic/publication review found no additional issue
  for these finite normal and +0 inputs.

## Prepared-value observations

The initial surface matched 507/618 typed controls. The retained
`prepared-initial-report.json` identifies all 111 former discrepancies: logical
values must return `Value`, explicit Missing must return `NA`, and array padding
must retain argument-order error precedence. The local surface now maps those
two input kinds recursively, then reuses the frozen ordered binary helper.
Blank cells still become zero. The numerical body does not change.

`IMPACT_ASSESSMENT.md` documents the evaluator-facing requirements and examples.
The function now declares `Custom`; its matching metadata golden row preserves
the file's existing absence of a final newline. The existing direct dispatch
route remains sufficient; numerical Q dispatch continues to use the kernel.

The typed discovery replay passes all 618 cases through both direct surface and
public dispatch. The metadata golden test and seven Lean jobs also pass. Fresh
432-case typed controls were generated after the prepared source freeze; the
qualified serialized replay initially matched 276 and exposed 156 reference-origin
failures. That frozen candidate's typed totals were 894/1,050, with all 156 failures retained explicitly.
All 273 heldout cases without multi-cell references agree; only 3/159 cases
containing multi-cell references agree. These controls cover scalar origins,
row/column/unit and mixed array shapes, explicit references, blanks, logicals,
text and errors.

The first 432-case capture is provisional because it overlapped another Excel
capture. It is retained as `prepared-provisional.json` and is not counted twice.
The qualified serial run `w111-mround-prepared-serial-typed-20260929` reproduces
all 432 Excel typed digests exactly. `prepared-serial-repeat-comparison.json`
records that comparison and verifies all five frozen prepared source hashes.
The 80-row caller-position/reference-materialization diagnostic distinguishes
the rules: baseline 26/80, unconditional non-unit-reference rejection 80/80,
caller-aligned intersection 55/80. Materialization by unary plus lifts, while
INDEX/IF reference-return controls still reject. Aligned callers do not cause
intersection. No new caller-context dependency is introduced.

The reference-origin refinement preserves original arguments through value
preparation and replaces only a reference resolving to a non-unit array with
scalar `Value`, in that argument position. The metadata now declares
`RefsVisibleInAdapter`; unit references, explicit arrays and the numerical body
are unchanged. The reference/array distinction therefore remains available to
the function instead of being erased by upstream materialization.

The refined candidate passes four numerical replay tests (47,964 observations),
four typed replay tests (1,514 observations through both direct/public routes),
the metadata golden test, and seven Lean jobs. `reference-candidate-freeze.json`
records the new source hashes and the unchanged numerical body. All prior failed
freezes/reports remain retained. The fresh 384-case reference-origin validation
packet, generated after this freeze and captured serially as
`w111-mround-reference-independent-typed-20260929`, matches 384/384. All seven
frozen source hashes remain unchanged (`reference-heldout-source-check.json`).
The provisional duplicate 432-case capture is excluded from these totals.

A separate read-only reference preparation review found no additional blocker
for the captured resolved-reference slice. True resolver failures remain an
unresolved host seam: value preparation resolves references before ordered
scalar error selection, unlike ordinary worksheet-error payloads in these controls.

Open lanes: multi-area/structured/external reference classes, further typed/coercion and evaluator
preparation controls; unsupported nonfinite/subnormal source carriers;
unprobed arithmetic inputs and other target architectures; evaluator integration
and any required cross-repository acknowledgment. The measured cutoff's internal
derivation remains unidentified. The prepared metadata change requires the
parent's canonical handoff and receiving-repository acknowledgment.

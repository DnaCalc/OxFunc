# Bitwise numeric and typed evidence, 2026-09-29

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: current-baseline shared numeric-text context; further array precedence
coverage; current-phase review/promotion; alternate Excel version, channel and
locale phases. The original numeric/typed packets below match, but the later
[promotion audit](PROMOTION_AUDIT.md) retains fifty current-baseline shared-text
discrepancies, ten per function. The full functions remain Divergent/Structural.

The current candidate agrees with 6,245 discovery, 1,568 input-qualified
discriminator and 26,433 independent numeric heldout rows. The raw discriminator
has 1,751 output agreements; 183 rows contain negative-zero source arguments that
Value2 converts to positive zero and do not count as same-input semantic evidence.
Those rows are retained unchanged. The heldout generator excluded changed-ingress
inputs and prior discovery/discriminator tuples before Excel capture. The separate
`../numeric-ingress.json` records the source/storage distinction.

Typed discovery first produced 160/170 exact outcomes and ten explicit-missing
argument mismatches. Family-specific missing-to-zero preparation resolves those
ten; `typed-discovery/outcomes/local-final.jsonl` replays all 170 against their
retained Excel outcomes. An independent 692-row typed heldout is 692/692 exact,
with live fixture readback checking numbers, text, logicals, blanks and errors.
Arrays are compared cellwise with exact numeric bits and typed error identity.

## Observed behavior and candidate

- BITAND, BITOR and BITXOR operands are finite integers from zero through
  `2^48-1`; fractional values return `#NUM!` rather than being truncated.
- BITLSHIFT and BITRSHIFT apply the same integer admission to their first input.
  A zero first input returns positive zero for every finite numeric shift.
- Shift counts truncate toward zero; a nonzero operand requires the truncated
  count's magnitude to be at most 53. Admitted magnitudes 49 through 53 return
  positive zero in either direction. Counts through magnitude 48 use ordinary
  direction reversal, with a pre-shift check rejecting results above `2^48-1`.
- Explicitly missing argument positions coerce to zero for all five functions.
  The existing exact-two-argument arity rule remains in place.

The 49-through-53 and zero-operand branches are reproducible black-box findings
that qualify the public [BITLSHIFT documentation](https://support.microsoft.com/en-us/excel/functions/bitlshift-function).
Operand integrality agrees with the public [BITAND documentation](https://support.microsoft.com/en-us/excel/functions/bitand-function).
No Excel binary or internal implementation was inspected.

The candidate uses a common integer kernel and a family-specific preparation
helper in `bit_common.rs`; shared numeric coercion policies were not changed.
`freeze-v1.json` records the numeric candidate before independent heldout capture.
`freeze-v2.json` records the missing-argument surface change before the typed
heldout capture. All numeric evidence was replayed after the second freeze.

## Axes and limits

Reference baseline: Excel 16.0 build 20430, 64-bit Windows, workbook Compatibility
Version 2. Channel is unavailable rather than inferred. Numeric capture used
uncached bulk Value2 input; typed capture recorded the 1900 date system and
`PrecisionAsDisplayed=false`. Numeric capture's older provenance does not record
those latter settings; numeric bitwise functions are date/locale independent,
and the separately checked typed ingress supports the storage behavior.

The numeric slice covers random integers, fractional rejection, 48-bit limits,
both shift directions, huge shift counts, and adjacent count boundaries. The typed
slice covers scalar text/logical/blank/missing/error coercion, cell references,
error ordering, and rectangular array broadcasting. Workbook/reference-engine
implementation outside the function surface remains outside this evidence.

## Reproduction and validation

`artifact-manifest.json` records original and retained file hashes. Numeric
answer files retain each exact input tuple and each live Excel answer; compacting
JSON does not regenerate or infer an answer. `final-numeric-judgement.json`
contains the production dispatch replay results for every retained numeric file.
Typed files preserve original cases, local/Excel outcomes, comparison records,
environment and runner/source hashes.

From the repository root:

```powershell
python smart-fuzzer/tools/w111/retain_bitwise_20260929.py
cargo test -p oxfunc_core --test bitwise_excel_parity_20260929
cargo test -p oxfunc_core --lib functions::bit_common::tests
cargo test --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --bin array_tranche_local_eval
```

The focused dispatch regressions pass (3 tests), common-kernel tests pass
(3 tests), and the repaired local-reference harness tests pass (3 tests).
From `formal/lean`, the following passes:

```powershell
lake build OxFunc.Functions.BitAndFn OxFunc.Functions.BitOrFn OxFunc.Functions.BitXorFn OxFunc.Functions.BitLshiftFn OxFunc.Functions.BitRshiftFn
```

`Bitwise.lean` supplies executable rational-input/integer-result models for
integrality, shifts, missing preparation, and the witnessed branch behavior;
the five bindings import that model. Binary64 ingress and the full text parser
are separate substrates, not duplicated by the rational kernel model. This packet
does not independently claim function-phase completion or perform the canonical
register promotion/checklist review.

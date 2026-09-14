# HO-FN-019 - Wire-Schema Id Constant And Validator (W110-3 / campaign W3.5)

Direction: `OxFunc -> OxXlPlay, OxReplay, DnaTreeCalc` (plus one doc line each in OxDoc and DnaOneCalc)
Source repo/workset: `OxFunc/W110` (bead `oxf-xvt5.3`)
Target repo/workset: the loaders and producers of the `comparison_value` envelope
Filed: `2026-09-14`
Status: `filed`

## What OxFunc now exports

`crates/oxfunc_value_types/src/lib.rs` (reachable as `oxfunc_value_types::*` and,
for crates that only depend on `oxfunc_core`, as `oxfunc_core::value::*`, which
re-exports the whole value-types crate):

```rust
pub const OXFUNC_VALUE_WIRE_SCHEMA_ID: &str;      // the id every producer writes into "wire_schema"
pub const OXFUNC_VALUE_WIRE_SCHEMA_VERSION: u32;  // the trailing .v<N> of the id, currently 1

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSchemaMismatch { pub found: String, pub expected: &'static str }
// implements Display and std::error::Error

pub fn validate_wire_schema_id(found: &str) -> Result<(), WireSchemaMismatch>;
```

Tests in the same crate pin that:

1. `OXFUNC_VALUE_WIRE_SCHEMA_ID` equals the historical hand-typed literal, so every
   retained artifact written before the constant existed still loads unchanged
   (`constant_equals_the_historical_literal_so_retained_artifacts_still_load`);
2. the version constant agrees with the id's `.v<N>` suffix
   (`version_constant_agrees_with_the_id_suffix`);
3. `validate_wire_schema_id` accepts exactly the constant and rejects a different
   id, a different version of the same family, a case variant, an unrelated
   schema id and the empty string with a typed `WireSchemaMismatch`
   (`validate_rejects_a_different_id_with_a_typed_error`);
4. no `.rs` file under the OxFunc workspace `crates/` tree spells the id as a bare
   literal outside the constant's definition and the historical pin
   (`no_bare_wire_schema_literal_outside_the_constant`). The invariant is textual,
   so the test is textual; it was exercised negatively during landing (a scratch
   file carrying the literal made it fail with a `file:line` pointer).

Every downstream repo compiles OxFunc from source through a path dependency, so
the constant is available on the next build with no version bump.

## Finding that corrects the bead premise

The bead (and campaign row W3.5) said the id was hand-typed "in 11+ places across
OxFunc, OxFml, OxCalc, OxXlPlay". Measured on 2026-09-14 with
`rg "oxfunc_value_types\.aligned_json"` over every sibling checkout under
`C:\Work\DnaCalc` (excluding `target*` and `.git`):

| Repo | Rust/PowerShell source sites | Doc mentions | Retained-artifact data files |
|---|---|---|---|
| OxFunc | **0** (the crate the id is named after never mentioned it) | 0 | 0 |
| OxFml | **0** | 0 | 0 |
| OxCalc | **0** | 0 | 0 |
| OxXlPlay | 3 Rust + 3 PowerShell | 1 | 44 `.json` |
| OxReplay | 3 Rust | 3 | 11 `.json` + 1 `.jsonl` |
| DnaTreeCalc | 11 Rust (all test code) | 2 (archived design notes) | 1 `.json` |
| DnaOneCalc (archived) | 8 Rust (the same eight sites, at the same line numbers, as the DnaTreeCalc bench-host file it was forked from) | 0 | 0 |
| OxDoc | 0 | 1 | 0 |

So the "in-repo occurrences replaced by the constant" step of the bead was
vacuous for OxFunc: there was nothing to replace, and the sweep test now keeps it
that way. The real spread is 28 source sites in four repos (20 excluding the
archived DnaOneCalc copies), none of them in OxFml or OxCalc. The retained-artifact
JSON/JSONL files carry the id as **data** and must never be rewritten; they are the
reason the constant is pinned to the historical literal.

## Every out-of-repo bare literal (file:line), with the suggested replacement

Line numbers are as measured on 2026-09-14; re-grep before editing.

### OxXlPlay (already depends on `oxfunc_value_types`)

| Site | Context | Suggested change |
|---|---|---|
| `src/oxxlplay-capture/src/lib.rs:13` | `const OXFUNC_ALIGNED_JSON_WIRE_SCHEMA: &str = "..."` — a local duplicate constant, with the comment "Switch this mirror validator to direct oxfunc_value_types serde helpers once the shared crate exposes an admitted wire surface" | Delete the local const; the comparison at `:230` becomes `oxfunc_value_types::validate_wire_schema_id(&envelope.wire_schema).map_err(|e| e.to_string())?` (or keep the local `Err(format!(..))` shape and compare against `OXFUNC_VALUE_WIRE_SCHEMA_ID`). |
| `src/oxxlplay-capture/src/lib.rs:454` | test fixture `"wire_schema": "..."` inside `serde_json::json!` | `"wire_schema": OXFUNC_VALUE_WIRE_SCHEMA_ID` |
| `src/oxxlplay-capture/src/lib.rs:518` | same, second test | same |
| `scripts/invoke-excel-observation.ps1:177` | `comparison_value_wire_schema = "..."` (source_metadata) | PowerShell cannot import a Rust const. Options for the OxXlPlay owner: (a) one script-level `$OxFuncValueWireSchemaId` declared once at the top and used at all three sites, with a pin test that compares it to the Rust constant (e.g. a tiny `cargo run` helper or a `cargo test` in `oxxlplay-capture` that reads the script and asserts the value); (b) have the Rust capture layer stamp `wire_schema` and drop it from the script. |
| `scripts/invoke-excel-observation.ps1:1504` | `wire_schema = "..."` on the emitted envelope | as above |
| `scripts/invoke-excel-observation.ps1:4567` | `comparison_value_wire_schema = "..."` | as above |
| `docs/upstream/NOTES_FOR_OXREPLAY.md:95` | prose | leave, or reword to name the constant |

### OxReplay (does not yet depend on OxFunc; this is the "admitted helper" BLK-REPLAY-003 waits for)

| Site | Context | Suggested change |
|---|---|---|
| `src/oxreplay-diff/src/lib.rs:771` | production: `.is_some_and(\|schema\| schema == "...")` decides whether an object is the OxFunc envelope | add the narrow path dependency on `../OxFunc/crates/oxfunc_value_types` and compare against `OXFUNC_VALUE_WIRE_SCHEMA_ID` (or `validate_wire_schema_id(schema).is_ok()`). Note the current code silently falls through to the legacy `value_kind`/`kind`/`type` parser for any other `wire_schema`; with the validator that fall-through can become a typed rejection for a *different version of the same family* while staying lenient for envelopes that carry no `wire_schema` at all. That behaviour change is OxReplay's decision. |
| `src/oxreplay-diff/src/lib.rs:1574` | test fixture | `OXFUNC_VALUE_WIRE_SCHEMA_ID` |
| `src/oxreplay-diff/src/lib.rs:1609` | test fixture | `OXFUNC_VALUE_WIRE_SCHEMA_ID` |
| `CURRENT_BLOCKERS.md:24` (BLK-REPLAY-003) | prose | the constant + validator is the first admitted OxFunc-owned piece of that helper surface; the typed-envelope serde surface itself is still not admitted (see "What this does not do"). |
| `docs/worksets/W007_HOST_ROLLOUT_EVIDENCE_COMPARISON_PLANNING.md:95` | prose | leave |
| `docs/test-runs/w011-a1-times-three-excel-compare-intake-baseline/README.md:47` | prose describing retained data | leave |

### DnaTreeCalc (depends on `oxfunc_core`; reach the constant as `oxfunc_core::value::OXFUNC_VALUE_WIRE_SCHEMA_ID`)

| Site | Context | Suggested change |
|---|---|---|
| `src/dnacalc-bench-host/src/services/verification_bundle.rs:4912` | `#[cfg(test)]` helper `fake_diff_report` | `OXFUNC_VALUE_WIRE_SCHEMA_ID` |
| `...:6835` | test `summarize_excel_capture_normalizes_published_formula_result_wrapper` | same |
| `...:6957` | test `materialize_compare_ready_normalized_replay_normalizes_comparison_value_wrapper` | same |
| `...:7020` | test `materialize_compare_ready_normalized_replay_preserves_raw_excel_numeric_lexeme` | same |
| `...:7194` | test `normalize_comparison_value_coalesces_logical_aliases` | same |
| `...:7226` | test `normalize_comparison_value_coalesces_nested_number_and_logical_aliases` | same |
| `...:7256` | test `normalize_comparison_value_decodes_aligned_text_utf16_payloads` | same |
| `...:7321` | test `normalize_comparison_value_coalesces_aligned_error_aliases` | same |
| `src/dnatreecalc-host/tests/active_table_corpus.rs:4523, :4533, :4542` | test helper `comparison_value_json` that *produces* envelopes | same |
| `docs/archive/grid-recon-2026-06/round2-design-fileBoundary.md:26`, `round2-recon-interop.md:31, :56` | archived design notes | leave |

Observation for the DnaTreeCalc owner, not a requirement: the bench-host
production path (`verification_bundle.rs` before line 4424) never reads
`wire_schema` at all; it recognises the envelope by
`boundary == "published_formula_result"` (`:3685`, `:3814`) and by the presence of
`utf16_code_units`. A loader that never reads the id cannot be told about a
version drift. Calling `validate_wire_schema_id` where the envelope is first
recognised would make a future `.v2` artifact a typed refusal instead of a silent
misread.

### DnaOneCalc (archived)

`src/dnaonecalc-host/src/services/verification_bundle.rs:4912, 6835, 6957, 7020,
7194, 7226, 7256, 7321` — the same eight test sites, at the same line numbers, as
the DnaTreeCalc bench-host file (the two files have since diverged elsewhere).
Listed for completeness; the repo is archived and no change is requested.

### OxDoc

`docs/OXDOC_REQUIREMENTS.md:80` — prose ("a defined lossless lowering to the
`oxfunc_value_types.aligned_json.v1` wire"). Leave, or name the constant.

## A future v2 bump

After this handoff is taken up, bumping the wire schema is one place: the two
adjacent constants in `crates/oxfunc_value_types/src/lib.rs`
(`OXFUNC_VALUE_WIRE_SCHEMA_ID` and `OXFUNC_VALUE_WIRE_SCHEMA_VERSION`; the
agreement test fails if only one is changed). Every Rust producer and loader that
imports the constant picks the new id up by recompiling; the PowerShell script is
the one site that still needs its own edit unless option (b) above is taken.

Two things a bump does **not** solve by itself, so they are decisions for the bump,
not for now:

1. `validate_wire_schema_id` accepts exactly one id. Once the constant says `.v2`,
   every retained `.v1` artifact (today 44 JSON files in OxXlPlay carrying about
   4,700 envelopes, 12 JSON/JSONL files in OxReplay carrying about 4,900, and one
   in DnaTreeCalc) is rejected by any loader that calls the validator. A bump
   therefore needs either a migration of retained artifacts or a loader-side
   "supported set" (the OxDoc requirements doc already describes that shape for
   its own `WireSchemaId`: reader/ingest negotiate declared-supported sets,
   mismatch is a typed error). The single-id validator is deliberately the
   narrow first step; it is not a negotiation surface.
2. The envelope's *shape* (`boundary`, `value.kind`, `utf16_code_units`, the
   14 worksheet error codes, `array` shape+cells, `reference`, `empty_cell`) is
   still defined only by OxXlPlay's mirror deserializer
   (`oxxlplay-capture/src/lib.rs`), OxReplay's hand parser and DnaTreeCalc's
   normalizer. This handoff names the id once; it does not admit a shared serde
   surface for the shape. That remains BLK-REPLAY-003's larger ask and is out of
   scope for W110-3.

## What this does not do

1. It adds no serde derives and no JSON encoder/decoder to `oxfunc_value_types`
   (the crate still has zero dependencies).
2. It changes no public signature and no behaviour on any OxFunc path.
3. It does not edit any sibling repo; every replacement above is the receiving
   repo's to make and to acknowledge.

## Acknowledgement

Per OxFunc `AGENTS.md` Rule 6, this handoff opens a dependency and does not close
one. Bead `oxf-xvt5.3` closes on the OxFunc-side outcome (constant, validator,
tests, this file); the register row stays `filed` until each receiving repo
replaces its sites and records the acknowledgement date.

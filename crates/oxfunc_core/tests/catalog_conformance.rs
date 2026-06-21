//! W105 oxf-y2uw.2 — Build-time catalog conformance check + monotonic placeholder ratchet.
//!
//! Operationalizes decision **W105-D1** (point 4) in
//! `docs/worksets/W105_DECLARATIVE_FUNCTION_SPEC_AND_GENERIC_EXECUTOR.md` and ODR-FN-004:
//! the function-signature/parameter identity of every built-in lives in three id-keyed
//! representations — `FunctionMeta.arity`, the `SignatureSeed` corpus, and the
//! `RegistryHelpSeed` corpus — joined only at projection time in `registry.rs`, with **no**
//! mutual-agreement check. The join is silent: a help parameter that fails the
//! `index + case-insensitive name` bind in `parameter_help_description` just disappears
//! (`-> None`) with no diagnostic. This test turns that drift class into a `cargo test`
//! failure.
//!
//! It is a single generic data-over-the-table test (in the spirit of the cross-surface
//! equivalence law of bead .1): it enumerates the live registry catalog once via the
//! `catalog_conformance_rows()` accessor and applies four checks to every row, so it scales
//! across all ~525 rows with no per-function test code. It is a pure guard — it changes no
//! catalog data and asserts only the *current* state plus monotonic ratchets.
//!
//! ## The four checks (W105-D1 point 4)
//!
//! 1. **Arity <-> signature seed.** For each real function signature the `SignatureSeed`
//!    parameter count is consistent with `FunctionMeta.arity`: `arity.min <= fixed params`,
//!    the seed carries a trailing repeat iff `arity.max > params.len()`, and no fixed-arity
//!    seed exceeds `arity.max`.
//! 2. **Help seed <-> signature seed (no silent drop).** Every raw `RegistryHelpSeed`
//!    parameter entry binds to exactly one signature parameter by `index` +
//!    case-insensitive `name` (the exact join used in `registry.rs::parameter_help_description`).
//!    A help entry that binds to nothing is an `orphaned_help_entry` — a BUILD BREAK.
//! 3. **Signature display <-> parameter names.** Each function `signature_display` begins
//!    with the canonical surface name and lists the ordered structured parameter names as an
//!    in-order subsequence (cf. `synthesized_signature_display`). A rename, reorder, dropped
//!    structured name, or surface/head mismatch is a BUILD BREAK.
//! 4. **Coverage ledger — the three W105-D1 named states, plus full accountability.** The
//!    decision names three states: `placeholder_signature` (synthesized `FUNCNAME(...)`
//!    display, no real parameter records — the W106 fill backlog), `known_missing_help` (a
//!    real signature with no function-level help/description — a LEGAL unfrozen-corpus gap),
//!    and `orphaned_help_entry` (help text binding to no signature parameter — a DEFECT).
//!    Only `known_missing_help` is an allowed gap; an orphan is always a build break. So that
//!    EVERY row lands in exactly one structural bucket, the ledger also carries
//!    `operator_surface` (operator-notation rows, see the scoping note) and `complete` (a real
//!    signature that already carries help); these are accountability buckets, not gaps.
//!
//! ## Ratchets (monotonic, baselines pinned to the live current-tree counts)
//!
//! * **Placeholder ratchet.** `placeholder_signature` count must be `<=` the pinned baseline,
//!   so the backlog may only shrink under W106 and no NEW placeholder can be introduced.
//! * **Display/seed count-divergence ratchet.** A function whose `signature_display`
//!   enumerates MORE parameters than the structured seed (the display ran ahead of the
//!   structured fill) is real but tolerable representational drift. Its count is pinned so it
//!   may only shrink and no NEW divergence can appear. Today exactly one row is in this state
//!   (`FUNC.GROUPBY`: display lists 8 params, seed carries 3).
//!
//! ## Scoping note: operators
//!
//! Excel operator surfaces (`FUNC.OP_*`) are a distinct signature category: their
//! `signature_display` is operator notation (`-A`, `A + B`, `A:B`, `@A`, ...), never the
//! `FUNCNAME(arg1, arg2)` shape, and their operands are not authored as positional seed
//! `parameters`. Checks 1 and 3 are framed entirely in function-signature terms and do not
//! model operators, so they are scoped to non-operator rows; operators are tracked as their
//! own `operator_surface` ledger state for full accountability. Checks 2 and 4 (orphan-help
//! and the help ledger) still apply to operators.
//!
//! ## Live ledger (measured from the registry on the current tree)
//!
//! The coverage ledger partitions all 525 rows exactly: `525 = 229 + 22 + 0 + 274`.
//!
//! * total registry rows: 525 (the contract doc's "528" is stale; the live catalog is 525)
//! * `placeholder_signature`: 229 (matches the doc's 229) — pinned ratchet baseline
//! * `operator_surface`: 22 (operator-notation rows, excluded from the function-shaped checks)
//! * `known_missing_help`: 0 (every non-operator real signature currently carries a
//!   `short_description`; this is still a LEGAL state should one regress)
//! * `complete`: 274 (real signature with help)
//! * `orphaned_help_entry`: 0 (no help text binds to a non-existent signature parameter)
//! * display/seed count-divergence: 1 (`FUNC.GROUPBY`) — pinned ratchet baseline

use oxfunc_core::registry::{CatalogConformanceRow, catalog_conformance_rows};

/// Pinned monotonic ratchet: the number of `placeholder_signature` rows may only shrink.
/// Measured live on the current tree (matches the W105-D1 / contract-doc "229").
const PLACEHOLDER_SIGNATURE_BASELINE: usize = 229;

/// Pinned monotonic ratchet: the number of function rows whose `signature_display` enumerates
/// more parameters than the structured seed may only shrink. Today exactly `FUNC.GROUPBY`.
const DISPLAY_SEED_COUNT_DIVERGENCE_BASELINE: usize = 1;

fn is_operator(row: &CatalogConformanceRow) -> bool {
    row.function_id.starts_with("FUNC.OP_")
}

/// A placeholder row: a synthesized `FUNCNAME(...)` display with no real parameter records
/// (W105-D1 / `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` §4.1).
fn is_placeholder(row: &CatalogConformanceRow) -> bool {
    row.signature_param_names.is_empty()
        && row.signature_display == format!("{}(...)", row.surface_name)
}

/// Tokenise a function `signature_display` body into bare parameter identifiers, dropping the
/// `[optional]` brackets and the trailing-repeat `...` ellipsis. Returns `None` if the display
/// is not the `SURFACE(...)` function shape headed by the canonical surface name.
fn display_param_tokens(row: &CatalogConformanceRow) -> Option<Vec<String>> {
    let open = row.signature_display.find('(')?;
    if !row.signature_display.ends_with(')') {
        return None;
    }
    let head = &row.signature_display[..open];
    if !head.eq_ignore_ascii_case(&row.surface_name) {
        return None;
    }
    let body = &row.signature_display[open + 1..row.signature_display.len() - 1];
    let tokens = body
        .split(',')
        .map(|raw| raw.trim().trim_matches(|c| c == '[' || c == ']').trim())
        .filter(|t| !t.is_empty() && *t != "...")
        .map(str::to_string)
        .collect();
    Some(tokens)
}

#[test]
fn catalog_conformance_and_placeholder_ratchet() {
    let rows = catalog_conformance_rows();
    assert!(
        !rows.is_empty(),
        "live registry catalog must not be empty — accessor returned no rows"
    );

    // Ledger tallies (every row lands in exactly one structural bucket).
    let mut placeholder_signature = 0usize;
    let mut known_missing_help = 0usize;
    let mut operator_surface = 0usize;
    let mut complete = 0usize;
    let mut display_seed_count_divergence = 0usize;

    // Collected hard failures (each is a build break).
    let mut orphaned_help: Vec<String> = Vec::new();
    let mut arity_failures: Vec<String> = Vec::new();
    let mut display_failures: Vec<String> = Vec::new();

    for row in &rows {
        let op = is_operator(row);
        let placeholder = is_placeholder(row);
        let params = row.signature_param_names.len();

        // ---- Check 2: orphan-help (applies to EVERY row, operators included) ----
        // Mirror the registry.rs::parameter_help_description join exactly: a help parameter
        // entry binds iff the signature parameter at `index` exists AND its name matches
        // case-insensitively. A help entry that binds to nothing degrades silently in the
        // public projection; here it is a defect.
        for (index, name, _has_desc) in &row.help_param_entries {
            let binds = row
                .signature_param_names
                .get(*index)
                .is_some_and(|sig_name| sig_name.eq_ignore_ascii_case(name));
            if !binds {
                orphaned_help.push(format!(
                    "{}: help param entry (index {index}, name {name:?}) binds to no signature parameter (signature names: {:?})",
                    row.function_id, row.signature_param_names
                ));
            }
        }

        // ---- Checks 1 & 3: function-signature-shaped, non-operator rows only ----
        if !op && !placeholder {
            // Check 1 — arity <-> signature seed.
            let implied_trailing = row.arity_max > params;
            let effective_trailing = row.signature_trailing_repeats || implied_trailing;
            if row.arity_min > params {
                arity_failures.push(format!(
                    "{}: arity.min {} exceeds fixed signature params {}",
                    row.function_id, row.arity_min, params
                ));
            }
            if !effective_trailing && params > row.arity_max {
                arity_failures.push(format!(
                    "{}: fixed-arity seed has {} params, exceeding arity.max {}",
                    row.function_id, params, row.arity_max
                ));
            }
            if row.signature_trailing_repeats && !implied_trailing {
                arity_failures.push(format!(
                    "{}: seed declares trailing_repeats but arity.max {} == params {} (no room to repeat)",
                    row.function_id, row.arity_max, params
                ));
            }

            // Check 3 — signature display <-> ordered parameter names.
            match display_param_tokens(row) {
                None => display_failures.push(format!(
                    "{}: signature_display {:?} is not the `{}(...)` function shape headed by the canonical surface name",
                    row.function_id, row.signature_display, row.surface_name
                )),
                Some(tokens) => {
                    // Every structured parameter name must appear, in order, among the display
                    // tokens (a rename/reorder/drop is a defect; a display that lists *more*
                    // params than the seed is the count-divergence ratchet state below).
                    let mut cursor = 0usize;
                    let mut ordered_subsequence = true;
                    for name in &row.signature_param_names {
                        match tokens[cursor..]
                            .iter()
                            .position(|t| t.eq_ignore_ascii_case(name))
                        {
                            Some(offset) => cursor += offset + 1,
                            None => {
                                ordered_subsequence = false;
                                break;
                            }
                        }
                    }
                    if !ordered_subsequence {
                        display_failures.push(format!(
                            "{}: structured parameter names {:?} are not an in-order subsequence of signature_display tokens {:?}",
                            row.function_id, row.signature_param_names, tokens
                        ));
                    } else if tokens.len() != params {
                        // Subsequence holds, but the display enumerates a different parameter
                        // count than the seed (today only GROUPBY: display ahead of fill).
                        display_seed_count_divergence += 1;
                    }
                }
            }
        }

        // ---- Check 4: coverage ledger (one structural bucket per row) ----
        if placeholder {
            placeholder_signature += 1;
        } else if op {
            operator_surface += 1;
        } else if !row.has_short_description {
            known_missing_help += 1;
        } else {
            complete += 1;
        }
    }

    // Sanity: the ledger partitions the catalog exactly.
    assert_eq!(
        placeholder_signature + operator_surface + known_missing_help + complete,
        rows.len(),
        "coverage ledger must partition every catalog row exactly once"
    );

    // --- Hard build breaks (orphans / arity / display defects) ---
    assert!(
        orphaned_help.is_empty(),
        "orphaned_help_entry is a build break (W105-D1): help text that binds to no signature \
         parameter is silently dropped by the projection. Offenders:\n{}",
        orphaned_help.join("\n")
    );
    assert!(
        arity_failures.is_empty(),
        "arity <-> signature-seed conformance failures (W105-D1):\n{}",
        arity_failures.join("\n")
    );
    assert!(
        display_failures.is_empty(),
        "signature display <-> parameter-name conformance failures (W105-D1):\n{}",
        display_failures.join("\n")
    );

    // --- Monotonic placeholder ratchet (the backlog may only shrink) ---
    assert!(
        placeholder_signature <= PLACEHOLDER_SIGNATURE_BASELINE,
        "placeholder ratchet tripped: {placeholder_signature} placeholder_signature rows exceed the \
         pinned baseline {PLACEHOLDER_SIGNATURE_BASELINE}. A NEW placeholder `FUNCNAME(...)` signature \
         was introduced; placeholders may only be burned down (W106), never added. If you legitimately \
         filled placeholders, lower PLACEHOLDER_SIGNATURE_BASELINE to the new count."
    );

    // --- Monotonic display/seed count-divergence ratchet (may only shrink) ---
    assert!(
        display_seed_count_divergence <= DISPLAY_SEED_COUNT_DIVERGENCE_BASELINE,
        "display/seed count-divergence ratchet tripped: {display_seed_count_divergence} function rows \
         have a signature_display enumerating more parameters than the structured seed, exceeding the \
         pinned baseline {DISPLAY_SEED_COUNT_DIVERGENCE_BASELINE} (today only FUNC.GROUPBY). A NEW \
         display/seed divergence was introduced; reconcile the display with the structured parameters."
    );
}

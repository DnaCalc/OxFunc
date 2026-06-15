# BUG-FUNC-040: Approximate/binary lookup aborts on any blank cell instead of skipping it

## Summary
- **Bug id**: `BUG-FUNC-040`
- **Opened**: `2026-06-11`
- **Status**: `fix_in_progress`
- **Owner workset**: `W102A.6` (W100-W102 cleanup pass)

## Source Refs
- **Reported against ref**: branch `w100-w102-cleanup-pass` working tree
- **Reproduced on ref**: same working tree (review digest
  `.tmp/review-digest/functions-text-lookup.md` finding F4)
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)
- **Ref notes**: review-digest evidence only; no fresh live-Excel COM replay was
  run for this fix. The blank-skipping direction is the unambiguous case the
  digest already pinned (the `=MATCH(9.99E307,A:A,1)` last-value idiom depends on
  it); error-cell and cross-type ordering remain open pending probe lanes.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `spec_mismatch`
- **Root cause summary**: the approximate/binary candidate-collection paths
  aborted the whole lookup the moment any cell yielded `None` from
  `prepared_lookup_candidate_comparable` (blanks, `Missing`, and errors all
  yield `None`). `collect_match_candidates` (match_fn.rs) returned
  `Err(NotAvailable)` and `collect_binary_candidates` (xmatch.rs) returned
  `Ok(None)` -> `Err(NotAvailable)`. So `MATCH(x, range, 1)` /
  `MATCH(x, range, -1)` over a range with even one blank cell was `#N/A`,
  `VLOOKUP`/`HLOOKUP` approximate inherited it via `eval_match_surface`, and
  `XMATCH` binary-search modes (search_mode `+-2`) inherited it via
  `collect_binary_candidates`. Excel's approximate/binary search *skips* blank
  cells; the canonical last-value idiom `=MATCH(9.99E307,A:A,1)` and the common
  trailing-blank range case rely on that.

## Reproduction
Review digest `functions-text-lookup.md` F4 (isReal=True, conf=high).
Conceptual Excel-vs-OxFunc table (analytical, not COM-replayed in this fix):

| Formula | OxFunc (before) | Excel |
| --- | --- | --- |
| `=MATCH(9.99E307,{1,2,<blank>,3},1)` | `#N/A` | `4` (last value `3`) |
| `=MATCH(2.5,{1,<blank>,2,3},1)` | `#N/A` | `3` (value `2`) |
| `=VLOOKUP(2.9,{1,10;2,20;<blank>,99;3,30},2)` | `#N/A` | `20` |
| `=XMATCH(2.5,{1,<blank>,2,3},1,2)` | `#N/A` | `4` (value `3`) |

## Fix Plan
In the current working-tree patch, blanks (`Empty`/`Missing` cells) are
skippable candidates in the approximate/binary paths instead of aborting the
lookup; the binary search runs over the comparable subset and maps the chosen
position back to the cell's original 1-based index.

- `crates/oxfunc_core/src/functions/xmatch.rs`: added
  `ApproximateCandidate` (Comparable | Blank), `prepared_approximate_candidate`
  (blank -> skip, error/unsupported -> abort), and
  `collect_approximate_comparables` returning `IndexedComparables { values,
  original_indices }`. `xmatch_binary_search` now collects via that helper and
  maps every result through `original_indices`. Removed
  `collect_binary_candidates`.
- `crates/oxfunc_core/src/functions/match_fn.rs`: `collect_match_candidates`
  now delegates to `collect_approximate_comparables`;
  `eval_match_approximate_prepared` maps the chosen filtered position back to
  the original index.
- `crates/oxfunc_core/src/functions/vhlookup_family.rs`: no logic change;
  approximate VLOOKUP/HLOOKUP inherit the working-tree MATCH behavior via
  `eval_match_surface`. New focused tests added.

Because the index map is the identity when there are no blanks
(`original_indices[p] == p`), every existing pinned lane (W10S2 unsorted /
duplicate-selection rows, exact modes) is unchanged.

### Out-of-scope (deliberately preserved as-is)
- **Error cells still abort to `#N/A`.** `prepared_approximate_candidate`
  returns `Err(NotAvailable)` for `Error` cells exactly as before. Excel's
  observed behavior for a *probed* error cell can propagate the error code
  (e.g. `#DIV/0!`) rather than skip it, so the error-cell repair needs an
  empirical probe lane before changing direction.
- **Cross-type (None-ordering) approximate matches still abort.**
  `comparable_order` returning `None` (e.g. numeric lookup vs text candidate)
  is unchanged — the digest's F5 notes the replacement policy (skip-within-
  bracket vs number<text<logical ordering) is unpinned and must not be a
  mechanical `None -> Greater` substitution.

## Validation
- `cargo test -p oxfunc_core --lib match` -> `330 passed; 0 failed`.
- `cargo test -p oxfunc_core --lib xmatch` -> `44 passed; 0 failed`.
- `cargo test -p oxfunc_core --lib vhlookup` -> `13 passed; 0 failed`.
- New lanes: ascending trailing-blank last-value idiom, ascending/descending
  interior-blank skip, error-cell-still-aborts (MATCH and XMATCH binary),
  VLOOKUP/HLOOKUP approximate blank-key skip. Pinned W10S2
  `follow_excel_duplicate_selection` and `follow_empirical_unsorted_lanes`
  lanes still pass.

## Similar-Risk Scan
- `XMATCH` linear modes (search_mode `1`/`-1`) already skip blanks correctly via
  `LookupCandidate::Skip` in `xmatch_scan_exact_or_approximate`; unchanged.
- `XMATCH` exact binary mode benefits from the same blank-skip (it routes
  through `xmatch_binary_search`); covered by the interior-blank exact lane.
- `LOOKUP` / `FREQUENCY` are a separate numeric-only substrate (digest F6,
  bounded under the W24 provisional slice) and are not touched here.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded (working-tree patch present; checkpoint not landed)
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] W102B probe matrix updated for error-cell and cross-type lanes
- [ ] handoff filed if W102B evidence shows a cross-repo contract change is required
- [x] linked reports updated

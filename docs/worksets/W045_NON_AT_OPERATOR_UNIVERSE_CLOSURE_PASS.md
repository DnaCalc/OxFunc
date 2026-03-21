# WORKSET - Non-@ Operator Universe Closure Pass (W45)

## 1. Purpose
Open one dedicated operator packet for all evaluable non-`@` operator surfaces.

This packet exists because operator work is currently fragmented:
1. `OP_ADD` was seeded in `W10`,
2. `OP_SPILL_REF` exists as an explicit operator slice,
3. `W014` owns only implicit intersection (`@`),
4. the remaining operator universe is still an undeclared backlog.

`W45` is the planning and execution owner for the non-`@` operator pass across:
1. canonical operator inventory,
2. contract and admission profile work,
3. Rust runtime implementation,
4. Lean/formal executable substrate work,
5. empirical Excel validation,
6. replay/test packet coverage,
7. library-context export refinement.

## 2. Provenance
Opened after:
1. the library-context snapshot export made operator coverage gaps visible in `W044`,
2. the current review concluded that a unified non-`@` operator packet is now needed,
3. `W014` remained the dedicated owner for implicit intersection and should stay separate.

Relevant context:
1. `docs/worksets/W010_TEN_FUNCTION_MIXED_SEAMS.md`
2. `docs/worksets/W014_IMPLICIT_INTERSECTION_OPERATOR.md`
3. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md`
4. `docs/function-lane/EXCEL_FUNCTION_DEFINITION_DISCUSSION.md`
5. `docs/function-lane/FUNCTION_SLICE_OP_ADD_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_OP_SPILL_REF_CONTRACT_PRELIM.md`
7. `docs/function-lane/W044_EXECUTION_RECORD.md`
8. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
9. operator/preference cross-check note:
   - future operator refresh passes should be checked against Microsoft's operator reference page:
     `https://support.microsoft.com/en-us/office/calculation-operators-and-precedence-in-excel-48be406d-4975-4d31-b2b8-7af9e0e2878a`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W45_NON_AT_OPERATOR_INVENTORY.csv`

Current total:
1. `22` operator rows.

Scope rule:
1. `@` / `OP_IMPLICIT_INTERSECTION` is explicitly excluded and remains owned by `W014`.
2. parse-only delimiters remain out of scope unless they are promoted to evaluable operator rows.

Current inventory members:
1. `OP_UNARY_PLUS`
2. `OP_NEGATE`
3. `OP_ADD`
4. `OP_SUBTRACT`
5. `OP_MULTIPLY`
6. `OP_DIVIDE`
7. `OP_POWER`
8. `OP_PERCENT`
9. `OP_CONCAT`
10. `OP_EQUAL`
11. `OP_NOT_EQUAL`
12. `OP_LESS_THAN`
13. `OP_LESS_EQUAL`
14. `OP_GREATER_THAN`
15. `OP_GREATER_EQUAL`
16. `OP_RANGE_REF`
17. `OP_INTERSECTION_REF`
18. `OP_UNION_REF`
19. `OP_SPILL_REF`
20. `OP_TRIM_REF_LEADING`
21. `OP_TRIM_REF_TRAILING`
22. `OP_TRIM_REF_BOTH`

Local naming note:
1. some rows are already doctrinally anchored (`OP_ADD`, `OP_SPILL_REF`, `OP_UNION_REF`, `OP_TRIM_REF(...)`);
2. some ids are still OxFunc-local provisional planning ids pending function-definition lane lock (`OP_UNARY_PLUS`, `OP_NEGATE`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_PERCENT`, `OP_CONCAT`, `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS_THAN`, `OP_LESS_EQUAL`, `OP_GREATER_THAN`, `OP_GREATER_EQUAL`, `OP_RANGE_REF`, `OP_INTERSECTION_REF`).

## 4. Why This Packet Matters
1. Operators are part of Excel semantic identity, not only grammar punctuation.
2. The operator universe exposes coercion, reference identity, array publication, caller context, comparison rules, and workbook-state/reference seams in a compact surface.
3. OxFml parse/bind work now needs a better downstream operator inventory than the current partial `W044` export.
4. A unified packet should stop the operator backlog from remaining a diffuse “undeclared operators” note.

## 5. In Scope
1. canonical or provisional operator inventory freeze for all non-`@` evaluable operators,
2. explicit operator admission and argument-preparation profiles,
3. Rust runtime implementation or uplift of existing operator kernels,
4. Lean/formal executable substrate work for the primary operator families,
5. empirical Excel probes and scenario manifests for each operator family,
6. packeted runtime requirements, execution record, evidence rows, and scope reconciliation,
7. `W044` snapshot export refinement so OxFml can see a materially better operator surface.

## 6. Out Of Scope
1. `@` / `SINGLE` / `_xlfn.SINGLE` and their caller-context scalarization seam,
2. parser-only delimiters that remain non-evaluable,
3. final OxFml grammar ownership lock for every operator token,
4. locale-specific list-separator punctuation that is not an evaluable operator row,
5. pretending every operator must share one identical runtime substrate.

## 7. Planned Execution Waves
1. Wave A: arithmetic, unary, and postfix numeric operators
   - `OP_UNARY_PLUS`, `OP_NEGATE`, `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_POWER`, `OP_PERCENT`
2. Wave B: concatenation and comparison operators
   - `OP_CONCAT`, `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS_THAN`, `OP_LESS_EQUAL`, `OP_GREATER_THAN`, `OP_GREATER_EQUAL`
3. Wave C: reference composition operators excluding `@`
   - `OP_RANGE_REF`, `OP_INTERSECTION_REF`, `OP_UNION_REF`, `OP_SPILL_REF`, `OP_TRIM_REF_*`
4. Wave D: packet-wide operator export, replay/evidence reconciliation, and remaining extraction decisions

## 8. Required Outputs
1. a machine-readable operator inventory and reconciliation ledger,
2. shared operator packet contract text plus per-family/per-operator slice contracts where needed,
3. Rust runtime modules and unit/integration tests,
4. Lean executable operator substrate modules aligned to the admitted slices,
5. native Excel empirical scenario manifests, runtime requirements, and replayable result artifacts,
6. packet execution record and scope reconciliation,
7. library-context snapshot export updates for operator rows and special-interface annotations,
8. OxFml handoff note only if this packet changes the seam materially.

## 9. Gate Criteria
This workset can only be reported `scope_complete` when:
1. every non-`@` evaluable operator row is either:
   - covered by declared runtime/formal/evidence artifacts in `W45`, or
   - explicitly extracted to a successor packet with a named owner,
2. no undeclared non-`@` operator remains as an unowned backlog note,
3. the arithmetic/comparison/reference operator families each have:
   - contract coverage,
   - Rust runtime coverage,
   - Lean/formal executable substrate coverage,
   - empirical Excel evidence for the admitted current-baseline slice,
4. `W044` or its successor export is updated so OxFml can consume materially better operator rows than the current partial export,
5. `W014` ownership remains cleanly separate for `@`.

## 10. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - initial packet planning state only; see `docs/function-lane/W45_EXECUTION_RECORD.md` for the executed packet state

## 11. Closure Result
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. resolution:
   - all `22` non-`@` operator rows are now covered in `W45`
   - no operator row was extracted to a successor packet
   - `W044` export has been refreshed and now exposes the full current non-`@` operator surface plus the separately modeled implicit-intersection row
6. orthogonal seam note:
   - legacy CSE array-formula (`{=...}`) context remains tracked with `W014` / cross-seam work and is not an open lane in declared `W45` scope

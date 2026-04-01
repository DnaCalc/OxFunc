# WORKSET - Hidden Ordinary Backlog Systematic Completion Plan (W57)

## 1. Purpose
Turn the remaining `185` hidden non-deferred ordinary backlog rows from a reconciled backlog list into an executable closure program that can be run family-by-family without lowering OxFunc evidence standards.

This packet exists to replace “one big hidden backlog” with:
1. a deterministic packet sequence,
2. explicit packet ownership,
3. shared substrate-first ordering,
4. family-level closure outputs,
5. extraction and blocker rules that prevent silent doctrine drift.

## 2. Position and Dependencies
Program position:
1. post-`W056` packet (`W57`).

Dependencies:
1. `W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
2. `W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
3. `W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
4. `W054_LEAN_FORMALIZATION_GAP_RECONCILIATION.md`
5. `docs/function-lane/W51_HIDDEN_NON_DEFERRED_BACKLOG_FIRST_PASS.csv`
6. `docs/function-lane/W44_DOCUMENTED_COMPLETE_SNAPSHOT_STALE_INVENTORY.csv`

## 3. Scope
In scope for `W57`:
1. the full current `185`-row hidden non-deferred ordinary backlog from `W051`,
2. execution packetization of those rows into quality-preserving family packets,
3. normalization of the seven grouped-name rows before semantic closure starts,
4. a shared output contract and verification contract for every successor packet,
5. an extraction rule for any row that proves host-sensitive, provider-sensitive, or otherwise non-ordinary.

Out of scope for `W57`:
1. closing the `17` deferred rows in `W050`,
2. re-opening already-closed seam-heavy rows unless concrete new evidence forces it,
3. silent catalog promotion without native/runtime/formal evidence,
4. alternate locale/version sweeps beyond the declared current reference baseline.

## 4. Current Target
Current target:
1. `185` hidden snapshot entries.
2. `0` operators.
3. all current `W051` membership after `W014` closure.
4. machine-clean execution counterpart after `W058`: `192` function rows.

Current packet split:
1. `P0` normalization and grouped-row cleanup (`7` grouped entries)
2. `P1` engineering conversions and Bessel family (`16`)
3. `P2` complex-number family (`26`)
4. `P3` statistical distributions and modern/legacy compat A (`29`)
5. `P4` statistical distributions and modern/legacy compat B (`35`)
6. `P5` date/time and business-day family (`18`)
7. `P6` financial core misc family (`15`)
8. `P7` database family (`12`)
9. `P8` text core and compatibility family (`9`)
10. `P9` math/matrix/rounding family (`15`)
11. `P10` lookup/logical residuals (`3`)

Machine-readable packet register:
1. `docs/function-lane/W57_PACKET_REGISTER.csv`
2. post-normalization successor split:
   - `docs/function-lane/W58_SUCCESSOR_PACKET_SPLIT.csv`

## 5. Execution Principle
The backlog should not be executed as one giant mega-batch claim. It should be executed as a sequence of bounded successor packets with shared rules.

Execution rules:
1. finish `P0` before any packet that depends on grouped-name normalization. This gate is now closed by `W058`.
2. execute substrate-heavy packets before their likely downstream dependents.
3. complete one packet’s full evidence bundle before claiming any member row complete.
4. if a row proves non-ordinary, extract it immediately with explicit successor ownership; do not leave it half-ordinary inside the packet.
5. update `W051` after each packet closure wave; do not wait until the end.
6. refresh export metadata only after a packet’s members are honestly supported.

## 6. Successor Packet Order
### Wave 0 - Backlog Normalization
1. `P0` grouped-name normalization and packet split finalization.

### Wave 1 - Shared Numeric Substrates
1. `P1` engineering conversions and Bessel family.
2. `P2` complex-number family.

Reason:
1. these packets likely establish shared conversion, radix, and complex-value helpers used across many rows.

### Wave 2 - Statistical Distribution Surfaces
1. `P3` statistical distributions and compat A.
2. `P4` statistical distributions and compat B.

Reason:
1. modern/legacy name pairs should close together so semantics, compat aliases, and catalog promotion stay aligned.

### Wave 3 - Core Calendar and Money
1. `P5` date/time and business-day family.
2. `P6` financial core misc family.

Reason:
1. date serial semantics and day-count helpers are likely shared dependencies for the financial residuals.

### Wave 4 - Grid and Text Residuals
1. `P7` database family.
2. `P8` text core and compatibility family.

Reason:
1. both are ordinary but evidence-heavy; they benefit from stable scalar/text/date substrates already being closed.

### Wave 5 - Matrix and Lookup Residuals
1. `P9` math/matrix/rounding family.
2. `P10` lookup/logical residuals.

Reason:
1. these are smaller but highly visible residuals and should close on top of already-stable array/reference/date/text behavior.

## 7. Mandatory Output Contract Per Successor Packet
Every successor packet created under `W57` must produce:
1. one workset spec under `docs/worksets/`
2. one family contract note under `docs/function-lane/`
3. one seeded native replay manifest
4. one runtime-requirements note
5. one execution record
6. Rust runtime/tests for the admitted current-baseline slice
7. Lean substrate/binding alignment for the packet’s primary semantic substrate
8. one evidence-registry row
9. `.tmp` native replay artifact(s)
10. `W051` removal/update for rows honestly closed by that packet
11. downstream snapshot/labeling refresh where packet members move from backlog to supported

## 8. Mandatory Verification Contract Per Packet
At minimum, every successor packet must run:
1. native Excel replay against the packet manifest
2. targeted Rust tests for the packet family
3. `lake build`
4. `cargo fmt --manifest-path crates/oxfunc_core/Cargo.toml`
5. `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml` when shared dispatch/export surfaces changed

When shared behavior changes materially, run:
1. broader targeted crate tests for the affected substrate
2. any relevant OxFml integration tests if the packet touches shared prepared-call or adapter consequences

## 9. Extraction Rules
Any row or subfamily must be extracted out of the ordinary program when evidence shows it is:
1. host/profile-sensitive,
2. provider-sensitive,
3. workbook-visibility or publication-sensitive,
4. absent on the current host baseline,
5. blocked by a cross-repo interface mismatch,
6. blocked by a missing semantic substrate that would distort the current packet claim.

Allowed per-row states inside the `W57` program:
1. `planned`
2. `in_progress`
3. `blocked`
4. `done`
5. `extracted`

## 10. Quality Rules
Highest-quality execution means:
1. modern and legacy compatibility pairs close together, not in separate ad hoc passes,
2. packet claims are substrate-based and evidence-backed, not catalog-driven,
3. every packet ends either `complete` or with explicit extractions/blockers,
4. `W054` is updated whenever Rust/Lean substrate coverage would otherwise drift,
5. the snapshot export never gets ahead of packet truth,
6. `W051` always equals the real remaining non-deferred backlog after each packet closes.

## 11. Packet Register Summary
See `docs/function-lane/W57_PACKET_REGISTER.csv`.

Recommended packet IDs and likely successor worksets:
1. `P0` -> `W058` grouped-row normalization and ordinary backlog split
2. `P1` -> `W059` engineering conversions and Bessel family
3. `P2` -> `W060` complex-number family
4. `P3` -> `W061` statistical distributions and compat A
5. `P4` -> `W062` statistical distributions and compat B
6. `P5` -> `W063` date/time and business-day family
7. `P6` -> `W064` financial core misc family
8. `P7` -> `W065` database family
9. `P8` -> `W066` text core and compatibility family
10. `P9` -> `W067` math/matrix/rounding family
11. `P10` -> `W068` lookup/logical residuals

The exact numbering can shift if intervening packets appear, but the sequence and member ownership should remain stable.

## 12. Exit Condition
`W57` is complete when:
1. every one of the `185` rows is assigned to a successor packet,
2. grouped-name normalization is fully resolved up front,
3. every successor packet has a declared output/verification contract,
4. no row remains as an unowned hidden backlog item.

The broader ordinary backlog program is complete only when every row is either:
1. `done`, or
2. `extracted` with explicit successor ownership and rationale.

## 13. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - successor packet execution `W059` through `W068` has not started yet
   - the machine-clean successor split now lives in `W58_SUCCESSOR_PACKET_SPLIT.csv`

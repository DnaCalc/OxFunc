# WORKSET - Power Zero-To-Zero Num Error Parity (W078)

## 1. Purpose
Own the bounded OxFunc-side review/correction lane for the prior shared power
kernel claim where an April 8, 2026 local Excel replay reported `#NUM!` for
`0^0` and `POWER(0,0)`, while the then-current local `POWER` / `OP_POWER` path
published `1`.

Fresh-review note:
1. this packet records a prior local finding and working-tree correction claim,
2. it must not be treated as a current known bug without fresh Excel
   confirmation against the active baseline,
3. if fresh replay shows no current divergence, the W078/BUG-FUNC-005 surfaces
   should be reconciled as stale rather than carried as active bug truth.

## 2. Why This Packet Exists
The earlier `POWER` repair packet fixed integer-publication drift but did not
pin the zero-to-zero domain row:
1. live Excel on 2026-04-08 reported both `=0^0` and `=POWER(0,0)` return
   `#NUM!`,
2. local `power_kernel` treated every zero exponent as `1`,
3. `POWER`, `OP_POWER`, the q-binary fast path, and the Lean executable model
   all inherited that same assumption.

## 3. Provenance
1. local live Excel COM replay on 2026-04-08
2. `docs/function-lane/W53_NUMERIC_FORENSICS_20260326.md`
3. `docs/function-lane/W45_EXECUTION_RECORD.md`
4. `docs/bugs/streams/BUG-FUNC-005_power_zero_to_zero_diverges_from_excel.md`
5. `formal/lean/OxFunc/Functions/PowerFn.lean`

## 4. Scope
In scope:
1. record the local Excel finding as a bug report and canonical bug stream,
2. correct the shared `power_kernel` zero-to-zero domain lane for both `POWER`
   and `OP_POWER`,
3. align the Lean executable model to the same rule,
4. add focused local runtime, surface, adapter-fixture, and Wave A empirical
   validation for `0^0`,
5. refresh current-gap and workset truth surfaces so `POWER` is not overclaimed.

Out of scope:
1. broader operator broadcast work already owned by `W074`,
2. downstream seam handoff unless a new cross-repo dependency is discovered,
3. new locale/channel sweeps beyond the current installed Excel baseline,
4. unrelated finance-family semantic changes beyond the narrow caller scan.

## 5. Initial Epic Lanes
1. bug intake and ownership registration
2. shared power-kernel correction
3. Lean executable-model alignment
4. focused local validation
5. current-gap and workset truth reconciliation

## 6. Closure Condition
`W078` is complete for declared scope only when:
1. fresh Excel replay either confirms the prior `0^0` / `POWER(0,0)` claim or
   invalidates it as stale,
2. the shared Rust and Lean power lanes agree with the freshly confirmed rule,
3. focused local validation is recorded across runtime, dispatch, adapter, and
   native Wave A evidence where the bug remains real,
4. `W051` and the bug/current-workset surfaces no longer overclaim `POWER`.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - fresh Excel confirmation review for the prior `POWER` bug claim
   - landed-ref promotion for the local `0^0 -> #NUM!` correction

# WORKSET - RATE Default-Guess Convergence Repair (W081)

## 1. Purpose
Own the bounded OxFunc-side repair for the reopened `RATE` lane where Excel
returns a small positive periodic rate for a mortgage-style omitted-guess case,
but the current local default-guess solver path fails with `#NUM!`.

## 2. Why This Packet Exists
The earlier financial time-value packet admitted `RATE` through a representative
seed inversion row but did not force the solver through a long-horizon
mortgage-style omitted-guess case:
1. live Excel on 2026-04-10 showed `=RATE(360,-1073.64,200000)` has underlying
   value `0.004166644536345589`,
2. local direct OxFunc replay on the same date showed the omitted-guess path
   returns `NoConvergence` and therefore publishes `#NUM!`,
3. nearby explicit guesses (`0.01`, `0.005`, `0.004`, `0.001`) all converge
   locally to the Excel-like value,
4. the open lane is therefore the default-guess / fallback robustness path, not
   a display-format misunderstanding.
5. the local `W081` correction now adds a bounded bracket-and-bisection
   fallback around the existing secant path and validates the mortgage-style
   row against live Excel plus the W24 witness packet, but that correction is
   still only on the working tree.

## 3. Provenance
1. user observation on 2026-04-10
2. live Excel COM replay on 2026-04-10
3. `docs/bugs/streams/BUG-FUNC-009_rate_default_guess_solver_no_convergence.md`
4. `docs/function-lane/FUNCTION_SLICE_FINANCIAL_TIME_VALUE_FAMILY_CONTRACT_PRELIM.md`
5. `docs/function-lane/W24_BATCH11_FINANCIAL_TIME_VALUE_EXECUTION_RECORD.md`

## 4. Scope
In scope:
1. record the reopened `RATE` lane as a canonical bug stream and bounded owner
   workset,
2. add replayable local evidence for the mortgage-style omitted-guess case,
3. characterize the current solver's default-guess and fallback behavior
   against adjacent Excel rows,
4. repair the omitted-guess convergence strategy for the reopened lane,
5. add focused validation and reconcile `W051` / finance truth surfaces.

Out of scope:
1. unrelated `IRR`, `XIRR`, or `XNPV` solver changes unless new evidence shows
   they share the same local failure mode,
2. locale/channel sweeps beyond the current installed Excel baseline,
3. downstream OxFml handoff unless the repair changes a seam contract rather
   than a local finance kernel.

## 5. Initial Epic Lanes
1. bug intake and ownership registration
2. omitted-guess versus explicit-guess replay characterization
3. local `RATE` solver robustness repair
4. focused validation
5. current-gap and workset truth reconciliation

## 6. Closure Condition
`W081` is complete for declared scope only when:
1. the reopened mortgage-style omitted-guess `RATE` lane matches Excel locally,
2. the admitted earlier seed inversion row remains aligned,
3. focused validation is recorded for omitted/default and nearby explicit guess
   lanes,
4. `W051` and the bug/workset surfaces no longer overclaim `RATE`.

## 7. Current Reading
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - bounded characterization of adjacent omitted-guess `RATE` rows
   - landed-ref promotion after the local fix

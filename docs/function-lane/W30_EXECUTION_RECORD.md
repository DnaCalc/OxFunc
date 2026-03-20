# W30 Execution Record - Locale/Profile-Sensitive Text And Number Functions

Status: `complete`
Workset: `W30`

## 1. Purpose
Record the current-boundary seam decision for the `W26` locale/profile-sensitive subset:
1. `ASC`
2. `DBCS`
3. `JIS`
4. `NUMBERVALUE`

## 2. Evidence Used
1. `CURRENT_BLOCKERS.md`
2. `docs/worksets/W030_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_FUNCTIONS.md`
3. `docs/function-lane/W30_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_INVENTORY.csv`
4. `docs/function-lane/W26_EXECUTION_RECORD.md`
5. `docs/function-lane/W26_RUNTIME_REQUIREMENTS.md`
6. `docs/function-lane/W26_SCOPE_RECONCILIATION.csv`
7. `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
8. `.tmp/w26-host-profile-provider-results.csv`
9. `crates/oxfunc_core/src/functions/text_compat_locale_family.rs`
10. `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`

## 3. Current-Boundary Findings
From `.tmp/w26-host-profile-provider-results.csv`:
1. `ASC("ＡＢＣ　１２３")` returned the input unchanged on the current host/profile.
2. `DBCS("ABC ｶﾞ")` also returned the input unchanged on the current host/profile.
3. `JIS("ABC ｶﾞ")` returned `#NAME?`, so the function is not admitted on this worksheet host baseline.
4. `NUMBERVALUE("1,234.5%")` returned `#VALUE!` when separator defaults were omitted, while the explicit-separator lane still returned `12.345`.

## 4. Reconciliation Decision
1. `ASC`, `DBCS`, and `JIS` move to successor `W034` as width-conversion host/profile capability work.
2. `NUMBERVALUE` moves to successor `W035` as locale-default parsing/profile work.
3. `W30` therefore completes as a seam-definition and reconciliation packet, not as function-phase closure for the four functions themselves.

## 5. Scope Reconciliation
See:
1. `docs/function-lane/W30_SCOPE_RECONCILIATION.csv`

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W30` scope after reconciliation

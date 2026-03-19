# W26 Execution Record - Locale/Profile And Provider-Sensitive Ordinary Outliers

Status: `complete`
Workset: `W26`

## 1. Purpose
Record the current starting position for the extracted locale/profile-sensitive and provider-sensitive ordinary outliers.

## 2. Current Packet Reading
1. `ASC`, `DBCS`, and `JIS` are not currently honest pure-function closure candidates on this host baseline.
2. `NUMBERVALUE` omitted defaults are locale/profile-sensitive.
3. `TRANSLATE` is provider-sensitive rather than a purely local deterministic text function.

## 3. Established Evidence
1. `BLK-FN-005` records the current host-profile-sensitive width-conversion findings.
2. `BLK-FN-006` records the `NUMBERVALUE` and `TRANSLATE` current-baseline findings.
3. `W24` already extracted these rows out of the ordinary mega-batch on that basis.

## 4. Executed Packet
Artifacts created or updated:
1. `docs/function-lane/W26_HOST_PROFILE_PROVIDER_SCENARIO_MANIFEST_SEED.csv`
2. `docs/function-lane/W26_RUNTIME_REQUIREMENTS.md`
3. `tools/w26-probe/run-w26-host-profile-provider-baseline.ps1`
4. `.tmp/w26-host-profile-provider-results.csv`
5. `docs/function-lane/W26_SCOPE_RECONCILIATION.csv`
6. `docs/worksets/W030_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_FUNCTIONS.md`
7. `docs/function-lane/W30_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_INVENTORY.csv`
8. `docs/worksets/W031_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS.md`
9. `docs/function-lane/W31_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS_INVENTORY.csv`
10. `docs/worksets/W026_DEFERRED_LOCALE_PROFILE_AND_PROVIDER_SENSITIVE_ORDINARY_OUTLIERS.md`
11. `docs/function-lane/W26_EXECUTION_RECORD.md`

## 5. Empirical Findings
From `.tmp/w26-host-profile-provider-results.csv`:
1. `ASC("ＡＢＣ　１２３")` returned the input unchanged on the current host/profile.
2. `DBCS("ABC ｶﾞ")` also returned the input unchanged on the current host/profile.
3. `JIS("ABC ｶﾞ")` returned `#NAME?`, so the function is not admitted on this worksheet host baseline.
4. `NUMBERVALUE("1,234.5%")` returned `#VALUE!` when separator defaults were omitted, while `NUMBERVALUE("1,234.5%",".",",")` returned `12.345`.
5. `TRANSLATE("hola","es","es")` returned `"hola"`, while `TRANSLATE("hello","en","es")` returned `#BUSY!`.

## 6. Reconciliation Result
1. `ASC`, `DBCS`, `JIS`, and `NUMBERVALUE` move to `W30`.
2. `TRANSLATE` moves to `W31`.
3. `W26` therefore completes as a characterization-and-extraction packet, not as an implementation-closure packet for the five functions themselves.

## 7. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W26` scope after reconciliation

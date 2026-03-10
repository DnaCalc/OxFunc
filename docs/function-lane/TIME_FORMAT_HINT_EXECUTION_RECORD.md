# Time Format-Hint Execution Record

Status: `complete-provisional`
Evidence ID: `W10-W12-TFMT-20260310`

## 1. Purpose
Record empirical evidence for post-evaluation caller-cell format-hint behavior on built-in `NOW()` and `TODAY()` under the current reference Excel baseline.

## 2. Executed Scope
Execution date:
1. `2026-03-10`

Executed commands:
1. `powershell -File tools/time-format-hint-probe/run-time-format-hint-suite.ps1 -OutDir .tmp`

Primary inputs:
1. `docs/function-lane/TIME_FORMAT_HINT_SCENARIO_MANIFEST_SEED.csv`

Primary outputs:
1. `.tmp/time-format-hint-results-default.csv`
2. `.tmp/time-format-hint-results-compat.csv`
3. `.tmp/time-format-hint-results.csv`
4. `.tmp/time-format-hint-summary.json`

## 3. Results Summary
1. dual-run scope: `default` and `compat_template`
2. target functions: `NOW`, `TODAY`
3. exercised caller-format states:
   - `General`
   - explicit custom numeric format `0.000`

## 4. Current Findings
1. `NOW()` entered into a `General` cell applies `yyyy/mm/dd hh:mm` in the observed baseline.
2. `TODAY()` entered into a `General` cell applies `yyyy/mm/dd` in the observed baseline.
3. Preformatted numeric cells retained their existing `0.000` format for both `NOW()` and `TODAY()` in the observed baseline.
4. These behaviors are part of function-semantic characterization, but application of the resulting format hint is an engine-surface responsibility and is not required from the XLL verification seam.

## 5. Recording Rule
1. Use this evidence when qualifying `NOW`/`TODAY` current-phase closure claims.
2. Treat locale/version differences in exact format strings as orthogonal validation-phase expansion unless they are declared in-scope for a specific pass.

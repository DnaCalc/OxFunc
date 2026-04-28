# Smart-Fuzzer Tools

Status: `tooling_sandbox`

Tracked tools in this directory are reproducible helpers for W088 exploration.
Generated outputs should normally go to `smart-fuzzer/cache/` or
`smart-fuzzer/runs/`, both ignored by default.

## Build-StaticRiskIndex.ps1

Builds a derived function risk index for exploration ordering. It consumes:

1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`,
2. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`,
3. `docs/bugs/BUG_STREAM_REGISTER.csv`,
4. `docs/function-lane/*SCENARIO_MANIFEST_SEED.csv`,
5. `docs/function-lane/*DEFERRED*INVENTORY*.csv`,
6. `crates/oxfunc_core/src/functions/*.rs`.

Default output:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-StaticRiskIndex.ps1
```

The default index path is `smart-fuzzer/cache/static-risk-index.json`. The
index is not semantic authority; it is a disposable exploration-ordering input.

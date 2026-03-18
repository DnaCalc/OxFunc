# W16 Batch 57 - Legacy Stats Alias Family

Status: `in_progress-self-contained`
Workset: `W16`
Evidence ID: `W16-BATCH57-LEGACY-STATS-ALIASES-20260316`

## Scope
1. `COVAR`
2. `MODE`
3. `PERCENTILE`
4. `PERCENTRANK`
5. `QUARTILE`
6. `LOGINV`

## Shape
1. This batch is intentionally limited to legacy names that are thin aliases over already-admitted OxFunc substrates.
2. `COVAR` wraps `COVARIANCE.P`.
3. `MODE` wraps `MODE.SNGL`.
4. `PERCENTILE`, `PERCENTRANK`, and `QUARTILE` wrap the `.INC` variants.
5. `LOGINV` wraps `LOGNORM.INV`.
6. Shared dispatch/catalog/Lean import wiring is out of scope for this owned-file pass.

## Executable Coverage
1. Rust unit tests pin seeded rows for `COVAR`, `MODE`, `PERCENTILE`, `PERCENTRANK`, `QUARTILE`, and `LOGINV`.
2. Rust unit tests also pin alias-local arity handling and worksheet-error mapping.
3. The Lean file is metadata-only and mirrors the Rust `FunctionMeta` shape for the six aliases.

## Open Lane
1. This note covers the self-contained family file only; registration in shared surfaces remains separate work because it is outside the owned-file scope.

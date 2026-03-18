# W16 Batch 63 - Depreciation Family

Status: `in_progress-self-contained`
Workset: `W16`
Evidence ID: `W16-BATCH63-DEPRECIATION-20260316`

## Scope
1. `SLN`
2. `SYD`
3. `DB`
4. `DDB`
5. `VDB`

## Shape
1. This batch is a scalar financial family with values-only pre-adapter semantics and optional-argument defaults handled inside the family surface.
2. `SLN` and `SYD` are direct formula kernels.
3. `DB` uses the documented fixed-declining-balance rate rounded to three decimals plus the documented first/last-period month adjustment.
4. `DDB` is implemented as a one-period declining-balance interval over the shared variable-declining kernel with `no_switch=TRUE`.
5. `VDB` supports partial periods and optional straight-line switching, matching the published Microsoft examples.

## Executable Coverage
1. Rust unit tests pin the Microsoft sample rows for `SLN`, `SYD`, `DB`, and `VDB`.
2. Rust unit tests pin `DDB` first- and second-period lanes, optional default handling, and worksheet-error mapping.
3. The `VDB` tests also pin the empirical/documented zero `start_period` examples despite the generic support-text positivity remark.
4. The Lean file is metadata-only and mirrors the Rust `FunctionMeta` profile for the family.

## Open Lane
1. This owned-file pass does not wire the family into shared dispatch/catalog/import surfaces, so crate-wide admission remains separate work.

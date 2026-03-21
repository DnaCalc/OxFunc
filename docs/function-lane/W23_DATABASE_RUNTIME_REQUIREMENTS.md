# W23 Database Runtime Requirements

Status: `provisional`
Workset: `W23`

## 1. Purpose
1. define the native worksheet requirements for the bounded database-family replay packet,
2. keep the database-grid setup reproducible,
3. make the packet independent of the unrelated host-sensitive residuals in `W23`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W23_DATABASE_SCENARIO_MANIFEST_SEED.csv`
2. runner: `tools/w23-probe/run-w23-database-baseline.ps1`

## 3. Worksheet Setup
1. `A1:D6` is the database grid with headers `Type`, `Salesperson`, `Sales`, `Units`.
2. `F1:F2` is the `Dav*` criteria range.
3. `H1:H2` is the `Produce` criteria range.
4. `J1:J2` is the unique `Buchanan` criteria range.
5. `L1:L2` is the no-match criteria range.
6. `N1:O3` is the duplicate-header OR criteria range.

## 4. Recording Rules
1. numeric lanes compare the underlying `Value2` against the seeded tolerance.
2. error lanes compare the displayed worksheet text.
3. results are exported to `.tmp/w23-database-results.csv`.

## 5. Out Of Scope
1. full criteria-formula evaluation against worksheet context,
2. locale/version sweep,
3. host-sensitive functions elsewhere in `W23`.

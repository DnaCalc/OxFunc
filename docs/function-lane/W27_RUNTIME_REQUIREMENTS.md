# W27 Runtime Requirements - Bond Core And Odd Bond Direct Excel Parity

Status: `provisional`
Workset: `W27`

## 1. Purpose
Pin the current-baseline direct Excel parity packet for the bond-core and odd-bond families extracted from `W24`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W27_BOND_ODD_BOND_SCENARIO_MANIFEST_SEED.csv`
2. Excel host baseline with worksheet formulas available through COM automation

## 3. Output
1. result artifact: `.tmp/w27-bond-odd-bond-results.csv`

## 4. Classification Rule
1. direct packet mismatches are finance-kernel semantic mismatches unless a documented XLL/host seam limitation applies
2. the public ExcelFinancialFunctions F# library may be used as a benchmark and structural comparison aid, but direct Excel worksheet evidence remains authoritative

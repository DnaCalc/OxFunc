# W41 Web Text/XML Runtime Requirements

Status: `provisional`
Workset: `W41`

## 1. Purpose
1. define the native worksheet requirements for the admitted `ENCODEURL` / `FILTERXML` slice,
2. keep this local slice separate from the true provider/cube lanes still open in `W41`.

## 2. Inputs
1. scenario manifest: `docs/function-lane/W41_WEB_TEXT_XML_SCENARIO_MANIFEST_SEED.csv`
2. runner: `tools/w41-probe/run-w41-web-text-xml-baseline.ps1`

## 3. Recording Rules
1. plain scalar results compare the displayed worksheet text.
2. spill lanes record the spill range text joined by `|` in row-major order.
3. error lanes compare the displayed worksheet text.
4. results are exported to `.tmp/w41-web-text-xml-results.csv`.

## 4. Out Of Scope
1. live web-service fetch semantics,
2. platform/version sweep,
3. cube/provider/add-in lanes elsewhere in `W41`.

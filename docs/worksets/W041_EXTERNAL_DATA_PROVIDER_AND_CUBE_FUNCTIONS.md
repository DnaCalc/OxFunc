# WORKSET - External Data Provider And Cube Functions (W41)

## 1. Purpose
Open the interesting packet for external-data, provider-bound, and cube-context functions.

This packet exists to group the high-interest functions whose semantics are shaped by provider availability, external services, COM/automation, or cube connections rather than by pure local kernels.

## 2. Provenance
Opened after the interesting-function review identified a coherent external/provider cluster separate from:
1. ordinary provider-language `TRANSLATE` work already deferred through `W031` / `W036`,
2. host/database metadata work in `W023`,
3. lambda and dynamic-array families in `W038` and `W039`.

Relevant context:
1. `docs/function-lane/INTERESTING_FUNCTIONS_INITIAL_CLASSIFICATION.csv`
2. `docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv`
3. `docs/worksets/W031_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS.md`
4. `docs/worksets/W036_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_BASELINE.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W41_EXTERNAL_DATA_PROVIDER_AND_CUBE_INVENTORY.csv`

Current total:
1. `12` functions.

Members:
1. `CUBEKPIMEMBER`
2. `CUBEMEMBER`
3. `CUBEMEMBERPROPERTY`
4. `CUBERANKEDMEMBER`
5. `CUBESET`
6. `CUBESETCOUNT`
7. `CUBEVALUE`
8. `ENCODEURL`
9. `FILTERXML`
10. `RTD`
11. `STOCKHISTORY`
12. `WEBSERVICE`

## 4. Why This Packet Matters
1. These functions expose some of Excel's least explicit machinery: provider presence, external connection truth, service failure, and capability-denied outcomes.
2. They are strong pressure tests for the availability / provider-failure taxonomy already discussed with OxFml.
3. They provide a natural place to distinguish:
   - parse/bind catalog presence,
   - runtime provider capability,
   - connection/service outcomes,
   - host/platform restrictions.
4. They are interesting precisely because they cannot be honestly reduced to ordinary pure kernels.

## 5. In Scope
1. empirical characterization of admitted current-baseline presence and failure surfaces,
2. explicit capability and provider taxonomy for the family,
3. packet-level contract work for provider-bound function classes,
4. scenario manifests, runtime requirements, execution record, and evidence registry rows,
5. honest seam statements separating local semantics from external provider truth.

## 6. Out Of Scope
1. full provider implementation or live external-service integration,
2. language-provider functions already split to `W036`,
3. unrelated host/database metadata work in `W023`,
4. dynamic-array or lambda-family semantics.

## 7. Gate Criteria
This workset can only be reported `scope_complete` when:
1. all inventory members have reproducible native current-host classification evidence,
2. the packet states the provider/cube capability taxonomy honestly,
3. the boundary between catalog presence, capability denial, and external/runtime failure is explicit,
4. successor packets are opened for any function classes that prove to require dedicated provider or connection seams.

## 8. Initial Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - no packet-specific scenario manifest yet
   - no provider/cube family contract yet
   - external-connection and provider-capability taxonomy still unstarted

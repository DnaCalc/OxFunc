# Foundation Editor Prompts From OxFunc

Status: `active`
Source lane: `OxFunc`
Scope: `W2 Floating-Point Characterization follow-up`

## 1. Purpose
Provide copy-ready prompts for Foundation editors to absorb W2 outputs into Foundation evidence/conformance assets.

## 2. Prompt - Add W2 FP Evidence Candidates
Use this prompt in Foundation repo editorial workflow:

```text
Add a W2 floating-point evidence intake section to the appropriate Foundation evidence intake area.

Source repository:
- OxFunc

Source commit anchors:
- 0804ea3 (W2 tooling + FP-C xlcall/xlfRegister hardening)

Candidate evidence items to review and potentially promote to EMP IDs:
1. EMP-CAND-FP-001
   - formula divide-by-zero/invalid operations normalize to worksheet errors rather than exposing IEEE infinities/NaNs.
2. EMP-CAND-FP-002
   - formula/reference lanes normalize tested tiny/subnormal candidates to worksheet zero.
3. EMP-CAND-FP-003
   - interop UDF ingress normalizes +/-infinity and NaN variants to #NUM! at worksheet surface.
4. EMP-CAND-FP-004
   - interop UDF ingress can surface -0 as display -0 even where formula lanes display 0.

Required metadata from OxFunc run:
- Execution date: 2026-03-05
- Excel build: 16.0 (build 19725)
- Channel: http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60
- Compatibility: default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51
- Locale: en-US
- Lean toolchain: leanprover/lean4:v4.28.0

Primary OxFunc evidence artifacts:
- docs/function-lane/FLOATING_POINT_EXECUTION_RECORD.md
- docs/function-lane/FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md
- docs/function-lane/FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv
- .tmp/fp-results-excel-all.csv
- .tmp/fp-results-lean.csv

Intake requirement:
- assign Foundation EMP IDs or record explicit rejection/defer rationale per candidate.
```

## 3. Prompt - Update Foundation Conformance Cross-Links
```text
Update Foundation conformance/index docs to cross-link W2 floating-point characterization artifacts from OxFunc.

Add references for:
- W2 execution record
- W2 deviation ledger
- W2 probe runtime requirements

Goal:
- make floating-point normalization behavior traceable from Foundation-level requirements to OxFunc replay artifacts.
```

## 4. Prompt - Add Source-Binding Rows for W2
```text
In Foundation SOURCE_BINDINGS / evidence index:
1. Add rows for W2 baseline runs and linked artifacts from OxFunc.
2. Capture version axes explicitly (Excel build/channel + compatibility version).
3. Mark scope as provisional baseline (single build/channel, single locale) and retain explicit deferred expansion notes.
```

## 5. Notes
1. These prompts intentionally separate evidence intake from policy acceptance.
2. Deferred expansions (additional builds/channels, compatibility variants, locales) remain open and should be tagged as pending in Foundation trackers.

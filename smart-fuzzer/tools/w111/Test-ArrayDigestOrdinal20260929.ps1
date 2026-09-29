# Offline regression of the actual runner comparison functions. Does not open Excel.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
$tokens = $null; $parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $repo "smart-fuzzer/tools/Run-ArraySupportTranche.ps1"), [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw "runner parse errors: $parseErrors" }
foreach ($definition in $ast.FindAll({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] }, $false)) {
    . ([scriptblock]::Create($definition.Extent.Text))
}
Import-Module (Join-Path $repo "smart-fuzzer/tools/CellRefBatch.psm1") -Force
$RunId = "offline-digest-ordinal"
$outDir = Join-Path $repo (".tmp/digest-ordinal-" + [guid]::NewGuid().ToString("N"))
[void](New-Item -ItemType Directory -Path $outDir)
$failureDir = Join-Path $outDir "failures"
[void](New-Item -ItemType Directory -Path $failureDir)
$pairs = @(
    @("text:ABC", "text:abc", "unexpected_mismatch"),
    @("array:1x2:[text:ABC|text:x]", "array:1x2:[text:abc|text:x]", "unexpected_mismatch"),
    @("array:1x1:[text:ABC]", "text:abc", "unexpected_mismatch"),
    @("array:1x1:[text:abc]", "text:abc", "adapter_or_seam_mismatch"),
    @("text:ABC", "text:ABC", "exact_typed_bit_match"),
    @("text:abc", "text:abc", "exact_typed_bit_match")
)
$cases = @()
for ($i = 0; $i -lt $pairs.Count; $i++) {
    $case = [pscustomobject]@{case_id="ordinal-$i";function_id="FUNC.TEST";canonical_surface_name="TEST";case_tag="ordinal";axis="text";formula_text='="ABC"'}
    if ($i -ge 4) {
        $case | Add-Member axis_pair_id "case-sensitive-axis"
        $case | Add-Member axis_role $(if ($i -eq 4) {"control"} else {"variant"})
        $case | Add-Member axis_group "text"
        $case | Add-Member axis_tag "letter-case"
    }
    $cases += $case
    foreach ($side in @("local", "excel")) {
        $digest = $pairs[$i][$(if ($side -eq "local") {0} else {1})]
        Add-JsonLine (Join-Path $outDir "$side.jsonl") @{case_id=$case.case_id;execution_status="ok";outcome=@{kind="text";value=$digest;digest_payload=$digest}}
    }
}
$rollup = Compare-ArrayOutcomes $cases (Join-Path $outDir "local.jsonl") (Join-Path $outDir "excel.jsonl") (Join-Path $outDir "comparisons.jsonl") $failureDir
$actual = Read-JsonLinesByCase (Join-Path $outDir "comparisons.jsonl")
for ($i=0; $i -lt $pairs.Count; $i++) {
    if ($actual["ordinal-$i"].classification -ne $pairs[$i][2]) {throw "Wrong comparison classification for ordinal-$i"}
}
if ($rollup.axis_witness_pairs.differentiated_pairs -ne 1) {throw "Case-distinct axis witness not distinguished"}
Write-Output "Passed 7 offline runner digest/array/seam/axis checks."

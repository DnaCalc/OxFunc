[CmdletBinding(PositionalBinding = $false)]
param()

# In-process self-test for CellRefBatch.psm1 helpers
# (Test-FormulaTextIsBitExactSafe and Get-StandardSeverityClass).
# Excel COM is NOT exercised here — this is a pure-logic test that the
# severity vocabulary in CHARTER §4.1 / SMART_FUZZER_DESIGN §1.1 is
# faithfully implemented.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Import-Module (Join-Path $scriptRoot "CellRefBatch.psm1") -Force

$failures = New-Object 'System.Collections.Generic.List[string]'
$passes = 0

function Assert-Equal {
    param([string] $Label, $Expected, $Actual)
    $script:totalCases += 1
    if (-not ($Expected -is [string])) { $Expected = [string]$Expected }
    if (-not ($Actual -is [string])) { $Actual = [string]$Actual }
    if ($Expected -eq $Actual) {
        $script:passes += 1
        Write-Host "  PASS  $Label"
    } else {
        $script:failures.Add("$Label : expected '$Expected' got '$Actual'") | Out-Null
        Write-Host "  FAIL  $Label : expected '$Expected' got '$Actual'" -ForegroundColor Red
    }
}
$script:totalCases = 0

Write-Host "Test-FormulaTextIsBitExactSafe"
$safeCases = @(
    '=ABS(-3)',
    '=ABS(-2)',
    '=TRUNC(12.34)',
    '=TRUNC(12.34,1)',
    '=PMT(0.005,360,200000)',
    '=POWER(2,10)',
    '=A1+B1',
    '=IF(A1>0,"yes","no")',
    '=ABS(1.5e10)'
)
foreach ($f in $safeCases) {
    $r = Test-FormulaTextIsBitExactSafe -FormulaText $f
    Assert-Equal "safe : '$f'" $true $r.safe
}

$unsafeCases = @(
    '=ABS(-140920.05717469757655635)',
    '=TAN(797601.5817469757655635)',
    '=GAMMA(-1.00011965486703613)',
    '=POWER(2.7182818284590451,3)'
)
foreach ($f in $unsafeCases) {
    $r = Test-FormulaTextIsBitExactSafe -FormulaText $f
    Assert-Equal "unsafe : '$f'" $false $r.safe
}

Write-Host ""
Write-Host "Get-StandardSeverityClass"

function _Number-Outcome {
    param([double] $V)
    return [ordered]@{ kind = "number"; value = $V; bits_hex = (Get-F64BitsHex $V) }
}

# 1. Exact bit match.
$a = _Number-Outcome 1.0; $b = _Number-Outcome 1.0
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "1: exact bit match"  "match" $r.severity_class

# 2. Numeric drift 1 ULP (no integer signal).
$a = _Number-Outcome 0.1; $b = _Number-Outcome ([System.BitConverter]::Int64BitsToDouble([System.BitConverter]::DoubleToInt64Bits(0.1) + 1))
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "2: numeric_drift_1ulp"  "numeric_drift_1ulp" $r.severity_class
Assert-Equal "2: no excel_imprecision_witness (local not integer)" $false ($r.sub_tags -contains "excel_imprecision_witness")

# 3. Excel-imprecision witness: local is exact integer, Excel is 1 ULP off.
$a = _Number-Outcome 1144066.0
$b = _Number-Outcome ([System.BitConverter]::Int64BitsToDouble([System.BitConverter]::DoubleToInt64Bits(1144066.0) + 1))
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "3: excel_imprecision_witness severity is numeric_drift_1ulp (still a bug)"  "numeric_drift_1ulp" $r.severity_class
Assert-Equal "3: excel_imprecision_witness sub-tag present" $true ($r.sub_tags -contains "excel_imprecision_witness")

# 4. Numeric drift > 1 ULP.
$a = _Number-Outcome 1.0; $b = _Number-Outcome 1.0000001
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "4: numeric_drift_gt1ulp"  "numeric_drift_gt1ulp" $r.severity_class

# 5. Signed-zero collapse.
$a = _Number-Outcome 0.0; $b = _Number-Outcome (-0.0)
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "5: signed-zero collapses to match" "match" $r.severity_class

# 6. Kind drift = structural mismatch.
$a = _Number-Outcome 5.0
$b = [ordered]@{ kind = "error"; code = "Value" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "6: kind_drift -> structural_mismatch"  "structural_mismatch" $r.severity_class

# 7. Same error code = match.
$a = [ordered]@{ kind = "error"; code = "Num" }
$b = [ordered]@{ kind = "error"; code = "Num" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "7: same error -> match" "match" $r.severity_class

# 8. Different error codes = structural mismatch.
$a = [ordered]@{ kind = "error"; code = "Num" }
$b = [ordered]@{ kind = "error"; code = "Value" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "8: error_code_drift -> structural_mismatch" "structural_mismatch" $r.severity_class

# 9. Logical mismatch = structural.
$a = [ordered]@{ kind = "logical"; value = $true }
$b = [ordered]@{ kind = "logical"; value = $false }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "9: logical_value_drift -> structural_mismatch" "structural_mismatch" $r.severity_class

# 10. Missing local outcome.
$r = Get-StandardSeverityClass -LocalOutcome $null -ExcelOutcome (_Number-Outcome 1.0)
Assert-Equal "10: missing local -> harness_blocked_local" "harness_blocked_local" $r.severity_class

# 11. Missing excel outcome.
$r = Get-StandardSeverityClass -LocalOutcome (_Number-Outcome 1.0) -ExcelOutcome $null
Assert-Equal "11: missing excel -> harness_blocked_excel" "harness_blocked_excel" $r.severity_class

# 12. Identical digest_payload (compound kind, e.g. arrays) → match.
$a = [ordered]@{ kind = "array"; rows = 2; cols = 1; digest_payload = "array:2x1:[number:0x3ff0000000000000|number:0x4008000000000000]" }
$b = [ordered]@{ kind = "array"; rows = 2; cols = 1; digest_payload = "array:2x1:[number:0x3ff0000000000000|number:0x4008000000000000]" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "12: array same digest -> match" "match" $r.severity_class

# 13. Array shape drift = structural mismatch.
$a = [ordered]@{ kind = "array"; rows = 2; cols = 1; digest_payload = "array:2x1:[x|y]" }
$b = [ordered]@{ kind = "array"; rows = 1; cols = 2; digest_payload = "array:1x2:[x|y]" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "13: array shape drift -> structural_mismatch" "structural_mismatch" $r.severity_class
Assert-Equal "13: array shape drift sub-tag" $true ($r.sub_tags -contains "array_shape_drift")

# 14. Array element drift (same shape, different digest) -> structural_mismatch.
$a = [ordered]@{ kind = "array"; rows = 2; cols = 1; digest_payload = "array:2x1:[x|y]" }
$b = [ordered]@{ kind = "array"; rows = 2; cols = 1; digest_payload = "array:2x1:[x|z]" }
$r = Get-StandardSeverityClass -LocalOutcome $a -ExcelOutcome $b
Assert-Equal "14: array element drift -> structural_mismatch" "structural_mismatch" $r.severity_class
Assert-Equal "14: array element drift sub-tag" $true ($r.sub_tags -contains "array_element_drift")

Write-Host ""
Write-Host "Summary: $passes / $($script:totalCases) passed, $($failures.Count) failed."
if ($failures.Count -gt 0) {
    foreach ($f in $failures) { Write-Host "FAIL: $f" -ForegroundColor Red }
    exit 1
}
exit 0

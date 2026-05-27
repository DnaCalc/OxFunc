[CmdletBinding(PositionalBinding = $false)]
param()

# Integration-level test: feed two synthetic cases (one safe, one unsafe)
# directly to Run-ArraySupportTranche's Excel-evaluation function, bypassing
# the local Rust evaluator. Verifies that the bit-exact safety guard fires
# on the unsafe case and lets the safe case through.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path

# Source the runner's helpers without invoking its main flow. We do this
# by dot-sourcing only its function definitions inside a parser-aware
# scope: simplest is to import the CellRefBatch module (which has the
# safety check) and run the targeted assertion.
Import-Module (Join-Path $scriptRoot "CellRefBatch.psm1") -Force

$unsafe = Test-FormulaTextIsBitExactSafe -FormulaText "=TAN(797601.5817469757655635)"
$safe   = Test-FormulaTextIsBitExactSafe -FormulaText "=TAN(0.5)"

if ($unsafe.safe -ne $false) {
    Write-Host "FAIL: unsafe literal not detected" -ForegroundColor Red
    exit 1
}
if ($safe.safe -ne $true) {
    Write-Host "FAIL: safe literal flagged" -ForegroundColor Red
    exit 1
}

# Also verify the Run-ArraySupportTranche file *uses* the guard, so a
# future refactor that drops the call would be caught.
$runnerPath = Join-Path $scriptRoot "Run-ArraySupportTranche.ps1"
$runnerText = Get-Content -Raw -LiteralPath $runnerPath
if ($runnerText -notmatch 'Test-FormulaTextIsBitExactSafe') {
    Write-Host "FAIL: Run-ArraySupportTranche.ps1 does not call Test-FormulaTextIsBitExactSafe" -ForegroundColor Red
    exit 1
}
if ($runnerText -notmatch 'Get-StandardSeverityClass') {
    Write-Host "FAIL: Run-ArraySupportTranche.ps1 does not call Get-StandardSeverityClass" -ForegroundColor Red
    exit 1
}

Write-Host "PASS: unsafe-literal guard detected"
Write-Host "PASS: safe-literal passed through"
Write-Host "PASS: Run-ArraySupportTranche.ps1 uses Test-FormulaTextIsBitExactSafe + Get-StandardSeverityClass"
exit 0

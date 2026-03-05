param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out
)

$ErrorActionPreference = "Stop"

$manifestPath = Resolve-Path -Path $Manifest -ErrorAction Stop
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$repoRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "..\..")
$leanRoot = Join-Path $repoRoot "formal\lean"
if (-not (Test-Path (Join-Path $leanRoot "lakefile.lean"))) {
    throw "Lean root not found: $leanRoot"
}

$leanToolchain = ""
$toolchainPath = Join-Path $leanRoot "lean-toolchain"
if (Test-Path $toolchainPath) {
    $leanToolchain = (Get-Content $toolchainPath -TotalCount 1).Trim()
}
if ([string]::IsNullOrWhiteSpace($leanToolchain)) {
    $leanToolchain = "unknown"
}

$rows = Import-Csv -Path $manifestPath
$leanRows = $rows | Where-Object {
    $_.status -in @("seed", "ready") -and
    ([string]$_.model_targets).ToLowerInvariant().Contains("lean-runtime")
}

$result = New-Object System.Collections.Generic.List[object]

Push-Location $leanRoot
try {
    $laneByScenario = @{}
    $scenarioIds = New-Object System.Collections.Generic.List[string]
    foreach ($row in $leanRows) {
        $sid = [string]$row.scenario_id
        $laneByScenario[$sid] = [string]$row.lane
        $scenarioIds.Add($sid)
    }

    if ($scenarioIds.Count -gt 0) {
        $rawLines = & lake exe fp_scenario_eval @scenarioIds
        if ($LASTEXITCODE -ne 0) {
            throw "lake exe fp_scenario_eval returned exit code $LASTEXITCODE"
        }

        $seen = @{}
        foreach ($line in $rawLines) {
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            $parts = $line -split '\|', 7
            if ($parts.Count -lt 3) { continue }
            $kind = [string]$parts[0]
            $scenarioId = [string]$parts[1]
            $seen[$scenarioId] = $true
            $lane = if ($laneByScenario.ContainsKey($scenarioId)) { $laneByScenario[$scenarioId] } else { "FP-A" }

            if ($kind -eq "ok" -and $parts.Count -ge 7) {
                $observedClass = [string]$parts[2]
                $primaryText = [string]$parts[3]
                $primaryValue2 = [string]$parts[4]
                $formulaHint = [string]$parts[5]
                $evalNotes = [string]$parts[6]

                $result.Add([PSCustomObject]@{
                        scenario_id = $scenarioId
                        lane = $lane
                        mode = "lean-runtime"
                        execution_status = "observed"
                        observed_class = $observedClass
                        excel_version = "lean-toolchain:$leanToolchain"
                        excel_channel = ""
                        compat_version = "lean-runtime"
                        locale_profile = "en-US"
                        runner_version = "fp-lean-baseline-ps1/0.1.0"
                        artifact_ref = ""
                        primary_cell = "A1"
                        primary_formula2 = $formulaHint
                        primary_value2 = $primaryValue2
                        primary_text = $primaryText
                        observed_cells = "A1{text=$primaryText;value2=$primaryValue2;type=UInt64Bits;formula2=$formulaHint}"
                        comparison_bools = ""
                        notes = "lean_eval=$evalNotes"
                    })
            }
            else {
                $msg = if ($parts.Count -ge 3) { [string]$parts[2] } else { "unknown lean evaluator error" }
                $result.Add([PSCustomObject]@{
                        scenario_id = $scenarioId
                        lane = $lane
                        mode = "lean-runtime"
                        execution_status = "failed"
                        observed_class = "pending_observation"
                        excel_version = "lean-toolchain:$leanToolchain"
                        excel_channel = ""
                        compat_version = "lean-runtime"
                        locale_profile = "en-US"
                        runner_version = "fp-lean-baseline-ps1/0.1.0"
                        artifact_ref = ""
                        primary_cell = ""
                        primary_formula2 = ""
                        primary_value2 = ""
                        primary_text = ""
                        observed_cells = ""
                        comparison_bools = ""
                        notes = "scenario failure: $msg"
                    })
            }
        }

        foreach ($scenarioId in $scenarioIds) {
            if (-not $seen.ContainsKey($scenarioId)) {
                $lane = if ($laneByScenario.ContainsKey($scenarioId)) { $laneByScenario[$scenarioId] } else { "FP-A" }
                $result.Add([PSCustomObject]@{
                        scenario_id = $scenarioId
                        lane = $lane
                        mode = "lean-runtime"
                        execution_status = "failed"
                        observed_class = "pending_observation"
                        excel_version = "lean-toolchain:$leanToolchain"
                        excel_channel = ""
                        compat_version = "lean-runtime"
                        locale_profile = "en-US"
                        runner_version = "fp-lean-baseline-ps1/0.1.0"
                        artifact_ref = ""
                        primary_cell = ""
                        primary_formula2 = ""
                        primary_value2 = ""
                        primary_text = ""
                        observed_cells = ""
                        comparison_bools = ""
                        notes = "scenario failure: no output line from lean evaluator"
                    })
            }
        }
    }
}
catch {
    foreach ($row in $leanRows) {
        $scenarioId = [string]$row.scenario_id
        if ($result | Where-Object { $_.scenario_id -eq $scenarioId }) { continue }
        $result.Add([PSCustomObject]@{
                scenario_id = $scenarioId
                lane = $row.lane
                mode = "lean-runtime"
                execution_status = "failed"
                observed_class = "pending_observation"
                excel_version = "lean-toolchain:$leanToolchain"
                excel_channel = ""
                compat_version = "lean-runtime"
                locale_profile = "en-US"
                runner_version = "fp-lean-baseline-ps1/0.1.0"
                artifact_ref = ""
                primary_cell = ""
                primary_formula2 = ""
                primary_value2 = ""
                primary_text = ""
                observed_cells = ""
                comparison_bools = ""
                notes = "scenario failure: $($_.Exception.Message)"
            })
    }
}
finally {
    Pop-Location
}

$result | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
Write-Host "Lean baseline run complete. Rows written: $($result.Count)"
Write-Host "Output: $outPath"

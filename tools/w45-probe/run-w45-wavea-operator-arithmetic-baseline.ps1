param(
    [string]$Manifest = "docs/function-lane/W45_WAVEA_OPERATOR_ARITHMETIC_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w45-wavea-operator-arithmetic-results.csv"
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

function Get-OperatorObservation {
    param($Cell)

    $hasSpill = $false
    try { [void]($hasSpill = [bool]$Cell.HasSpill()) } catch {}
    $range = if ($hasSpill) { $Cell.SpillingToRange } else { $Cell }
    [void]($spillCells = @($range.Cells))
    [void]($spillText = foreach ($spillCell in $spillCells) { [string]$spillCell.Text })
    [void]($spillRows = @($spillCells | ForEach-Object { [int]$_.Row } | Sort-Object -Unique).Count)
    [void]($spillCols = @($spillCells | ForEach-Object { [int]$_.Column } | Sort-Object -Unique).Count)

    [pscustomobject]@{
        anchor_text = [string]$Cell.Text
        anchor_value2 = $Cell.Value2
        has_spill = $hasSpill
        spill_rows = $spillRows
        spill_cols = $spillCols
        spill_text = ($spillText -join "|")
    }
}

function Test-Expectation {
    param($Row, $Observation)

    if ($Row.expected_error) {
        return $Observation.anchor_text -eq $Row.expected_error
    }

    if ($Row.expected_spill_text -or $Row.expected_spill_rows -or $Row.expected_spill_cols) {
        [void]($textMatch = $Observation.spill_text -eq $Row.expected_spill_text)
        [void]($rowMatch = ($Row.expected_spill_rows -eq "") -or ($Observation.spill_rows -eq [int]$Row.expected_spill_rows))
        [void]($colMatch = ($Row.expected_spill_cols -eq "") -or ($Observation.spill_cols -eq [int]$Row.expected_spill_cols))
        return $textMatch -and $rowMatch -and $colMatch
    }

    if ($Row.expected_text) {
        return $Observation.anchor_text -eq $Row.expected_text
    }

    if ($Row.expected_value2 -ne "") {
        $expected = [double]$Row.expected_value2
        $tol = if ($Row.tolerance -eq "") { 0.0 } else { [double]$Row.tolerance }
        $actual = [double]$Observation.anchor_value2
        return [math]::Abs($actual - $expected) -le $tol
    }

    return $false
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

try {
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    $ws.Cells.NumberFormat = "General"

    $results = foreach ($row in $rows) {
        [void]($ws.Cells.Clear())
        [void]($cell = $ws.Range("D1"))
        [void]($cell.Formula2 = "=" + $row.formula)
        Start-Sleep -Milliseconds 50
        [void]($observation = Get-OperatorObservation -Cell $cell)

        [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            formula = "=" + $row.formula
            text = $observation.anchor_text
            value2 = $observation.anchor_value2
            has_spill = $observation.has_spill
            spill_rows = $observation.spill_rows
            spill_cols = $observation.spill_cols
            spill_text = $observation.spill_text
            expected_text = $row.expected_text
            expected_value2 = $row.expected_value2
            expected_error = $row.expected_error
            expected_spill_text = $row.expected_spill_text
            expected_spill_rows = $row.expected_spill_rows
            expected_spill_cols = $row.expected_spill_cols
            tolerance = $row.tolerance
            matches_expected = Test-Expectation -Row $row -Observation $observation
            notes = $row.notes
        }
    }

    $outDir = Split-Path $outPath -Parent
    if (-not (Test-Path $outDir)) {
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    }
    $results | Export-Csv -NoTypeInformation -Path $outPath
}
finally {
    if ($wb) { $wb.Close($false) }
    $excel.Quit()
    if ($ws) { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null }
    if ($wb) { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null }
    if ($excel) { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null }
}

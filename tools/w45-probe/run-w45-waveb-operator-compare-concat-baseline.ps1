param(
    [string]$OutPath = ".tmp/w45-waveb-operator-compare-concat-results.csv"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\")
Set-Location $repoRoot

$manifest = Import-Csv "docs/function-lane/W45_WAVEB_OPERATOR_COMPARE_CONCAT_SCENARIO_MANIFEST_SEED.csv"
New-Item -ItemType Directory -Force -Path (Split-Path $OutPath) | Out-Null

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

    return $Observation.anchor_text -eq $Row.expected_text
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

try {
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    [void]($ws.Range("A1").Clear())

    $rows = foreach ($row in $manifest) {
        [void]($ws.Cells.Clear())
        [void]($cell = $ws.Range("D1"))
        [void]($cell.Formula2 = $row.formula)
        Start-Sleep -Milliseconds 50
        [void]($observation = Get-OperatorObservation -Cell $cell)
        [pscustomobject]@{
            scenario_id = $row.scenario_id
            wave = $row.wave
            formula = $row.formula
            expected_text = $row.expected_text
            expected_error = $row.expected_error
            expected_spill_text = $row.expected_spill_text
            expected_spill_rows = $row.expected_spill_rows
            expected_spill_cols = $row.expected_spill_cols
            actual_text = $observation.anchor_text
            has_spill = $observation.has_spill
            spill_rows = $observation.spill_rows
            spill_cols = $observation.spill_cols
            spill_text = $observation.spill_text
            match = Test-Expectation -Row $row -Observation $observation
        }
    }

    $rows | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $OutPath
    $rows
}
finally {
    if ($wb) { $wb.Close($false) }
    $excel.Quit()
    if ($ws) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) }
    if ($wb) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) }
    if ($excel) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) }
}

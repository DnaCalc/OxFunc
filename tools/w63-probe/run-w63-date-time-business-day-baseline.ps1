param(
    [string]$Manifest = "docs/function-lane/W63_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w63-date-time-business-day-results.csv"
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$manifestPath = Join-Path $repoRoot $Manifest
$outPath = Join-Path $repoRoot $Out
$outDir = Split-Path -Parent $outPath
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$rows = Import-Csv $manifestPath
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$excel.ScreenUpdating = $false
$excel.EnableEvents = $false
$wb = $null
$ws = $null

try {
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    $ws.Cells.NumberFormat = "General"
    $ws.Range("H1").Formula2 = "=DATE(2024,1,1)"
    $ws.Range("H2").Formula2 = "=DATE(2006,1,2)"
    $ws.Range("H3").Formula2 = "=DATE(2006,1,16)"
    $excel.CalculateFull()

    $excelExe = Join-Path $excel.Path "EXCEL.EXE"
    $productVersion = (Get-Item $excelExe).VersionInfo.ProductVersion
    $excelVersion = [string]$excel.Version

    $results = @()
    $rowIndex = 1
    foreach ($row in $rows) {
        $probeCell = $ws.Cells.Item($rowIndex, 6)
        $probeCell.Formula2 = "=" + [string]$row.formula
        $excel.CalculateFull()

        $text = [string]$probeCell.Text
        $value2 = if ($null -eq $probeCell.Value2) { "" } else { [string]$probeCell.Value2 }
        $expectedText = [string]$row.expected_text
        $observedForMatch = if ($expectedText.StartsWith("#")) { $text } else { $value2 }
        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            formula = "=" + [string]$row.formula
            text = $text
            value2 = $value2
            expected_text = $expectedText
            matches_expected = ($observedForMatch -ceq $expectedText)
            excel_version = $excelVersion
            excel_product_version = $productVersion
            notes = $row.notes
        }
        $rowIndex++
    }

    $results | ConvertTo-Csv -NoTypeInformation | Set-Content -Path $outPath -Encoding UTF8
    Get-Content $outPath
}
finally {
    if ($wb -ne $null) {
        $wb.Close($false)
        Release-ComObjectSafe -Obj $ws
        Release-ComObjectSafe -Obj $wb
    }
    if ($excel -ne $null) {
        $excel.Quit()
        Release-ComObjectSafe -Obj $excel
    }
}

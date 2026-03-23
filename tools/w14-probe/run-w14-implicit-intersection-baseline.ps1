param(
    [string]$Manifest = "docs/function-lane/W14_IMPLICIT_INTERSECTION_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w14-implicit-intersection-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

function Convert-ExcelValueToText($value) {
    if ($null -eq $value) { return "" }
    if ($value -is [int]) {
        switch ([int]$value) {
            -2146826246 { return "#N/A" }
            -2146826273 { return "#VALUE!" }
            -2146826281 { return "#DIV/0!" }
            -2146826259 { return "#NAME?" }
            -2146826265 { return "#NUM!" }
            -2146826288 { return "#NULL!" }
            -2146826266 { return "#REF!" }
            -2146826287 { return "#GETTING_DATA" }
        }
    }
    return [string]$value
}

function Reset-Sheet($sheet) {
    $sheet.Cells.Clear() | Out-Null
    $sheet.Cells.NumberFormat = "General"
}

function Set-Assignments($sheet, [string]$assignments) {
    if ([string]::IsNullOrWhiteSpace($assignments)) {
        return
    }
    foreach ($assignment in ($assignments -split '\|')) {
        if ([string]::IsNullOrWhiteSpace($assignment)) { continue }
        $parts = $assignment -split ':=', 2
        $target = $parts[0].Trim()
        $expr = $parts[1].Trim()
        $cell = $sheet.Range($target)
        if ($expr.StartsWith("=") -or $expr.Contains("(") -or $expr.Contains("#") -or $expr.StartsWith("@")) {
            $cell.Formula2 = $expr
            if (-not $expr.StartsWith("=")) {
                $cell.Formula2 = "=" + $expr
            }
        } elseif ($expr -match '^[+-]?\d+(\.\d+)?$') {
            $cell.Value2 = [double]$expr
        } else {
            $cell.Value2 = $expr
        }
    }
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)
$ws.Name = "w14_probe"

$results = @()
foreach ($row in $rows) {
    Reset-Sheet $ws
    Set-Assignments $ws $row.value_setup

    $formulaCellRef = ($row.formula_setup -split ':=', 2)[0].Trim()
    $formulaText = ($row.formula_setup -split ':=', 2)[1].Trim()
    $formulaCell = $ws.Range($formulaCellRef)
    $formulaCell.Formula2 = "=" + $formulaText
    $excel.CalculateFull()

    $value2 = $formulaCell.Value2
    $text = Convert-ExcelValueToText $value2
    $storedFormula = [string]$formulaCell.Formula
    $storedFormula2 = [string]$formulaCell.Formula2

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        status = $row.status
        formula = "=" + $formulaText
        value_text = $text
        stored_formula = $storedFormula
        stored_formula2 = $storedFormula2
        expected = $row.expected_observable
        notes = $row.notes
    }
}

$outDir = Split-Path $outPath -Parent
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$results | Export-Csv -NoTypeInformation -Path $outPath

$wb.Close($false)
$excel.Quit()
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null

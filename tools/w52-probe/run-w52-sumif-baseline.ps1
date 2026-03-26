param(
    [string]$Manifest = "docs/function-lane/W52_SUMIF_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w52-sumif-results.csv"
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

function Convert-ManifestScalar {
    param([string]$Expr)
    if ($Expr -match '^".*"$') {
        return $Expr.Substring(1, $Expr.Length - 2).Replace('""', '"')
    }
    if ($Expr -eq "TRUE") { return $true }
    if ($Expr -eq "FALSE") { return $false }
    $n = 0.0
    if ([double]::TryParse($Expr, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$n)) {
        return $n
    }
    return $Expr
}

function Parse-Assignments {
    param([string]$Raw)
    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }
    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, '^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$')
        if (-not $m.Success) {
            throw "Invalid setup assignment '$trimmed'."
        }
        $items += [pscustomobject]@{
            Cell = $m.Groups[1].Value
            Expr = $m.Groups[2].Value.Trim()
        }
    }
    return $items
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
    $results = @()

    foreach ($row in $rows) {
        $ws.Cells.Clear() | Out-Null
        foreach ($assignment in (Parse-Assignments -Raw ([string]$row.setup_values))) {
            $target = $ws.Range([string]$assignment.Cell)
            $expr = [string]$assignment.Expr
            if ($expr.StartsWith("FORMULA(") -and $expr.EndsWith(")")) {
                $target.Formula2 = $expr.Substring(8, $expr.Length - 9)
                continue
            }
            $scalar = Convert-ManifestScalar -Expr $expr
            if ($scalar -is [string]) {
                $target.Value = $scalar
            } else {
                $target.Value2 = $scalar
            }
        }

        $probeCell = $ws.Range("F1")
        $probeCell.Formula2 = "=" + [string]$row.formula
        $excel.CalculateFull()

        $text = [string]$probeCell.Text
        $value2 = if ($null -eq $probeCell.Value2) { "" } else { [string]$probeCell.Value2 }
        $expectedText = [string]$row.expected_text
        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            formula = "=" + [string]$row.formula
            text = $text
            value2 = $value2
            expected_text = $expectedText
            matches_expected = ($text -ceq $expectedText)
            notes = $row.notes
        }
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

param(
    [string]$Manifest = "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w15-cell-host-pre-results.csv",
    [string]$WorkbookTemplate = "",
    [string]$RunLabel = "default"
)

$ErrorActionPreference = "Stop"

function Parse-Assignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, '^\s*((?:[^!:=|]+!)?[A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$')
        if (-not $m.Success) {
            throw "Invalid setup assignment '$trimmed' (expected CELL:=VALUE_OR_FORMULA)."
        }
        $items += [pscustomobject]@{
            Cell = $m.Groups[1].Value
            Expr = $m.Groups[2].Value.Trim()
        }
    }
    return $items
}

function Parse-FormatAssignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, '^\s*((?:[^!:=|]+!)?[A-Z]{1,3}[0-9]+)\.(\w+)\s*:=\s*(.+)$')
        if (-not $m.Success) {
            throw "Invalid format assignment '$trimmed' (expected CELL.Property:=VALUE)."
        }
        $items += [pscustomobject]@{
            Cell = $m.Groups[1].Value
            Property = $m.Groups[2].Value
            Expr = $m.Groups[3].Value.Trim()
        }
    }
    return $items
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

function Get-OrCreateWorksheet {
    param(
        [object]$Workbook,
        [string]$SheetName
    )

    foreach ($sheet in @($Workbook.Worksheets)) {
        if ([string]$sheet.Name -eq $SheetName) {
            return $sheet
        }
    }

    $sheet = $Workbook.Worksheets.Add()
    $sheet.Name = $SheetName
    return $sheet
}

function Resolve-RangeRef {
    param(
        [object]$Workbook,
        [object]$DefaultWorksheet,
        [string]$RefText
    )

    if ($RefText -match '^(?<sheet>[^!]+)!(?<cell>[A-Z]{1,3}[0-9]+)$') {
        $sheet = Get-OrCreateWorksheet -Workbook $Workbook -SheetName $Matches.sheet
        return $sheet.Range($Matches.cell)
    }

    return $DefaultWorksheet.Range($RefText)
}

function Apply-ValueAssignment {
    param(
        [object]$Workbook,
        [object]$Worksheet,
        [object]$Assignment
    )

    $target = Resolve-RangeRef -Workbook $Workbook -DefaultWorksheet $Worksheet -RefText ([string]$Assignment.Cell)
    $expr = [string]$Assignment.Expr
    if ($expr.StartsWith("=")) {
        $target.Formula2 = $expr
        return
    }
    $scalar = Convert-ManifestScalar -Expr $expr
    if ($scalar -is [string]) {
        $target.Value = $scalar
        return
    }
    $target.Value2 = $scalar
}

function Apply-FormatAssignment {
    param(
        [object]$Workbook,
        [object]$Worksheet,
        [object]$Assignment
    )

    $target = Resolve-RangeRef -Workbook $Workbook -DefaultWorksheet $Worksheet -RefText ([string]$Assignment.Cell)
    $rawExpr = [string]$Assignment.Expr
    $value = Convert-ManifestScalar -Expr $rawExpr

    switch ($Assignment.Property) {
        "NumberFormat" {
            if ($rawExpr -match '^".*"$') {
                $target.NumberFormat = [string]$value
            } else {
                $target.NumberFormat = $rawExpr
            }
            return
        }
        "HorizontalAlignment" { $target.HorizontalAlignment = [int]$value; return }
        "Locked" { $target.Locked = [bool]$value; return }
        "ColumnWidth" { $target.ColumnWidth = [double]$value; return }
        default { throw "Unsupported format property '$($Assignment.Property)'." }
    }
}

function Get-WorkbookDescriptor {
    param([object]$Workbook)

    try { $calcVersion = [string]$Workbook.CalculationVersion } catch { $calcVersion = "" }
    try { $checkCompatibility = [string]$Workbook.CheckCompatibility } catch { $checkCompatibility = "" }
    try { $fileFormat = [string]$Workbook.FileFormat } catch { $fileFormat = "" }
    return "default|CalculationVersion=$calcVersion|CheckCompatibility=$checkCompatibility|FileFormat=$fileFormat"
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\\..")
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

try {
    if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
        $templatePath = if ([System.IO.Path]::IsPathRooted($WorkbookTemplate)) {
            $WorkbookTemplate
        } else {
            Join-Path $repoRoot $WorkbookTemplate
        }
        $templatePath = [System.IO.Path]::GetFullPath($templatePath)
        $saveDir = Join-Path $env:TEMP "oxfunc-w15-cell"
        if (-not (Test-Path $saveDir)) {
            New-Item -ItemType Directory -Path $saveDir | Out-Null
        }
        $savePath = Join-Path $saveDir ($RunLabel + "-cell-host-probe" + [System.IO.Path]::GetExtension($templatePath))
        Copy-Item $templatePath $savePath -Force
        $wb = $excel.Workbooks.Open($savePath)
    } else {
        $wb = $excel.Workbooks.Add()
    }
    $ws = $wb.Worksheets.Item(1)
    if ([string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
        $saveDir = Join-Path $env:TEMP "oxfunc-w15-cell"
        if (-not (Test-Path $saveDir)) {
            New-Item -ItemType Directory -Path $saveDir | Out-Null
        }
        $savePath = Join-Path $saveDir "cell-host-probe.xlsx"
        $wb.SaveAs($savePath, 51)
    }
    $compatDescriptor = Get-WorkbookDescriptor -Workbook $wb

    $results = @()
    foreach ($row in $rows) {
        foreach ($sheet in @($wb.Worksheets)) {
            $sheet.Cells.Clear() | Out-Null
        }
        foreach ($assignment in (Parse-Assignments -Raw ([string]$row.setup_values))) {
            Apply-ValueAssignment -Workbook $wb -Worksheet $ws -Assignment $assignment
        }
        foreach ($assignment in (Parse-FormatAssignments -Raw ([string]$row.setup_formats))) {
            Apply-FormatAssignment -Workbook $wb -Worksheet $ws -Assignment $assignment
        }

        if (-not [string]::IsNullOrWhiteSpace([string]$row.active_selection)) {
            (Resolve-RangeRef -Workbook $wb -DefaultWorksheet $ws -RefText ([string]$row.active_selection)).Select() | Out-Null
        }

        $target = $ws.Range("A1")
        $target.Formula = "=" + [string]$row.formula
        $ws.Calculate()

        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            run_label = $RunLabel
            compat_descriptor = $compatDescriptor
            formula = $target.Formula
            text = $target.Text
            value2 = $target.Value2
            expected_status = $row.expected_status
            expected_observable = $row.expected_observable
        }
    }

    $results | ConvertTo-Csv -NoTypeInformation | Set-Content -Path $outPath -Encoding UTF8
    Get-Content $outPath
}
finally {
    if ($wb -ne $null) {
        $wb.Close($true)
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) | Out-Null
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) | Out-Null
    }
    $excel.Quit()
    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) | Out-Null
}

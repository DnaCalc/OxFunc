param(
    [string]$Manifest = "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w15-xll-bridge-results.csv",
    [string]$XllPath = "",
    [switch]$BuildIfMissing,
    [string]$WorkbookTemplate = "",
    [string]$RunLabel = "default"
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

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

function Convert-ObservableValue {
    param([object]$Value)

    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    return [string]$Value
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

if ([string]::IsNullOrWhiteSpace($XllPath)) {
    $XllPath = Join-Path $repoRoot "tools\\xll-addin\\bin\\OxFunc64.xll"
}
$xllPathFull = [System.IO.Path]::GetFullPath($XllPath)

if (-not (Test-Path $xllPathFull)) {
    if ($BuildIfMissing) {
        & (Join-Path $repoRoot "tools\\xll-addin\\build-oxfunc-xll.ps1") -Profile release
    }
    if (-not (Test-Path $xllPathFull)) {
        throw "XLL not found: $xllPathFull"
    }
}

$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$excel.ScreenUpdating = $false
$excel.EnableEvents = $false

try {
    if (-not $excel.RegisterXLL($xllPathFull)) {
        throw "RegisterXLL returned false for: $xllPathFull"
    }

    if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
        $templatePath = if ([System.IO.Path]::IsPathRooted($WorkbookTemplate)) {
            $WorkbookTemplate
        } else {
            Join-Path $repoRoot $WorkbookTemplate
        }
        $templatePath = [System.IO.Path]::GetFullPath($templatePath)
        $saveDir = Join-Path $env:TEMP "oxfunc-w15-xll"
        if (-not (Test-Path $saveDir)) {
            New-Item -ItemType Directory -Path $saveDir | Out-Null
        }
        $savePath = Join-Path $saveDir ($RunLabel + "-w15-xll-bridge" + [System.IO.Path]::GetExtension($templatePath))
        Copy-Item $templatePath $savePath -Force
        $wb = $excel.Workbooks.Open($savePath)
    } else {
        $wb = $excel.Workbooks.Add()
    }
    $ws = $wb.Worksheets.Item(1)
    if ([string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
        $saveDir = Join-Path $env:TEMP "oxfunc-w15-xll"
        if (-not (Test-Path $saveDir)) {
            New-Item -ItemType Directory -Path $saveDir | Out-Null
        }
        $savePath = Join-Path $saveDir "w15-xll-bridge.xlsx"
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

        $bridgeCell = $ws.Range("A1")
        $nativeCell = $ws.Range("B1")

        $bridgeCell.Formula = "=" + [string]$row.bridge_formula
        $nativeCell.Formula = "=" + [string]$row.native_formula
        $excel.CalculateFull()

        $bridgeText = [string]$bridgeCell.Text
        $bridgeValue2 = Convert-ObservableValue -Value $bridgeCell.Value2
        $nativeText = [string]$nativeCell.Text
        $nativeValue2 = Convert-ObservableValue -Value $nativeCell.Value2
        $observedEqual = ($bridgeText -ceq $nativeText) -or ($bridgeValue2 -ceq $nativeValue2)

        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            run_label = $RunLabel
            compat_descriptor = $compatDescriptor
            bridge_formula = $bridgeCell.Formula
            native_formula = $nativeCell.Formula
            bridge_text = $bridgeText
            bridge_value2 = $bridgeValue2
            native_text = $nativeText
            native_value2 = $nativeValue2
            observed_equal = $observedEqual
            relation_status = if ($observedEqual) { "matched" } else { "mismatched" }
            expected_relation = $row.expected_relation
            notes = $row.notes
        }
    }

    $results | ConvertTo-Csv -NoTypeInformation | Set-Content -Path $outPath -Encoding UTF8
    Get-Content $outPath
}
finally {
    if ($wb -ne $null) {
        $wb.Close($true)
        Release-ComObjectSafe -Obj $ws
        Release-ComObjectSafe -Obj $wb
    }
    $excel.Quit()
    Release-ComObjectSafe -Obj $excel
}

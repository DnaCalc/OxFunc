param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string]$XllPath = "",

    [switch]$BuildIfMissing
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

function Close-WorkbookSafe {
    param([object]$Workbook)
    if ($null -ne $Workbook) {
        try { $Workbook.Close($false) | Out-Null } catch {}
    }
}

function Convert-ScalarToString {
    param([object]$Value)
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    if ($Value -is [Array]) { return "[array]" }
    return [string]$Value
}

function Parse-Assignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid setup assignment '$trimmed' (expected CELL:=VALUE_OR_FORMULA)."
        }
        $items += [PSCustomObject]@{
            Cell = $m.Groups[1].Value
            Expr = $m.Groups[2].Value.Trim()
        }
    }
    return $items
}

function Apply-Assignment {
    param(
        [object]$Worksheet,
        [object]$Assignment
    )

    $expr = [string]$Assignment.Expr
    if ($expr.StartsWith("=")) {
        $Worksheet.Range($Assignment.Cell).Formula2 = $expr
        return
    }
    if ($expr -match '^".*"$') {
        $Worksheet.Range($Assignment.Cell).Value = $expr.Substring(1, $expr.Length - 2).Replace('""', '"')
        return
    }
    if ($expr -eq "TRUE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $true
        return
    }
    if ($expr -eq "FALSE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $false
        return
    }

    $n = 0.0
    if ([double]::TryParse($expr, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$n)) {
        $Worksheet.Range($Assignment.Cell).Value2 = $n
        return
    }

    $Worksheet.Range($Assignment.Cell).Value = $expr
}

function Get-SheetMacroPrefix {
    param([object]$Worksheet)
    $name = [string]$Worksheet.Name
    if ($name -match "[\s']") {
        return "'" + $name.Replace("'", "''") + "'"
    }
    return $name
}

function Invoke-NativeMacro {
    param(
        [object]$Excel,
        [object]$Worksheet,
        [string]$MacroText
    )

    $macro = $MacroText.Trim()
    $getCellMatch = [regex]::Match($macro, '^GET\.CELL\((\d+)\s*,\s*([A-Z]{1,3}[0-9]+)\)$')
    if ($getCellMatch.Success) {
        $typeNum = $getCellMatch.Groups[1].Value
        $cellRef = $getCellMatch.Groups[2].Value
        $target = $Worksheet.Range($cellRef)
        $sheetPrefix = Get-SheetMacroPrefix -Worksheet $Worksheet
        $r1c1Ref = "$sheetPrefix!R$($target.Row)C$($target.Column)"
        return $Excel.ExecuteExcel4Macro("GET.CELL($typeNum,$r1c1Ref)")
    }

    return $Excel.ExecuteExcel4Macro($macro)
}

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

if ([string]::IsNullOrWhiteSpace($XllPath)) {
    $XllPath = Join-Path $PSScriptRoot "bin\OxFunc64.xll"
}
$xllPathFull = [System.IO.Path]::GetFullPath($XllPath)

if (-not (Test-Path $xllPathFull)) {
    if ($BuildIfMissing) {
        & (Join-Path $PSScriptRoot "build-oxfunc-xll.ps1") -Profile release
    }
    if (-not (Test-Path $xllPathFull)) {
        throw "XLL not found: $xllPathFull"
    }
}

$scenarios = Import-Csv -Path $manifestPath | Where-Object { $_.status -in @("ready", "seed") }
if (-not $scenarios -or $scenarios.Count -eq 0) {
    throw "Manifest has no runnable scenario rows: $manifestPath"
}

$excel = $null
$workbook = $null
$worksheet = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    if (-not $excel.RegisterXLL($xllPathFull)) {
        throw "RegisterXLL returned false for: $xllPathFull"
    }

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)
    $worksheet.Activate() | Out-Null

    $results = New-Object System.Collections.Generic.List[object]

    foreach ($scenario in $scenarios) {
        $worksheet.Cells.Clear() | Out-Null
        foreach ($assignment in (Parse-Assignments -Raw ([string]$scenario.setup_values))) {
            Apply-Assignment -Worksheet $worksheet -Assignment $assignment
        }

        $bridgeText = ""
        $bridgeValue = ""
        $nativeText = ""
        $executionStatus = "observed"
        $notes = [string]$scenario.notes

        try {
            $worksheet.Range("A1").Formula2 = "=" + [string]$scenario.bridge_formula
            $excel.CalculateFull()
            $bridgeRange = $worksheet.Range("A1")
            $bridgeText = [string]$bridgeRange.Text
            $bridgeValue = Convert-ScalarToString -Value $bridgeRange.Value2

            $nativeRaw = Invoke-NativeMacro -Excel $excel -Worksheet $worksheet -MacroText ([string]$scenario.native_macro)
            $nativeText = Convert-ScalarToString -Value $nativeRaw
        }
        catch {
            $executionStatus = "failed"
            $notes = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
        }

        $observedEqual = ($bridgeValue -ceq $nativeText) -or ($bridgeText -ceq $nativeText)
        $relationStatus = if ($executionStatus -eq "observed" -and $observedEqual) { "matched" } elseif ($executionStatus -eq "observed") { "mismatched" } else { "failed" }

        $results.Add([PSCustomObject]@{
            scenario_id = [string]$scenario.scenario_id
            lane = [string]$scenario.lane
            execution_status = $executionStatus
            bridge_formula = [string]$scenario.bridge_formula
            native_macro = [string]$scenario.native_macro
            bridge_text = $bridgeText
            bridge_value2 = $bridgeValue
            native_macro_value = $nativeText
            observed_equal = $observedEqual
            relation_status = $relationStatus
            notes = $notes
        })
    }

    $results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "XLL GET-info probe complete."
    Write-Host "Output: $outPath"
}
finally {
    Close-WorkbookSafe -Workbook $workbook
    Release-ComObjectSafe -Obj $worksheet
    Release-ComObjectSafe -Obj $workbook
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

param(
    [string]$Manifest = "docs/function-lane/W65_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w65-database-results.csv"
)

$ErrorActionPreference = "Stop"

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

function Set-Grid {
    param(
        [object]$Worksheet,
        [int]$StartRow,
        [int]$StartCol,
        [object[]]$Grid
    )

    for ($r = 0; $r -lt $Grid.Count; $r++) {
        for ($c = 0; $c -lt $Grid[$r].Count; $c++) {
            $value = $Grid[$r][$c]
            $cell = $Worksheet.Cells.Item($StartRow + $r, $StartCol + $c)
            if ($null -eq $value) {
                $cell.ClearContents()
            } elseif ($value -is [int] -or $value -is [long] -or $value -is [double] -or $value -is [decimal]) {
                $cell.Value2 = [double]$value
            } elseif ($value -is [string] -and $value.Length -gt 0 -and "=<>".Contains($value.Substring(0,1))) {
                $cell.Formula = "'" + $value
            } else {
                $cell.Value2 = [string]$value
            }
        }
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

    $seedRows = @(
        [object[]]@("Type","Salesperson","Sales","Units"),
        [object[]]@("Meat","Davolio",450,8),
        [object[]]@("Produce","Buchanan",6328,10),
        [object[]]@("Produce","Davolio",6544,$null),
        [object[]]@("Dairy","David",834,7),
        [object[]]@("Produce","Davis",3000,5)
    )
    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 1 -Grid $seedRows

    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 6 -Grid @([object[]]@("Salesperson"), [object[]]@("Dav"))
    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 8 -Grid @([object[]]@("Type"), [object[]]@("=Produce"))
    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 10 -Grid @([object[]]@("Salesperson"), [object[]]@("=Buchanan"))
    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 12 -Grid @([object[]]@("Salesperson"), [object[]]@("=Nobody"))
    Set-Grid -Worksheet $ws -StartRow 1 -StartCol 14 -Grid @(
        [object[]]@("Sales","Sales"),
        [object[]]@(">6000","<6500"),
        [object[]]@("<500",$null)
    )

    $excel.CalculateFull()
    $excelExe = Join-Path $excel.Path "EXCEL.EXE"
    $productVersion = (Get-Item $excelExe).VersionInfo.ProductVersion
    $excelVersion = [string]$excel.Version

    $results = @()
    $rowIndex = 1
    foreach ($row in $rows) {
        $probeCell = $ws.Cells.Item($rowIndex, 20)
        $probeCell.NumberFormat = "0.000000000000000"
        $probeCell.Formula2 = "=" + [string]$row.formula
        $excel.CalculateFull()

        $observed = ""
        $matches = $false
        if ($row.expected_kind -eq "number") {
            $value = [double]$probeCell.Value2
            $observed = [string]$value
            $matches = ([math]::Abs($value - [double]$row.expected_value) -le [double]$row.tolerance)
        } else {
            $observed = [string]$probeCell.Text
            $matches = ($observed -ceq [string]$row.expected_value)
        }

        $results += [pscustomobject]@{
            scenario_id = $row.scenario_id
            lane = $row.lane
            formula = "=" + [string]$row.formula
            expected_kind = $row.expected_kind
            observed = $observed
            expected_value = [string]$row.expected_value
            tolerance = [string]$row.tolerance
            matches_expected = $matches
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

param(
    [string]$Manifest = "docs/function-lane/W23_DATABASE_SCENARIO_MANIFEST_SEED.csv",
    [string]$Out = ".tmp/w23-database-results.csv"
)

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$manifestPath = Join-Path $root $Manifest
$outPath = Join-Path $root $Out
$rows = Import-Csv $manifestPath

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
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
for ($r = 0; $r -lt $seedRows.Count; $r++) {
    for ($c = 0; $c -lt $seedRows[$r].Count; $c++) {
        $value = $seedRows[$r][$c]
        if ($null -eq $value) {
            $ws.Cells.Item($r + 1, $c + 1).ClearContents()
        } else {
            if ($value -is [int] -or $value -is [long] -or $value -is [double] -or $value -is [decimal]) {
                $ws.Cells.Item($r + 1, $c + 1).Value2 = [double]$value
            } else {
                $ws.Cells.Item($r + 1, $c + 1).Value2 = [string]$value
            }
        }
    }
}

$critDav = @([object[]]@("Salesperson"), [object[]]@("Dav"))
$critProduce = @([object[]]@("Type"), [object[]]@("=Produce"))
$critUnique = @([object[]]@("Salesperson"), [object[]]@("=Buchanan"))
$critNone = @([object[]]@("Salesperson"), [object[]]@("=Nobody"))
$critRangeOr = @(
    [object[]]@("Sales","Sales"),
    [object[]]@(">6000","<6500"),
    [object[]]@("<500",$null)
)

function Set-Grid($startRow, $startCol, $grid) {
    for ($r = 0; $r -lt $grid.Count; $r++) {
        for ($c = 0; $c -lt $grid[$r].Count; $c++) {
            $value = $grid[$r][$c]
            if ($null -eq $value) {
                $ws.Cells.Item($startRow + $r, $startCol + $c).ClearContents()
            } else {
                if ($value -is [int] -or $value -is [long] -or $value -is [double] -or $value -is [decimal]) {
                    $ws.Cells.Item($startRow + $r, $startCol + $c).Value2 = [double]$value
                } elseif ($value -is [string] -and $value.Length -gt 0 -and "=<>".Contains($value.Substring(0,1))) {
                    $ws.Cells.Item($startRow + $r, $startCol + $c).Formula = "'" + $value
                } else {
                    $ws.Cells.Item($startRow + $r, $startCol + $c).Value2 = [string]$value
                }
            }
        }
    }
}

Set-Grid 1 6 $critDav
Set-Grid 1 8 $critProduce
Set-Grid 1 10 $critUnique
Set-Grid 1 12 $critNone
Set-Grid 1 14 $critRangeOr

$results = @()
$rowIndex = 1
foreach ($row in $rows) {
    $cell = $ws.Cells.Item($rowIndex, 20)
    $cell.NumberFormat = "0.000000000000000"
    $cell.Formula2 = "=" + $row.formula
    $excel.Calculate()

    $matches = $false
    $observed = ""
    if ($row.expected_kind -eq "number") {
        $value = [double]$cell.Value2
        $observed = [string]$value
        $tol = [double]$row.tolerance
        $matches = ([math]::Abs($value - [double]$row.expected_value) -le $tol)
    } else {
        $observed = [string]$cell.Text
        $matches = ($observed -eq $row.expected_value)
    }

    $results += [pscustomobject]@{
        scenario_id = $row.scenario_id
        lane = $row.lane
        formula = "=" + $row.formula
        expected_kind = $row.expected_kind
        observed = $observed
        expected_value = $row.expected_value
        matches_expected = $matches
        notes = $row.notes
    }
    $rowIndex++
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

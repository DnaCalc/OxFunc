param(
    [string]$Out = ".tmp/w110-and-or-error-precedence-probe-results.csv"
)

# oxf-xvt5.14 (W110) — AND/OR/XOR evaluate every argument: does a LATER argument's error
# surface, which error wins when SEVERAL arguments are errors, and what happens to a direct
# text argument. Same shape as tools/w24-probe/run-w24-batch01-switch-baseline.ps1: a hidden
# Excel instance, one formula per row in column A, .Text and .Value2 read back, nothing saved.
# Helper cells C1:C3 / D1:D3 carry TRUE / #DIV/0! / #N/A and FALSE / #DIV/0! / #N/A for the
# reference rows. Retained rows: .tmp/ (gitignored); the pins live in
# crates/oxfunc_core/src/functions/argument_laziness_golden.rs.

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$outPath = Join-Path $root $Out

$groups = [ordered]@{
    "later-argument-error" = @(
        '=OR(TRUE,1/0)',
        '=OR(1/0,TRUE)',
        '=OR(TRUE,NA())',
        '=AND(FALSE,1/0)',
        '=AND(1/0,FALSE)',
        '=XOR(TRUE,1/0)',
        '=XOR(1/0,TRUE)'
    )
    "two-errors-no-deciding-value" = @(
        '=OR(1/0,NA())',
        '=OR(NA(),1/0)',
        '=AND(1/0,NA())',
        '=AND(NA(),1/0)',
        '=XOR(1/0,NA())',
        '=XOR(NA(),1/0)'
    )
    "two-errors-after-deciding-value" = @(
        '=OR(TRUE,1/0,NA())',
        '=OR(TRUE,NA(),1/0)',
        '=AND(FALSE,1/0,NA())',
        '=AND(FALSE,NA(),1/0)',
        '=XOR(TRUE,1/0,NA())',
        '=XOR(TRUE,NA(),1/0)'
    )
    "other-error-code-pairs-both-orders" = @(
        '=OR(SQRT(-1),1/0)',
        '=OR(1/0,SQRT(-1))',
        '=OR("x"+1,1/0)',
        '=OR(1/0,"x"+1)',
        '=OR(NA(),SQRT(-1))',
        '=OR(SQRT(-1),NA())',
        '=OR(OFFSET(A1,-1,0),1/0)',
        '=OR(1/0,OFFSET(A1,-1,0))',
        '=OR(NA(),"x"+1)',
        '=OR("x"+1,NA())',
        '=AND(SQRT(-1),1/0)',
        '=AND(1/0,SQRT(-1))',
        '=AND("x"+1,NA())',
        '=AND(NA(),"x"+1)',
        '=AND(OFFSET(A1,-1,0),NA())',
        '=AND(NA(),OFFSET(A1,-1,0))'
    )
    "three-errors" = @(
        '=OR(SQRT(-1),NA(),1/0)',
        '=OR(NA(),1/0,SQRT(-1))',
        '=AND(1/0,SQRT(-1),NA())'
    )
    "errors-in-array-constant-or-reference" = @(
        '=OR({TRUE,#DIV/0!})',
        '=OR({TRUE,#DIV/0!,#N/A})',
        '=OR({TRUE,#N/A,#DIV/0!})',
        '=AND({FALSE,#DIV/0!})',
        '=AND({FALSE,#N/A,#DIV/0!})',
        '=OR(C1:C2)',
        '=OR(C1:C3)',
        '=OR(C1,C3,C2)',
        '=AND(D1:D2)',
        '=AND(D1:D3)',
        '=AND(D1,D3,D2)',
        '=OR(TRUE,C2)',
        '=AND(FALSE,D2)'
    )
    "direct-text-with-error" = @(
        '=OR("x",1/0)',
        '=OR(1/0,"x")',
        '=AND("x",NA())',
        '=AND(NA(),"x")',
        '=OR("x",TRUE,1/0)',
        '=OR(TRUE,"x",NA())'
    )
    "direct-text-alone-or-with-a-logical" = @(
        '=OR("x")',
        '=AND("x")',
        '=XOR("x")',
        '=OR("x","y")',
        '=AND("x","y")',
        '=OR("x",TRUE)',
        '=OR("x",FALSE)',
        '=AND("x",TRUE)',
        '=AND("x",FALSE)',
        '=OR(TRUE,"x")',
        '=AND(FALSE,"x")',
        '=OR(FALSE,"x")',
        '=AND(TRUE,"x")',
        '=XOR(TRUE,"x")',
        '=XOR("x",TRUE)',
        '=XOR(FALSE,"x")',
        '=OR(0,"x")',
        '=AND(1,"x")',
        '=OR("x",0)',
        '=AND("x",1)'
    )
    "direct-text-logical-and-numeric-spellings" = @(
        '=OR("TRUE")',
        '=OR("FALSE")',
        '=AND("TRUE")',
        '=AND("FALSE")',
        '=OR(FALSE,"TRUE")',
        '=AND(TRUE,"FALSE")',
        '=OR(FALSE,"true")',
        '=OR(FALSE,"TrUe")',
        '=OR("1")',
        '=OR("0")',
        '=AND("1")',
        '=AND("0")',
        '=OR("2")',
        '=OR(FALSE,"1")',
        '=AND(TRUE,"0")',
        '=OR(FALSE,"2")',
        '=OR(TRUE,"1")',
        '=AND(FALSE,"1")',
        '=XOR("1")',
        '=XOR(TRUE,"1")',
        '=XOR("x","1")',
        '=OR("x","1")',
        '=OR("1","x")',
        '=AND("x","0")',
        '=OR(TRUE,"1.5")',
        '=OR(FALSE,"1.5")',
        '=OR(FALSE," 1 ")',
        '=OR(FALSE,"1e0")',
        '=OR(FALSE,"$1")',
        '=OR(FALSE,"1/2")',
        '=OR(FALSE,"12/31/2020")',
        '=OR(FALSE,"")',
        '=AND(TRUE,"")',
        '=OR("")'
    )
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)

$ws.Range("C1").Formula2 = '=TRUE'
$ws.Range("C2").Formula2 = '=1/0'
$ws.Range("C3").Formula2 = '=NA()'
$ws.Range("D1").Formula2 = '=FALSE'
$ws.Range("D2").Formula2 = '=1/0'
$ws.Range("D3").Formula2 = '=NA()'

$version = [string]$excel.Version
$build = [string]$excel.Build
$stamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$results = @()
$rowIndex = 1
foreach ($group in $groups.Keys) {
    foreach ($f in $groups[$group]) {
        $cell = $ws.Cells.Item($rowIndex, 1)
        $cell.Formula2 = $f
        $results += [pscustomobject]@{
            group = $group
            formula = $f
            text = [string]$cell.Text
            value2 = $cell.Value2
            excel_version = $version
            excel_build = $build
            probed_utc = $stamp
        }
        $rowIndex++
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

$results | Format-Table group, formula, text, value2 -AutoSize | Out-String -Width 200

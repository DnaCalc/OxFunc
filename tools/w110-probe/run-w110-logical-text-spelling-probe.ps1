param(
    [string]$Out = ".tmp/w110-logical-text-spelling-probe-results.csv"
)

# oxf-xvt5.15 (W110, catalog G1-02) — which DIRECT text arguments AND/OR/XOR coerce to a
# logical. The oxf-xvt5.14 probe (run-w110-and-or-error-precedence-probe.ps1, groups
# direct-text-*) showed that "TRUE"/"FALSE" coerce case-insensitively and every other direct
# text — numeric text included — is ignored. Open questions settled here: surrounding
# whitespace, more case variants and non-ASCII look-alikes, text COMPUTED by an expression
# (is it "direct"?), a logical spelling arriving through a cell reference or an array
# constant, a coerced spelling next to an error, and whether a coerced spelling counts as a
# seen value for the no-value #VALUE! rule. Same shape as the sibling script: a hidden Excel
# instance, one formula per row in column A, .Text and .Value2 read back, nothing saved.
# Helper cells E1 / E2 hold the TEXT values TRUE / FALSE (number format "@" before the
# assignment, guarded by the ISTEXT rows). Retained rows: .tmp/ (gitignored); the pins live
# in crates/oxfunc_core/src/functions/argument_laziness_golden.rs.

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$outPath = Join-Path $root $Out

$groups = [ordered]@{
    "whitespace-around-logical-spelling" = @(
        '=OR(" TRUE")',
        '=OR("TRUE ")',
        '=OR(" TRUE ")',
        '=AND(" FALSE ")',
        '=OR(FALSE," TRUE")',
        '=OR(FALSE,"TRUE ")',
        '=OR(FALSE," TRUE ")',
        '=AND(TRUE," FALSE")',
        '=AND(TRUE,"FALSE ")',
        '=AND(TRUE," FALSE ")',
        '=OR(FALSE,CHAR(9)&"TRUE")',
        '=OR(FALSE,"TRUE"&CHAR(10))',
        '=OR(FALSE,CHAR(160)&"TRUE")',
        '=OR(FALSE,TRIM(" TRUE "))'
    )
    "case-variants-and-look-alikes" = @(
        '=OR("True")',
        '=OR("true")',
        '=AND("False")',
        '=AND("false")',
        '=AND(TRUE,"fAlSe")',
        '=OR(FALSE,"tRUE")',
        '=XOR("TRUE")',
        '=XOR("true")',
        '=XOR("FALSE")',
        '=AND(TRUE,"FAL"&UNICHAR(383)&"E")',
        '=OR(FALSE,UNICHAR(65332)&UNICHAR(65330)&UNICHAR(65333)&UNICHAR(65317))',
        '=OR(FALSE,"TRUE.")',
        '=OR(FALSE,"TRUE1")',
        '=OR(FALSE,"T")',
        '=OR(FALSE,"Yes")',
        '=OR(FALSE,"On")',
        '=OR(FALSE,"=TRUE")',
        '=OR(FALSE,"TRUE()")'
    )
    "computed-text-is-direct" = @(
        '=OR(FALSE,"TR"&"UE")',
        '=OR(FALSE,"x"&"y")',
        '=OR("x"&"y")',
        '=AND(TRUE,UPPER("false"))',
        '=OR(FALSE,LOWER("TRUE"))',
        '=OR(FALSE,LEFT("TRUEx",4))',
        '=OR(FALSE,TEXT(1,"0"))',
        '=OR(FALSE,IF(TRUE,"TRUE"))',
        '=OR(FALSE,IF(TRUE,"x"))',
        '=OR(IF(TRUE,"x"))'
    )
    "logical-spelling-from-reference-or-array" = @(
        '=ISTEXT(E1)',
        '=ISTEXT(E2)',
        '=OR(E1)',
        '=OR(FALSE,E1)',
        '=AND(TRUE,E2)',
        '=OR(E1:E2)',
        '=OR(FALSE,E1:E2)',
        '=OR({"TRUE"})',
        '=OR(FALSE,{"TRUE"})',
        '=AND(TRUE,{"FALSE"})',
        '=OR({"TRUE",FALSE})',
        '=OR(FALSE,INDEX({"TRUE"},1))',
        '=OR(FALSE,E1&"")',
        '=OR(FALSE,T(E1))',
        '=OR(FALSE,VALUETOTEXT(E1))'
    )
    "logical-spelling-next-to-an-error" = @(
        '=OR("TRUE",1/0)',
        '=OR(1/0,"TRUE")',
        '=AND("FALSE",NA())',
        '=AND(NA(),"FALSE")',
        '=XOR("TRUE",1/0)',
        '=OR(FALSE,"TRUE",NA())'
    )
    "coerced-spelling-counts-as-a-seen-value" = @(
        '=OR("FALSE","FALSE")',
        '=AND("TRUE","TRUE")',
        '=XOR("TRUE","TRUE")',
        '=XOR("TRUE","TRUE","TRUE")',
        '=XOR("TRUE","x")',
        '=XOR("FALSE","x")',
        '=OR("FALSE","x")',
        '=AND("x","TRUE")',
        '=OR("TRUE",0)',
        '=AND("FALSE",1)',
        '=OR("x","FALSE")',
        '=AND("TRUE","x","1")'
    )
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)

$ws.Range("E1").NumberFormat = "@"
$ws.Range("E1").Value2 = "TRUE"
$ws.Range("E2").NumberFormat = "@"
$ws.Range("E2").Value2 = "FALSE"

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

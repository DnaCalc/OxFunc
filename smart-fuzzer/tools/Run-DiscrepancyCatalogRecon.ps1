[CmdletBinding()]
param(
    [string]$CaseSet = "smart-fuzzer/corpus/discrepancy-recon/catalog-row-recon-v0.json",
    [string]$RunId = "catalog-row-recon-current"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$inv = [Globalization.CultureInfo]::InvariantCulture
Import-Module (Join-Path $PSScriptRoot "CellRefBatch.psm1") -Force

function Resolve-RepoPath([string]$path) {
    if ([IO.Path]::IsPathRooted($path)) { return $path }
    return Join-Path $repo $path
}
function To-LocalCell($value) {
    if ($value -is [string]) { return [ordered]@{kind="text";value=$value} }
    if ($value -is [bool]) { return [ordered]@{kind="logical";value=[bool]$value} }
    return [ordered]@{kind="number";value=[double]$value}
}
function To-LocalArg($arg) {
    if ($arg.kind -ne "array") { return $arg }
    $typedRows = [Collections.ArrayList]::new()
    foreach ($row in $arg.rows) {
        [void]$typedRows.Add([object[]]@($row | ForEach-Object { To-LocalCell $_ }))
    }
    return [ordered]@{kind="array";rows=$typedRows.ToArray()}
}
function Col([int]$number) {
    $s = ""; $n = $number
    while ($n -gt 0) { $m = ($n - 1) % 26; $s = [char](65 + $m) + $s; $n = [int](($n - $m) / 26) }
    return $s
}
function Extract-LocalOutcome($outcome, $resultIndex) {
    if ($null -eq $resultIndex) { return $outcome }
    if ($outcome.kind -ne "array") { return $outcome }
    $r = [int]$resultIndex[0] - 1; $c = [int]$resultIndex[1] - 1
    return $outcome.cells[$r][$c]
}

$caseDoc = Get-Content (Resolve-RepoPath $CaseSet) -Raw | ConvertFrom-Json
$cases = @($caseDoc.cases)
$runDir = Join-Path $repo "smart-fuzzer/runs/$RunId"
$caseDir = Join-Path $runDir "cases"; $outcomeDir = Join-Path $runDir "outcomes"; $comparisonDir = Join-Path $runDir "comparisons"
@($caseDir,$outcomeDir,$comparisonDir) | ForEach-Object { New-Item -ItemType Directory -Force -Path $_ | Out-Null }

$localCasePath = Join-Path $caseDir "cases.jsonl"
$localLines = foreach ($case in $cases) {
    $record = [ordered]@{
        case_id = $case.case_id
        function_id = "FUNC.$($case.function_name)"
        formula_text = "=$($case.function_name)(cell-refs)"
        args = @($case.args | ForEach-Object { To-LocalArg $_ })
    }
    $record | ConvertTo-Json -Compress -Depth 20
}
$localLines | Set-Content -Path $localCasePath -Encoding utf8

$localPath = Join-Path $outcomeDir "local.jsonl"
& cargo run -q --release --manifest-path (Join-Path $repo "smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml") --bin array_tranche_local_eval -- --cases $localCasePath --out $localPath | Out-Null
if ($LASTEXITCODE -ne 0) { throw "local evaluator failed: $LASTEXITCODE" }
$localMap = @{}
foreach ($line in Get-Content $localPath) {
    if ($line.Trim()) { $o = $line | ConvertFrom-Json; $localMap[$o.case_id] = $o.outcome }
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false; $excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add(); $ws = $wb.Worksheets.Item(1)
$excelMap = @{}
try {
    for ($i=0; $i -lt $cases.Count; $i++) {
        $case = $cases[$i]; $row = 1 + 4*$i; $col = 3; $refs = @()
        foreach ($arg in $case.args) {
            if ($arg.kind -eq "array") {
                $rows = @($arg.rows); $rowCount = $rows.Count; $colCount = @($rows[0]).Count
                for ($rr=0; $rr -lt $rowCount; $rr++) {
                    for ($cc=0; $cc -lt $colCount; $cc++) {
                        $v = $rows[$rr][$cc]
                        $ws.Cells.Item($row+$rr,$col+$cc).Value2 = if ($v -is [string]) { [string]$v } else { [double]$v }
                    }
                }
                $refs += "$(Col $col)$row`:$(Col ($col+$colCount-1))$($row+$rowCount-1)"
                $col += $colCount + 1
            } else {
                if ($arg.kind -eq "text") {
                    $ws.Cells.Item($row,$col).Value2 = [string]$arg.value
                } elseif ($arg.kind -eq "logical") {
                    $ws.Cells.Item($row,$col).Value2 = [bool]$arg.value
                } else {
                    $ws.Cells.Item($row,$col).Value2 = [double]$arg.value
                }
                $refs += "$(Col $col)$row"
                $col++
            }
        }
        $call = "$($case.function_name)($($refs -join ','))"
        $resultIndexProp = $case.PSObject.Properties["result_index"]
        if ($null -ne $resultIndexProp) {
            $resultIndex = @($resultIndexProp.Value)
            $call = "INDEX($call,$($resultIndex[0]),$($resultIndex[1]))"
        }
        $ws.Cells.Item($row,1).Formula2 = "=$call"
        $ws.Cells.Item($row,2).Formula2 = "=IF(ISERROR(A$row),ERROR.TYPE(A$row),`"`" )"
    }
    $excel.CalculateFull()
    for ($i=0; $i -lt $cases.Count; $i++) {
        $case = $cases[$i]; $row = 1 + 4*$i
        $excelMap[$case.case_id] = ConvertTo-ExcelOutcome -Value $ws.Cells.Item($row,1).Value2 -ErrorType $ws.Cells.Item($row,2).Value2
    }
    $environment = [ordered]@{
        excel_version = [string]$excel.Version
        excel_build = [string]$excel.Build
        workbook_compatibility = $(try { [string]$wb.CompatibilityVersion } catch { "unknown" })
        excel_input_plumbing = "cell_value2"
    }
}
finally {
    $wb.Close($false); $excel.Quit()
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($ws)
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($wb)
    [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel)
}

$excelPath = Join-Path $outcomeDir "excel.jsonl"
$comparisonPath = Join-Path $comparisonDir "comparisons.jsonl"
$excelLines = @(); $comparisonLines = @(); $summaryRows = @()
foreach ($case in $cases) {
    $resultIndexProp = $case.PSObject.Properties["result_index"]
    $resultIndex = if ($null -ne $resultIndexProp) { @($resultIndexProp.Value) } else { $null }
    $lo = Extract-LocalOutcome $localMap[$case.case_id] $resultIndex
    $xo = $excelMap[$case.case_id]
    $class = Get-StandardSeverityClass -LocalOutcome $lo -ExcelOutcome $xo
    $excelLines += ([ordered]@{case_id=$case.case_id;row_id=$case.row_id;outcome=$xo} | ConvertTo-Json -Compress -Depth 12)
    $comparison = [ordered]@{
        case_id = $case.case_id; row_id = $case.row_id; function_name = $case.function_name
        local_digest = $lo.digest_payload; excel_digest = $xo.digest_payload
        severity_class = $class.severity_class
        ulp_distance = if ($class.Contains("ulp_distance")) { $class.ulp_distance } else { $null }
    }
    $comparisonLines += ($comparison | ConvertTo-Json -Compress -Depth 8)
    $summaryRows += [pscustomobject]$comparison
}
$excelLines | Set-Content -Path $excelPath -Encoding utf8
$comparisonLines | Set-Content -Path $comparisonPath -Encoding utf8

$rowRollup = @($summaryRows | Group-Object row_id | ForEach-Object {
    $ulpValues = @($_.Group | Where-Object {$null -ne $_.ulp_distance} | ForEach-Object {[double]$_.ulp_distance})
    [ordered]@{
        row_id=$_.Name; cases=$_.Count
        matches=@($_.Group | Where-Object severity_class -eq "match").Count
        numeric_1ulp=@($_.Group | Where-Object severity_class -eq "numeric_drift_1ulp").Count
        numeric_gt1ulp=@($_.Group | Where-Object severity_class -eq "numeric_drift_gt1ulp").Count
        structural=@($_.Group | Where-Object severity_class -eq "structural_mismatch").Count
        max_ulp=if ($ulpValues.Count -gt 0) { ($ulpValues | Measure-Object -Maximum).Maximum } else { $null }
    }
})
$rollup = [ordered]@{
    schema_version="oxfunc.discrepancy_recon_rollup.v0"; run_id=$RunId
    environment=$environment; cases=$cases.Count; catalog_rows=$rowRollup.Count
    row_rollup=$rowRollup
}
$rollup | ConvertTo-Json -Depth 12 | Set-Content -Path (Join-Path $runDir "rollup.json") -Encoding utf8
$rowRollup | ForEach-Object {[pscustomobject]$_} | Export-Csv -NoTypeInformation -Encoding utf8 -Path (Join-Path $runDir "row-rollup.csv")
$rowRollup | ForEach-Object {[pscustomobject]$_} | Format-Table row_id,cases,matches,numeric_1ulp,numeric_gt1ulp,structural,max_ulp -AutoSize | Out-Host
Write-Host "Excel $($environment.excel_version) build $($environment.excel_build), Compatibility $($environment.workbook_compatibility)" -ForegroundColor Cyan
Write-Host "Run -> $runDir" -ForegroundColor Cyan

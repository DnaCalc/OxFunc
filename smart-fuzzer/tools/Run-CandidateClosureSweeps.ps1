[CmdletBinding()]
param(
    [ValidateSet("All","Atanh","TBillYield")][string]$Lane = "All",
    [string]$RunId = "candidate-closure-current"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Import-Module (Join-Path $PSScriptRoot "CellRefBatch.psm1") -Force

function Add-UniqueDouble([Collections.Generic.Dictionary[uint64,double]]$map,[double]$value) {
    if (-not [double]::IsNaN($value) -and -not [double]::IsInfinity($value)) {
        $bits = [BitConverter]::ToUInt64([BitConverter]::GetBytes($value),0)
        if (-not $map.ContainsKey($bits)) { $map.Add($bits,$value) }
    }
}
function Number-Arg([double]$value) { [ordered]@{kind="number";value=$value} }

$cases = [Collections.ArrayList]::new()
if ($Lane -in @("All","Atanh")) {
    $values = [Collections.Generic.Dictionary[uint64,double]]::new()
    foreach ($x in @(0.0,-0.0,0.1,-0.1,0.2,-0.2,0.5,-0.5,0.9,-0.9,0.99,-0.99,0.999999,-0.999999,1e-300,-1e-300,1e-16,-1e-16,1e-12,-1e-12,1e-8,-1e-8)) { Add-UniqueDouble $values ([double]$x) }
    Add-UniqueDouble $values ([BitConverter]::Int64BitsToDouble(1))
    Add-UniqueDouble $values (-[BitConverter]::Int64BitsToDouble(1))
    for ($i=-63; $i -le 63; $i++) { Add-UniqueDouble $values ([double]$i / 64.0) }
    for ($k=2; $k -le 52; $k++) {
        $x = 1.0 - [Math]::Pow(2.0,-$k)
        Add-UniqueDouble $values $x; Add-UniqueDouble $values (-$x)
    }
    # Deterministic irregular central points; arithmetic is recorded by input bits.
    for ($i=1; $i -le 128; $i++) {
        $numerator = (($i * 104729) % 999983) - 499991
        Add-UniqueDouble $values ([double]$numerator / 500000.0)
    }
    $index=0
    foreach ($entry in $values.GetEnumerator() | Sort-Object Key) {
        [void]$cases.Add([ordered]@{
            lane="ATANH"; case_id=("atanh-{0:D4}" -f $index); function_name="ATANH"
            args=@((Number-Arg $entry.Value))
        }); $index++
    }
}

if ($Lane -in @("All","TBillYield")) {
    $settlements = @(36526,36951,37621,38352,39538,40179,40968,42063,42794,43890,44561,45351,45716,46081)
    $durations = @(1,2,7,28,30,31,60,90,91,180,182,183,364,365)
    $prices = @(50.0,80.0,90.0,95.0,98.0,98.45,99.0,99.5,99.9,99.99,100.0)
    $index=0
    foreach ($settlement in $settlements) {
        foreach ($duration in $durations) {
            foreach ($price in $prices) {
                [void]$cases.Add([ordered]@{
                    lane="TBILLYIELD"; case_id=("tbillyield-{0:D5}" -f $index); function_name="TBILLYIELD"
                    args=@((Number-Arg $settlement),(Number-Arg ($settlement+$duration)),(Number-Arg $price))
                }); $index++
            }
        }
    }
}

$runDir = Join-Path $repo "smart-fuzzer/runs/$RunId"
$caseDir=Join-Path $runDir "cases"; $outDir=Join-Path $runDir "outcomes"; $cmpDir=Join-Path $runDir "comparisons"
@($caseDir,$outDir,$cmpDir) | ForEach-Object {New-Item -ItemType Directory -Force -Path $_ | Out-Null}
$casePath=Join-Path $caseDir "cases.jsonl"; $localPath=Join-Path $outDir "local.jsonl"
$caseLines = foreach ($case in $cases) {
    [ordered]@{case_id=$case.case_id;function_id="FUNC.$($case.function_name)";formula_text="=$($case.function_name)(cell-refs)";args=$case.args} | ConvertTo-Json -Compress -Depth 8
}
$caseLines | Set-Content -Path $casePath -Encoding utf8
& cargo run -q --release --manifest-path (Join-Path $repo "smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml") --bin array_tranche_local_eval -- --cases $casePath --out $localPath | Out-Null
if ($LASTEXITCODE -ne 0) {throw "local evaluator failed: $LASTEXITCODE"}
$local=@{}; foreach($line in Get-Content $localPath){if($line.Trim()){$o=$line|ConvertFrom-Json;$local[$o.case_id]=$o.outcome}}

$candidates=@($cases | ForEach-Object {[ordered]@{function_name=$_.function_name;args=@($_.args|ForEach-Object {[double]$_.value})}})
$excel=Invoke-ExcelCellRefBatch -Candidates $candidates
if($excel.blocked){throw "Excel sweep blocked: $($excel.blocker)"}

$excelLines=@(); $comparisonLines=@(); $rows=@()
for($i=0;$i -lt $cases.Count;$i++){
    $case=$cases[$i];$lo=$local[$case.case_id];$xo=$excel.outcomes[$i]
    $class=Get-StandardSeverityClass -LocalOutcome $lo -ExcelOutcome $xo
    $argBits=@($case.args|ForEach-Object {Get-F64BitsHex ([double]$_.value)})
    $record=[ordered]@{
        lane=$case.lane;case_id=$case.case_id;function_name=$case.function_name;arg_bits=($argBits -join "|")
        local_digest=$lo.digest_payload;excel_digest=$xo.digest_payload;severity_class=$class.severity_class
        ulp_distance=if($class.Contains("ulp_distance")){$class.ulp_distance}else{$null}
    }
    $rows += [pscustomobject]$record
    $comparisonLines += ($record|ConvertTo-Json -Compress -Depth 6)
    $excelLines += ([ordered]@{case_id=$case.case_id;outcome=$xo}|ConvertTo-Json -Compress -Depth 6)
}
$comparisonLines|Set-Content -Encoding utf8 -Path (Join-Path $cmpDir "comparisons.jsonl")
$excelLines|Set-Content -Encoding utf8 -Path (Join-Path $outDir "excel.jsonl")
$rollup=@($rows|Group-Object lane|ForEach-Object {
    $mismatch=@($_.Group|Where-Object severity_class -ne "match")
    [ordered]@{lane=$_.Name;cases=$_.Count;matches=$_.Count-$mismatch.Count;mismatches=$mismatch.Count;max_ulp=if($mismatch.Count){($mismatch|Measure-Object ulp_distance -Maximum).Maximum}else{$null}}
})
[ordered]@{schema_version="oxfunc.candidate_closure_rollup.v0";run_id=$RunId;environment=$excel.environment;lanes=$rollup}|ConvertTo-Json -Depth 10|Set-Content -Encoding utf8 -Path (Join-Path $runDir "rollup.json")
$rollup|ForEach-Object {[pscustomobject]$_}|Export-Csv -NoTypeInformation -Encoding utf8 -Path (Join-Path $runDir "rollup.csv")
$rollup|ForEach-Object {[pscustomobject]$_}|Format-Table -AutoSize|Out-Host
Write-Host "Excel $($excel.environment.excel_version) build $($excel.environment.excel_build), Compatibility $($excel.environment.workbook_compatibility)" -ForegroundColor Cyan
Write-Host "Run -> $runDir" -ForegroundColor Cyan

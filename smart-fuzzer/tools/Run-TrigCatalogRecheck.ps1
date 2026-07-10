[CmdletBinding()]
param(
    [string]$Out = ".tmp/trig-catalog-recheck.csv"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$inv = [System.Globalization.CultureInfo]::InvariantCulture
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

Import-Module (Join-Path $PSScriptRoot "CellRefBatch.psm1") -Force

$witnesses = @(
    @{ id="cos.49"; fn="COS"; value=49.214601836 },
    @{ id="cos.149"; fn="COS"; value=149.214601836 },
    @{ id="cot.-307"; fn="COT"; value=-307.07 },
    @{ id="tan.797601"; fn="TAN"; value=797601.58 },
    @{ id="sin.961281"; fn="SIN"; value=961281.44 },
    @{ id="sin.100000"; fn="SIN"; value=100000.0 },
    @{ id="cos.100000"; fn="COS"; value=100000.0 },
    @{ id="tan.100000"; fn="TAN"; value=100000.0 },
    @{ id="cot.100000"; fn="COT"; value=100000.0 },
    @{ id="sec.100000"; fn="SEC"; value=100000.0 },
    @{ id="csc.100000"; fn="CSC"; value=100000.0 },
    @{ id="sin.limit-1"; fn="SIN"; value=134217727.0 },
    @{ id="sin.limit"; fn="SIN"; value=134217728.0 }
)

$tmpDir = Join-Path $repo ".tmp"
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
$casePath = Join-Path $tmpDir "trig-catalog-recheck-cases.jsonl"
$localPath = Join-Path $tmpDir "trig-catalog-recheck-local.jsonl"

$caseLines = foreach ($w in $witnesses) {
    [ordered]@{
        case_id = $w.id
        function_id = "FUNC.$($w.fn)"
        formula_text = "=$($w.fn)(cell-ref)"
        args = @([ordered]@{ kind="number"; value=[double]$w.value })
    } | ConvertTo-Json -Compress -Depth 6
}
Set-Content -Path $casePath -Value $caseLines -Encoding utf8

& cargo run -q --release --manifest-path (Join-Path $repo "smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml") --bin array_tranche_local_eval -- --cases $casePath --out $localPath | Out-Null
if ($LASTEXITCODE -ne 0) { throw "OxFunc local evaluator failed with exit $LASTEXITCODE" }

$local = @{}
foreach ($line in Get-Content $localPath) {
    if ($line.Trim()) {
        $record = $line | ConvertFrom-Json
        $local[$record.case_id] = $record.outcome
    }
}

$candidates = @($witnesses | ForEach-Object {
    [ordered]@{ function_name=$_.fn; args=@([double]$_.value) }
})
$excel = Invoke-ExcelCellRefBatch -Candidates $candidates
if ($excel.blocked) { throw "Excel verification blocked: $($excel.blocker)" }

$rows = for ($i = 0; $i -lt $witnesses.Count; $i++) {
    $w = $witnesses[$i]
    $lo = $local[$w.id]
    $xo = $excel.outcomes[$i]
    $classification = Get-StandardSeverityClass -LocalOutcome $lo -ExcelOutcome $xo
    [pscustomobject]@{
        case_id = $w.id
        function = $w.fn
        input = ([double]$w.value).ToString("R", $inv)
        input_bits = Get-F64BitsHex ([double]$w.value)
        oxfunc = $lo.digest_payload
        excel = $xo.digest_payload
        classification = $classification.severity_class
        ulp = if ($classification.Contains("ulp_distance")) { $classification.ulp_distance } else { $null }
    }
}

$outPath = if ([IO.Path]::IsPathRooted($Out)) { $Out } else { Join-Path $repo $Out }
$parent = Split-Path -Parent $outPath
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$rows | Export-Csv -Path $outPath -NoTypeInformation -Encoding utf8
$rows | Format-Table case_id, function, classification, ulp -AutoSize | Out-Host
Write-Host "Excel: $($excel.environment.excel_version) build $($excel.environment.excel_build)" -ForegroundColor Cyan
Write-Host "Ledger -> $outPath" -ForegroundColor Cyan

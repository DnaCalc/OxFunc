<#
.SYNOPSIS
  Two-way bit-exact probe (OxFunc local-eval vs live Excel COM) for unary numeric
  functions — the catastrophic-band elementary/trig rows that the F# library does
  not cover.

.DESCRIPTION
  Each case is @{ id; fn; arg } where fn is the Excel/OxFunc function name and arg a
  scalar double. The same double drives both engines (Excel via Range.Value2 cell-ref;
  OxFunc via round-trip R-format JSON). Output: per-case OxFunc bits, Excel bits, exact
  match, ULP distance. Excel results are read by .NET type (number vs error code).

  Pass -Cases to supply your own list; otherwise a default catastrophic-band set runs.
#>
[CmdletBinding()]
param(
    [array]$Cases,
    [string]$LedgerOut = ".tmp/elem-probe-ledger.csv"
)
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$inv = [System.Globalization.CultureInfo]::InvariantCulture

if (-not $Cases) {
    $Cases = @(
        @{ id='tan.w';   fn='TAN'; arg=797601.58 },
        @{ id='sin.w';   fn='SIN'; arg=961281.44 },
        @{ id='cot.w';   fn='COT'; arg=-307.07 },
        @{ id='sin.sm';  fn='SIN'; arg=0.5 },
        @{ id='tan.sm';  fn='TAN'; arg=1.2 },
        @{ id='cos.w';   fn='COS'; arg=797601.58 },
        @{ id='sin.mid'; fn='SIN'; arg=12345.678 },
        @{ id='tan.mid'; fn='TAN'; arg=100000.0 }
    )
}

function Ensure-Parent([string]$p) {
    $d = Split-Path -Parent $p
    if ($d -and -not (Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null }
}
function Num-Lit($v) { ([double]$v).ToString('R', $inv) }
function Hex-Of-Int64([long]$b) { '0x{0:x16}' -f $b }
function Hex-Of-Double([double]$d) { Hex-Of-Int64 ([System.BitConverter]::DoubleToInt64Bits($d)) }
function Int64-Of-Hex([string]$hex) {
    $h = $hex; if ($h.StartsWith('0x')) { $h = $h.Substring(2) }
    $u = [System.Convert]::ToUInt64($h, 16)
    return [System.BitConverter]::ToInt64([System.BitConverter]::GetBytes($u), 0)
}
function Ordered([long]$b) { if ($b -lt 0) { return [decimal][long]::MinValue - [decimal]$b } else { return [decimal]$b } }
function Ulp-Distance([string]$a, [string]$b) {
    if (-not $a -or -not $b -or $a.StartsWith('err') -or $b.StartsWith('err')) { return $null }
    return [Math]::Abs((Ordered (Int64-Of-Hex $a)) - (Ordered (Int64-Of-Hex $b)))
}

# --- OxFunc ---
$lines = foreach ($c in $Cases) {
    $argJson = '{"kind":"number","value":' + (Num-Lit $c.arg) + '}'
    if ($c.ContainsKey('arg2')) {
        $argJson += ',{"kind":"number","value":' + (Num-Lit $c.arg2) + '}'
    }
    '{"case_id":"' + $c.id + '","function_id":"FUNC.' + $c.fn + '","formula_text":"' + $c.id +
    '","args":[' + $argJson + ']}'
}
$casesPath = ".tmp/elem-probe-ox-cases.jsonl"
$outPath = ".tmp/elem-probe-ox-out.jsonl"
Ensure-Parent $casesPath
Set-Content -Path $casesPath -Value $lines -Encoding utf8
& cargo run -q --release --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --bin array_tranche_local_eval -- --cases $casesPath --out $outPath | Out-Null
if ($LASTEXITCODE -ne 0) { throw "OxFunc local-eval failed." }
$oxMap = @{}
foreach ($line in (Get-Content $outPath)) {
    if (-not $line.Trim()) { continue }
    $o = $line | ConvertFrom-Json
    $oc = $o.outcome
    if ($oc.kind -eq 'number') { $oxMap[$o.case_id] = $oc.bits_hex }
    elseif ($oc.kind -eq 'error') { $oxMap[$o.case_id] = 'err:' + $oc.code }
    else { $oxMap[$o.case_id] = 'err:' + $oc.kind }
}

# --- Excel ---
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add()
$ws = $wb.Worksheets.Item(1)
$exMap = @{}
try {
    $r = 0
    foreach ($c in $Cases) {
        $r++
        $ws.Cells.Item($r, 2).Value2 = [double]$c.arg
        if ($c.ContainsKey('arg2')) {
            $ws.Cells.Item($r, 3).Value2 = [double]$c.arg2
            $ws.Cells.Item($r, 1).Formula = '=' + $c.fn + '(B' + $r + ',C' + $r + ')'
        } else {
            $ws.Cells.Item($r, 1).Formula = '=' + $c.fn + '(B' + $r + ')'
        }
    }
    $ws.Columns.Item(1).ColumnWidth = 100
    $r = 0
    foreach ($c in $Cases) {
        $r++
        $v2 = $ws.Cells.Item($r, 1).Value2
        if ($v2 -is [double]) { $exMap[$c.id] = Hex-Of-Double ([double]$v2) }
        elseif ($null -eq $v2) { $exMap[$c.id] = 'err:#EMPTY' }
        else { $exMap[$c.id] = 'err:' + ([string]$ws.Cells.Item($r, 1).Text) }
    }
}
finally {
    $wb.Close($false); $excel.Quit()
    [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws)
    [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb)
    [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel)
}

Ensure-Parent $LedgerOut
$ledger = foreach ($c in $Cases) {
    $ox = [string]$oxMap[$c.id]; $ex = [string]$exMap[$c.id]
    [pscustomobject]@{
        case_id = $c.id
        fn      = $c.fn
        arg     = Num-Lit $c.arg
        ox      = $ox
        excel   = $ex
        match   = ($ox -eq $ex)
        ulp     = Ulp-Distance $ox $ex
    }
}
$ledger | Export-Csv -Path $LedgerOut -NoTypeInformation -Encoding UTF8
$ledger | Format-Table case_id, fn, ox, excel, match, ulp -AutoSize | Out-Host

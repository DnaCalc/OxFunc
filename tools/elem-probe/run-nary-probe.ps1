<#
.SYNOPSIS
  Bit-exact two-way probe (OxFunc local-eval vs live Excel COM) for N-ary scalar
  functions — used to quantify the unranked NUM-L statistical/special rows.

.DESCRIPTION
  Each case is @{ id; fn; args=@(v1,v2,...) }. The same doubles drive both engines
  (Excel via Range.Value2 cell-refs B,C,D,...; OxFunc via round-trip R-format JSON).
  Output: per-case OxFunc bits, Excel bits, exact match, ULP distance.
#>
[CmdletBinding()]
param([array]$Cases, [string]$LedgerOut = ".tmp/nary-probe-ledger.csv")
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$inv = [System.Globalization.CultureInfo]::InvariantCulture

if (-not $Cases) {
    $Cases = @(
        @{ id='normsdist';    fn='NORMSDIST';    args=@(1.5) },
        @{ id='normsinv';     fn='NORMSINV';     args=@(0.975) },
        @{ id='chidist';      fn='CHIDIST';      args=@(5.0, 3.0) },
        @{ id='chiinv';       fn='CHIINV';       args=@(0.05, 3.0) },
        @{ id='fdist';        fn='FDIST';        args=@(3.0, 5.0, 10.0) },
        @{ id='finv';         fn='FINV';         args=@(0.05, 5.0, 10.0) },
        @{ id='gammadist';    fn='GAMMADIST';    args=@(5.0, 2.0, 1.5, 1.0) },
        @{ id='gammainv';     fn='GAMMAINV';     args=@(0.5, 2.0, 1.5) },
        @{ id='hypgeomdist';  fn='HYPGEOMDIST';  args=@(1.0, 4.0, 8.0, 20.0) },
        @{ id='negbinomdist'; fn='NEGBINOMDIST'; args=@(10.0, 5.0, 0.25) },
        @{ id='tdist';        fn='TDIST';        args=@(2.0, 10.0, 2.0) },
        @{ id='tinv';         fn='TINV';         args=@(0.05, 10.0) },
        @{ id='betainv';      fn='BETAINV';      args=@(0.5, 2.0, 3.0) },
        @{ id='betadist';     fn='BETADIST';     args=@(0.5, 2.0, 3.0) },
        @{ id='bessely';      fn='BESSELY';      args=@(2.5, 1.0) },
        @{ id='gamma';        fn='GAMMA';        args=@(-1.00012) }
    )
}

function Ensure-Parent([string]$p) { $d = Split-Path -Parent $p; if ($d -and -not (Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null } }
function Num-Lit($v) { ([double]$v).ToString('R', $inv) }
function Hex-Of-Int64([long]$b) { '0x{0:x16}' -f $b }
function Hex-Of-Double([double]$d) { Hex-Of-Int64 ([System.BitConverter]::DoubleToInt64Bits($d)) }
function Int64-Of-Hex([string]$h) { if ($h.StartsWith('0x')) { $h = $h.Substring(2) }; [System.BitConverter]::ToInt64([System.BitConverter]::GetBytes([System.Convert]::ToUInt64($h, 16)), 0) }
function Ordered([long]$b) { if ($b -lt 0) { return [decimal][long]::MinValue - [decimal]$b } else { return [decimal]$b } }
function Ulp-Distance($a, $b) { if (-not $a -or -not $b -or $a.StartsWith('err') -or $b.StartsWith('err')) { return $null }; return [Math]::Abs((Ordered (Int64-Of-Hex $a)) - (Ordered (Int64-Of-Hex $b))) }
function Col([int]$i) { [char](66 + $i) }  # 0->B, 1->C, ...

# --- OxFunc ---
$lines = foreach ($c in $Cases) {
    $argJson = ($c.args | ForEach-Object { '{"kind":"number","value":' + (Num-Lit $_) + '}' }) -join ','
    '{"case_id":"' + $c.id + '","function_id":"FUNC.' + $c.fn + '","formula_text":"' + $c.id + '","args":[' + $argJson + ']}'
}
$casesPath = ".tmp/nary-probe-ox-cases.jsonl"; $outPath = ".tmp/nary-probe-ox-out.jsonl"
Ensure-Parent $casesPath
Set-Content -Path $casesPath -Value $lines -Encoding utf8
& cargo run -q --release --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --bin array_tranche_local_eval -- --cases $casesPath --out $outPath | Out-Null
if ($LASTEXITCODE -ne 0) { throw "OxFunc local-eval failed." }
$oxMap = @{}
foreach ($line in (Get-Content $outPath)) {
    if (-not $line.Trim()) { continue }
    $o = $line | ConvertFrom-Json; $oc = $o.outcome
    if ($oc.kind -eq 'number') { $oxMap[$o.case_id] = $oc.bits_hex }
    elseif ($oc.kind -eq 'error') { $oxMap[$o.case_id] = 'err:' + $oc.code }
    else { $oxMap[$o.case_id] = 'err:' + $oc.kind }
}

# --- Excel ---
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false; $excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add(); $ws = $wb.Worksheets.Item(1)
$exMap = @{}
try {
    $r = 0
    foreach ($c in $Cases) {
        $r++
        $refs = @()
        for ($i = 0; $i -lt $c.args.Count; $i++) {
            $ws.Cells.Item($r, 2 + $i).Value2 = [double]$c.args[$i]
            $refs += (Col $i) + $r
        }
        $ws.Cells.Item($r, 1).Formula = '=' + $c.fn + '(' + ($refs -join ',') + ')'
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
    [pscustomobject]@{ case_id = $c.id; fn = $c.fn; ox = $ox; excel = $ex; match = ($ox -eq $ex); ulp = Ulp-Distance $ox $ex }
}
$ledger | Export-Csv -Path $LedgerOut -NoTypeInformation -Encoding UTF8
$ledger | Format-Table case_id, fn, ox, excel, match, ulp -AutoSize | Out-Host

<#
.SYNOPSIS
  Bit-exact probe (OxFunc local-eval vs live Excel COM) for the array matrix
  functions MINVERSE / MMULT — quantifies the unranked NUM-L matrix rows.

.DESCRIPTION
  Each case is @{ id; fn; m1=@(@(..),..); m2=... }. Matrices drive both engines with
  identical doubles (Excel via Range.Value2 + a CSE array formula; OxFunc via array
  JSON args). Output: per-case max/whichcell ULP across the result matrix.
#>
[CmdletBinding()]
param([array]$Cases, [string]$LedgerOut = ".tmp/matrix-probe-ledger.csv")
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$inv = [System.Globalization.CultureInfo]::InvariantCulture

if (-not $Cases) {
    $Cases = @(
        @{ id='minv.2x2'; fn='MINVERSE'; m1=@(@(4.0, 3.0), @(6.0, 3.0)) },
        @{ id='minv.3x3'; fn='MINVERSE'; m1=@(@(1.0, 2.0, 3.0), @(0.0, 1.0, 4.0), @(5.0, 6.0, 0.0)) },
        @{ id='minv.3x3b'; fn='MINVERSE'; m1=@(@(2.0, -1.0, 0.0), @(-1.0, 2.0, -1.0), @(0.0, -1.0, 2.0)) },
        @{ id='minv.4x4'; fn='MINVERSE'; m1=@(@(4.0, 2.0, 1.0, 3.0), @(3.0, 5.0, 2.0, 1.0), @(1.0, 1.0, 6.0, 2.0), @(2.0, 3.0, 1.0, 7.0)) },
        @{ id='mmult.2x2'; fn='MMULT'; m1=@(@(1.0, 2.0), @(3.0, 4.0)); m2=@(@(5.0, 6.0), @(7.0, 8.0)) },
        @{ id='mmult.2x3'; fn='MMULT'; m1=@(@(1.0, 2.0, 3.0), @(4.0, 5.0, 6.0)); m2=@(@(7.0, 8.0), @(9.0, 10.0), @(11.0, 12.0)) },
        @{ id='mmult.frac'; fn='MMULT'; m1=@(@(0.1, 0.2), @(0.3, 0.4)); m2=@(@(1.5, 2.5), @(3.5, 4.5)) }
    )
}

function Ensure-Parent([string]$p) { $d = Split-Path -Parent $p; if ($d -and -not (Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null } }
function Num-Lit($v) { ([double]$v).ToString('R', $inv) }
function Hex-Of-Double([double]$d) { '0x{0:x16}' -f [System.BitConverter]::DoubleToInt64Bits($d) }
function Int64-Of-Hex([string]$h) { if ($h.StartsWith('0x')) { $h = $h.Substring(2) }; [System.BitConverter]::ToInt64([System.BitConverter]::GetBytes([System.Convert]::ToUInt64($h, 16)), 0) }
function Ordered([long]$b) { if ($b -lt 0) { return [decimal][long]::MinValue - [decimal]$b } else { return [decimal]$b } }
function Ulp-Distance($a, $b) { [Math]::Abs((Ordered (Int64-Of-Hex $a)) - (Ordered (Int64-Of-Hex $b))) }
function Mat-Json($m) {
    $rows = foreach ($row in $m) { '[' + (($row | ForEach-Object { '{"kind":"number","value":' + (Num-Lit $_) + '}' }) -join ',') + ']' }
    '{"kind":"array","rows":[' + ($rows -join ',') + ']}'
}
function Col-Letter([int]$col) { $s = ''; $n = $col; while ($n -gt 0) { $m = ($n - 1) % 26; $s = [char](65 + $m) + $s; $n = [int](($n - $m) / 26) }; $s }

# --- OxFunc ---
$lines = foreach ($c in $Cases) {
    $args = (Mat-Json $c.m1)
    if ($c.ContainsKey('m2')) { $args += ',' + (Mat-Json $c.m2) }
    '{"case_id":"' + $c.id + '","function_id":"FUNC.' + $c.fn + '","formula_text":"' + $c.id + '","args":[' + $args + ']}'
}
$casesPath = ".tmp/matrix-probe-ox-cases.jsonl"; $outPath = ".tmp/matrix-probe-ox-out.jsonl"
Ensure-Parent $casesPath
Set-Content -Path $casesPath -Value $lines -Encoding utf8
& cargo run -q --release --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --bin array_tranche_local_eval -- --cases $casesPath --out $outPath | Out-Null
if ($LASTEXITCODE -ne 0) { throw "OxFunc local-eval failed." }
$oxMap = @{}
foreach ($line in (Get-Content $outPath)) {
    if (-not $line.Trim()) { continue }
    $o = $line | ConvertFrom-Json; $oc = $o.outcome
    if ($oc.kind -eq 'array') {
        $grid = @()
        foreach ($r in $oc.cells) { $grid += , ($r | ForEach-Object { if ($_.kind -eq 'number') { $_.bits_hex } else { 'err:' + $_.kind } }) }
        $oxMap[$o.case_id] = $grid
    } else { $oxMap[$o.case_id] = 'err:' + $oc.kind }
}

# --- Excel ---
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false; $excel.DisplayAlerts = $false
$wb = $excel.Workbooks.Add(); $ws = $wb.Worksheets.Item(1)
$exMap = @{}
try {
    $row = 1
    $meta = @{}
    foreach ($c in $Cases) {
        $r1 = $c.m1.Count; $c1 = $c.m1[0].Count
        # place m1 at (row, col=2)
        for ($i = 0; $i -lt $r1; $i++) { for ($j = 0; $j -lt $c1; $j++) { $ws.Cells.Item($row + $i, 2 + $j).Value2 = [double]$c.m1[$i][$j] } }
        $m1ref = 'B' + $row + ':' + (Col-Letter (1 + $c1)) + ($row + $r1 - 1)
        if ($c.ContainsKey('m2')) {
            $r2 = $c.m2.Count; $c2 = $c.m2[0].Count
            $m2col = 2 + $c1 + 1
            for ($i = 0; $i -lt $r2; $i++) { for ($j = 0; $j -lt $c2; $j++) { $ws.Cells.Item($row + $i, $m2col + $j).Value2 = [double]$c.m2[$i][$j] } }
            $m2ref = (Col-Letter $m2col) + $row + ':' + (Col-Letter ($m2col + $c2 - 1)) + ($row + $r2 - 1)
            $outRows = $r1; $outCols = $c2; $formula = "=MMULT($m1ref,$m2ref)"
            $outCol = $m2col + $c2 + 1
        } else {
            $outRows = $r1; $outCols = $c1; $formula = "=MINVERSE($m1ref)"
            $outCol = 2 + $c1 + 1
        }
        $outRange = $ws.Range((Col-Letter $outCol) + $row + ':' + (Col-Letter ($outCol + $outCols - 1)) + ($row + $outRows - 1))
        $outRange.FormulaArray = $formula
        $meta[$c.id] = @{ outCol = $outCol; row = $row; rows = $outRows; cols = $outCols }
        $row += [Math]::Max($outRows, $c.m1.Count) + 2
    }
    foreach ($c in $Cases) {
        $mm = $meta[$c.id]
        $grid = @()
        for ($i = 0; $i -lt $mm.rows; $i++) {
            $r = @()
            for ($j = 0; $j -lt $mm.cols; $j++) {
                $v2 = $ws.Cells.Item($mm.row + $i, $mm.outCol + $j).Value2
                if ($v2 -is [double]) { $r += Hex-Of-Double ([double]$v2) } else { $r += 'err' }
            }
            $grid += , $r
        }
        $exMap[$c.id] = $grid
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
    $ox = $oxMap[$c.id]; $ex = $exMap[$c.id]
    $maxUlp = 0.0; $cells = 0; $exactCells = 0
    for ($i = 0; $i -lt $ox.Count; $i++) {
        for ($j = 0; $j -lt $ox[$i].Count; $j++) {
            $cells++
            if ($ox[$i][$j] -eq $ex[$i][$j]) { $exactCells++ }
            elseif (-not $ox[$i][$j].StartsWith('err') -and -not $ex[$i][$j].StartsWith('err')) {
                $u = Ulp-Distance $ox[$i][$j] $ex[$i][$j]
                if ($u -gt $maxUlp) { $maxUlp = $u }
            }
        }
    }
    [pscustomobject]@{ case_id = $c.id; fn = $c.fn; cells = $cells; exact = $exactCells; max_ulp = $maxUlp }
}
$ledger | Export-Csv -Path $LedgerOut -NoTypeInformation -Encoding UTF8
$ledger | Format-Table case_id, fn, cells, exact, max_ulp -AutoSize | Out-Host

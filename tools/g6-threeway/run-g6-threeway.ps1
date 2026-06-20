<#
.SYNOPSIS
  Bit-exact three-way check (OxFunc / F# ExcelFinancialFunctions / live Excel) over
  the open G6 financial discrepancy witnesses.

.DESCRIPTION
  For each witness in g6-cases.ps1 the same double-precision inputs are driven into
  three engines and the IEEE-754 results compared bit-for-bit:
    * OxFunc  - smart-fuzzer array_tranche_local_eval (emits bits_hex)
    * F#      - dotnet fsi against the built ExcelFinancialFunctions.dll
    * Excel   - live COM, inputs via Range.Value2 cell-refs (no literal re-rounding)

  Output: a ledger CSV (per-case digests + ULP distances + comparison class) and a
  console summary. Class vocabulary:
    all_bit_exact      ox==excel and fs==excel
    ox_exact_fs_off    ox==excel, fs!=excel   (OxFunc already matches Excel)
    fs_exact_ox_off    fs==excel, ox!=excel   (F# is a repair roadmap)
    both_off_ox_eq_fs  neither matches Excel but ox==fs (Excel is idiosyncratic)
    all_diverge        all three differ

.NOTES
  Bit-exactness, not closeness: equality is hex-of-bits, ULP is informational.
#>
[CmdletBinding()]
param(
    [string]$LedgerOut = ".tmp/g6-threeway-ledger.csv",
    [string]$FSharpDll = ".tmp/ExcelFinancialFunctions/src/ExcelFinancialFunctions/bin/Debug/netstandard2.0/ExcelFinancialFunctions.dll",
    [string]$FSharpProject = ".tmp/ExcelFinancialFunctions/src/ExcelFinancialFunctions/ExcelFinancialFunctions.fsproj",
    [string[]]$Only,          # restrict to these case ids (smoke test)
    [switch]$SkipFSharp,
    [switch]$SkipExcel
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$inv = [System.Globalization.CultureInfo]::InvariantCulture

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $scriptDir 'g6-cases.ps1') | Out-Null
$cases = $G6Cases
if ($Only) { $cases = $cases | Where-Object { $Only -contains $_.id } }
Write-Host "G6 three-way: $($cases.Count) case(s)." -ForegroundColor Cyan

function Ensure-Parent([string]$p) {
    $d = Split-Path -Parent $p
    if ($d -and -not (Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null }
}
function Num-Lit($v) { ([double]$v).ToString('R', $inv) }
function Bits-Lit($v) { [System.BitConverter]::DoubleToInt64Bits([double]$v).ToString($inv) }
function Hex-Of-Int64([long]$b) { '0x{0:x16}' -f $b }
function Hex-Of-Double([double]$d) { Hex-Of-Int64 ([System.BitConverter]::DoubleToInt64Bits($d)) }
function Int64-Of-Hex([string]$hex) {
    $h = $hex; if ($h.StartsWith('0x')) { $h = $h.Substring(2) }
    $u = [System.Convert]::ToUInt64($h, 16)
    return [System.BitConverter]::ToInt64([System.BitConverter]::GetBytes($u), 0)
}
function Ordered([long]$b) { if ($b -lt 0) { return [decimal][long]::MinValue - [decimal]$b } else { return [decimal]$b } }
function Ulp-Distance([string]$hexA, [string]$hexB) {
    if (-not $hexA -or -not $hexB) { return $null }
    $a = Ordered (Int64-Of-Hex $hexA)
    $b = Ordered (Int64-Of-Hex $hexB)
    return [Math]::Abs($a - $b)
}
function Col-Letter([int]$col) {
    $s = ''; $n = $col
    while ($n -gt 0) { $m = ($n - 1) % 26; $s = [char](65 + $m) + $s; $n = [int](($n - $m) / 26) }
    return $s
}

# --- OxFunc -------------------------------------------------------------------
function Invoke-OxFunc($caseList) {
    $lines = foreach ($c in $caseList) {
        $argJson = foreach ($a in $c.args) {
            if ($a.t -eq 'array') {
                $cells = ($a.v | ForEach-Object { '{"kind":"number","value":' + (Num-Lit $_) + '}' }) -join ','
                '{"kind":"array","rows":[[' + $cells + ']]}'
            } else {
                '{"kind":"number","value":' + (Num-Lit $a.v) + '}'
            }
        }
        '{"case_id":"' + $c.id + '","function_id":"FUNC.' + $c.xlFn + '","formula_text":"' + $c.id + '","args":[' + ($argJson -join ',') + ']}'
    }
    $casesPath = ".tmp/g6-threeway-ox-cases.jsonl"
    $outPath = ".tmp/g6-threeway-ox-out.jsonl"
    Ensure-Parent $casesPath
    Set-Content -Path $casesPath -Value $lines -Encoding utf8
    & cargo run -q --release --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --bin array_tranche_local_eval -- --cases $casesPath --out $outPath | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "OxFunc local-eval failed (exit $LASTEXITCODE)." }
    $map = @{}
    foreach ($line in (Get-Content $outPath)) {
        if (-not $line.Trim()) { continue }
        $o = $line | ConvertFrom-Json
        $oc = $o.outcome
        switch ($oc.kind) {
            'number' { $map[$o.case_id] = @{ kind='number'; hex=$oc.bits_hex; raw=$oc.value } }
            'error'  { $map[$o.case_id] = @{ kind='error'; code=$oc.code; raw=$oc.code } }
            default  { $map[$o.case_id] = @{ kind='other'; code=$oc.kind; raw=$oc.digest_payload } }
        }
    }
    return $map
}

# --- F# -----------------------------------------------------------------------
function Invoke-FSharp($caseList) {
    if (-not (Test-Path $FSharpDll)) {
        Write-Host "Building F# library..." -ForegroundColor DarkGray
        & dotnet build $FSharpProject -v minimal | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "F# build failed." }
    }
    $dll = (Resolve-Path $FSharpDll).Path
    $sb = [System.Text.StringBuilder]::new()
    [void]$sb.AppendLine('#r @"' + $dll + '"')
    [void]$sb.AppendLine('open System')
    [void]$sb.AppendLine('open Excel.FinancialFunctions')
    foreach ($c in $caseList) {
        $argExpr = foreach ($a in $c.args) {
            switch ($a.t) {
                'num'    { 'BitConverter.Int64BitsToDouble(' + (Bits-Lit $a.v) + 'L)' }
                'date'   { 'DateTime.FromOADate(' + (Num-Lit $a.v) + ')' }
                'freq'   { 'enum<Frequency>(' + [int]$a.v + ')' }
                'basis'  { 'enum<DayCountBasis>(' + [int]$a.v + ')' }
                'paydue' { 'enum<PaymentDue>(' + [int]$a.v + ')' }
                'array'  { '[| ' + (($a.v | ForEach-Object { 'BitConverter.Int64BitsToDouble(' + (Bits-Lit $_) + 'L)' }) -join '; ') + ' |]' }
            }
        }
        $call = 'Financial.' + $c.fsFn + '(' + ($argExpr -join ', ') + ')'
        [void]$sb.AppendLine('try')
        [void]$sb.AppendLine('    let v = ' + $call)
        [void]$sb.AppendLine('    printfn "' + $c.id + '|number|%d" (BitConverter.DoubleToInt64Bits(v))')
        [void]$sb.AppendLine('with ex ->')
        [void]$sb.AppendLine('    printfn "' + $c.id + '|error|%s" (ex.GetType().Name)')
    }
    $fsxPath = ".tmp/g6-threeway-probe.fsx"
    Ensure-Parent $fsxPath
    Set-Content -Path $fsxPath -Value $sb.ToString() -Encoding utf8
    $raw = & dotnet fsi $fsxPath 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host ($raw | Out-String) -ForegroundColor Red
        throw "dotnet fsi failed."
    }
    $map = @{}
    foreach ($line in $raw) {
        $t = [string]$line
        if ($t -notmatch '^[^|]+\|(number|error)\|') { continue }
        $p = $t.Split('|')
        if ($p[1] -eq 'number') { $map[$p[0]] = @{ kind='number'; hex=(Hex-Of-Int64 ([long]$p[2])) } }
        else { $map[$p[0]] = @{ kind='error'; code=$p[2] } }
    }
    return $map
}

# --- Excel --------------------------------------------------------------------
function Invoke-Excel($caseList) {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)
    $map = @{}
    try {
        $r = 0
        foreach ($c in $caseList) {
            $r++
            $col = 3
            $refs = @()
            foreach ($a in $c.args) {
                if ($a.t -eq 'array') {
                    $start = $col
                    foreach ($el in $a.v) { $ws.Cells.Item($r, $col).Value2 = [double]$el; $col++ }
                    $refs += ((Col-Letter $start) + $r + ':' + (Col-Letter ($col - 1)) + $r)
                } else {
                    $ws.Cells.Item($r, $col).Value2 = [double]$a.v
                    $refs += ((Col-Letter $col) + $r)
                    $col++
                }
            }
            $ws.Cells.Item($r, 1).Formula = '=' + $c.xlFn + '(' + ($refs -join ',') + ')'
        }
        $ws.Columns.Item(1).ColumnWidth = 100   # avoid ######## column-width artifact on .Text
        $r = 0
        foreach ($c in $caseList) {
            $r++
            $cell = $ws.Cells.Item($r, 1)
            $v2 = $cell.Value2
            # A numeric result comes back as [double]; an Excel error comes back as an
            # int error code (its .Text is the #CODE! string). Type, not .Text, is the
            # reliable discriminator (########, $-signs, thousands separators all fool .Text).
            if ($v2 -is [double]) {
                $map[$c.id] = @{ kind='number'; hex=(Hex-Of-Double ([double]$v2)) }
            } elseif ($null -eq $v2) {
                $map[$c.id] = @{ kind='error'; code='#EMPTY' }
            } else {
                $map[$c.id] = @{ kind='error'; code=([string]$cell.Text) }
            }
        }
        return $map
    }
    finally {
        $wb.Close($false); $excel.Quit()
        [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws)
        [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb)
        [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel)
    }
}

# --- compare / classify -------------------------------------------------------
function Is-Number($d) { $d -and $d.kind -eq 'number' }
function Engine-Digest($d) {
    if (-not $d) { return '(missing)' }
    if ($d.kind -eq 'number') { return $d.hex }
    if ($d.kind -eq 'error') { return 'err:' + $d.code }
    return $d.raw
}
function Matches($a, $b) {
    if (-not $a -or -not $b) { return $false }
    if ((Is-Number $a) -and (Is-Number $b)) { return $a.hex -eq $b.hex }
    if ($a.kind -eq 'error' -and $b.kind -eq 'error') { return $true }   # any error == any error
    return $false
}

Ensure-Parent $LedgerOut
$oxMap = Invoke-OxFunc $cases
$fsMap = if ($SkipFSharp) { @{} } else { Invoke-FSharp $cases }
$exMap = if ($SkipExcel) { @{} } else { Invoke-Excel $cases }

$ledger = foreach ($c in $cases) {
    $ox = $oxMap[$c.id]; $fs = $fsMap[$c.id]; $ex = $exMap[$c.id]
    $oxEx = Matches $ox $ex
    $fsEx = Matches $fs $ex
    $oxFs = Matches $ox $fs
    $class =
        if ($SkipExcel) { 'no_excel' }
        elseif ($oxEx -and $fsEx) { 'all_bit_exact' }
        elseif ($oxEx -and -not $fsEx) { 'ox_exact_fs_off' }
        elseif ($fsEx -and -not $oxEx) { 'fs_exact_ox_off' }
        elseif (-not $oxEx -and -not $fsEx -and $oxFs) { 'both_off_ox_eq_fs' }
        else { 'all_diverge' }
    [pscustomobject]@{
        case_id   = $c.id
        row       = $c.row
        fn        = $c.xlFn
        ox        = Engine-Digest $ox
        fs        = if ($SkipFSharp) { '(skip)' } else { Engine-Digest $fs }
        excel     = if ($SkipExcel) { '(skip)' } else { Engine-Digest $ex }
        ulp_ox_excel = if ((Is-Number $ox) -and (Is-Number $ex)) { Ulp-Distance $ox.hex $ex.hex } else { $null }
        ulp_fs_excel = if ((Is-Number $fs) -and (Is-Number $ex)) { Ulp-Distance $fs.hex $ex.hex } else { $null }
        class     = $class
        residual  = $c.residual
    }
}

$ledger | Export-Csv -Path $LedgerOut -NoTypeInformation -Encoding UTF8
Write-Host "`nLedger -> $LedgerOut" -ForegroundColor Cyan
$ledger | Format-Table case_id, fn, class, ulp_ox_excel, ulp_fs_excel -AutoSize | Out-Host

if (-not $SkipExcel) {
    Write-Host "`nSummary:" -ForegroundColor Cyan
    $ledger | Group-Object class | Sort-Object Name | ForEach-Object {
        Write-Host ("  {0,-20} {1}" -f $_.Name, $_.Count)
    }
}

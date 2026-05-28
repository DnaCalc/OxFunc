[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RunId = "",
    [int]    $Samples = 500
)

# Statistical-profile harness for the stochastic uniform-draw family
# (RAND, RANDBETWEEN, RANDARRAY) — the surfaces that CANNOT be bit-compared
# per draw. This is a DIFFERENT comparison policy from the bit-exact value
# runner:
#
#   * OxFunc's RAND family delegates the actual draw to a host
#     `RandomProvider`; OxFunc owns the *contract* (bounds, shape, integer
#     mapping, argument handling), not the randomness. The local evaluator
#     is driven with the deterministic `fixed_0_5` provider, so its output
#     is a fixed point used to check the contract (in-bounds, correct shape,
#     integer-ness), not to match Excel value-for-value.
#   * Excel's RNG cannot be pinned via COM, so the Excel side is SAMPLED
#     (N draws) and reduced to a profile (bounds, mean, integer fraction,
#     distinct set, spill shape) plus a coarse uniformity check.
#
# The verdict is `statistical_profile_consistent` when both sides satisfy
# the same contract (same bounds, same integer rule, same shape), and the
# Excel sample is plausibly uniform. It is NOT a bit-exact closure claim.
# v0: contract + coarse uniformity. A finer distribution test (KS /
# chi-square against OxFunc driven by a varying provider) is a follow-up.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptPath "..\..")).Path
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = "rand-statistical-profile-" + (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ")
}
$RunRoot = Join-Path $RepoRoot "smart-fuzzer\runs\$RunId"
if (Test-Path $RunRoot) { throw "run directory already exists: $RunRoot" }
[void](New-Item -ItemType Directory -Path $RunRoot -Force)
$RollupPath = Join-Path $RunRoot "rollup.json"

# --------------------------------------------------------------------------
# Excel sampling.
# --------------------------------------------------------------------------
function Get-ExcelRandProfile {
    $excel = $null; $workbook = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        $excel.Visible = $false
        $excel.DisplayAlerts = $false
        $excel.ScreenUpdating = $false
        $workbook = $excel.Workbooks.Add()
        $ws = $workbook.Worksheets.Item(1)
        $env = [ordered]@{ version = [string]$excel.Version; build = [string]$excel.Build }

        # --- RAND(): sample N draws ---
        $randCell = $ws.Range("A1"); $randCell.Formula2 = "=RAND()"
        $randVals = New-Object 'System.Collections.Generic.List[double]'
        for ($i = 0; $i -lt $Samples; $i++) { $ws.Calculate() | Out-Null; $randVals.Add([double]$randCell.Value2) }

        # --- RANDBETWEEN(1,6): sample N draws ---
        $rbCell = $ws.Range("A2"); $rbCell.Formula2 = "=RANDBETWEEN(1,6)"
        $rbVals = New-Object 'System.Collections.Generic.List[double]'
        for ($i = 0; $i -lt $Samples; $i++) { $ws.Calculate() | Out-Null; $rbVals.Add([double]$rbCell.Value2) }

        # --- RANDARRAY(2,3): shape + bounds (sample a few recalcs) ---
        $ws.Cells.Clear() | Out-Null
        $raCell = $ws.Range("A1"); $raCell.Formula2 = "=RANDARRAY(2,3)"; $ws.Calculate() | Out-Null
        $raSpill = $raCell.SpillingToRange
        $raRows = [int]$raSpill.Rows.Count; $raCols = [int]$raSpill.Columns.Count
        $raVals = New-Object 'System.Collections.Generic.List[double]'
        foreach ($c in $raSpill.Cells) { $raVals.Add([double]$c.Value2) }

        # --- RANDARRAY(2,2,1,6,TRUE): whole numbers in [1,6] ---
        $ws.Cells.Clear() | Out-Null
        $rawCell = $ws.Range("A1"); $rawCell.Formula2 = "=RANDARRAY(2,2,1,6,TRUE)"; $ws.Calculate() | Out-Null
        $rawSpill = $rawCell.SpillingToRange
        $rawRows = [int]$rawSpill.Rows.Count; $rawCols = [int]$rawSpill.Columns.Count
        $rawVals = New-Object 'System.Collections.Generic.List[double]'
        foreach ($c in $rawSpill.Cells) { $rawVals.Add([double]$c.Value2) }

        return [ordered]@{
            environment = $env
            rand        = (Get-ScalarProfile $randVals)
            randbetween = (Get-ScalarProfile $rbVals)
            randarray   = [ordered]@{ rows = $raRows; cols = $raCols; min = ($raVals | Measure-Object -Minimum).Minimum; max = ($raVals | Measure-Object -Maximum).Maximum }
            randarray_whole = [ordered]@{ rows = $rawRows; cols = $rawCols; all_integer = (Test-AllInteger $rawVals); min = ($rawVals | Measure-Object -Minimum).Minimum; max = ($rawVals | Measure-Object -Maximum).Maximum }
        }
    } finally {
        if ($null -ne $workbook) { $workbook.Close($false) | Out-Null }
        if ($null -ne $excel) { $excel.Quit() | Out-Null }
        [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($excel) | Out-Null 2>$null
        [GC]::Collect(); [GC]::WaitForPendingFinalizers()
    }
}

function Test-AllInteger { param([object]$Vals) foreach ($v in $Vals) { if ([math]::Floor([double]$v) -ne [double]$v) { return $false } } return $true }

function Get-ScalarProfile {
    param([object]$Vals)
    $arr = @($Vals | ForEach-Object { [double]$_ })
    $n = $arr.Count
    $stats = $arr | Measure-Object -Average -Minimum -Maximum
    # six-bucket uniformity over the observed [min,max] span (coarse).
    $buckets = New-Object 'int[]' 6
    $lo = 0.0; $hi = 1.0
    foreach ($v in $arr) {
        $idx = [int][math]::Floor((($v - $lo) / ($hi - $lo)) * 6)
        if ($idx -lt 0) { $idx = 0 }; if ($idx -gt 5) { $idx = 5 }
        $buckets[$idx] += 1
    }
    $expected = $n / 6.0
    $chi = 0.0; foreach ($b in $buckets) { $chi += (($b - $expected) * ($b - $expected)) / $expected }
    return [ordered]@{
        count = $n
        min = $stats.Minimum
        max = $stats.Maximum
        mean = $stats.Average
        all_integer = (Test-AllInteger $arr)
        distinct = (@($arr | Sort-Object -Unique))
        bucket_counts = $buckets
        chi_square_uniform_6 = [math]::Round($chi, 3)
    }
}

# --------------------------------------------------------------------------
# OxFunc local contract (deterministic, fixed_0_5 provider).
# --------------------------------------------------------------------------
function Get-LocalRandContract {
    $cases = @(
        [ordered]@{ schema_version="oxfunc.smart_fuzzer.scenario_seed_case.v0"; run_id="assigned"; tranche_id="rand-contract"; case_id="rand-rand"; function_id="FUNC.RAND"; canonical_surface_name="RAND"; formula_text="=RAND()"; args=@(); cell_fixture=@(); random_provider="fixed_0_5"; category="Math and trigonometry functions" }
        [ordered]@{ schema_version="oxfunc.smart_fuzzer.scenario_seed_case.v0"; run_id="assigned"; tranche_id="rand-contract"; case_id="rand-randbetween"; function_id="FUNC.RANDBETWEEN"; canonical_surface_name="RANDBETWEEN"; formula_text="=RANDBETWEEN(1,6)"; args=@(([ordered]@{kind="number";value=1}),([ordered]@{kind="number";value=6})); cell_fixture=@(); random_provider="fixed_0_5"; category="Math and trigonometry functions" }
        [ordered]@{ schema_version="oxfunc.smart_fuzzer.scenario_seed_case.v0"; run_id="assigned"; tranche_id="rand-contract"; case_id="rand-randarray"; function_id="FUNC.RANDARRAY"; canonical_surface_name="RANDARRAY"; formula_text="=RANDARRAY(2,3)"; args=@(([ordered]@{kind="number";value=2}),([ordered]@{kind="number";value=3})); cell_fixture=@(); random_provider="fixed_0_5"; category="Math and trigonometry functions" }
        [ordered]@{ schema_version="oxfunc.smart_fuzzer.scenario_seed_case.v0"; run_id="assigned"; tranche_id="rand-contract"; case_id="rand-randarray-whole"; function_id="FUNC.RANDARRAY"; canonical_surface_name="RANDARRAY"; formula_text="=RANDARRAY(2,2,1,6,TRUE)"; args=@(([ordered]@{kind="number";value=2}),([ordered]@{kind="number";value=2}),([ordered]@{kind="number";value=1}),([ordered]@{kind="number";value=6}),([ordered]@{kind="logical";value=$true})); cell_fixture=@(); random_provider="fixed_0_5"; category="Math and trigonometry functions" }
    )
    $caseSet = [ordered]@{ schema_version="oxfunc.smart_fuzzer.scenario_seed_case_set.v0"; cases=$cases }
    $caseDir = Join-Path $RunRoot "cases"; [void](New-Item -ItemType Directory -Path $caseDir -Force)
    $casesPath = Join-Path $caseDir "rand-contract-cases.jsonl"
    $caseSet.cases | ForEach-Object { $_ | ConvertTo-Json -Depth 12 -Compress } | Set-Content -LiteralPath $casesPath -Encoding UTF8
    $localOut = Join-Path $RunRoot "local-contract.jsonl"
    $manifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
    & cargo run --quiet --manifest-path $manifest --bin array_tranche_local_eval -- --cases $casesPath --out $localOut
    if ($LASTEXITCODE -ne 0) { throw "local rand contract eval failed ($LASTEXITCODE)" }
    $byCase = @{}
    foreach ($line in (Get-Content -LiteralPath $localOut)) { if ($line.Trim()) { $o = $line | ConvertFrom-Json; $byCase[[string]$o.case_id] = $o } }
    return $byCase
}

Write-Host "Sampling Excel RAND family ($Samples draws each)..."
$excelProfile = Get-ExcelRandProfile
Write-Host "Running OxFunc local contract (fixed_0_5)..."
$local = Get-LocalRandContract

# --------------------------------------------------------------------------
# Contract verdicts.
# --------------------------------------------------------------------------
$findings = New-Object 'System.Collections.Generic.List[object]'
function Add-Verdict { param([string]$Surface,[string]$Verdict,[string]$Detail) $findings.Add([ordered]@{ surface=$Surface; verdict=$Verdict; detail=$Detail }) | Out-Null }

# RAND: Excel in [0,1), mean ~0.5, plausibly uniform; local fixed point in [0,1).
$r = $excelProfile.rand
$randExcelOk = ($r.min -ge 0) -and ($r.max -lt 1) -and ([math]::Abs($r.mean - 0.5) -lt 0.1) -and ($r.chi_square_uniform_6 -lt 20)
Add-Verdict "RAND" $(if ($randExcelOk) {"statistical_profile_consistent"} else {"statistical_profile_mismatch"}) "excel: n=$($r.count) min=$([math]::Round($r.min,4)) max=$([math]::Round($r.max,4)) mean=$([math]::Round($r.mean,4)) chi6=$($r.chi_square_uniform_6); local(fixed_0_5)=$($local['rand-rand'].outcome.value)"

# RANDBETWEEN(1,6): Excel integers, distinct set subset of {1..6}; local integer in [1,6].
$rb = $excelProfile.randbetween
$rbDistinctOk = ((@($rb.distinct | Where-Object { $_ -lt 1 -or $_ -gt 6 -or ([math]::Floor($_) -ne $_) })).Count -eq 0)
$rbLocalVal = [double]$local['rand-randbetween'].outcome.value
$rbLocalOk = ($rbLocalVal -ge 1 -and $rbLocalVal -le 6 -and [math]::Floor($rbLocalVal) -eq $rbLocalVal)
Add-Verdict "RANDBETWEEN" $(if ($rb.all_integer -and $rbDistinctOk -and $rbLocalOk) {"statistical_profile_consistent"} else {"statistical_profile_mismatch"}) "excel: integers in [$($rb.min),$($rb.max)] distinct=$($rb.distinct -join '/'); local(fixed_0_5)=$rbLocalVal"

# RANDARRAY(2,3): shape 2x3, bounds [0,1); local same shape.
$ra = $excelProfile.randarray
$raLocal = $local['rand-randarray'].outcome
$raLocalShape = if ($raLocal.PSObject.Properties.Name -contains 'rows') { "$($raLocal.rows)x$($raLocal.cols)" } else { "scalar" }
$raOk = ($ra.rows -eq 2 -and $ra.cols -eq 3 -and $ra.min -ge 0 -and $ra.max -lt 1 -and $raLocalShape -eq "2x3")
Add-Verdict "RANDARRAY" $(if ($raOk) {"statistical_profile_consistent"} else {"statistical_profile_mismatch"}) "excel: shape $($ra.rows)x$($ra.cols) bounds [$([math]::Round($ra.min,3)),$([math]::Round($ra.max,3))]; local shape $raLocalShape"

# RANDARRAY(2,2,1,6,TRUE): shape 2x2, integers in [1,6]; local same shape.
$raw = $excelProfile.randarray_whole
$rawLocal = $local['rand-randarray-whole'].outcome
$rawLocalShape = if ($rawLocal.PSObject.Properties.Name -contains 'rows') { "$($rawLocal.rows)x$($rawLocal.cols)" } else { "scalar" }
$rawOk = ($raw.rows -eq 2 -and $raw.cols -eq 2 -and $raw.all_integer -and $raw.min -ge 1 -and $raw.max -le 6 -and $rawLocalShape -eq "2x2")
Add-Verdict "RANDARRAY.whole" $(if ($rawOk) {"statistical_profile_consistent"} else {"statistical_profile_mismatch"}) "excel: shape $($raw.rows)x$($raw.cols) all_int=$($raw.all_integer) in [$($raw.min),$($raw.max)]; local shape $rawLocalShape"

$consistent = (@($findings | Where-Object { $_.verdict -eq "statistical_profile_consistent" })).Count
$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.rand_statistical_profile.v0"
    run_id = $RunId
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    samples_per_function = $Samples
    comparison_policy = "statistical_profile_consistency_not_bit_exact"
    excel_environment = $excelProfile.environment
    excel_profile = $excelProfile
    verdicts = $findings.ToArray()
    summary = [ordered]@{ surfaces = $findings.Count; consistent = $consistent; mismatch = ($findings.Count - $consistent) }
}
$rollup | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $RollupPath -Encoding UTF8

Write-Host ""
Write-Host "RAND statistical-profile verdicts:"
foreach ($f in $findings) { "  {0,-18} {1,-34} {2}" -f $f.surface, $f.verdict, $f.detail }
Write-Host ""
Write-Host "consistent=$consistent / $($findings.Count); rollup=$RollupPath"

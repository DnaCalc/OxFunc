[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string] $TrancheId = "typed-arg-probes-v0"
)

# Synthesize typed-argument probe cases for the harness-blocked
# financial / bond / depreciation / annuity / date / statistical-test /
# unit-conversion surfaces in the function status map.
#
# These landed in `harness_blocked` because the naive numeric-fill probe
# could not form a *valid* call: they need correctly typed arguments
# (date serials in date order, rates, basis codes, redemption/par,
# frequency, unit strings, paired sample arrays). The fix is a curated
# per-surface argument vector drawn from the published (clean-room) Excel
# function signatures, with standard valid values.
#
# The formula_text is DERIVED from the argument vector by a single token
# builder, so the local Rust evaluator (which dispatches on function_id +
# args) and the Excel COM runner (which parses formula_text) always see
# the identical invocation. All literals are short / bit-exact-safe
# (CHARTER §4.1 plumbing rule): date serials are integers, rates are
# short decimals, no long round-trip-risky literals.
#
# A probe that yields the same error on both sides (e.g. both #NUM!) is a
# legitimate match — the goal is structural agreement, not a non-error
# result.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\typed-arg-probes-v0.json"
}

# ---- value constructors ----
function Num { param([double]$V) [ordered]@{ kind = "number"; value = $V } }
function Txt { param([string]$V) [ordered]@{ kind = "text"; value = $V } }
function Lgl { param([bool]$V)   [ordered]@{ kind = "logical"; value = $V } }
function NumRow {
    param([double[]]$Vals)
    $cells = New-Object 'System.Collections.ArrayList'
    foreach ($v in $Vals) { [void]$cells.Add((Num $v)) }
    $rows = New-Object 'System.Collections.ArrayList'
    [void]$rows.Add($cells)
    [ordered]@{ kind = "array"; rows = $rows }
}

# ---- formula token builder (must mirror Excel literal syntax) ----
function Tok {
    param($A)
    switch ([string]$A.kind) {
        "number"  { $s = [string]$A.value; return $s }
        "text"    { return '"' + ([string]$A.value) + '"' }
        "logical" { if ($A.value) { return "TRUE" } else { return "FALSE" } }
        "array"   {
            $rowToks = @()
            foreach ($row in $A.rows) {
                $cellToks = @()
                foreach ($c in $row) { $cellToks += (Tok $c) }
                $rowToks += ($cellToks -join ",")
            }
            return "{" + ($rowToks -join ";") + "}"
        }
        default { return [string]$A.value }
    }
}

# ---- standard valid inputs ----
# Date serials (1900 system), kept in valid order: issue < settlement < maturity.
$Issue   = 43831  # 2020-01-01
$Settle  = 44013  # 2020-07-01
$First   = 44197  # 2021-01-01 (first interest / first period)
$Mature  = 44562  # 2022-01-01
$Rate    = 0.05
$Coupon  = 0.05
$Yld     = 0.06
$Pr      = 95
$Redeem  = 100
$Par     = 1000
$Freq    = 2
$Basis   = 0

# ---- curated per-surface specs: Name, Category, Args[] ----
$specs = @(
    # --- accrued interest ---
    @{ N="ACCRINT";   Cat="Financial functions"; Args=@((Num $Issue),(Num $First),(Num $Settle),(Num $Coupon),(Num $Par),(Num $Freq),(Num $Basis)) }
    @{ N="ACCRINTM";  Cat="Financial functions"; Args=@((Num $Issue),(Num $Settle),(Num $Coupon),(Num $Par),(Num $Basis)) }

    # --- depreciation ---
    @{ N="AMORDEGRC"; Cat="Financial functions"; Args=@((Num 1000),(Num $Issue),(Num $First),(Num 100),(Num 1),(Num 0.15),(Num $Basis)) }
    @{ N="AMORLINC";  Cat="Financial functions"; Args=@((Num 1000),(Num $Issue),(Num $First),(Num 100),(Num 1),(Num 0.15),(Num $Basis)) }

    # --- coupon-day family: (settlement, maturity, frequency, [basis]) ---
    @{ N="COUPDAYBS"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }
    @{ N="COUPDAYS";  Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }
    @{ N="COUPDAYSNC";Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }
    @{ N="COUPNCD";   Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }
    @{ N="COUPNUM";   Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }
    @{ N="COUPPCD";   Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Freq),(Num $Basis)) }

    # --- bond price / yield family ---
    @{ N="PRICE";     Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Coupon),(Num $Yld),(Num $Redeem),(Num $Freq),(Num $Basis)) }
    @{ N="YIELD";     Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Coupon),(Num $Pr),(Num $Redeem),(Num $Freq),(Num $Basis)) }
    @{ N="PRICEDISC"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Rate),(Num $Redeem),(Num $Basis)) }
    @{ N="YIELDDISC"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Pr),(Num $Redeem),(Num $Basis)) }
    @{ N="PRICEMAT";  Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $Coupon),(Num $Yld),(Num $Basis)) }
    @{ N="YIELDMAT";  Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $Coupon),(Num $Pr),(Num $Basis)) }
    @{ N="DISC";      Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Pr),(Num $Redeem),(Num $Basis)) }
    @{ N="INTRATE";   Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num 1000),(Num 1100),(Num $Basis)) }
    @{ N="RECEIVED";  Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num 1000),(Num $Rate),(Num $Basis)) }
    @{ N="DURATION";  Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Coupon),(Num $Yld),(Num $Freq),(Num $Basis)) }
    @{ N="MDURATION"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Coupon),(Num $Yld),(Num $Freq),(Num $Basis)) }

    # --- odd-period bond family: (settlement, maturity, issue, first/last_coupon, rate, yld/pr, redemption, frequency, [basis]) ---
    @{ N="ODDFPRICE"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $First),(Num $Coupon),(Num $Yld),(Num $Redeem),(Num $Freq),(Num $Basis)) }
    @{ N="ODDFYIELD"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $First),(Num $Coupon),(Num $Pr),(Num $Redeem),(Num $Freq),(Num $Basis)) }
    @{ N="ODDLPRICE"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $Coupon),(Num $Yld),(Num $Redeem),(Num $Freq),(Num $Basis)) }
    @{ N="ODDLYIELD"; Cat="Financial functions"; Args=@((Num $Settle),(Num $Mature),(Num $Issue),(Num $Coupon),(Num $Pr),(Num $Redeem),(Num $Freq),(Num $Basis)) }

    # --- annuity / cashflow ---
    @{ N="FV";        Cat="Financial functions"; Args=@((Num $Rate),(Num 10),(Num -100)) }
    @{ N="PV";        Cat="Financial functions"; Args=@((Num $Rate),(Num 10),(Num -100)) }
    @{ N="NPER";      Cat="Financial functions"; Args=@((Num $Rate),(Num -100),(Num 1000)) }
    @{ N="ISPMT";     Cat="Financial functions"; Args=@((Num $Rate),(Num 1),(Num 10),(Num 1000)) }
    @{ N="CUMIPMT";   Cat="Financial functions"; Args=@((Num $Rate),(Num 10),(Num 1000),(Num 1),(Num 10),(Num 0)) }
    @{ N="CUMPRINC";  Cat="Financial functions"; Args=@((Num $Rate),(Num 10),(Num 1000),(Num 1),(Num 10),(Num 0)) }
    @{ N="NPV";       Cat="Financial functions"; Args=@((Num $Rate),(Num 100),(Num 200),(Num 300)) }
    @{ N="MIRR";      Cat="Financial functions"; Args=@((NumRow @(-1000,200,300,400)),(Num $Rate),(Num 0.06)) }
    @{ N="FVSCHEDULE";Cat="Financial functions"; Args=@((Num 1000),(NumRow @(0.05,0.06,0.07))) }
    @{ N="XIRR";      Cat="Financial functions"; Args=@((NumRow @(-1000,500,600)),(NumRow @($Issue,$Settle,$Mature))) }
    @{ N="XNPV";      Cat="Financial functions"; Args=@((Num $Rate),(NumRow @(-1000,500,600)),(NumRow @($Issue,$Settle,$Mature))) }
    @{ N="RRI";       Cat="Financial functions"; Args=@((Num 10),(Num 1000),(Num 2000)) }
    @{ N="PDURATION"; Cat="Financial functions"; Args=@((Num $Rate),(Num 1000),(Num 2000)) }
    @{ N="EFFECT";    Cat="Financial functions"; Args=@((Num $Rate),(Num 4)) }
    @{ N="NOMINAL";   Cat="Financial functions"; Args=@((Num $Rate),(Num 4)) }

    # --- date ---
    @{ N="DATEDIF";   Cat="Date and time functions"; Args=@((Num $Issue),(Num $Mature),(Txt "Y")) }
    @{ N="DAYS360";   Cat="Date and time functions"; Args=@((Num $Issue),(Num $Mature),(Lgl $false)) }

    # --- statistical paired-sample tests ---
    @{ N="CHISQ.TEST";Cat="Statistical functions"; Args=@((NumRow @(10,20,30)),(NumRow @(12,18,30))) }
    @{ N="CHITEST";   Cat="Statistical functions"; Args=@((NumRow @(10,20,30)),(NumRow @(12,18,30))) }
    @{ N="F.TEST";    Cat="Statistical functions"; Args=@((NumRow @(1,2,3,4)),(NumRow @(2,4,6,8))) }
    @{ N="FTEST";     Cat="Statistical functions"; Args=@((NumRow @(1,2,3,4)),(NumRow @(2,4,6,8))) }
    @{ N="T.TEST";    Cat="Statistical functions"; Args=@((NumRow @(1,2,3,4)),(NumRow @(2,3,4,5)),(Num 2),(Num 1)) }
    @{ N="TTEST";     Cat="Statistical functions"; Args=@((NumRow @(1,2,3,4)),(NumRow @(2,3,4,5)),(Num 2),(Num 1)) }
    @{ N="ZTEST";     Cat="Statistical functions"; Args=@((NumRow @(1,2,3,4,5)),(Num 3)) }
    @{ N="WEIBULL";   Cat="Statistical functions"; Args=@((Num 2),(Num 1),(Num 1),(Lgl $true)) }
    @{ N="WEIBULL.DIST";Cat="Statistical functions"; Args=@((Num 2),(Num 1),(Num 1),(Lgl $true)) }

    # --- unit conversion ---
    @{ N="CONVERT";   Cat="Engineering functions"; Args=@((Num 1),(Txt "m"),(Txt "ft")) }

    # --- ratio (Excel 2024: PERCENTOF(subset, total) = sum(subset)/sum(total)) ---
    @{ N="PERCENTOF"; Cat="Statistical functions"; Args=@((NumRow @(1,2)),(NumRow @(1,2,3,4))) }
)

$cases = New-Object 'System.Collections.Generic.List[object]'
$seq = 0
foreach ($spec in $specs) {
    $seq += 1
    $toks = @(); foreach ($a in $spec.Args) { $toks += (Tok $a) }
    $formula = "=$($spec.N)(" + ($toks -join ",") + ")"
    $caseId = ("typedarg-{0:D5}-{1}" -f $seq, ($spec.N.ToLower() -replace '[^a-z0-9]','_'))
    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = $TrancheId
        case_id = $caseId
        function_id = "FUNC.$($spec.N)"
        canonical_surface_name = $spec.N
        case_tag = "typed_arg_probe:baseline"
        axis = "typed_arg_probe"
        expected_probe_class = "typed_value_call"
        formula_text = $formula
        args = @($spec.Args)
        cell_fixture = @()
        formula_cell = $null
        category = $spec.Cat
        blocked_or_deferred_lanes = @()
        known_deviation_tags = @()
    }
    $cases.Add($case) | Out-Null
}

$surfaces = (@($cases | ForEach-Object { $_.canonical_surface_name } | Sort-Object -Unique))
$out = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_case_set.v0"
    authority = "non_semantic_exploration_input"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    generator = "Build-TypedArgProbes.ps1"
    tranche_id = $TrancheId
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases.ToArray()
    tranches = @([ordered]@{ tranche_id = $TrancheId; case_ids = @($cases | ForEach-Object { $_.case_id }) })
    skipped = @()
    summary = [ordered]@{ case_count = $cases.Count; surfaces_covered = $surfaces.Count }
}
$out | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Cases:    $($cases.Count)"
Write-Host "Surfaces: $($surfaces.Count)"
Write-Host "Output:   $OutputPath"

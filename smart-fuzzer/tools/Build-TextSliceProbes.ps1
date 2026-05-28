[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string] $TrancheId = "text-slice-probes-v0"
)

# Synthesize text-slice probe cases for the harness-blocked
# search / position / extract / replace / split text surfaces.
#
# These landed in `harness_blocked` because the naive numeric-fill probe
# fed a number where a text or position argument was required
# (e.g. FIND(within_text=2) is invalid). The fix is a curated per-surface
# argument vector with a real text payload and valid position/pattern
# arguments, drawn from the published (clean-room) Excel signatures.
#
# formula_text is DERIVED from the argument vector by one token builder so
# the local Rust evaluator and the Excel COM runner see the identical
# invocation. All literals are short text / small integers (bit-exact-safe).

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\text-slice-probes-v0.json"
}

function Num { param([double]$V) [ordered]@{ kind = "number"; value = $V } }
function Txt { param([string]$V) [ordered]@{ kind = "text"; value = $V } }
function Lgl { param([bool]$V)   [ordered]@{ kind = "logical"; value = $V } }

function Tok {
    param($A)
    switch ([string]$A.kind) {
        "number"  { return [string]$A.value }
        "text"    { return '"' + ([string]$A.value) + '"' }
        "logical" { if ($A.value) { return "TRUE" } else { return "FALSE" } }
        default   { return [string]$A.value }
    }
}

# Curated per-surface specs. Each spec: Name, Category, Tag, Args[].
$specs = @(
    # --- find / search (find_text, within_text, [start_num]) ---
    @{ N="FIND";    Tag="char_in_text";   Args=@((Txt "b"),(Txt "abcde")) }
    @{ N="FINDB";   Tag="char_in_text";   Args=@((Txt "b"),(Txt "abcde")) }
    @{ N="SEARCH";  Tag="ci_char_in_text";Args=@((Txt "B"),(Txt "abcde")) }
    @{ N="SEARCHB"; Tag="ci_char_in_text";Args=@((Txt "B"),(Txt "abcde")) }
    @{ N="FIND";    Tag="not_found_na";   Args=@((Txt "z"),(Txt "abcde")) }

    # --- slice (text, start, num) / (text, num) ---
    @{ N="MID";     Tag="middle";  Args=@((Txt "abcde"),(Num 2),(Num 3)) }
    @{ N="MIDB";    Tag="middle";  Args=@((Txt "abcde"),(Num 2),(Num 3)) }
    @{ N="LEFT";    Tag="prefix";  Args=@((Txt "abcde"),(Num 2)) }
    @{ N="LEFTB";   Tag="prefix";  Args=@((Txt "abcde"),(Num 2)) }
    @{ N="RIGHT";   Tag="suffix";  Args=@((Txt "abcde"),(Num 2)) }
    @{ N="RIGHTB";  Tag="suffix";  Args=@((Txt "abcde"),(Num 2)) }

    # --- length ---
    @{ N="LEN";     Tag="length";  Args=@((Txt "abcde")) }
    @{ N="LENB";    Tag="length";  Args=@((Txt "abcde")) }

    # --- replace (old, start, num, new) ---
    @{ N="REPLACE"; Tag="mid_replace"; Args=@((Txt "abcde"),(Num 2),(Num 2),(Txt "XY")) }
    @{ N="REPLACEB";Tag="mid_replace"; Args=@((Txt "abcde"),(Num 2),(Num 2),(Txt "XY")) }

    # --- regex family (text, pattern, ...) — Excel 2024+ ---
    @{ N="REGEXTEST";    Tag="digits";  Args=@((Txt "abc123"),(Txt "\d+")) }
    @{ N="REGEXEXTRACT"; Tag="digits";  Args=@((Txt "abc123"),(Txt "\d+")) }
    @{ N="REGEXREPLACE"; Tag="digits";  Args=@((Txt "abc123"),(Txt "\d+"),(Txt "X")) }

    # --- text after/before/split (text, delimiter) ---
    @{ N="TEXTAFTER";  Tag="delim"; Args=@((Txt "a-b-c"),(Txt "-")) }
    @{ N="TEXTBEFORE"; Tag="delim"; Args=@((Txt "a-b-c"),(Txt "-")) }
    @{ N="TEXTSPLIT";  Tag="delim"; Args=@((Txt "a,b,c"),(Txt ",")) }
)

$cases = New-Object 'System.Collections.Generic.List[object]'
$seq = 0
foreach ($spec in $specs) {
    $seq += 1
    $toks = @(); foreach ($a in $spec.Args) { $toks += (Tok $a) }
    $formula = "=$($spec.N)(" + ($toks -join ",") + ")"
    $caseId = ("textslice-{0:D5}-{1}-{2}" -f $seq, ($spec.N.ToLower() -replace '[^a-z0-9]','_'), $spec.Tag)
    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = $TrancheId
        case_id = $caseId
        function_id = "FUNC.$($spec.N)"
        canonical_surface_name = $spec.N
        case_tag = "text_slice_probe:$($spec.Tag)"
        axis = "text_slice_probe"
        expected_probe_class = "typed_text_call"
        formula_text = $formula
        args = @($spec.Args)
        cell_fixture = @()
        formula_cell = $null
        category = "Text functions"
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
    generator = "Build-TextSliceProbes.ps1"
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

[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string] $TrancheId = "reference-operator-probes-v0"
)

# Probe the reference OPERATORS (the value-comparison-floor remainder of
# the unswept operator set):
#   OP_RANGE_REF (':'), OP_INTERSECTION_REF (' '),
#   OP_SPILL_REF ('#'), OP_IMPLICIT_INTERSECTION ('@').
#
# These dispatch as FUNC.OP_* value calls taking reference args. Per the
# Rust surface-dispatch tests, they CONSTRUCT / COMBINE references and
# return an `EvalValue::Reference` (e.g. OP_RANGE_REF(B2,A1) -> Area
# "A1:B2"); they do not materialise to a value. Excel, evaluating the same
# infix form in a cell, auto-materialises (spill / implicit intersection).
# So the value-comparison harness is expected to show local=reference vs
# Excel=materialised-value — the reference-materialisation boundary, the
# same family as OFFSET / XLOOKUP (Tranche B §3.3 / task #8). This probe
# confirms that empirically rather than assuming it.
#
# OP_TRIM_REF_{LEADING,TRAILING,BOTH} use the newest range-trim syntax and
# a spill/host context; they are recorded as deferred (newest-syntax +
# host) rather than probed blindly here.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\reference-operator-probes-v0.json"
}

function Num { param([double]$V) [ordered]@{ kind = "number"; value = $V } }
function Ref { param([string]$Kind, [string]$Target) [ordered]@{ kind = "reference"; reference_kind = $Kind; target = $Target } }
function ColNums {
    param([double[]]$Vals, [string]$Target)
    $rows = New-Object 'System.Collections.ArrayList'
    foreach ($v in $Vals) { $cells = New-Object 'System.Collections.ArrayList'; [void]$cells.Add((Num $v)); [void]$rows.Add($cells) }
    [ordered]@{ target = $Target; value = [ordered]@{ kind = "array"; rows = $rows } }
}

# A1:A4 column vector [1;2;3;4] for range / intersection / spill / II probes.
$colFix = (ColNums @(1,2,3,4) "A1:A4")

$specs = @(
    @{ N="OP_RANGE_REF";            Tag="range_colon";      F="=A1:A4";    Args=@((Ref "A1" "A1"),(Ref "A1" "A4")); Fix=@($colFix) }
    @{ N="OP_INTERSECTION_REF";     Tag="space_intersect";  F="=A1:A4 A3:A4"; Args=@((Ref "Area" "A1:A4"),(Ref "Area" "A3:A4")); Fix=@($colFix) }
    @{ N="OP_IMPLICIT_INTERSECTION";Tag="at_implicit";      F="=@A1:A4";   Args=@((Ref "Area" "A1:A4")); Fix=@($colFix) }
    @{ N="OP_SPILL_REF";            Tag="hash_spill";       F="=A1#";      Args=@((Ref "A1" "A1")); Fix=@($colFix) }
)

$cases = New-Object 'System.Collections.Generic.List[object]'
$seq = 0
foreach ($spec in $specs) {
    $seq += 1
    $caseId = ("refop-{0:D5}-{1}-{2}" -f $seq, ($spec.N.ToLower() -replace '[^a-z0-9]','_'), $spec.Tag)
    $cases.Add([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = $TrancheId
        case_id = $caseId
        function_id = "FUNC.$($spec.N)"
        canonical_surface_name = $spec.N
        case_tag = "reference_operator_probe:$($spec.Tag)"
        axis = "reference_operator_probe"
        expected_probe_class = "reference_operator_call"
        formula_text = $spec.F
        args = @($spec.Args)
        cell_fixture = @($spec.Fix)
        formula_cell = $null
        category = "Operators"
        blocked_or_deferred_lanes = @()
        known_deviation_tags = @()
    }) | Out-Null
}

$out = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_case_set.v0"
    authority = "non_semantic_exploration_input"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    generator = "Build-ReferenceOperatorProbes.ps1"
    tranche_id = $TrancheId
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases.ToArray()
    tranches = @([ordered]@{ tranche_id = $TrancheId; case_ids = @($cases | ForEach-Object { $_.case_id }) })
    skipped = @(
        [ordered]@{ name = "OP_TRIM_REF_LEADING";  reason = "newest range-trim syntax + spill/host context; deferred" }
        [ordered]@{ name = "OP_TRIM_REF_TRAILING"; reason = "newest range-trim syntax + spill/host context; deferred" }
        [ordered]@{ name = "OP_TRIM_REF_BOTH";     reason = "newest range-trim syntax + spill/host context; deferred" }
    )
    summary = [ordered]@{ case_count = $cases.Count; surfaces_covered = $cases.Count }
}
$out | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Cases: $($cases.Count)  Output: $OutputPath"

[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath
)

# Structural-axis probes for the scalar/value operators (OP_*), which the
# function-probe generator skips (entry_kind = built_in_operator). Excel
# evaluates these as infix/prefix/postfix; OxFunc dispatches them as
# FUNC.OP_* value calls. Each probe emits an equivalent (formula_text,
# args) pair.
#
# Reference operators (OP_RANGE_REF ':', OP_INTERSECTION_REF ' ',
# OP_UNION_REF ',', OP_SPILL_REF '#', OP_TRIM_REF_*, OP_IMPLICIT_INTERSECTION
# '@') are NOT covered here — they need a reference-fixture generator.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\operator-structural-probes-v0.json"
}

# Operator table: name, arity kind, infix symbol.
$ops = @(
    @{ name = "OP_ADD";           kind = "binary";        sym = "+"  },
    @{ name = "OP_SUBTRACT";      kind = "binary";        sym = "-"  },
    @{ name = "OP_MULTIPLY";      kind = "binary";        sym = "*"  },
    @{ name = "OP_DIVIDE";        kind = "binary";        sym = "/"  },
    @{ name = "OP_POWER";         kind = "binary";        sym = "^"  },
    @{ name = "OP_CONCAT";        kind = "binary";        sym = "&"  },
    @{ name = "OP_EQUAL";         kind = "binary";        sym = "="  },
    @{ name = "OP_NOT_EQUAL";     kind = "binary";        sym = "<>" },
    @{ name = "OP_GREATER_THAN";  kind = "binary";        sym = ">"  },
    @{ name = "OP_GREATER_EQUAL"; kind = "binary";        sym = ">=" },
    @{ name = "OP_LESS_THAN";     kind = "binary";        sym = "<"  },
    @{ name = "OP_LESS_EQUAL";    kind = "binary";        sym = "<=" },
    @{ name = "OP_NEGATE";        kind = "unary_prefix";  sym = "-"  },
    @{ name = "OP_UNARY_PLUS";    kind = "unary_prefix";  sym = "+"  },
    @{ name = "OP_PERCENT";       kind = "unary_postfix"; sym = "%"  }
)

function New-NumberArg  { param([double]$V) return [ordered]@{ kind = "number"; value = $V } }
$arrayArg   = [ordered]@{ kind = "array"; rows = @(@([ordered]@{kind="number";value=2}), @([ordered]@{kind="number";value=3})) }
$arrayArgB  = [ordered]@{ kind = "array"; rows = @(@([ordered]@{kind="number";value=4}), @([ordered]@{kind="number";value=5})) }
$errArg     = [ordered]@{ kind = "error"; code = "NA" }
$textNumArg = [ordered]@{ kind = "text"; value = "2" }
$logicalArg = [ordered]@{ kind = "logical"; value = $true }

function Token {
    param($Arg)
    switch ([string]$Arg.kind) {
        "number"  { return [string]$Arg.value }
        "text"    { return '"' + ([string]$Arg.value) + '"' }
        "logical" { if ($Arg.value) { return "TRUE" } else { return "FALSE" } }
        "error"   { return "NA()" }
        "array"   {
            $rowsT = @()
            foreach ($row in $Arg.rows) { $cellT = @(); foreach ($c in $row) { $cellT += [string]$c.value }; $rowsT += ($cellT -join ",") }
            return "{" + ($rowsT -join ";") + "}"
        }
    }
}

function New-Formula {
    param([string]$OpKind, [string]$Sym, $A0, $A1)
    switch ($OpKind) {
        "binary"        { return "=" + (Token $A0) + $Sym + (Token $A1) }
        "unary_prefix"  { return "=" + $Sym + (Token $A0) }
        "unary_postfix" { return "=" + (Token $A0) + $Sym }
    }
}

$baseN = New-NumberArg 2
$baseM = New-NumberArg 3

$cases = New-Object 'System.Collections.Generic.List[object]'
$caseIds = New-Object 'System.Collections.Generic.List[string]'
$seq = 0

foreach ($op in $ops) {
    $isBinary = ($op.kind -eq "binary")

    # Each probe: tag, arg0 override, arg1 override ($null = baseline).
    $probes = New-Object 'System.Collections.Generic.List[object]'
    [void]$probes.Add(@{ tag = "baseline_scalar"; a0 = $null;      a1 = $null })
    [void]$probes.Add(@{ tag = "arg0_array_lift"; a0 = $arrayArg;  a1 = $null })
    [void]$probes.Add(@{ tag = "arg0_error_na";   a0 = $errArg;    a1 = $null })
    [void]$probes.Add(@{ tag = "arg0_text_number";a0 = $textNumArg;a1 = $null })
    [void]$probes.Add(@{ tag = "arg0_logical";    a0 = $logicalArg;a1 = $null })
    if ($isBinary) {
        [void]$probes.Add(@{ tag = "both_array_elementwise"; a0 = $arrayArg; a1 = $arrayArgB })
        [void]$probes.Add(@{ tag = "arg1_error_na";          a0 = $null;     a1 = $errArg })
    }

    foreach ($probe in $probes) {
        $a0 = if ($null -ne $probe.a0) { $probe.a0 } else { $baseN }
        $a1 = if ($null -ne $probe.a1) { $probe.a1 } else { $baseM }
        if ($isBinary) {
            $argVec = @($a0, $a1)
            $formula = New-Formula $op.kind $op.sym $a0 $a1
        } else {
            $argVec = @($a0)
            $formula = New-Formula $op.kind $op.sym $a0 $null
        }
        $seq += 1
        $caseId = ("operator-{0:D4}-{1}-{2}" -f $seq, ($op.name.ToLower()), $probe.tag)
        $cases.Add([ordered]@{
            schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
            run_id = "assigned_by_runner"
            tranche_id = "operator-structural-probes-v0"
            case_id = $caseId
            function_id = "FUNC.$($op.name)"
            canonical_surface_name = $op.name
            case_tag = "operator_probe:$($probe.tag)"
            axis = "operator_structural_probe"
            expected_probe_class = "operator_structural_axis"
            formula_text = $formula
            args = $argVec
            category = "Operators"
            blocked_or_deferred_lanes = @()
            known_deviation_tags = @()
        }) | Out-Null
        $caseIds.Add($caseId) | Out-Null
    }
}

$out = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_case_set.v0"
    authority = "non_semantic_exploration_input"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    generator = "Build-OperatorStructuralProbes.ps1"
    tranche_id = "operator-structural-probes-v0"
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases.ToArray()
    tranches = @([ordered]@{ tranche_id = "operator-structural-probes-v0"; case_ids = $caseIds.ToArray() })
    skipped = @()
    summary = [ordered]@{
        case_count = $cases.Count
        operator_count = $ops.Count
        note = "scalar/value operators only; reference operators need a fixture-aware generator"
    }
}
$out | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Operators: $($ops.Count)"
Write-Host "Cases:     $($cases.Count)"
Write-Host "Output:    $OutputPath"

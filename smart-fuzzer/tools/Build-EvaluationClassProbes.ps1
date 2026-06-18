[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string] $TrancheId = "evaluation-class-probes-v0"
)

# Category-2 evaluation-class probe generator (ODR-FN-002 / W104).
#
# These probes target the *evaluation-class* axis of context-free invocations:
# kind, coercion, and error shape/code — the structural behavior that is a bug
# under CHARTER §4.1 when it diverges from Excel, independent of bit-exactness.
#
#   1. coercion:logical_to_number   — TRUE/FALSE coerced as direct numeric args
#   2. coercion:text_to_number      — numeric-looking text coerced as direct args
#   3. coercion:array_no_coerce     — inline-array logical/text NOT coerced (contrast)
#   4. error:generation             — the function itself must produce the error code
#   5. error:propagation            — an error argument must propagate / be handled
#   6. kind:returned_value          — returned-value kind (number vs text vs logical)
#   7. shape:scalar_array_lift      — scalar function lifted over an inline array
#
# Output is the existing `oxfunc.smart_fuzzer.scenario_seed_case_set.v0` case-set
# consumed by Run-ArraySupportTranche.ps1, which runs the Rust local evaluator
# (array_tranche_local_eval) and the Excel COM runner over the identical
# invocation. The local side dispatches on function_id + args; the Excel side
# parses formula_text. The single token builder below derives formula_text from
# args so both sides see the same call.
#
# All literals are short / bit-exact-safe (CHARTER §4.1 plumbing rule); none need
# a cell fixture. Blank-cell coercion and bare-operator forms (=1+TRUE) need a
# reference fixture or operator surface and are deferred to a follow-up generator.
#
# A probe that yields the same error on both sides (e.g. both #NUM!) is a match —
# the goal is structural agreement, not a non-error result.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\evaluation-class-probes-v0.json"
}

# ---- value constructors ----
function Num { param([double]$V) [ordered]@{ kind = "number"; value = $V } }
function Txt { param([string]$V) [ordered]@{ kind = "text"; value = $V } }
function Lgl { param([bool]$V)   [ordered]@{ kind = "logical"; value = $V } }
# Error argument. Code is the symbolic WorksheetErrorCode name (Div0, Value, Num,
# NA, Ref, Name, Null) — accepted by the Rust evaluator and mapped to an Excel
# error literal by the token builder below.
function ErrV { param([string]$Code) [ordered]@{ kind = "error"; code = $Code } }
# Generic single-row inline array of already-constructed typed cells.
# Note: PowerShell variables are case-insensitive, so the local list must NOT be
# named $cells (it would alias the $Cells parameter and wipe it before the loop).
function Row {
    param([object[]]$Cells)
    $cellList = New-Object 'System.Collections.ArrayList'
    foreach ($c in $Cells) { [void]$cellList.Add($c) }
    $rowList = New-Object 'System.Collections.ArrayList'
    [void]$rowList.Add($cellList)
    [ordered]@{ kind = "array"; rows = $rowList }
}

# ---- error code -> Excel error literal ----
$ErrLiteral = @{
    "Div0"  = "#DIV/0!"; "Value" = "#VALUE!"; "Num" = "#NUM!"; "NA" = "#N/A"
    "Ref"   = "#REF!";   "Name"  = "#NAME?";  "Null" = "#NULL!"
}

# ---- formula token builder (must mirror Excel literal syntax) ----
function Tok {
    param($A)
    switch ([string]$A.kind) {
        "number"  { return [string]$A.value }
        "text"    { return '"' + ([string]$A.value) + '"' }
        "logical" { if ($A.value) { return "TRUE" } else { return "FALSE" } }
        "error"   {
            $lit = $ErrLiteral[[string]$A.code]
            if ($null -eq $lit) { throw "unmapped error code: $($A.code)" }
            return $lit
        }
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

# ---- curated probe specs: Name, Cat, Tag, Args[], Expected (Excel, doc only) ----
$specs = @(
    # --- coercion: logical -> number (direct args ARE coerced) ---
    @{ N="SUM";     Cat="Math and trigonometry functions"; Tag="coercion:logical_to_number"; Args=@((Lgl $true),(Lgl $true));            Expected="2" }
    @{ N="SUM";     Cat="Math and trigonometry functions"; Tag="coercion:logical_to_number"; Args=@((Num 1),(Lgl $true));                Expected="2" }
    @{ N="PRODUCT"; Cat="Math and trigonometry functions"; Tag="coercion:logical_to_number"; Args=@((Lgl $true),(Num 3));                Expected="3" }
    @{ N="N";       Cat="Information functions";            Tag="coercion:logical_to_number"; Args=@((Lgl $true));                        Expected="1" }

    # --- coercion: numeric-looking text -> number (direct args ARE coerced) ---
    # NOTE: VALUE("2") is deliberately NOT here. The first local run of this lane
    # showed VALUE returns #VALUE! locally because value_fn.rs parses text through a
    # LocaleFormatContext the local harness does not supply (W082 locale-format
    # seam). VALUE is therefore locale-context-sensitive -> Category 1, not 2; it
    # would read as a false Excel mismatch in this set. SUM("2",3) below covers the
    # argument-prep text coercion that IS context-free.
    @{ N="SUM";     Cat="Math and trigonometry functions"; Tag="coercion:text_to_number"; Args=@((Txt "2"),(Num 3));                     Expected="5" }
    @{ N="PRODUCT"; Cat="Math and trigonometry functions"; Tag="coercion:text_to_number"; Args=@((Txt "2"),(Txt "3"));                   Expected="6" }
    @{ N="N";       Cat="Information functions";            Tag="coercion:text_to_number"; Args=@((Txt "2"));                            Expected="0 (N of text is 0)" }

    # --- coercion contrast: inline-array logical/text are NOT coerced by SUM ---
    @{ N="SUM";     Cat="Math and trigonometry functions"; Tag="coercion:array_no_coerce"; Args=@((Row @((Num 1),(Lgl $true))));         Expected="1 (array TRUE ignored)" }
    @{ N="SUM";     Cat="Math and trigonometry functions"; Tag="coercion:array_no_coerce"; Args=@((Row @((Txt "2"),(Num 3))));          Expected="3 (array text ignored)" }

    # --- error generation: function itself must yield the error code ---
    @{ N="SQRT";    Cat="Math and trigonometry functions"; Tag="error:generation"; Args=@((Num -1));                                     Expected="#NUM!" }
    @{ N="ABS";     Cat="Math and trigonometry functions"; Tag="error:generation"; Args=@((Txt "x"));                                    Expected="#VALUE!" }
    @{ N="MOD";     Cat="Math and trigonometry functions"; Tag="error:generation"; Args=@((Num 1),(Num 0));                              Expected="#DIV/0!" }
    @{ N="CHOOSE";  Cat="Lookup and reference functions";  Tag="error:generation"; Args=@((Num 0),(Num 1));                             Expected="#VALUE!" }
    @{ N="LOG";     Cat="Math and trigonometry functions"; Tag="error:generation"; Args=@((Num -1));                                     Expected="#NUM!" }

    # --- error propagation / handling: an error argument flows through ---
    @{ N="SUM";       Cat="Math and trigonometry functions"; Tag="error:propagation"; Args=@((ErrV "NA"),(Num 1));                       Expected="#N/A" }
    @{ N="IFERROR";   Cat="Logical functions";               Tag="error:propagation"; Args=@((ErrV "Div0"),(Num 99));                    Expected="99" }
    @{ N="ISERROR";   Cat="Information functions";            Tag="error:propagation"; Args=@((ErrV "Value"));                           Expected="TRUE" }
    @{ N="ERROR.TYPE";Cat="Information functions";            Tag="error:propagation"; Args=@((ErrV "Num"));                             Expected="6" }
    @{ N="N";         Cat="Information functions";            Tag="error:propagation"; Args=@((ErrV "NA"));                              Expected="#N/A" }

    # --- kind: returned-value kind under IF / type functions ---
    @{ N="IF";       Cat="Logical functions";     Tag="kind:returned_value"; Args=@((Lgl $true),(Num 1),(Txt "a"));                      Expected="1 (number)" }
    @{ N="IF";       Cat="Logical functions";     Tag="kind:returned_value"; Args=@((Lgl $false),(Num 1),(Txt "a"));                     Expected="a (text)" }
    @{ N="T";        Cat="Text functions";        Tag="kind:returned_value"; Args=@((Num 1));                                            Expected="empty text (T of number)" }
    @{ N="ISNUMBER"; Cat="Information functions";  Tag="kind:returned_value"; Args=@((Num 1));                                            Expected="TRUE" }

    # --- shape: scalar function lifted over an inline array ---
    @{ N="ABS";      Cat="Math and trigonometry functions"; Tag="shape:scalar_array_lift"; Args=@((Row @((Num -1),(Num -2))));           Expected="{1,2}" }
    @{ N="SQRT";     Cat="Math and trigonometry functions"; Tag="shape:scalar_array_lift"; Args=@((Row @((Num 4),(Num 9))));            Expected="{2,3}" }
)

$cases = New-Object 'System.Collections.Generic.List[object]'
$seq = 0
foreach ($spec in $specs) {
    $seq += 1
    $toks = @(); foreach ($a in $spec.Args) { $toks += (Tok $a) }
    $formula = "=$($spec.N)(" + ($toks -join ",") + ")"
    $tagSlug = ($spec.Tag -replace '[^a-z0-9]','_')
    $caseId = ("evalclass-{0:D5}-{1}-{2}" -f $seq, ($spec.N.ToLower() -replace '[^a-z0-9]','_'), $tagSlug)
    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
        run_id = "assigned_by_runner"
        tranche_id = $TrancheId
        case_id = $caseId
        function_id = "FUNC.$($spec.N)"
        canonical_surface_name = $spec.N
        case_tag = $spec.Tag
        axis = "evaluation_class_probe"
        expected_probe_class = "typed_value_call"
        formula_text = $formula
        args = @($spec.Args)
        cell_fixture = @()
        formula_cell = $null
        category = $spec.Cat
        expected_excel_note = $spec.Expected
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
    generator = "Build-EvaluationClassProbes.ps1"
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

param(
    [string]$InventoryPath = "smart-fuzzer/cache/dimension-inventory-v0.json",
    [string]$OutputPath = "smart-fuzzer/cache/scenario-seed-executable-cases-v0.json",
    [int]$MaxSeedsPerSurface = 5,
    [int]$MaxTotalCases = 0,
    [switch]$IncludeBlocked,
    [switch]$IncludeKnownDeviations
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptPath "..\..")
$RepoRoot = $RepoRoot.Path
Set-Location $RepoRoot

$Invariant = [System.Globalization.CultureInfo]::InvariantCulture

function New-JsonArray {
    param([object[]]$Items)
    $list = New-Object 'System.Collections.Generic.List[object]'
    foreach ($item in $Items) {
        [void]$list.Add($item)
    }
    return $list
}

function New-NumberValue {
    param([double]$Value)
    return [ordered]@{ kind = "number"; value = $Value }
}

function New-TextValue {
    param([string]$Value)
    return [ordered]@{ kind = "text"; value = $Value }
}

function New-LogicalValue {
    param([bool]$Value)
    return [ordered]@{ kind = "logical"; value = $Value }
}

function New-ErrorValue {
    param([string]$Code)
    return [ordered]@{ kind = "error"; code = $Code }
}

function New-MissingValue {
    return [ordered]@{ kind = "missing_arg" }
}

function New-ArrayValue {
    param($Rows)
    $rowList = New-Object 'System.Collections.Generic.List[object]'
    foreach ($row in $Rows) {
        [void]$rowList.Add($row)
    }
    return [ordered]@{ kind = "array"; rows = $rowList }
}

function Add-Skip {
    param(
        [Parameter(Mandatory = $true)]$Skipped,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reason,
        [string]$SurfaceId = "",
        [string]$Category = ""
    )
    [void]$Skipped.Add([ordered]@{
        surface_id = $SurfaceId
        name = $Name
        category = $Category
        reason = $Reason
    })
}

function Get-ValueField {
    param(
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Name
    )
    if ($Value -is [System.Collections.IDictionary]) {
        Write-Output -NoEnumerate $Value[$Name]
        return
    }
    Write-Output -NoEnumerate $Value.PSObject.Properties[$Name].Value
}

function Test-NameBoundary {
    param([string]$Text, [int]$Index, [int]$Length)
    if ($Index -gt 0) {
        $before = $Text[$Index - 1]
        if ($before -match '[A-Za-z0-9_.]') {
            return $false
        }
    }
    $after = $Index + $Length
    if ($after -ge $Text.Length -or $Text[$after] -ne '(') {
        return $false
    }
    return $true
}

function Find-TargetCallArgs {
    param(
        [string]$Formula,
        [string]$FunctionName
    )
    $text = $Formula.Trim()
    if ($text.StartsWith("=")) {
        $text = $text.Substring(1)
    }

    $nameLength = $FunctionName.Length
    $inString = $false
    for ($i = 0; $i -le $text.Length - $nameLength - 1; $i += 1) {
        $ch = $text[$i]
        if ($ch -eq '"') {
            if ($inString -and $i + 1 -lt $text.Length -and $text[$i + 1] -eq '"') {
                $i += 1
                continue
            }
            $inString = -not $inString
            continue
        }
        if ($inString) {
            continue
        }
        if ([string]::Compare($text, $i, $FunctionName, 0, $nameLength, $true, $Invariant) -ne 0) {
            continue
        }
        if (-not (Test-NameBoundary $text $i $nameLength)) {
            continue
        }

        $open = $i + $nameLength
        $depth = 0
        $braceDepth = 0
        $innerStart = $open + 1
        for ($j = $open; $j -lt $text.Length; $j += 1) {
            $cj = $text[$j]
            if ($cj -eq '"') {
                if ($inString -and $j + 1 -lt $text.Length -and $text[$j + 1] -eq '"') {
                    $j += 1
                    continue
                }
                $inString = -not $inString
                continue
            }
            if ($inString) {
                continue
            }
            switch ($cj) {
                '(' { $depth += 1 }
                ')' {
                    $depth -= 1
                    if ($depth -eq 0 -and $braceDepth -eq 0) {
                        return $text.Substring($innerStart, $j - $innerStart)
                    }
                }
                '{' { $braceDepth += 1 }
                '}' { $braceDepth -= 1 }
            }
        }
    }
    return $null
}

function Split-TopLevel {
    param([string]$Text, [char]$Separator)
    $parts = New-Object 'System.Collections.Generic.List[string]'
    $depth = 0
    $braceDepth = 0
    $inString = $false
    $start = 0
    for ($i = 0; $i -lt $Text.Length; $i += 1) {
        $ch = $Text[$i]
        if ($ch -eq '"') {
            if ($inString -and $i + 1 -lt $Text.Length -and $Text[$i + 1] -eq '"') {
                $i += 1
                continue
            }
            $inString = -not $inString
            continue
        }
        if ($inString) {
            continue
        }
        switch ($ch) {
            '(' { $depth += 1 }
            ')' { $depth -= 1 }
            '{' { $braceDepth += 1 }
            '}' { $braceDepth -= 1 }
            default {
                if ($ch -eq $Separator -and $depth -eq 0 -and $braceDepth -eq 0) {
                    [void]$parts.Add($Text.Substring($start, $i - $start).Trim())
                    $start = $i + 1
                }
            }
        }
    }
    [void]$parts.Add($Text.Substring($start).Trim())
    return $parts
}

function ConvertFrom-FormulaLiteral {
    param([string]$Text)
    $trimmed = $Text.Trim()
    if ([string]::IsNullOrWhiteSpace($trimmed)) {
        return New-MissingValue
    }
    if ($trimmed -match '^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[Ee][+-]?\d+)?$') {
        return New-NumberValue ([double]::Parse($trimmed, $Invariant))
    }
    if ($trimmed -match '^(?i:true)$') {
        return New-LogicalValue $true
    }
    if ($trimmed -match '^(?i:false)$') {
        return New-LogicalValue $false
    }
    if ($trimmed.StartsWith('"') -and $trimmed.EndsWith('"')) {
        $inner = $trimmed.Substring(1, $trimmed.Length - 2).Replace('""', '"')
        return New-TextValue $inner
    }
    switch -Regex ($trimmed) {
        '^#NULL!$' { return New-ErrorValue "Null" }
        '^#DIV/0!$' { return New-ErrorValue "Div0" }
        '^#VALUE!$' { return New-ErrorValue "Value" }
        '^#REF!$' { return New-ErrorValue "Ref" }
        '^#NAME\?$' { return New-ErrorValue "Name" }
        '^#NUM!$' { return New-ErrorValue "Num" }
        '^#N/A$' { return New-ErrorValue "NA" }
        '^#SPILL!$' { return New-ErrorValue "Spill" }
        '^#CALC!$' { return New-ErrorValue "Calc" }
    }
    if ($trimmed.StartsWith("{") -and $trimmed.EndsWith("}")) {
        $inner = $trimmed.Substring(1, $trimmed.Length - 2)
        $rows = New-Object 'System.Collections.Generic.List[object]'
        foreach ($rowText in (Split-TopLevel $inner ';')) {
            $row = New-Object 'System.Collections.Generic.List[object]'
            foreach ($cellText in (Split-TopLevel $rowText ',')) {
                $cell = ConvertFrom-FormulaLiteral $cellText
                $cellKind = [string](Get-ValueField $cell "kind")
                if ($cellKind -eq "array" -or $cellKind -eq "missing_arg") {
                    throw "nested or missing array cell literal is unsupported: $cellText"
                }
                [void]$row.Add($cell)
            }
            [void]$rows.Add($row)
        }
        return New-ArrayValue $rows
    }
    throw "unsupported formula literal: $Text"
}

function Get-FormulaCandidates {
    param($Row)
    $fields = @("formula", "native_formula", "lhs_formula", "rhs_formula", "formula_setup", "cell_formula_or_action")
    $records = New-Object 'System.Collections.Generic.List[object]'
    foreach ($field in $fields) {
        if (-not ($Row.PSObject.Properties.Name -contains $field)) {
            continue
        }
        $text = [string]$Row.$field
        if ([string]::IsNullOrWhiteSpace($text)) {
            continue
        }
        $candidates = New-Object 'System.Collections.Generic.List[string]'
        $trimmed = $text.Trim()
        if ($trimmed.StartsWith("=")) {
            [void]$candidates.Add($trimmed)
        } elseif ($trimmed -match '^[A-Za-z][A-Za-z0-9.]*\s*\(') {
            [void]$candidates.Add("=$trimmed")
        } elseif ($trimmed -match ':=') {
            foreach ($part in ($trimmed -split '[|]')) {
                $assignIndex = $part.IndexOf(":=", [System.StringComparison]::Ordinal)
                if ($assignIndex -lt 0) {
                    continue
                }
                $rhs = $part.Substring($assignIndex + 2).Trim()
                if ([string]::IsNullOrWhiteSpace($rhs)) {
                    continue
                }
                if ($rhs.StartsWith("=")) {
                    [void]$candidates.Add($rhs)
                } elseif ($rhs -match '^[A-Za-z][A-Za-z0-9.]*\s*\(') {
                    [void]$candidates.Add("=$rhs")
                }
            }
        }

        foreach ($formula in $candidates) {
            [void]$records.Add([ordered]@{
                field = $field
                formula = $formula
            })
        }
    }
    return $records
}

$inventory = Get-Content (Resolve-Path (Join-Path $RepoRoot $InventoryPath)) -Raw | ConvertFrom-Json
$surfaces = @($inventory.surfaces)

$formulaCandidates = New-Object 'System.Collections.Generic.List[object]'
foreach ($path in Get-ChildItem (Join-Path $RepoRoot "docs/function-lane") -Filter "*SCENARIO_MANIFEST_SEED.csv") {
    foreach ($row in Import-Csv $path.FullName) {
        $row | Add-Member -NotePropertyName manifest_path -NotePropertyValue ($path.FullName.Substring($RepoRoot.Length + 1)) -Force
        foreach ($candidate in (Get-FormulaCandidates $row)) {
            [void]$formulaCandidates.Add([ordered]@{
                manifest_path = [string]$row.manifest_path
                scenario_id = [string]$row.scenario_id
                field = [string]$candidate.field
                formula = [string]$candidate.formula
            })
        }
    }
}

$cases = New-Object 'System.Collections.Generic.List[object]'
$tranches = @{}
$skipped = New-Object 'System.Collections.Generic.List[object]'
$seen = @{}
$caseIndex = 1

foreach ($surface in ($surfaces | Sort-Object canonical_surface_name)) {
    $surfaceId = [string]$surface.surface_id
    $name = [string]$surface.canonical_surface_name
    $category = [string]$surface.category
    $blocked = @($surface.blocked_or_deferred_lanes)
    $known = @($surface.known_deviation_tags)
    if ($blocked.Count -gt 0 -and -not $IncludeBlocked) {
        Add-Skip $skipped $name "blocked_or_deferred" $surfaceId $category
        continue
    }
    if ($known.Count -gt 0 -and -not $IncludeKnownDeviations) {
        Add-Skip $skipped $name "known_deviation" $surfaceId $category
        continue
    }

    $matchingCandidates = @($formulaCandidates | Where-Object {
        ([string]$_.formula).IndexOf($name, [System.StringComparison]::OrdinalIgnoreCase) -ge 0
    })
    if ($matchingCandidates.Count -eq 0) {
        Add-Skip $skipped $name "no_manifest_formula" $surfaceId $category
        continue
    }

    $surfaceCaseCount = 0
    $sawCandidate = $false
    foreach ($candidate in $matchingCandidates) {
        if ($surfaceCaseCount -ge $MaxSeedsPerSurface) {
            break
        }
        $argText = Find-TargetCallArgs ([string]$candidate.formula) $name
        if ($null -eq $argText) {
            continue
        }
        $sawCandidate = $true
        $argTexts = if ([string]::IsNullOrWhiteSpace($argText)) {
            @()
        } else {
            @(Split-TopLevel $argText ',')
        }
        $typedArgs = New-Object 'System.Collections.Generic.List[object]'
        $parseFailed = $false
        foreach ($arg in $argTexts) {
            try {
                [void]$typedArgs.Add((ConvertFrom-FormulaLiteral $arg))
            } catch {
                $parseFailed = $true
                break
            }
        }
        if ($parseFailed) {
            continue
        }

        $formulaText = "=$name($argText)"
        $dedupeKey = "$surfaceId|$formulaText"
        if ($seen.ContainsKey($dedupeKey)) {
            continue
        }
        $seen[$dedupeKey] = $true

        $slug = (($name -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant()).Trim('-')
        $caseId = "w089-seed-{0:d5}-{1}" -f $caseIndex, $slug
        [void]$cases.Add([ordered]@{
            schema_version = "oxfunc.smart_fuzzer.scenario_seed_case.v0"
            run_id = "assigned_by_runner"
            tranche_id = "w089-scenario-seed-executable-v0"
            case_id = $caseId
            function_id = $surfaceId
            canonical_surface_name = $name
            case_tag = "manifest_literal_seed"
            axis = "scenario_seed_literal_args"
            expected_probe_class = "scalar_or_array_literal_seed"
            formula_text = $formulaText
            args = [object[]]$typedArgs.ToArray()
            source_manifest = [string]$candidate.manifest_path
            source_formula_field = [string]$candidate.field
            source_scenario_id = [string]$candidate.scenario_id
            category = $category
            blocked_or_deferred_lanes = $blocked
            known_deviation_tags = $known
        })

        $trancheKey = (($category -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant()).Trim('-')
        if ([string]::IsNullOrWhiteSpace($trancheKey)) {
            $trancheKey = "uncategorized"
        }
        if (-not $tranches.ContainsKey($trancheKey)) {
            $tranches[$trancheKey] = [ordered]@{
                tranche_id = "w089-scenario-seed-$trancheKey"
                category = $category
                case_ids = New-Object 'System.Collections.Generic.List[string]'
                surfaces = New-Object 'System.Collections.Generic.HashSet[string]'
            }
        }
        [void]$tranches[$trancheKey].case_ids.Add($caseId)
        [void]$tranches[$trancheKey].surfaces.Add($name)

        $caseIndex += 1
        $surfaceCaseCount += 1
        if ($MaxTotalCases -gt 0 -and $cases.Count -ge $MaxTotalCases) {
            break
        }
    }
    if ($surfaceCaseCount -eq 0) {
        $reason = if ($sawCandidate) { "no_parseable_literal_seed" } else { "no_target_call" }
        Add-Skip $skipped $name $reason $surfaceId $category
    }
    if ($MaxTotalCases -gt 0 -and $cases.Count -ge $MaxTotalCases) {
        break
    }
}

$trancheRows = New-Object 'System.Collections.Generic.List[object]'
foreach ($key in ($tranches.Keys | Sort-Object)) {
    $value = $tranches[$key]
    [void]$trancheRows.Add([ordered]@{
        tranche_id = $value.tranche_id
        category = $value.category
        surface_count = $value.surfaces.Count
        case_count = $value.case_ids.Count
        case_ids = $value.case_ids
    })
}

$result = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.scenario_seed_executable_cases.v0"
    authority = "derived_w089_execution_input_not_semantic_truth"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    source_inventory = $InventoryPath
    tranche_id = "w089-scenario-seed-executable-v0"
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    include_blocked = [bool]$IncludeBlocked
    include_known_deviations = [bool]$IncludeKnownDeviations
    cases = $cases
    tranches = $trancheRows
    skipped = $skipped
    summary = [ordered]@{
        case_count = $cases.Count
        tranche_count = $trancheRows.Count
        skipped_count = $skipped.Count
        max_seeds_per_surface = $MaxSeedsPerSurface
        max_total_cases = $MaxTotalCases
    }
}

$resolvedOutput = Join-Path $RepoRoot $OutputPath
$parent = Split-Path -Parent $resolvedOutput
if (-not (Test-Path $parent)) {
    [void](New-Item -ItemType Directory -Path $parent -Force)
}
$result | ConvertTo-Json -Depth 100 | Set-Content -Path $resolvedOutput -Encoding UTF8
Write-Host "wrote $resolvedOutput"
Write-Host "cases=$($cases.Count) tranches=$($trancheRows.Count) skipped=$($skipped.Count)"

param(
    [string]$InventoryPath = "smart-fuzzer/cache/array-support-candidate-inventory-v0.json",
    [string]$OutputPath = "smart-fuzzer/cache/array-support-successor-executable-tranches-v0.json",
    [string]$RiskBands = "high,medium,low",
    [int]$MaxSeedsPerSurface = 1,
    [int]$MaxCasesPerSurface = 3
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptPath "..\..")
$RepoRoot = $RepoRoot.Path
Set-Location $RepoRoot

$Invariant = [System.Globalization.CultureInfo]::InvariantCulture
$AllowedRiskBands = @{}
foreach ($band in $RiskBands.Split(",") | ForEach-Object { $_.Trim() } | Where-Object { $_ }) {
    $AllowedRiskBands[$band] = $true
}

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

function ConvertTo-FormulaLiteral {
    param($Value)
    switch ([string](Get-ValueField $Value "kind")) {
        "number" { return ([double](Get-ValueField $Value "value")).ToString("G17", $Invariant) }
        "text" { return '"' + ([string](Get-ValueField $Value "value")).Replace('"', '""') + '"' }
        "logical" { if ([bool](Get-ValueField $Value "value")) { return "TRUE" } else { return "FALSE" } }
        "missing_arg" { return "" }
        "error" {
            switch ([string](Get-ValueField $Value "code")) {
                "Null" { return "#NULL!" }
                "Div0" { return "#DIV/0!" }
                "Value" { return "#VALUE!" }
                "Ref" { return "#REF!" }
                "Name" { return "#NAME?" }
                "Num" { return "#NUM!" }
                "NA" { return "#N/A" }
                "Spill" { return "#SPILL!" }
                "Calc" { return "#CALC!" }
                default { return "#VALUE!" }
            }
        }
        "array" {
            $rowTexts = New-Object 'System.Collections.Generic.List[string]'
            foreach ($row in (Get-ValueField $Value "rows")) {
                $cellTexts = New-Object 'System.Collections.Generic.List[string]'
                foreach ($cell in $row) {
                    [void]$cellTexts.Add((ConvertTo-FormulaLiteral $cell))
                }
                [void]$rowTexts.Add(($cellTexts.ToArray() -join ","))
            }
            return "{" + (($rowTexts.ToArray()) -join ";") + "}"
        }
        default { throw "unsupported value kind: $(Get-ValueField $Value "kind")" }
    }
}

function New-DuplicateArrayValue {
    param($ScalarValue)
    $row = New-Object 'System.Collections.Generic.List[object]'
    [void]$row.Add($ScalarValue)
    [void]$row.Add($ScalarValue)
    $rows = New-Object 'System.Collections.Generic.List[object]'
    [void]$rows.Add($row)
    return New-ArrayValue $rows
}

function Copy-Args {
    param([object[]]$Args)
    $json = $Args | ConvertTo-Json -Depth 80 -Compress
    return [object[]]($json | ConvertFrom-Json)
}

$inventory = Get-Content (Resolve-Path (Join-Path $RepoRoot $InventoryPath)) -Raw | ConvertFrom-Json
$manifestRows = New-Object 'System.Collections.Generic.List[object]'
foreach ($path in Get-ChildItem (Join-Path $RepoRoot "docs/function-lane") -Filter "*SCENARIO_MANIFEST_SEED.csv") {
    foreach ($row in Import-Csv $path.FullName) {
        if (-not ($row.PSObject.Properties.Name -contains "formula") -or [string]::IsNullOrWhiteSpace([string]$row.formula)) {
            continue
        }
        $row | Add-Member -NotePropertyName manifest_path -NotePropertyValue ($path.FullName.Substring($RepoRoot.Length + 1)) -Force
        [void]$manifestRows.Add($row)
    }
}

$covered = @{}
foreach ($name in @(
    "ROUND","ROUNDDOWN","ROUNDUP","TRUNC","CEILING","CEILING.MATH","CEILING.PRECISE",
    "FLOOR","FLOOR.MATH","FLOOR.PRECISE","ISO.CEILING","ATAN2","BASE","MROUND"
)) {
    $covered[$name] = $true
}

$cases = New-Object 'System.Collections.Generic.List[object]'
$tranches = @{}
$skipped = New-Object 'System.Collections.Generic.List[object]'
$caseIndex = 1

foreach ($surface in @($inventory.rows)) {
    $riskBand = [string]$surface.risk_band
    $name = [string]$surface.canonical_surface_name
    if (-not $AllowedRiskBands.ContainsKey($riskBand)) {
        continue
    }
    if ($covered.ContainsKey($name)) {
        continue
    }
    if (@($surface.blocked_or_deferred_lanes).Count -gt 0 -or @($surface.known_deviation_tags).Count -gt 0) {
        [void]$skipped.Add([ordered]@{ surface_id = $surface.surface_id; name = $name; reason = "blocked_or_known_deviation" })
        continue
    }

    $matchingSeeds = @($manifestRows | Where-Object { [string]$_.formula -match [regex]::Escape($name) })
    $seedCount = 0
    $surfaceCaseCount = 0
    $surfaceHadParseableSeed = $false

    foreach ($seed in $matchingSeeds) {
        if ($seedCount -ge $MaxSeedsPerSurface -or $surfaceCaseCount -ge $MaxCasesPerSurface) {
            break
        }
        $argText = Find-TargetCallArgs ([string]$seed.formula) $name
        if ($null -eq $argText) {
            continue
        }
        $argTexts = @(Split-TopLevel $argText ',')
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
        $surfaceHadParseableSeed = $true
        $seedCount += 1

        for ($argIndex = 0; $argIndex -lt $typedArgs.Count; $argIndex += 1) {
            if ($surfaceCaseCount -ge $MaxCasesPerSurface) {
                break
            }
            $argValue = $typedArgs[$argIndex]
            $argKind = [string](Get-ValueField $argValue "kind")
            if ($argKind -eq "array" -or $argKind -eq "missing_arg") {
                continue
            }

            $mutatedArgs = New-Object 'System.Collections.Generic.List[object]'
            for ($i = 0; $i -lt $typedArgs.Count; $i += 1) {
                if ($i -eq $argIndex) {
                    [void]$mutatedArgs.Add((New-DuplicateArrayValue $typedArgs[$i]))
                } else {
                    [void]$mutatedArgs.Add($typedArgs[$i])
                }
            }

            $formulaArgs = New-Object 'System.Collections.Generic.List[string]'
            foreach ($arg in $mutatedArgs) {
                [void]$formulaArgs.Add((ConvertTo-FormulaLiteral $arg))
            }
            $slug = (($name -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant()).Trim('-')
            $caseId = "w090-successor-{0:d5}-{1}-arg{2}" -f $caseIndex, $slug, ($argIndex + 1)
            $case = [ordered]@{
                schema_version = "oxfunc.smart_fuzzer.array_case.v0"
                run_id = "assigned_by_runner"
                tranche_id = "w090-successor-all-other-executable-v0"
                case_id = $caseId
                function_id = [string]$surface.surface_id
                canonical_surface_name = $name
                case_tag = "manifest_scalar_arg_$($argIndex + 1)_as_array"
                axis = "one_array_arg:arg$($argIndex + 1)"
                expected_probe_class = "shape_preserving_or_value_error"
                formula_text = "=$name($($formulaArgs.ToArray() -join ','))"
                args = [object[]]$mutatedArgs.ToArray()
                source_manifest = [string]$seed.manifest_path
                source_scenario_id = [string]$seed.scenario_id
                risk_band = $riskBand
                category = [string]$surface.category
            }
            [void]$cases.Add($case)

            $trancheKey = (($surface.category -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant()).Trim('-')
            if ([string]::IsNullOrWhiteSpace($trancheKey)) {
                $trancheKey = "uncategorized"
            }
            if (-not $tranches.ContainsKey($trancheKey)) {
                $tranches[$trancheKey] = [ordered]@{
                    tranche_id = "w090-successor-$trancheKey"
                    category = [string]$surface.category
                    case_ids = New-Object 'System.Collections.Generic.List[string]'
                    surfaces = New-Object 'System.Collections.Generic.HashSet[string]'
                }
            }
            [void]$tranches[$trancheKey].case_ids.Add($caseId)
            [void]$tranches[$trancheKey].surfaces.Add($name)
            $caseIndex += 1
            $surfaceCaseCount += 1
        }
    }

    if (-not $surfaceHadParseableSeed) {
        [void]$skipped.Add([ordered]@{
            surface_id = [string]$surface.surface_id
            name = $name
            risk_band = $riskBand
            category = [string]$surface.category
            reason = "no_parseable_manifest_seed"
        })
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
    schema_version = "oxfunc.smart_fuzzer.array_support_successor_executable_tranches.v0"
    authority = "derived_w090_successor_execution_input_not_semantic_truth"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    source_inventory = $InventoryPath
    risk_bands = @($AllowedRiskBands.Keys)
    tranche_id = "w090-successor-all-other-executable-v0"
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    cases = $cases
    tranches = $trancheRows
    skipped = $skipped
    summary = [ordered]@{
        case_count = $cases.Count
        tranche_count = $trancheRows.Count
        skipped_count = $skipped.Count
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

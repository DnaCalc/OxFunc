[CmdletBinding()]
param(
    [string] $RepoRoot,
    [string] $OutputPath,
    [string[]] $PilotFunctionId = @("FUNC.PMT", "FUNC.PPMT")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
if (-not $OutputPath) {
    $OutputPath = Join-Path $RepoRoot "smart-fuzzer\cache\static-risk-index.json"
}
$OutputPath = [System.IO.Path]::GetFullPath($OutputPath)

function Join-RepoPath {
    param([string] $RelativePath)
    return Join-Path $RepoRoot $RelativePath
}

function Get-RepoRelativePath {
    param([string] $Path)

    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetFullPath($RepoRoot)
    if (-not $root.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $root = $root + [System.IO.Path]::DirectorySeparatorChar
    }

    if ($full.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
        return ($full.Substring($root.Length) -replace "\\", "/")
    }

    return $full
}

function Read-RequiredCsv {
    param([string] $RelativePath)

    $path = Join-RepoPath $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Required input not found: $RelativePath"
    }

    return @(Import-Csv -LiteralPath $path)
}

function New-StringSet {
    $set = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase
    )
    return ,$set
}

function Add-MapSetValue {
    param(
        [hashtable] $Map,
        [string] $Key,
        [string] $Value
    )

    if ([string]::IsNullOrWhiteSpace($Key) -or [string]::IsNullOrWhiteSpace($Value)) {
        return
    }

    if (-not $Map.ContainsKey($Key)) {
        $Map[$Key] = New-StringSet
    }

    [void] $Map[$Key].Add($Value)
}

function Get-MapSetValues {
    param(
        [hashtable] $Map,
        [string] $Key
    )

    if (-not $Map.ContainsKey($Key)) {
        return @()
    }

    return @($Map[$Key] | Sort-Object)
}

function Add-Tag {
    param(
        [System.Collections.Generic.HashSet[string]] $Tags,
        [string] $Tag
    )

    if (-not [string]::IsNullOrWhiteSpace($Tag)) {
        [void] $Tags.Add($Tag)
    }
}

function Get-GitValue {
    param([string[]] $GitArgs)

    try {
        $value = (& git @GitArgs 2>$null)
        if ($LASTEXITCODE -ne 0) {
            return $null
        }
        return (($value -join "`n").Trim())
    }
    catch {
        return $null
    }
}

function Get-FunctionMentionPattern {
    param([string] $Name)

    return "(?<![A-Z0-9_])$([regex]::Escape($Name))(?![A-Z0-9_])"
}

function Test-FunctionMention {
    param(
        [string] $Text,
        [string] $PathText,
        [string] $Name
    )

    $escaped = [regex]::Escape($Name)
    if ($Name.Length -le 3) {
        return ($Text -match "(?<![A-Z0-9_])$escaped\s*\(")
    }

    if ($Text -match (Get-FunctionMentionPattern $Name)) {
        return $true
    }

    $lowerName = [regex]::Escape($Name.ToLowerInvariant())
    return ($PathText -match "(?i)(^|[\\/_-])$lowerName($|[\\/_-])")
}

$libraryPath = "docs\function-lane\OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv"
$catalogPath = "docs\function-lane\FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv"
$bugRegisterPath = "docs\bugs\BUG_STREAM_REGISTER.csv"

$libraryRows = Read-RequiredCsv $libraryPath
$catalogRows = Read-RequiredCsv $catalogPath
$bugRows = Read-RequiredCsv $bugRegisterPath

$catalogByName = @{}
foreach ($row in $catalogRows) {
    if (-not [string]::IsNullOrWhiteSpace($row.function_name)) {
        $catalogByName[$row.function_name.ToUpperInvariant()] = $row
    }
}

$functionNames = @()
$functionIdsByName = @{}
foreach ($row in $libraryRows) {
    if ($row.surface_stable_id -like "FUNC.*" -and -not [string]::IsNullOrWhiteSpace($row.canonical_surface_name)) {
        $name = $row.canonical_surface_name.ToUpperInvariant()
        $functionIdsByName[$name] = $row.surface_stable_id
    }
}
$functionNames = @($functionIdsByName.Keys | Sort-Object)
$mentionFunctionNames = @($functionNames | Where-Object { $_.Length -ge 3 })

$scenarioRefsByFunction = @{}
$scenarioFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "docs\function-lane") -Filter "*SCENARIO_MANIFEST_SEED.csv" -File)
foreach ($file in $scenarioFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    foreach ($match in [regex]::Matches($content, "\b([A-Z][A-Z0-9\.]{1,31})\s*\(")) {
        $name = $match.Groups[1].Value.ToUpperInvariant()
        if ($functionIdsByName.ContainsKey($name)) {
            Add-MapSetValue $scenarioRefsByFunction $functionIdsByName[$name] $relative
        }
    }
}

$deferredRefsByFunction = @{}
$deferredFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "docs\function-lane") -Filter "*DEFERRED*INVENTORY*.csv" -File)
foreach ($file in $deferredFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    foreach ($name in $mentionFunctionNames) {
        if (Test-FunctionMention -Text $content -PathText $file.Name -Name $name) {
            Add-MapSetValue $deferredRefsByFunction $functionIdsByName[$name] $relative
        }
    }
}

$bugRefsByFunction = @{}
$bugTagsByFunction = @{}
$bugById = @{}
foreach ($bug in $bugRows) {
    $bugById[$bug.bug_id] = $bug
    $haystack = "$($bug.title) $($bug.notes) $($bug.stream_path)"
    foreach ($name in $mentionFunctionNames) {
        if (Test-FunctionMention -Text $bug.title -PathText $bug.stream_path -Name $name) {
            $functionId = $functionIdsByName[$name]
            Add-MapSetValue $bugRefsByFunction $functionId $bug.bug_id
            Add-MapSetValue $bugTagsByFunction $functionId "bug_status:$($bug.status)"
            Add-MapSetValue $bugTagsByFunction $functionId "bug_owner_workset:$($bug.owner_workset)"
            if ($haystack -match "(?i)fresh Excel confirmation|fresh confirmation|stale") {
                Add-MapSetValue $bugTagsByFunction $functionId "fresh_confirmation_required"
            }
        }
    }
}

$sourceRefsByFunction = @{}
$sourceHintsByFunction = @{}
$sourceRiskPatterns = @(
    @{ tag = "source:todo_or_unimplemented"; pattern = "\b(todo!|unimplemented!)\s*\(" },
    @{ tag = "source:panic_path"; pattern = "\bpanic!\s*\(" },
    @{ tag = "source:unwrap_or_expect"; pattern = "\.(unwrap|expect)\s*\(" },
    @{ tag = "source:numeric_transcendental"; pattern = "\.(powf|ln|exp|log|sqrt)\s*\(" },
    @{ tag = "source:iteration_or_solver"; pattern = "(?i)\b(loop|while|newton|solver|max_iter|iteration|tolerance|epsilon)\b" },
    @{ tag = "source:array_or_reference_logic"; pattern = "(?i)\b(array|spill|reference|range|resolver|broadcast)\b" }
)

$sourceFiles = @(Get-ChildItem -LiteralPath (Join-RepoPath "crates\oxfunc_core\src\functions") -Filter "*.rs" -File)
foreach ($file in $sourceFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw
    $relative = Get-RepoRelativePath $file.FullName
    $functionIds = @(
        [regex]::Matches($content, "FUNC\.[A-Z0-9\._]+") |
            ForEach-Object { $_.Value.ToUpperInvariant() } |
            Sort-Object -Unique
    )

    if ($functionIds.Count -eq 0) {
        continue
    }

    $fileTags = @()
    foreach ($riskPattern in $sourceRiskPatterns) {
        if ($content -match $riskPattern.pattern) {
            $fileTags += $riskPattern.tag
        }
    }

    foreach ($functionId in $functionIds) {
        Add-MapSetValue $sourceRefsByFunction $functionId $relative
        foreach ($tag in $fileTags) {
            Add-MapSetValue $sourceHintsByFunction $functionId $tag
        }
    }
}

$pilotSet = New-StringSet
foreach ($id in $PilotFunctionId) {
    [void] $pilotSet.Add($id)
}

$records = @()
foreach ($row in ($libraryRows | Sort-Object canonical_surface_name)) {
    $functionId = $row.surface_stable_id
    if (-not ($functionId -like "FUNC.*")) {
        continue
    }

    $name = $row.canonical_surface_name
    $catalog = $null
    if ($catalogByName.ContainsKey($name.ToUpperInvariant())) {
        $catalog = $catalogByName[$name.ToUpperInvariant()]
    }

    $tags = New-StringSet
    $score = 0
    $bugRefs = @(Get-MapSetValues $bugRefsByFunction $functionId)
    $bugTags = @(Get-MapSetValues $bugTagsByFunction $functionId)
    $scenarioRefs = @(Get-MapSetValues $scenarioRefsByFunction $functionId)
    $deferredRefs = @(Get-MapSetValues $deferredRefsByFunction $functionId)
    $sourceRefs = @(Get-MapSetValues $sourceRefsByFunction $functionId)
    $sourceHints = @(Get-MapSetValues $sourceHintsByFunction $functionId)

    if ($pilotSet.Contains($functionId)) {
        $score += 100
        Add-Tag $tags "pilot_focus"
    }

    if ($bugRefs.Count -gt 0) {
        Add-Tag $tags "bug_stream_mentioned"
        foreach ($bugId in $bugRefs) {
            $bug = $bugById[$bugId]
            switch ($bug.status) {
                "validated_local" { $score += 60 }
                "investigating" { $score += 25 }
                "handed_off" { $score += 20 }
                default { $score += 10 }
            }
        }
        foreach ($tag in $bugTags) {
            Add-Tag $tags $tag
        }
    }

    if ($deferredRefs.Count -gt 0) {
        $score += 25
        Add-Tag $tags "deferred_inventory_mentioned"
    }

    if ($scenarioRefs.Count -eq 0) {
        $score += 5
        Add-Tag $tags "no_seed_manifest_mentions"
    }
    else {
        Add-Tag $tags "seed_manifest_mentioned"
    }

    if ($row.interesting -eq "true") {
        $score += 10
        Add-Tag $tags "catalog_interesting"
    }

    if ([string]::IsNullOrWhiteSpace($row.arity_min) -or [string]::IsNullOrWhiteSpace($row.arity_max)) {
        $score += 8
        Add-Tag $tags "metadata_missing_arity"
    }

    if ($row.metadata_status -ne "function_meta_extracted") {
        $score += 5
        Add-Tag $tags "metadata_not_extracted"
    }

    if ($row.coercion_lift_profile -match "(?i)array|reference|custom") {
        $score += 8
        Add-Tag $tags "complex_coercion_or_lift_profile"
    }

    if ($row.host_interaction_class -and $row.host_interaction_class -ne "None") {
        $score += 15
        Add-Tag $tags "host_interaction"
    }

    if ($row.determinism_class -and $row.determinism_class -ne "Deterministic") {
        $score += 15
        Add-Tag $tags "non_deterministic"
    }

    if ($row.volatility_class -and $row.volatility_class -ne "NonVolatile") {
        $score += 15
        Add-Tag $tags "volatile"
    }

    foreach ($hint in $sourceHints) {
        Add-Tag $tags $hint
        switch ($hint) {
            "source:todo_or_unimplemented" { $score += 30 }
            "source:panic_path" { $score += 20 }
            "source:unwrap_or_expect" { $score += 12 }
            "source:iteration_or_solver" { $score += 10 }
            "source:array_or_reference_logic" { $score += 8 }
            "source:numeric_transcendental" { $score += 5 }
            default { $score += 3 }
        }
    }

    $catalogRef = $null
    if ($catalog) {
        $catalogRef = $catalogPath
    }

    $records += [ordered]@{
        function_id = $functionId
        canonical_surface_name = $name
        category = $row.category
        arity_min = $row.arity_min
        arity_max = $row.arity_max
        metadata_status = $row.metadata_status
        arg_preparation_profile = $row.arg_preparation_profile
        coercion_lift_profile = $row.coercion_lift_profile
        kernel_signature_class = $row.kernel_signature_class
        determinism_class = $row.determinism_class
        volatility_class = $row.volatility_class
        host_interaction_class = $row.host_interaction_class
        risk = [ordered]@{
            score = $score
            tags = @($tags | Sort-Object)
        }
        refs = [ordered]@{
            library_context = $libraryPath
            catalog = $catalogRef
            bug_streams = $bugRefs
            scenario_manifests = $scenarioRefs
            deferred_inventories = $deferredRefs
            source_files = $sourceRefs
        }
        source_hints = $sourceHints
    }
}

$orderedRecords = @($records | Sort-Object @{ Expression = { $_.risk.score }; Descending = $true }, canonical_surface_name)
$topRecords = @(
    $orderedRecords |
        Select-Object -First 25 |
        ForEach-Object {
            [ordered]@{
                function_id = $_.function_id
                canonical_surface_name = $_.canonical_surface_name
                score = $_.risk.score
                tags = $_.risk.tags
            }
        }
)

$index = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.static_risk_index.v0"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    authority = "derived_exploration_index_not_semantic_truth"
    git_revision = Get-GitValue @("rev-parse", "HEAD")
    git_status_short_digest_source = Get-GitValue @("status", "--short")
    inputs = [ordered]@{
        library_context = $libraryPath
        catalog = $catalogPath
        bug_register = $bugRegisterPath
        scenario_manifest_count = $scenarioFiles.Count
        deferred_inventory_count = $deferredFiles.Count
        source_file_count = $sourceFiles.Count
        pilot_function_ids = $PilotFunctionId
    }
    summary = [ordered]@{
        function_count = $orderedRecords.Count
        functions_with_bug_refs = @($orderedRecords | Where-Object { $_.refs.bug_streams.Count -gt 0 }).Count
        functions_with_scenario_refs = @($orderedRecords | Where-Object { $_.refs.scenario_manifests.Count -gt 0 }).Count
        functions_with_deferred_refs = @($orderedRecords | Where-Object { $_.refs.deferred_inventories.Count -gt 0 }).Count
        top_risk = $topRecords
    }
    functions = $orderedRecords
}

$outputDir = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$index | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "Wrote $OutputPath"
Write-Host "Functions indexed: $($orderedRecords.Count)"
Write-Host "Top risk functions:"
foreach ($record in ($topRecords | Select-Object -First 10)) {
    Write-Host ("  {0} {1} score={2} tags={3}" -f $record.function_id, $record.canonical_surface_name, $record.score, (($record.tags -join ", ")))
}

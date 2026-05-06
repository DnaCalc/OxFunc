$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$CatalogPath = Join-Path $RepoRoot "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv"
$OutputPath = Join-Path $RepoRoot "crates/oxfunc_core/src/registry_help_seed.rs"
$RichWitnessPaths = @(
    "docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json",
    "docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_MIXED_TRANCHE_A.json"
) | ForEach-Object { Join-Path $RepoRoot $_ }

function New-Seed($FunctionId) {
    [ordered]@{
        function_id = $FunctionId
        short_description = $null
        long_description = $null
        parameters = @{}
    }
}

function Get-Seed($Seeds, $FunctionId) {
    if (-not $Seeds.ContainsKey($FunctionId)) {
        $Seeds[$FunctionId] = New-Seed $FunctionId
    }
    return $Seeds[$FunctionId]
}

function Is-GenericWitnessText($Text) {
    if ([string]::IsNullOrWhiteSpace($Text)) {
        return $true
    }
    return $Text -match "Seeded from|pending fuller tranche|witness tranche seeded|Ordinary .* seed|Generated W71|tranche seed"
}

function Escape-RustString($Text) {
    if ($null -eq $Text) {
        return $null
    }
    return ($Text -replace "\\", "\\") `
        -replace '"', '\"' `
        -replace "`r", "" `
        -replace "`n", "\n"
}

function Rust-OptionString($Text) {
    if ([string]::IsNullOrWhiteSpace($Text)) {
        return "None"
    }
    return "Some(`"$(Escape-RustString $Text)`")"
}

$Seeds = @{}

foreach ($Row in Import-Csv $CatalogPath) {
    if ([string]::IsNullOrWhiteSpace($Row.function_name)) {
        continue
    }

    $FunctionName = $Row.function_name.Trim().ToUpperInvariant()
    $FunctionId = "FUNC.$FunctionName"
    $Seed = Get-Seed $Seeds $FunctionId

    if (-not [string]::IsNullOrWhiteSpace($Row.description)) {
        $Seed.short_description = $Row.description.Trim()
    }
}

foreach ($WitnessPath in $RichWitnessPaths) {
    if (-not (Test-Path $WitnessPath)) {
        continue
    }

    $Witness = Get-Content $WitnessPath -Raw | ConvertFrom-Json
    foreach ($Entry in @($Witness.entries)) {
        if ([string]::IsNullOrWhiteSpace($Entry.surface_stable_id)) {
            continue
        }

        $FunctionId = [string]$Entry.surface_stable_id
        $Seed = Get-Seed $Seeds $FunctionId

        if ([string]::IsNullOrWhiteSpace($Seed.short_description) -and -not (Is-GenericWitnessText $Entry.help_summary)) {
            $Seed.short_description = ([string]$Entry.help_summary).Trim()
        }

        if (-not (Is-GenericWitnessText $Entry.help_detail)) {
            $Seed.long_description = ([string]$Entry.help_detail).Trim()
        }

        foreach ($Arg in @($Entry.arg_specs)) {
            if ([string]::IsNullOrWhiteSpace($Arg.arg_name)) {
                continue
            }

            $ArgIndex = [int]$Arg.arg_index
            if ($ArgIndex -gt 0) {
                $ArgIndex = $ArgIndex - 1
            }

            $ParameterSeed = [ordered]@{
                index = $ArgIndex
                name = ([string]$Arg.arg_name).Trim()
                short_description = $null
            }

            if (-not [string]::IsNullOrWhiteSpace($Arg.arg_behavior_note)) {
                $ParameterSeed.short_description = ([string]$Arg.arg_behavior_note).Trim()
            }

            $Seed.parameters[$ArgIndex] = $ParameterSeed
        }
    }
}

$Lines = New-Object System.Collections.Generic.List[string]
$Lines.Add("// Auto-generated from current OxFunc function catalog and rich V2 witness seeds.")
$Lines.Add("// Do not hand-edit help data here; update the source artifacts and rerun tools/generate-registry-help-seed.ps1.")
$Lines.Add("")
$Lines.Add("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
$Lines.Add("pub(crate) struct ParameterHelpSeed {")
$Lines.Add("    pub index: usize,")
$Lines.Add("    pub name: &'static str,")
$Lines.Add("    pub short_description: Option<&'static str>,")
$Lines.Add("}")
$Lines.Add("")
$Lines.Add("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
$Lines.Add("pub(crate) struct RegistryHelpSeed {")
$Lines.Add("    pub function_id: &'static str,")
$Lines.Add("    pub short_description: Option<&'static str>,")
$Lines.Add("    pub long_description: Option<&'static str>,")
$Lines.Add("    pub parameters: &'static [ParameterHelpSeed],")
$Lines.Add("}")
$Lines.Add("")
$Lines.Add("pub(crate) const REGISTRY_HELP_SEEDS: &[RegistryHelpSeed] = &[")

foreach ($FunctionId in ($Seeds.Keys | Sort-Object)) {
    $Seed = $Seeds[$FunctionId]
    $HasFunctionHelp = -not [string]::IsNullOrWhiteSpace($Seed.short_description) -or -not [string]::IsNullOrWhiteSpace($Seed.long_description)
    $HasParameters = $Seed.parameters.Count -gt 0
    if (-not $HasFunctionHelp -and -not $HasParameters) {
        continue
    }

    $Lines.Add("    RegistryHelpSeed {")
    $Lines.Add("        function_id: `"$(Escape-RustString $FunctionId)`",")
    $Lines.Add("        short_description: $(Rust-OptionString $Seed.short_description),")
    $Lines.Add("        long_description: $(Rust-OptionString $Seed.long_description),")

    if ($HasParameters) {
        $Lines.Add("        parameters: &[")
        foreach ($Parameter in ($Seed.parameters.Values | Sort-Object index)) {
            $Lines.Add("            ParameterHelpSeed {")
            $Lines.Add("                index: $($Parameter.index),")
            $Lines.Add("                name: `"$(Escape-RustString $Parameter.name)`",")
            $Lines.Add("                short_description: $(Rust-OptionString $Parameter.short_description),")
            $Lines.Add("            },")
        }
        $Lines.Add("        ],")
    } else {
        $Lines.Add("        parameters: &[],")
    }

    $Lines.Add("    },")
}

$Lines.Add("];")
$Lines.Add("")
$Lines.Add("pub(crate) fn registry_help_seed_for_id(function_id: &str) -> Option<&'static RegistryHelpSeed> {")
$Lines.Add("    REGISTRY_HELP_SEEDS")
$Lines.Add("        .iter()")
$Lines.Add("        .find(|seed| seed.function_id.eq_ignore_ascii_case(function_id))")
$Lines.Add("}")

Set-Content -Path $OutputPath -Value $Lines -Encoding UTF8

$Generated = $Seeds.Values | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_.short_description) -or
    -not [string]::IsNullOrWhiteSpace($_.long_description) -or
    $_.parameters.Count -gt 0
}
$LongCount = @($Generated | Where-Object { -not [string]::IsNullOrWhiteSpace($_.long_description) }).Count
$ParamFunctionCount = @($Generated | Where-Object { $_.parameters.Count -gt 0 }).Count
$ParamCount = ($Generated | ForEach-Object { $_.parameters.Count } | Measure-Object -Sum).Sum

Write-Host "Generated $(@($Generated).Count) registry help seeds."
Write-Host "Generated $LongCount long-description seeds."
Write-Host "Generated $ParamCount parameter-help seeds across $ParamFunctionCount functions."

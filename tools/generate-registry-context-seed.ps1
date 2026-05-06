param(
    [string]$CsvPath = "docs\function-lane\OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv",
    [string]$OutPath = "crates\oxfunc_core\src\registry_context_seed.rs"
)

$ErrorActionPreference = "Stop"

function RustStringLiteral([string]$value) {
    if ([string]::IsNullOrEmpty($value)) {
        return "None"
    }
    $escaped = $value.Replace("\", "\\").Replace('"', '\"')
    return "Some(`"$escaped`".to_string())"
}

function Expand-SurfaceStableIds([string]$surfaceStableId) {
    if ([string]::IsNullOrWhiteSpace($surfaceStableId)) {
        return @()
    }

    $parts = @($surfaceStableId.Split(",") | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    if ($parts.Count -eq 0) {
        return @()
    }

    $prefix = ""
    if ($parts[0].StartsWith("FUNC.")) {
        $prefix = "FUNC."
    }

    return @(
        foreach ($part in $parts) {
            if ($part.StartsWith("FUNC.")) {
                $part
            } else {
                "$prefix$part"
            }
        }
    )
}

function SurfaceNameFromFunctionId([string]$functionId) {
    if ($functionId.StartsWith("FUNC.")) {
        return $functionId.Substring(5)
    }
    return $functionId
}

$repoRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "..")
$csvPathFull = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $CsvPath))
$outPathFull = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $OutPath))

$rows = Import-Csv -LiteralPath $csvPathFull
$fields = @(
    "lane_id",
    "entry_kind",
    "registration_source_kind",
    "surface_stable_id",
    "xlcall_builtin_symbol",
    "xlcall_builtin_code",
    "canonical_surface_name",
    "name_resolution_table_ref",
    "semantic_trait_profile_ref",
    "gating_profile_ref",
    "version_marker",
    "category",
    "interesting",
    "metadata_status",
    "special_interface_kind",
    "admission_interface_kind",
    "preparation_owner",
    "runtime_boundary_kind",
    "interface_contract_ref",
    "source_catalog_ref"
)

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("// Auto-generated from docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv.")
$lines.Add("// Do not hand-edit metadata here; update the source CSV and rerun tools/generate-registry-context-seed.ps1.")
$lines.Add("// The legacy arity_shape_note column is intentionally not emitted into the runtime registry API.")
$lines.Add("")
$lines.Add("use crate::registry::FunctionRegistryMetadata;")
$lines.Add("")
$lines.Add("pub(crate) fn registry_metadata_for_id(function_id: &str) -> FunctionRegistryMetadata {")
$lines.Add("    match function_id {")

$seen = New-Object System.Collections.Generic.HashSet[string]
foreach ($row in $rows) {
    foreach ($functionId in (Expand-SurfaceStableIds $row.surface_stable_id)) {
        if (-not $seen.Add($functionId)) {
            throw "Duplicate registry metadata id after expansion: $functionId"
        }

        $id = $functionId.Replace("\", "\\").Replace('"', '\"')
        $lines.Add("        `"$id`" => FunctionRegistryMetadata {")
        foreach ($field in $fields) {
            $value = $row.$field
            if ($field -eq "surface_stable_id") {
                $value = $functionId
            } elseif ($field -eq "canonical_surface_name") {
                $value = SurfaceNameFromFunctionId $functionId
            }
            $lines.Add("            ${field}: $(RustStringLiteral $value),")
        }
        $lines.Add("        },")
    }
}

$lines.Add("        _ => FunctionRegistryMetadata {")
$lines.Add("            surface_stable_id: Some(function_id.to_string()),")
$lines.Add("            ..FunctionRegistryMetadata::default()")
$lines.Add("        },")
$lines.Add("    }")
$lines.Add("}")

$outDir = Split-Path -Path $outPathFull -Parent
if ($outDir -and -not (Test-Path -LiteralPath $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

[System.IO.File]::WriteAllText($outPathFull, ($lines -join "`r`n") + "`r`n", [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote $outPathFull"

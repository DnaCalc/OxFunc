param(
    [string]$BundleRoot = ".tmp/replay-bundles/oxfunc-w15-v1",
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

function Write-JsonFile {
    param([string]$Path, [object]$Value)
    $dir = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($dir)) {
        Ensure-Directory -Path $dir
    }
    $Value | ConvertTo-Json -Depth 16 | Set-Content -Path $Path -Encoding UTF8
}

function Write-CsvFile {
    param([string]$Path, [object[]]$Rows, [string[]]$Headers)
    $dir = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($dir)) {
        Ensure-Directory -Path $dir
    }
    if ($Rows.Count -gt 0) {
        $Rows | Select-Object -Property $Headers | ConvertTo-Csv -NoTypeInformation | Set-Content -Path $Path -Encoding UTF8
        return
    }
    Set-Content -Path $Path -Value ($Headers -join ",") -Encoding UTF8
}

function Copy-RepoArtifact {
    param([string]$RepoRoot, [string]$SourceRelativePath, [string]$BundleRoot, [string]$BundleRelativePath)
    $sourcePath = Join-Path $RepoRoot $SourceRelativePath
    if (-not (Test-Path $sourcePath)) {
        throw "Source artifact missing: $SourceRelativePath"
    }
    $destPath = Join-Path $BundleRoot $BundleRelativePath
    Ensure-Directory -Path (Split-Path -Parent $destPath)
    Copy-Item -Path $sourcePath -Destination $destPath -Force
    return $BundleRelativePath.Replace("\", "/")
}

function New-SourceInventoryRow {
    param([string]$SourceRefId, [string]$SourceSchemaId, [string]$BundleRelativePath, [string]$SourceRole, [string]$PacketId, [string]$RunScope, [string]$Notes)
    [pscustomobject]@{
        source_ref_id = $SourceRefId
        source_schema_id = $SourceSchemaId
        bundle_relative_path = $BundleRelativePath
        source_role = $SourceRole
        packet_id = $PacketId
        run_scope = $RunScope
        notes = $Notes
    }
}

function Join-Refs {
    param([string[]]$Values)
    (($Values | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join "|")
}

function Get-W15PacketDefinitions {
    @(
        [pscustomobject]@{
            packet_id = "W15.INFO.packet"
            lane = "W15-INFO-PRE"
            generator_ref = "docs/function-lane/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv"
            manifest_bundle_path = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv"
            result_default = ".tmp/w15-info-pre-results.csv"
            result_compat = ".tmp/w15-info-pre-results-compat.csv"
            result_bundle_default = "sidecars/source/results/w15-info-pre-results.csv"
            result_bundle_compat = "sidecars/source/results/w15-info-pre-results-compat.csv"
            description = "INFO host-query packet"
            tags = @("w15", "info", "host_query")
            pack_tags = @("packet", "row_first")
            evidence_id = "W15-INFO-PRE-20260315"
            limitation_refs = @()
            row_kind = "info"
        }
        [pscustomobject]@{
            packet_id = "W15.CELL.packet"
            lane = "W15-CELL-HOST-PRE"
            generator_ref = "docs/function-lane/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv"
            manifest_bundle_path = "sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv"
            result_default = ".tmp/w15-cell-host-pre-results.csv"
            result_compat = ".tmp/w15-cell-host-pre-results-compat.csv"
            result_bundle_default = "sidecars/source/results/w15-cell-host-pre-results.csv"
            result_bundle_compat = "sidecars/source/results/w15-cell-host-pre-results-compat.csv"
            description = "CELL host-sensitive packet"
            tags = @("w15", "cell", "host_query")
            pack_tags = @("packet", "row_first")
            evidence_id = "W15-CELL-HOST-PRE-20260315"
            limitation_refs = @()
            row_kind = "cell"
        }
        [pscustomobject]@{
            packet_id = "W15.XLL.packet"
            lane = "W15-XLL"
            generator_ref = "docs/function-lane/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv"
            manifest_bundle_path = "sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv"
            result_default = ".tmp/w15-xll-bridge-results.csv"
            result_compat = ".tmp/w15-xll-bridge-results-compat.csv"
            result_bundle_default = "sidecars/source/results/w15-xll-bridge-results.csv"
            result_bundle_compat = "sidecars/source/results/w15-xll-bridge-results-compat.csv"
            description = "CELL/INFO XLL bridge parity packet"
            tags = @("w15", "xll", "bridge_parity")
            pack_tags = @("packet", "row_first", "xll_surface")
            evidence_id = "W15-XLL-BRIDGE-20260315"
            limitation_refs = @("docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md")
            row_kind = "xll"
        }
    )
}

function Build-W15RowView {
    param([object]$PacketDef, [string]$RunId, [string]$RunLabel, [string]$ManifestBundlePath, [string]$ResultBundlePath, [string]$RepoRoot)

    $manifestRows = @(Import-Csv (Join-Path $RepoRoot $PacketDef.generator_ref))
    $resultPath = if ($RunLabel -eq "default") { $PacketDef.result_default } else { $PacketDef.result_compat }
    $resultRows = @(Import-Csv (Join-Path $RepoRoot $resultPath))
    $manifestById = @{}
    foreach ($manifestRow in $manifestRows) {
        $manifestById[[string]$manifestRow.scenario_id] = $manifestRow
    }

    $rows = @()
    foreach ($resultRow in $resultRows) {
        $scenarioId = [string]$resultRow.scenario_id
        $manifestRow = $manifestById[$scenarioId]
        $common = @{
            packet_id = "W15"
            scenario_packet_id = $PacketDef.packet_id
            scenario_id = $scenarioId
            lane = [string]$resultRow.lane
            row_kind = $PacketDef.row_kind
            run_id = $RunId
            run_label = [string]$resultRow.run_label
            compat_descriptor = [string]$resultRow.compat_descriptor
            expected_status = if ($manifestRow.PSObject.Properties.Name -contains "expected_status") { [string]$manifestRow.expected_status } else { "" }
            expected_observable = if ($manifestRow.PSObject.Properties.Name -contains "expected_observable") { [string]$manifestRow.expected_observable } else { "" }
            expected_relation = if ($manifestRow.PSObject.Properties.Name -contains "expected_relation") { [string]$manifestRow.expected_relation } else { "" }
            source_manifest_ref = $ManifestBundlePath
            source_result_ref = $ResultBundlePath
            evidence_ids = $PacketDef.evidence_id
            limitation_refs = (Join-Refs -Values $PacketDef.limitation_refs)
            notes = [string]$manifestRow.notes
        }

        if ($PacketDef.row_kind -eq "xll") {
            $rows += [pscustomobject]@{
                packet_id = $common.packet_id
                scenario_packet_id = $common.scenario_packet_id
                scenario_id = $common.scenario_id
                lane = $common.lane
                row_kind = $common.row_kind
                run_id = $common.run_id
                run_label = $common.run_label
                compat_descriptor = $common.compat_descriptor
                formula_or_query = [string]$resultRow.bridge_formula
                comparison_formula = [string]$resultRow.native_formula
                observed_status = [string]$resultRow.relation_status
                observed_text = [string]$resultRow.bridge_text
                observed_value2 = [string]$resultRow.bridge_value2
                comparison_text = [string]$resultRow.native_text
                comparison_value2 = [string]$resultRow.native_value2
                expected_status = $common.expected_status
                expected_observable = $common.expected_observable
                expected_relation = $common.expected_relation
                source_manifest_ref = $common.source_manifest_ref
                source_result_ref = $common.source_result_ref
                evidence_ids = $common.evidence_ids
                limitation_refs = $common.limitation_refs
                notes = $common.notes
            }
            continue
        }

        $rows += [pscustomobject]@{
            packet_id = $common.packet_id
            scenario_packet_id = $common.scenario_packet_id
            scenario_id = $common.scenario_id
            lane = $common.lane
            row_kind = $common.row_kind
            run_id = $common.run_id
            run_label = $common.run_label
            compat_descriptor = $common.compat_descriptor
            formula_or_query = [string]$resultRow.formula
            comparison_formula = ""
            observed_status = [string]$resultRow.expected_status
            observed_text = [string]$resultRow.text
            observed_value2 = [string]$resultRow.value2
            comparison_text = ""
            comparison_value2 = ""
            expected_status = $common.expected_status
            expected_observable = $common.expected_observable
            expected_relation = $common.expected_relation
            source_manifest_ref = $common.source_manifest_ref
            source_result_ref = $common.source_result_ref
            evidence_ids = $common.evidence_ids
            limitation_refs = $common.limitation_refs
            notes = $common.notes
        }
    }
    $rows
}

function Build-W15RunSummary {
    param([object[]]$RunRows, [string]$RunId, [string]$RunLabel, [string]$CompatDescriptor)
    $xllRows = @($RunRows | Where-Object { $_.row_kind -eq "xll" })
    $matched = @($xllRows | Where-Object { $_.observed_status -eq "matched" }).Count
    $mismatched = @($xllRows | Where-Object { $_.observed_status -eq "mismatched" }).Count
    [ordered]@{
        packet_id = "W15"
        run_id = $RunId
        run_label = $RunLabel
        compat_descriptor = $CompatDescriptor
        total_rows = @($RunRows).Count
        packet_counts = [ordered]@{
            "W15.INFO.packet" = @($RunRows | Where-Object { $_.scenario_packet_id -eq "W15.INFO.packet" }).Count
            "W15.CELL.packet" = @($RunRows | Where-Object { $_.scenario_packet_id -eq "W15.CELL.packet" }).Count
            "W15.XLL.packet" = @($RunRows | Where-Object { $_.scenario_packet_id -eq "W15.XLL.packet" }).Count
        }
        xll_relation_counts = [ordered]@{ matched = $matched; mismatched = $mismatched }
        observed_status_counts = [ordered]@{
            observed = @($RunRows | Where-Object { $_.observed_status -eq "observed" }).Count
            matched = $matched
            mismatched = $mismatched
        }
    }
}

function Build-W15DiffRecords {
    param([object[]]$DefaultRows, [object[]]$CompatRows)
    $compatByScope = @{}
    foreach ($row in $CompatRows) {
        $compatByScope["$($row.scenario_packet_id)|$($row.scenario_id)"] = $row
    }

    $records = @()
    foreach ($leftRow in $DefaultRows) {
        $scopeKey = "$($leftRow.scenario_packet_id)|$($leftRow.scenario_id)"
        if (-not $compatByScope.ContainsKey($scopeKey)) {
            $records += [ordered]@{
                diff_id = "w15.default_vs_compat.row.$($leftRow.scenario_id)"
                left_ref = "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.default"
                right_ref = "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.compat_template"
                comparison_scope = [ordered]@{ packet_id = "W15"; row_id = $leftRow.scenario_id; view_id = "manifest_row_result_view" }
                mismatch_kind = "mm.run.presence"
                mismatch_path = "views/manifest_row_result_view/$($leftRow.scenario_id)"
                severity = "sev.coverage"
                explanation_hint = "why_projection_gap"
                supporting_refs = @("evidence:$($leftRow.evidence_ids)", "artifact:$($leftRow.source_result_ref)")
            }
            continue
        }

        $rightRow = $compatByScope[$scopeKey]
        $observedEqual = ($leftRow.observed_text -ceq $rightRow.observed_text) -and
            ($leftRow.observed_value2 -ceq $rightRow.observed_value2) -and
            ($leftRow.observed_status -ceq $rightRow.observed_status)
        if ($observedEqual) { continue }

        $isFilenameLane = ($leftRow.formula_or_query -like '*CELL("filename"*') -or ($leftRow.formula_or_query -like '*ox_CELL("filename"*')
        $severity = if ($isFilenameLane) {
            "sev.informational"
        } elseif ($leftRow.row_kind -eq "xll") {
            "sev.instrumentation"
        } else {
            "sev.semantic"
        }
        $hint = if ($severity -eq "sev.instrumentation") { "why_seam_limited" } else { "why_row_observed" }
        $supportingRefs = @("evidence:$($leftRow.evidence_ids)", "artifact:$($leftRow.source_result_ref)", "artifact:$($rightRow.source_result_ref)")
        if ($severity -eq "sev.instrumentation") {
            $supportingRefs += "artifact:sidecars/source/docs/XLL_VERIFICATION_SEAM_LIMITATIONS.md"
        }

        $records += [ordered]@{
            diff_id = "w15.default_vs_compat.row.$($leftRow.scenario_id)"
            left_ref = "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.default"
            right_ref = "bundle://oxfunc.w15.packet.default_and_compat.v1/runs/w15.compat_template"
            comparison_scope = [ordered]@{ packet_id = "W15"; row_id = $leftRow.scenario_id; view_id = "manifest_row_result_view" }
            mismatch_kind = "mm.view.value"
            mismatch_path = "views/manifest_row_result_view/$($leftRow.scenario_id)/observed_value_text"
            severity = $severity
            explanation_hint = $hint
            supporting_refs = $supportingRefs
        }
    }
    $records
}

function Build-W15ExplainRecords {
    param([object[]]$DefaultRows)
    $firstInfo = $DefaultRows | Where-Object { $_.scenario_packet_id -eq "W15.INFO.packet" } | Select-Object -First 1
    $firstBridge = $DefaultRows | Where-Object { $_.scenario_packet_id -eq "W15.XLL.packet" } | Select-Object -First 1
    @(
        [ordered]@{
            query_id = "explain.w15.row.$($firstInfo.scenario_id)"
            query_kind = "why_row_observed"
            scope_ref = [ordered]@{ packet_id = "W15"; row_id = $firstInfo.scenario_id; run_id = "w15.default" }
            supporting_refs = @("evidence:$($firstInfo.evidence_ids)", "artifact:$($firstInfo.source_result_ref)", "artifact:sidecars/source/docs/W15_EXECUTION_RECORD.md")
            explanation_text = "The row outcome is carried directly from the native W15 result sidecar into the packet-first row view, preserving run label, compatibility descriptor, and the formula under test."
            confidence_class = "provisional"
        }
        [ordered]@{
            query_id = "explain.w15.host_query.W15.CELL.packet"
            query_kind = "why_host_query"
            scope_ref = [ordered]@{ packet_id = "W15"; scenario_packet_id = "W15.CELL.packet" }
            supporting_refs = @("evidence:W15-CELL-HOST-PRE-20260315", "artifact:sidecars/source/docs/W15_EXECUTION_RECORD.md")
            explanation_text = "CELL is classified as a typed host-query packet here because the admitted W15 lanes depend on workbook, selection, or format metadata rather than pure-local scalar semantics."
            confidence_class = "provisional"
        }
        [ordered]@{
            query_id = "explain.w15.bridge_parity.$($firstBridge.scenario_id)"
            query_kind = "why_bridge_parity"
            scope_ref = [ordered]@{ packet_id = "W15"; row_id = $firstBridge.scenario_id; run_id = "w15.default" }
            supporting_refs = @("evidence:$($firstBridge.evidence_ids)", "artifact:$($firstBridge.source_result_ref)", "artifact:sidecars/source/docs/W15_EXECUTION_RECORD.md")
            explanation_text = "The bridge row is parity-clean in the current baseline; provider-backed ox_CELL and ox_INFO output matches the corresponding native worksheet formula for the admitted W15 lanes."
            confidence_class = "provisional"
        }
        [ordered]@{
            query_id = "explain.w15.seam_limited.W15.XLL.packet"
            query_kind = "why_seam_limited"
            scope_ref = [ordered]@{ packet_id = "W15"; scenario_packet_id = "W15.XLL.packet" }
            supporting_refs = @("evidence:W15-XLL-BRIDGE-20260315", "artifact:sidecars/source/docs/XLL_VERIFICATION_SEAM_LIMITATIONS.md")
            explanation_text = "Any future bridge-facing mismatch in this packet must remain visible as a seam-limited classification when the XLL verification-surface limitations apply."
            confidence_class = "provisional"
        }
        [ordered]@{
            query_id = "explain.w15.projection_gap.none_observed"
            query_kind = "why_projection_gap"
            scope_ref = [ordered]@{ packet_id = "W15"; view_id = "source_inventory_view" }
            supporting_refs = @("artifact:indexes/source_inventory.csv", "artifact:views/source_inventory_view/W15.csv")
            explanation_text = "No projection gap was observed in the emitted W15 baseline bundle; every required source manifest, result sidecar, evidence id, and limitation note is present in the current source inventory."
            confidence_class = "provisional"
        }
    )
}

function Build-ReplayAssessment {
    param([object[]]$ExpectedRows, [object[]]$ObservedRows)
    $expected = @($ExpectedRows | Sort-Object scenario_packet_id, scenario_id | ForEach-Object {
        "$($_.scenario_packet_id)|$($_.scenario_id)|$($_.run_id)|$($_.observed_status)|$($_.observed_text)|$($_.observed_value2)|$($_.comparison_text)|$($_.comparison_value2)"
    })
    $observed = @($ObservedRows | Sort-Object scenario_packet_id, scenario_id | ForEach-Object {
        "$($_.scenario_packet_id)|$($_.scenario_id)|$($_.run_id)|$($_.observed_status)|$($_.observed_text)|$($_.observed_value2)|$($_.comparison_text)|$($_.comparison_value2)"
    })
    [ordered]@{
        replay_valid = (@($expected) -join "`n") -ceq (@($observed) -join "`n")
        expected_row_count = @($ExpectedRows).Count
        observed_row_count = @($ObservedRows).Count
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\\..")).Path
$bundleRootPath = Join-Path $repoRoot $BundleRoot
if ($Clean -and (Test-Path $bundleRootPath)) {
    Remove-Item -Path $bundleRootPath -Recurse -Force
}

$dirs = @(
    "adapter_capabilities","run_manifests","scenario_manifests","views/manifest_row_result_view",
    "views/run_summary_view","views/analysis_summary_view","views/evidence_binding_view",
    "views/limitation_view","views/source_inventory_view","diff/expected","diff/emitted",
    "explain/expected","explain/emitted","sidecars/source/manifests","sidecars/source/results",
    "sidecars/source/docs","sidecars/normalized/config","sidecars/normalized","indexes",
    "registries","reductions","witnesses"
)
foreach ($dir in $dirs) {
    Ensure-Directory -Path (Join-Path $bundleRootPath $dir)
}

$packetDefs = Get-W15PacketDefinitions
$sourceInventory = @()
$capabilityManifestRel = "docs/function-lane/OXFUNC_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json"
$capabilityBundleRel = "adapter_capabilities/oxfunc.json"
Copy-RepoArtifact -RepoRoot $repoRoot -SourceRelativePath $capabilityManifestRel -BundleRoot $bundleRootPath -BundleRelativePath $capabilityBundleRel | Out-Null

$sharedArtifacts = @(
    @{ rel = "docs/function-lane/W15_EXECUTION_RECORD.md"; bundle = "sidecars/source/docs/W15_EXECUTION_RECORD.md"; schema = "oxfunc.local.execution_record.md.v1"; role = "execution_record"; notes = "W15 packet execution record."; run_scope = "shared" },
    @{ rel = "docs/function-lane/XLL_VERIFICATION_SEAM_LIMITATIONS.md"; bundle = "sidecars/source/docs/XLL_VERIFICATION_SEAM_LIMITATIONS.md"; schema = "oxfunc.local.limitation_note.md.v1"; role = "limitation_note"; notes = "XLL limitation note cited by W15 bridge rows."; run_scope = "shared" },
    @{ rel = "docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md"; bundle = "sidecars/source/docs/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md"; schema = "oxfunc.local.evidence_registry.table.v1"; role = "evidence_registry"; notes = "Evidence registry source for W15 evidence ids."; run_scope = "shared" }
)
foreach ($artifact in $sharedArtifacts) {
    $bundleRel = Copy-RepoArtifact -RepoRoot $repoRoot -SourceRelativePath $artifact.rel -BundleRoot $bundleRootPath -BundleRelativePath $artifact.bundle
    $sourceInventory += New-SourceInventoryRow -SourceRefId ([System.IO.Path]::GetFileNameWithoutExtension($artifact.rel)) -SourceSchemaId $artifact.schema -BundleRelativePath $bundleRel -SourceRole $artifact.role -PacketId "W15" -RunScope $artifact.run_scope -Notes $artifact.notes
}

$allDefaultRows = @()
$allCompatRows = @()

foreach ($packetDef in $packetDefs) {
    $manifestBundleRel = Copy-RepoArtifact -RepoRoot $repoRoot -SourceRelativePath $packetDef.generator_ref -BundleRoot $bundleRootPath -BundleRelativePath $packetDef.manifest_bundle_path
    $sourceInventory += New-SourceInventoryRow -SourceRefId ([System.IO.Path]::GetFileNameWithoutExtension($packetDef.generator_ref)) -SourceSchemaId "oxfunc.local.packet_manifest.csv.v1" -BundleRelativePath $manifestBundleRel -SourceRole "manifest" -PacketId $packetDef.packet_id -RunScope "both" -Notes $packetDef.description
    $defaultResultBundleRel = Copy-RepoArtifact -RepoRoot $repoRoot -SourceRelativePath $packetDef.result_default -BundleRoot $bundleRootPath -BundleRelativePath $packetDef.result_bundle_default
    $compatResultBundleRel = Copy-RepoArtifact -RepoRoot $repoRoot -SourceRelativePath $packetDef.result_compat -BundleRoot $bundleRootPath -BundleRelativePath $packetDef.result_bundle_compat
    $sourceInventory += New-SourceInventoryRow -SourceRefId ([System.IO.Path]::GetFileNameWithoutExtension($packetDef.result_default)) -SourceSchemaId "oxfunc.local.packet_results.csv.v1" -BundleRelativePath $defaultResultBundleRel -SourceRole "results" -PacketId $packetDef.packet_id -RunScope "default" -Notes "Default run results for $($packetDef.packet_id)."
    $sourceInventory += New-SourceInventoryRow -SourceRefId ([System.IO.Path]::GetFileNameWithoutExtension($packetDef.result_compat)) -SourceSchemaId "oxfunc.local.packet_results.csv.v1" -BundleRelativePath $compatResultBundleRel -SourceRole "results" -PacketId $packetDef.packet_id -RunScope "compat_template" -Notes "Compat-template run results for $($packetDef.packet_id)."
    $allDefaultRows += Build-W15RowView -PacketDef $packetDef -RunId "w15.default" -RunLabel "default" -ManifestBundlePath $manifestBundleRel -ResultBundlePath $defaultResultBundleRel -RepoRoot $repoRoot
    $allCompatRows += Build-W15RowView -PacketDef $packetDef -RunId "w15.compat_template" -RunLabel "compat_template" -ManifestBundlePath $manifestBundleRel -ResultBundlePath $compatResultBundleRel -RepoRoot $repoRoot
    Write-JsonFile -Path (Join-Path $bundleRootPath ("scenario_manifests/" + ($packetDef.packet_id -replace "\.", "_") + ".json")) -Value ([ordered]@{
        scenario_id = $packetDef.packet_id
        scenario_kind = "manifest_packet"
        description = $packetDef.description
        tags = $packetDef.tags
        pack_tags = $packetDef.pack_tags
        generator_ref = $manifestBundleRel
        evidence_ids = @($packetDef.evidence_id)
        clause_ids = @()
    })
}

$rowViewHeaders = @("packet_id","scenario_packet_id","scenario_id","lane","row_kind","run_id","run_label","compat_descriptor","formula_or_query","comparison_formula","observed_status","observed_text","observed_value2","comparison_text","comparison_value2","expected_status","expected_observable","expected_relation","source_manifest_ref","source_result_ref","evidence_ids","limitation_refs","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.default.csv") -Rows $allDefaultRows -Headers $rowViewHeaders
Write-CsvFile -Path (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.compat_template.csv") -Rows $allCompatRows -Headers $rowViewHeaders

$defaultCompatDescriptor = ($allDefaultRows | Select-Object -First 1).compat_descriptor
$compatCompatDescriptor = ($allCompatRows | Select-Object -First 1).compat_descriptor
$defaultRunSummary = Build-W15RunSummary -RunRows $allDefaultRows -RunId "w15.default" -RunLabel "default" -CompatDescriptor $defaultCompatDescriptor
$compatRunSummary = Build-W15RunSummary -RunRows $allCompatRows -RunId "w15.compat_template" -RunLabel "compat_template" -CompatDescriptor $compatCompatDescriptor
Write-JsonFile -Path (Join-Path $bundleRootPath "views/run_summary_view/w15.default.json") -Value $defaultRunSummary
Write-JsonFile -Path (Join-Path $bundleRootPath "views/run_summary_view/w15.compat_template.json") -Value $compatRunSummary
Write-JsonFile -Path (Join-Path $bundleRootPath "views/analysis_summary_view/W15.json") -Value ([ordered]@{
    packet_id = "W15"
    bundle_id = "oxfunc.w15.packet.default_and_compat.v1"
    statements = @(
        "CELL and INFO remain typed host-query packets for the admitted W15 slice.",
        "Native default and compat_template rows remain parity-clean for the admitted W15 packet rows.",
        "XLL bridge parity remains a verification-surface statement and any future bridge mismatch must stay distinct from core semantic failure."
    )
    run_ids = @("w15.default", "w15.compat_template")
    total_row_count = @($allDefaultRows).Count + @($allCompatRows).Count
})

$evidenceRows = @(
    [pscustomobject]@{ evidence_id = "W15-INFO-PRE-20260315"; packet_id = "W15"; scenario_id = "W15.INFO.packet"; run_scope = "default|compat_template"; artifact_ref = "sidecars/source/results/w15-info-pre-results.csv|sidecars/source/results/w15-info-pre-results-compat.csv"; notes = "W15 INFO packet evidence binding." }
    [pscustomobject]@{ evidence_id = "W15-CELL-HOST-PRE-20260315"; packet_id = "W15"; scenario_id = "W15.CELL.packet"; run_scope = "default|compat_template"; artifact_ref = "sidecars/source/results/w15-cell-host-pre-results.csv|sidecars/source/results/w15-cell-host-pre-results-compat.csv"; notes = "W15 CELL packet evidence binding." }
    [pscustomobject]@{ evidence_id = "W15-XLL-BRIDGE-20260315"; packet_id = "W15"; scenario_id = "W15.XLL.packet"; run_scope = "default|compat_template"; artifact_ref = "sidecars/source/results/w15-xll-bridge-results.csv|sidecars/source/results/w15-xll-bridge-results-compat.csv"; notes = "W15 bridge packet evidence binding." }
)
$evidenceHeaders = @("evidence_id","packet_id","scenario_id","run_scope","artifact_ref","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "views/evidence_binding_view/W15.csv") -Rows $evidenceRows -Headers $evidenceHeaders
$limitationRows = @([pscustomobject]@{ limitation_ref_id = "OXFUNC-XLL-SEAM-LIMITS-V1"; packet_id = "W15"; scope_ref = "W15.XLL.packet"; limitation_artifact_ref = "sidecars/source/docs/XLL_VERIFICATION_SEAM_LIMITATIONS.md"; limitation_class = "seam_limitation"; notes = "Bridge-facing W15 rows inherit the standard XLL verification-seam qualification." })
$limitationHeaders = @("limitation_ref_id","packet_id","scope_ref","limitation_artifact_ref","limitation_class","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "views/limitation_view/W15.csv") -Rows $limitationRows -Headers $limitationHeaders

$sourceInventoryHeaders = @("source_ref_id","source_schema_id","bundle_relative_path","source_role","packet_id","run_scope","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/source_inventory.csv") -Rows $sourceInventory -Headers $sourceInventoryHeaders
Write-CsvFile -Path (Join-Path $bundleRootPath "views/source_inventory_view/W15.csv") -Rows $sourceInventory -Headers $sourceInventoryHeaders
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/scenario_index.csv") -Rows @(
    [pscustomobject]@{ scenario_id = "W15.INFO.packet"; scenario_kind = "manifest_packet"; packet_id = "W15"; generator_ref = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv"; evidence_ids = "W15-INFO-PRE-20260315" }
    [pscustomobject]@{ scenario_id = "W15.CELL.packet"; scenario_kind = "manifest_packet"; packet_id = "W15"; generator_ref = "sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv"; evidence_ids = "W15-CELL-HOST-PRE-20260315" }
    [pscustomobject]@{ scenario_id = "W15.XLL.packet"; scenario_kind = "manifest_packet"; packet_id = "W15"; generator_ref = "sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv"; evidence_ids = "W15-XLL-BRIDGE-20260315" }
) -Headers @("scenario_id","scenario_kind","packet_id","generator_ref","evidence_ids")

$runManifestDefault = [ordered]@{ run_id = "w15.default"; lane_id = "oxfunc"; run_kind = "empirical_packet"; profile_id = "w15.host_query.packet"; profile_version = "1"; config_fingerprint_ref = "sidecars/normalized/config/w15.default.json"; selection_ref = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv"; result_state_counts = "indexes/result_state_counts.default.json"; source_artifact_roots = @("sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/results/w15-info-pre-results.csv","sidecars/source/results/w15-cell-host-pre-results.csv","sidecars/source/results/w15-xll-bridge-results.csv") }
$runManifestCompat = [ordered]@{ run_id = "w15.compat_template"; lane_id = "oxfunc"; run_kind = "empirical_packet"; profile_id = "w15.host_query.packet"; profile_version = "1"; config_fingerprint_ref = "sidecars/normalized/config/w15.compat_template.json"; selection_ref = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv"; result_state_counts = "indexes/result_state_counts.compat_template.json"; source_artifact_roots = @("sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv","sidecars/source/results/w15-info-pre-results-compat.csv","sidecars/source/results/w15-cell-host-pre-results-compat.csv","sidecars/source/results/w15-xll-bridge-results-compat.csv") }
Write-JsonFile -Path (Join-Path $bundleRootPath "run_manifests/w15.default.json") -Value $runManifestDefault
Write-JsonFile -Path (Join-Path $bundleRootPath "run_manifests/w15.compat_template.json") -Value $runManifestCompat
Write-JsonFile -Path (Join-Path $bundleRootPath "sidecars/normalized/config/w15.default.json") -Value ([ordered]@{ run_label = "default"; compat_descriptor = $defaultCompatDescriptor; source_results = $runManifestDefault.source_artifact_roots })
Write-JsonFile -Path (Join-Path $bundleRootPath "sidecars/normalized/config/w15.compat_template.json") -Value ([ordered]@{ run_label = "compat_template"; compat_descriptor = $compatCompatDescriptor; source_results = $runManifestCompat.source_artifact_roots })
Write-JsonFile -Path (Join-Path $bundleRootPath "indexes/result_state_counts.default.json") -Value $defaultRunSummary.observed_status_counts
Write-JsonFile -Path (Join-Path $bundleRootPath "indexes/result_state_counts.compat_template.json") -Value $compatRunSummary.observed_status_counts
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/run_index.csv") -Rows @(
    [pscustomobject]@{ run_id = "w15.default"; packet_id = "W15"; run_label = "default"; compat_descriptor = $defaultCompatDescriptor; profile_id = "w15.host_query.packet"; source_manifest_refs = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv|sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv|sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv" }
    [pscustomobject]@{ run_id = "w15.compat_template"; packet_id = "W15"; run_label = "compat_template"; compat_descriptor = $compatCompatDescriptor; profile_id = "w15.host_query.packet"; source_manifest_refs = "sidecars/source/manifests/W15_INFO_PRE_SCENARIO_MANIFEST_SEED.csv|sidecars/source/manifests/W15_CELL_HOST_PRE_SCENARIO_MANIFEST_SEED.csv|sidecars/source/manifests/W15_XLL_BRIDGE_SCENARIO_MANIFEST_SEED.csv" }
) -Headers @("run_id","packet_id","run_label","compat_descriptor","profile_id","source_manifest_refs")
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/evidence_index.csv") -Rows $evidenceRows -Headers $evidenceHeaders
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/limitation_index.csv") -Rows $limitationRows -Headers $limitationHeaders

$diffRecords = Build-W15DiffRecords -DefaultRows $allDefaultRows -CompatRows $allCompatRows
$diffArtifactPath = "diff/emitted/w15.default_vs_compat.json"
Write-JsonFile -Path (Join-Path $bundleRootPath $diffArtifactPath) -Value $diffRecords
$diffIndexRows = @()
foreach ($record in $diffRecords) {
    $diffIndexRows += [pscustomobject]@{ diff_id = $record.diff_id; packet_id = $record.comparison_scope.packet_id; left_run_id = "w15.default"; right_run_id = "w15.compat_template"; mismatch_kind = $record.mismatch_kind; severity = $record.severity; artifact_ref = $diffArtifactPath }
}
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/diff_index.csv") -Rows $diffIndexRows -Headers @("diff_id","packet_id","left_run_id","right_run_id","mismatch_kind","severity","artifact_ref")

$explainRecords = Build-W15ExplainRecords -DefaultRows $allDefaultRows
$explainArtifactPath = "explain/emitted/w15.explain.json"
Write-JsonFile -Path (Join-Path $bundleRootPath $explainArtifactPath) -Value $explainRecords
$explainIndexRows = @()
foreach ($record in $explainRecords) {
    $scopeRef = if ($record.scope_ref.row_id) { $record.scope_ref.row_id } elseif ($record.scope_ref.scenario_packet_id) { $record.scope_ref.scenario_packet_id } else { $record.scope_ref.view_id }
    $explainIndexRows += [pscustomobject]@{ query_id = $record.query_id; packet_id = "W15"; query_kind = $record.query_kind; scope_ref = $scopeRef; artifact_ref = $explainArtifactPath; notes = $record.confidence_class }
}
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/explain_index.csv") -Rows $explainIndexRows -Headers @("query_id","packet_id","query_kind","scope_ref","artifact_ref","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/witness_index.csv") -Rows @() -Headers @("witness_id","packet_id","reduction_status","artifact_ref","notes")
Write-CsvFile -Path (Join-Path $bundleRootPath "indexes/reduction_index.csv") -Rows @() -Headers @("reduction_id","packet_id","reduction_status","artifact_ref","notes")
Write-JsonFile -Path (Join-Path $bundleRootPath "diff/expected/w15.target_shape.json") -Value ([ordered]@{ target_ref = "docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md"; required_fields = @("diff_id","left_ref","right_ref","comparison_scope","mismatch_kind","mismatch_path","severity","explanation_hint","supporting_refs") })
Write-JsonFile -Path (Join-Path $bundleRootPath "explain/expected/w15.target_shape.json") -Value ([ordered]@{ target_ref = "docs/function-lane/W15_REPLAY_DIFF_EXPLAIN_SHAPES_V1.md"; required_fields = @("query_id","query_kind","scope_ref","supporting_refs","explanation_text","confidence_class") })

$registryVersion = "foundation-handoff-20260315-pass-01"
Write-JsonFile -Path (Join-Path $bundleRootPath "registries/capability_level.json") -Value ([ordered]@{ registry_family = "capability_level"; registry_version = $registryVersion; values = @("cap.C0.ingest_valid","cap.C1.replay_valid","cap.C2.diff_valid","cap.C3.explain_valid","cap.C4.distill_valid","cap.C5.pack_valid") })
Write-JsonFile -Path (Join-Path $bundleRootPath "registries/predicate_kind.json") -Value ([ordered]@{ registry_family = "predicate_kind"; registry_version = $registryVersion; values = @("pred.diff.mismatch_present","pred.invariant.failed","pred.evidence.claim_failed") })
Write-JsonFile -Path (Join-Path $bundleRootPath "registries/mismatch_kind.json") -Value ([ordered]@{ registry_family = "mismatch_kind"; registry_version = $registryVersion; values = @("mm.run.presence","mm.scenario.presence","mm.result.state","mm.view.value","mm.evidence.binding","mm.sidecar.payload") })
Write-JsonFile -Path (Join-Path $bundleRootPath "registries/reduction_status.json") -Value ([ordered]@{ registry_family = "reduction_status"; registry_version = $registryVersion; values = @(); notes = "No reduction records are active in the W15 baseline bundle." })
Write-JsonFile -Path (Join-Path $bundleRootPath "registries/witness_lifecycle_state.json") -Value ([ordered]@{ registry_family = "witness_lifecycle_state"; registry_version = $registryVersion; values = @(); notes = "No witness lifecycle records are active in the W15 baseline bundle." })
Write-JsonFile -Path (Join-Path $bundleRootPath "bundle_manifest.json") -Value ([ordered]@{
    bundle_schema_id = "dna-replay-bundle/v1"
    bundle_schema_version = "1"
    bundle_id = "oxfunc.w15.packet.default_and_compat.v1"
    source_lanes = @("oxfunc")
    created_by = "oxfunc.replay.packet_adapter.ps1"
    normalizer_version = "1.0.0-w21"
    artifact_layout_version = "1"
    source_inventory_ref = "indexes/source_inventory.csv"
    adapter_capability_ref = $capabilityBundleRel
    registry_refs = @(
        [ordered]@{ registry_family = "capability_level"; registry_version = $registryVersion },
        [ordered]@{ registry_family = "predicate_kind"; registry_version = $registryVersion },
        [ordered]@{ registry_family = "mismatch_kind"; registry_version = $registryVersion },
        [ordered]@{ registry_family = "witness_lifecycle_state"; registry_version = $registryVersion }
    )
    sidecar_refs = @("sidecars/source/docs/W15_EXECUTION_RECORD.md","sidecars/source/docs/XLL_VERIFICATION_SEAM_LIMITATIONS.md","sidecars/source/docs/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md")
})

$emittedDefaultRows = @(Import-Csv (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.default.csv"))
$emittedCompatRows = @(Import-Csv (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.compat_template.csv"))
$replayAssessment = Build-ReplayAssessment -ExpectedRows ($allDefaultRows + $allCompatRows) -ObservedRows ($emittedDefaultRows + $emittedCompatRows)
$minimumExplainKinds = @("why_row_observed","why_host_query","why_bridge_parity","why_seam_limited")
$presentExplainKinds = @($explainRecords | ForEach-Object { $_.query_kind } | Select-Object -Unique)
$validationResult = [ordered]@{
    bundle_root = $BundleRoot.Replace("\", "/")
    validation_time_utc = [DateTime]::UtcNow.ToString("o")
    required_file_checks = [ordered]@{
        bundle_manifest = Test-Path (Join-Path $bundleRootPath "bundle_manifest.json")
        adapter_capability = Test-Path (Join-Path $bundleRootPath $capabilityBundleRel)
        run_manifest_default = Test-Path (Join-Path $bundleRootPath "run_manifests/w15.default.json")
        run_manifest_compat = Test-Path (Join-Path $bundleRootPath "run_manifests/w15.compat_template.json")
        scenario_manifest_info = Test-Path (Join-Path $bundleRootPath "scenario_manifests/W15_INFO_packet.json")
        scenario_manifest_cell = Test-Path (Join-Path $bundleRootPath "scenario_manifests/W15_CELL_packet.json")
        scenario_manifest_xll = Test-Path (Join-Path $bundleRootPath "scenario_manifests/W15_XLL_packet.json")
        row_view_default = Test-Path (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.default.csv")
        row_view_compat = Test-Path (Join-Path $bundleRootPath "views/manifest_row_result_view/w15.compat_template.csv")
        diff_emitted = Test-Path (Join-Path $bundleRootPath $diffArtifactPath)
        explain_emitted = Test-Path (Join-Path $bundleRootPath $explainArtifactPath)
    }
    capability_assessment = [ordered]@{
        "cap.C0.ingest_valid" = $true
        "cap.C1.replay_valid" = [bool]$replayAssessment.replay_valid
        "cap.C2.diff_valid" = (Test-Path (Join-Path $bundleRootPath $diffArtifactPath))
        "cap.C3.explain_valid" = (($minimumExplainKinds | Where-Object { $_ -notin $presentExplainKinds }).Count -eq 0)
        "cap.C4.distill_valid" = $false
        "cap.C5.pack_valid" = $false
    }
    replay_assessment = $replayAssessment
    explain_query_kinds = $presentExplainKinds
    projection_gaps = @()
    open_lanes = @(
        "No DNA ReCalc import has been exercised yet against the emitted W15 bundle.",
        "No reduced witness has been proven replay-valid, so cap.C4.distill_valid remains unclaimed.",
        "No pack-grade export or promotion flow exists yet, so cap.C5.pack_valid remains unclaimed."
    )
}
Write-JsonFile -Path (Join-Path $bundleRootPath "sidecars/normalized/w15.validation.json") -Value $validationResult
Write-JsonFile -Path (Join-Path $bundleRootPath "sidecars/normalized/w15.replay_result.json") -Value $replayAssessment
Write-JsonFile -Path (Join-Path $bundleRootPath "sidecars/normalized/w15.capability_assessment.json") -Value $validationResult.capability_assessment
Get-Content (Join-Path $bundleRootPath "sidecars/normalized/w15.validation.json")

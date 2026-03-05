param(
    [Parameter(Mandatory = $true)]
    [string]$ExcelResults,

    [Parameter(Mandatory = $true)]
    [string]$LeanResults,

    [string]$Ledger = "docs/function-lane/FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv"
)

$ErrorActionPreference = "Stop"

function Get-Classification {
    param(
        [object]$ExcelRow,
        [object]$LeanRow
    )

    $excelClass = [string]$ExcelRow.observed_class
    $excelText = [string]$ExcelRow.primary_text
    $excelValue2 = [string]$ExcelRow.primary_value2
    $leanClass = [string]$LeanRow.observed_class
    $leanText = [string]$LeanRow.primary_text
    $leanBits = [string]$LeanRow.primary_value2

    if ($excelClass.StartsWith("error:") -and $leanClass.StartsWith("value:")) {
        return @{
            relation_status = "divergent"
            divergence_class = "excel_error_surface_normalization"
            implication_level = "high"
            contract_action = "document_and_bound"
            notes = "excel=$excelClass lean=$leanClass"
        }
    }

    if ($excelClass.StartsWith("value") -and $leanClass -eq "value:-0" -and $excelText -ne "-0") {
        return @{
            relation_status = "divergent"
            divergence_class = "signed_zero_surface_normalization"
            implication_level = "medium"
            contract_action = "document_and_bound"
            notes = "excel_text=$excelText lean_class=$leanClass"
        }
    }

    if ($excelClass.StartsWith("value") -and $leanClass -eq "value:finite") {
        $excelIsZero = $false
        try {
            $excelIsZero = ([double]$excelValue2 -eq 0.0)
        }
        catch {
            $excelIsZero = $false
        }

        if ($excelIsZero -and $leanBits -ne "0") {
            return @{
                relation_status = "divergent"
                divergence_class = "subnormal_or_tiny_flush_to_zero"
                implication_level = "medium"
                contract_action = "document_and_bound"
                notes = "excel_value2=$excelValue2 lean_bits=$leanBits lean_text=$leanText"
            }
        }
    }

    return @{
        relation_status = "aligned"
        divergence_class = "aligned"
        implication_level = "low"
        contract_action = "none"
        notes = "excel=$excelClass/$excelText lean=$leanClass/$leanText"
    }
}

$excelPath = (Resolve-Path $ExcelResults).Path
$leanPath = (Resolve-Path $LeanResults).Path
$ledgerPath = (Resolve-Path $Ledger).Path

$excelRows = Import-Csv $excelPath | Where-Object { $_.execution_status -eq "observed" }
$leanRows = Import-Csv $leanPath | Where-Object { $_.execution_status -eq "observed" }
$ledgerRows = Import-Csv $ledgerPath

$excelByScenario = @{}
foreach ($row in $excelRows) {
    $excelByScenario[[string]$row.scenario_id] = $row
}
$leanByScenario = @{}
foreach ($row in $leanRows) {
    $leanByScenario[[string]$row.scenario_id] = $row
}

$updated = foreach ($row in $ledgerRows) {
    $scenarioId = [string]$row.scenario_id
    $excelRow = $null
    $leanRow = $null
    if ($excelByScenario.ContainsKey($scenarioId)) { $excelRow = $excelByScenario[$scenarioId] }
    if ($leanByScenario.ContainsKey($scenarioId)) { $leanRow = $leanByScenario[$scenarioId] }

    if ($null -eq $excelRow -or $null -eq $leanRow) {
        [PSCustomObject]@{
            deviation_id = $row.deviation_id
            scenario_id = $scenarioId
            excel_observation_ref = if ($null -ne $excelRow) { "$excelPath#$scenarioId" } else { "" }
            lean_observation_ref = if ($null -ne $leanRow) { "$leanPath#$scenarioId" } else { "" }
            relation_status = "pending"
            divergence_class = "unclassified"
            implication_level = "unknown"
            contract_action = "pending"
            status = "draft"
            notes = "missing observation row(s) for comparison"
        }
        continue
    }

    $classification = Get-Classification -ExcelRow $excelRow -LeanRow $leanRow
    [PSCustomObject]@{
        deviation_id = $row.deviation_id
        scenario_id = $scenarioId
        excel_observation_ref = "$excelPath#$scenarioId"
        lean_observation_ref = "$leanPath#$scenarioId"
        relation_status = $classification.relation_status
        divergence_class = $classification.divergence_class
        implication_level = $classification.implication_level
        contract_action = $classification.contract_action
        status = "observed"
        notes = $classification.notes
    }
}

$updated | Export-Csv -Path $ledgerPath -NoTypeInformation -Encoding UTF8
Write-Host "Deviation ledger updated: $ledgerPath"

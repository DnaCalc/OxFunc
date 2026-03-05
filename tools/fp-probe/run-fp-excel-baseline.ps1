param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("FP-A", "FP-B", "FP-D"),

    [string]$WorkbookTemplate = "",

    [string]$ArtifactRoot = ".tmp/fp-artifacts",

    [string]$XllPath = ""
)

$ErrorActionPreference = "Stop"

function Get-ExcelChannel {
    $paths = @(
        "HKLM:\SOFTWARE\Microsoft\Office\ClickToRun\Configuration",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Office\ClickToRun\Configuration"
    )
    foreach ($path in $paths) {
        if (Test-Path $path) {
            $props = Get-ItemProperty -Path $path
            if ($props.UpdateChannel) { return [string]$props.UpdateChannel }
            if ($props.CDNBaseUrl) { return [string]$props.CDNBaseUrl }
        }
    }
    return ""
}

function Get-CompatibilityDescriptor {
    param([object]$Workbook)

    $calcVersion = ""
    $checkCompatibility = ""
    $fileFormat = ""
    try { $calcVersion = [string]$Workbook.CalculationVersion } catch { $calcVersion = "" }
    try { $checkCompatibility = [string]$Workbook.CheckCompatibility } catch { $checkCompatibility = "" }
    try { $fileFormat = [string]$Workbook.FileFormat } catch { $fileFormat = "" }

    return "default|CalculationVersion=$calcVersion|CheckCompatibility=$checkCompatibility|FileFormat=$fileFormat"
}

function Release-ComObjectSafe {
    param([object]$Obj)
    if ($null -ne $Obj) {
        try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($Obj) } catch {}
    }
}

function Close-WorkbookSafe {
    param([object]$Workbook)
    if ($null -ne $Workbook) {
        try { $Workbook.Close($false) | Out-Null } catch {}
    }
}

function Parse-OpAssignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }

        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid op_setup entry '$trimmed' (expected CELL:=FORMULA)."
        }

        $formula = $m.Groups[2].Value.Trim()
        if (-not $formula.StartsWith("=")) {
            $formula = "=$formula"
        }

        $items += [PSCustomObject]@{
            Cell = $m.Groups[1].Value
            Formula = $formula
        }
    }

    return $items
}

function Parse-FormulaSpec {
    param([string]$Raw)

    if ([string]::IsNullOrWhiteSpace($Raw)) { return $null }
    if ($Raw -eq "none") { return $null }

    $m = [regex]::Match($Raw, "^\s*([A-Z]{1,3}[0-9]+)\s+formula:\s*(=.*)$")
    if (-not $m.Success) { return $null }

    return [PSCustomObject]@{
        Cell = $m.Groups[1].Value
        Formula = $m.Groups[2].Value
    }
}

function Get-LegacyAssignments {
    param([object]$Row)

    $formulaSpecs = @()
    foreach ($fieldName in @("sheet_setup", "cell_formula_or_action", "comparison_cells")) {
        $spec = Parse-FormulaSpec -Raw ([string]$Row.$fieldName)
        if ($null -ne $spec) { $formulaSpecs += $spec }
    }
    return $formulaSpecs
}

function Parse-ObserveCells {
    param(
        [string]$Raw,
        [object[]]$FallbackAssignments
    )

    $cells = @()
    if (-not [string]::IsNullOrWhiteSpace($Raw)) {
        foreach ($entry in ($Raw -split '\|')) {
            $trimmed = $entry.Trim()
            if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
            if (-not ($trimmed -match '^[A-Z]{1,3}[0-9]+$')) {
                throw "Invalid op_observe_cells entry '$trimmed'."
            }
            $cells += $trimmed
        }
    }

    if ($cells.Count -eq 0 -and $FallbackAssignments.Count -gt 0) {
        $cells = @($FallbackAssignments | ForEach-Object { $_.Cell } | Select-Object -Unique)
    }

    return $cells
}

function Convert-Value2ToString {
    param([object]$Value)
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    return [string]$Value
}

function Get-CellSnapshot {
    param(
        [object]$Worksheet,
        [string]$Cell
    )

    $range = $Worksheet.Range($Cell)
    $text = [string]$range.Text
    $formula = [string]$range.Formula
    $formula2 = $formula
    try { $formula2 = [string]$range.Formula2 } catch { $formula2 = $formula }
    $value2Raw = $range.Value2
    $value2 = Convert-Value2ToString -Value $value2Raw
    $value2Type = if ($null -eq $value2Raw) { "null" } else { $value2Raw.GetType().Name }

    return [PSCustomObject]@{
        cell = $Cell
        text = $text
        formula = $formula
        formula2 = $formula2
        value2 = $value2
        value2_type = $value2Type
    }
}

function Serialize-Snapshots {
    param([object[]]$Snapshots)
    if ($null -eq $Snapshots -or $Snapshots.Count -eq 0) { return "" }
    return ($Snapshots | ForEach-Object {
        "$($_.cell){text=$($_.text);value2=$($_.value2);type=$($_.value2_type);formula2=$($_.formula2)}"
    }) -join "|"
}

function New-ScenarioWorkbook {
    param(
        [object]$Excel,
        [string]$ScenarioId,
        [string]$TemplatePath,
        [string]$ScenarioArtifactDir
    )

    if ([string]::IsNullOrWhiteSpace($TemplatePath)) {
        return [PSCustomObject]@{
            Workbook = $Excel.Workbooks.Add()
            ArtifactRef = ""
        }
    }

    $ext = [System.IO.Path]::GetExtension($TemplatePath)
    if ([string]::IsNullOrWhiteSpace($ext)) { $ext = ".xlsx" }
    $copyPath = Join-Path $ScenarioArtifactDir "$ScenarioId.template$ext"
    Copy-Item -Path $TemplatePath -Destination $copyPath -Force

    return [PSCustomObject]@{
        Workbook = $Excel.Workbooks.Open($copyPath)
        ArtifactRef = $copyPath
    }
}

$manifestPath = Resolve-Path -Path $Manifest -ErrorAction Stop
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$artifactRootPath = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootPath)) {
    New-Item -ItemType Directory -Path $artifactRootPath | Out-Null
}

$templatePathResolved = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $templatePathResolved = (Resolve-Path -Path $WorkbookTemplate -ErrorAction Stop).Path
}

$xllPathResolved = ""
if (-not [string]::IsNullOrWhiteSpace($XllPath)) {
    $xllPathResolved = (Resolve-Path -Path $XllPath -ErrorAction Stop).Path
}

$rows = Import-Csv -Path $manifestPath
if (-not $rows -or $rows.Count -eq 0) {
    throw "Manifest has no data rows: $manifestPath"
}

$excel = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel

    $fpCLaneEnabled = ($Lanes -contains "FP-C")
    $fpCReady = $true
    $fpCFailureReason = ""
    if ($fpCLaneEnabled) {
        if ([string]::IsNullOrWhiteSpace($xllPathResolved)) {
            $fpCReady = $false
            $fpCFailureReason = "XllPath parameter is empty."
        }
        else {
            try {
                $registered = $excel.RegisterXLL($xllPathResolved)
                if (-not $registered) {
                    throw "Excel RegisterXLL returned false."
                }
            }
            catch {
                $fpCReady = $false
                $fpCFailureReason = [string]$_.Exception.Message
            }
        }
    }

    $result = New-Object System.Collections.Generic.List[object]

    foreach ($row in $rows) {
        if ($row.status -notin @("seed", "blocked_harness", "ready")) { continue }
        if ($Lanes -notcontains $row.lane) { continue }

        if ($row.lane -eq "FP-C" -and -not $fpCReady) {
            $result.Add([PSCustomObject]@{
                    scenario_id = $row.scenario_id
                    lane = $row.lane
                    mode = "excel-baseline"
                    execution_status = "blocked_missing_xll"
                    observed_class = "pending_observation"
                    excel_version = $excelVersionFull
                    excel_channel = $excelChannel
                    compat_version = ""
                    locale_profile = "en-US"
                    runner_version = "fp-excel-baseline-ps1/0.2.0"
                    artifact_ref = $xllPathResolved
                    primary_cell = ""
                    primary_formula2 = ""
                    primary_value2 = ""
                    primary_text = ""
                    observed_cells = ""
                    comparison_bools = ""
                    notes = "FP-C lane requested but XLL is unavailable or registration failed. reason=$fpCFailureReason"
                })
            continue
        }

        $workbook = $null
        $worksheet = $null
        $compatVersion = ""
        $artifactRef = ""

        try {
            $scenarioArtifactDir = Join-Path $artifactRootPath $row.scenario_id
            if (-not (Test-Path $scenarioArtifactDir)) {
                New-Item -ItemType Directory -Path $scenarioArtifactDir | Out-Null
            }

            $wbInfo = New-ScenarioWorkbook -Excel $excel -ScenarioId $row.scenario_id -TemplatePath $templatePathResolved -ScenarioArtifactDir $scenarioArtifactDir
            $workbook = $wbInfo.Workbook
            $artifactRef = [string]$wbInfo.ArtifactRef
            $worksheet = $workbook.Worksheets.Item(1)
            $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

            $assignments = @()
            if ($row.PSObject.Properties.Name -contains "op_setup") {
                $assignments = Parse-OpAssignments -Raw ([string]$row.op_setup)
            }
            if ($assignments.Count -eq 0) {
                $assignments = Get-LegacyAssignments -Row $row
            }

            if ($assignments.Count -eq 0) {
                $result.Add([PSCustomObject]@{
                    scenario_id = $row.scenario_id
                    lane = $row.lane
                    mode = "excel-baseline"
                    execution_status = "skipped_no_formula"
                    observed_class = "pending_observation"
                    excel_version = $excelVersionFull
                    excel_channel = $excelChannel
                    compat_version = $compatVersion
                    locale_profile = "en-US"
                    runner_version = "fp-excel-baseline-ps1/0.2.0"
                    artifact_ref = $artifactRef
                    primary_cell = ""
                    primary_formula2 = ""
                    primary_value2 = ""
                    primary_text = ""
                    observed_cells = ""
                    comparison_bools = ""
                    notes = "No formula specs parsed from manifest fields."
                })
                continue
            }

            $observeCells = @()
            if ($row.PSObject.Properties.Name -contains "op_observe_cells") {
                $observeCells = Parse-ObserveCells -Raw ([string]$row.op_observe_cells) -FallbackAssignments $assignments
            }
            if ($observeCells.Count -eq 0) {
                $observeCells = @($assignments | ForEach-Object { $_.Cell } | Select-Object -Unique)
            }

            $primaryCell = [string]$row.op_primary_cell
            if ([string]::IsNullOrWhiteSpace($primaryCell)) {
                $primaryCell = $assignments[0].Cell
            }

            foreach ($spec in $assignments) {
                try {
                    $worksheet.Range($spec.Cell).Formula = $spec.Formula
                }
                catch {
                    throw "formula set failed at $($spec.Cell) with '$($spec.Formula)': $($_.Exception.Message)"
                }
            }

            $action = [string]$row.op_action
            if ([string]::IsNullOrWhiteSpace($action)) {
                $action = "calculate"
            }

            $preSnapshots = @()
            $postSnapshots = @()
            $notesExtra = ""

            if ($action -eq "calculate") {
                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $postSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }
            }
            elseif ($action -eq "save_reopen_recalc") {
                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $preSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }

                $xlsxPath = Join-Path $scenarioArtifactDir "$($row.scenario_id).xlsx"
                $workbook.SaveAs($xlsxPath, 51) # xlOpenXMLWorkbook
                $artifactRef = $xlsxPath

                Close-WorkbookSafe -Workbook $workbook
                Release-ComObjectSafe -Obj $worksheet
                Release-ComObjectSafe -Obj $workbook
                $worksheet = $null
                $workbook = $null

                $workbook = $excel.Workbooks.Open($xlsxPath)
                $worksheet = $workbook.Worksheets.Item(1)
                $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook
                $excel.CalculateFull()

                foreach ($cell in $observeCells) {
                    $postSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }

                $notesExtra = "pre_save=" + (Serialize-Snapshots -Snapshots $preSnapshots)
            }
            elseif ($action -eq "csv_roundtrip_values") {
                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $preSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }

                $primaryRange = $worksheet.Range($primaryCell)
                $primaryRange.Value2 = $primaryRange.Value2

                $csvPath = Join-Path $scenarioArtifactDir "$($row.scenario_id).csv"
                $workbook.SaveAs($csvPath, 6) # xlCSV
                $artifactRef = $csvPath

                Close-WorkbookSafe -Workbook $workbook
                Release-ComObjectSafe -Obj $worksheet
                Release-ComObjectSafe -Obj $workbook
                $worksheet = $null
                $workbook = $null

                $workbook = $excel.Workbooks.Open($csvPath)
                $worksheet = $workbook.Worksheets.Item(1)
                $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook
                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $postSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }
                $csvLine = ""
                try { $csvLine = (Get-Content -Path $csvPath -TotalCount 1) } catch { $csvLine = "" }
                $notesExtra = "pre_export=" + (Serialize-Snapshots -Snapshots $preSnapshots) + "; csv_line=$csvLine"
            }
            else {
                throw "Unsupported op_action '$action'."
            }

            $primarySnapshot = $postSnapshots | Where-Object { $_.cell -eq $primaryCell } | Select-Object -First 1
            if ($null -eq $primarySnapshot -and $postSnapshots.Count -gt 0) {
                $primarySnapshot = $postSnapshots[0]
            }
            if ($null -eq $primarySnapshot) {
                throw "No post-action snapshots captured."
            }

            $observedClass = if ($primarySnapshot.text.StartsWith("#")) { "error:$($primarySnapshot.text)" } else { "value" }
            $comparisonBools = ($postSnapshots | Where-Object { $_.text -in @("TRUE", "FALSE") } | ForEach-Object { "$($_.cell)=$($_.text)" }) -join "|"

            $notesCombined = "action=$action; setup=$($row.op_setup); " + $notesExtra
            $result.Add([PSCustomObject]@{
                    scenario_id = $row.scenario_id
                    lane = $row.lane
                    mode = "excel-baseline"
                    execution_status = "observed"
                    observed_class = $observedClass
                    excel_version = $excelVersionFull
                    excel_channel = $excelChannel
                    compat_version = $compatVersion
                    locale_profile = "en-US"
                    runner_version = "fp-excel-baseline-ps1/0.2.0"
                    artifact_ref = $artifactRef
                    primary_cell = $primarySnapshot.cell
                    primary_formula2 = $primarySnapshot.formula2
                    primary_value2 = $primarySnapshot.value2
                    primary_text = $primarySnapshot.text
                    observed_cells = (Serialize-Snapshots -Snapshots $postSnapshots)
                    comparison_bools = $comparisonBools
                    notes = $notesCombined.Trim()
                })
        }
        catch {
            $result.Add([PSCustomObject]@{
                scenario_id = $row.scenario_id
                lane = $row.lane
                mode = "excel-baseline"
                execution_status = "failed"
                observed_class = "pending_observation"
                excel_version = $excelVersionFull
                excel_channel = $excelChannel
                compat_version = $compatVersion
                locale_profile = "en-US"
                runner_version = "fp-excel-baseline-ps1/0.2.0"
                artifact_ref = $artifactRef
                primary_cell = ""
                primary_formula2 = ""
                primary_value2 = ""
                primary_text = ""
                observed_cells = ""
                comparison_bools = ""
                notes = "scenario failure: $($_.Exception.Message)"
            })
        }
        finally {
            Close-WorkbookSafe -Workbook $workbook
            Release-ComObjectSafe -Obj $worksheet
            Release-ComObjectSafe -Obj $workbook
        }
    }

    $result | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "Excel baseline run complete. Rows written: $($result.Count)"
    Write-Host "Output: $outPath"
}
finally {
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

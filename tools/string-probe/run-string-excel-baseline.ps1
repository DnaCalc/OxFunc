param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("STR-C", "STR-D", "STR-E"),

    [string]$ArtifactRoot = ".tmp/string-artifacts",

    [string]$WorkbookTemplate = "",

    [string]$RunLabel = "default"
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

function Convert-Value2ToString {
    param([object]$Value)
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    return [string]$Value
}

function Parse-FormulaAssignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }

        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid formula_setup entry '$trimmed' (expected CELL:=FORMULA)."
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

function Parse-ValueAssignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }

        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid value_setup entry '$trimmed' (expected CELL:=VALUE_EXPR)."
        }

        $items += [PSCustomObject]@{
            Cell = $m.Groups[1].Value
            ValueExpr = $m.Groups[2].Value.Trim()
        }
    }

    return $items
}

function Parse-ObserveCells {
    param(
        [string]$Raw,
        [string]$PrimaryCell,
        [object[]]$FormulaAssignments,
        [object[]]$ValueAssignments
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

    if ($cells.Count -eq 0 -and -not [string]::IsNullOrWhiteSpace($PrimaryCell)) {
        $cells += $PrimaryCell
    }
    if ($cells.Count -eq 0 -and $FormulaAssignments.Count -gt 0) {
        $cells += @($FormulaAssignments | ForEach-Object { $_.Cell } | Select-Object -Unique)
    }
    if ($cells.Count -eq 0 -and $ValueAssignments.Count -gt 0) {
        $cells += @($ValueAssignments | ForEach-Object { $_.Cell } | Select-Object -Unique)
    }

    return @($cells | Select-Object -Unique)
}

function Resolve-ValueExpression {
    param([string]$Expr)

    $raw = $Expr.Trim()

    $repeatMatch = [regex]::Match($raw, '^repeat\(([^,]+),(\d+)\)$')
    if ($repeatMatch.Success) {
        $token = $repeatMatch.Groups[1].Value.Trim()
        $count = [int]$repeatMatch.Groups[2].Value

        if ($count -lt 0) {
            throw "Negative repeat count not allowed in '$Expr'."
        }

        switch ($token) {
            "x" { return ("x" * $count) }
            "space" { return (" " * $count) }
            "emoji_grinning" {
                $unit = (([char]0xD83D).ToString() + ([char]0xDE00).ToString())
                return ($unit * $count)
            }
            default { throw "Unknown repeat token '$token' in '$Expr'." }
        }
    }

    $tokenMatch = [regex]::Match($raw, '^token\(([^\)]+)\)$')
    if ($tokenMatch.Success) {
        $token = $tokenMatch.Groups[1].Value.Trim()

        switch ($token) {
            "csv_quotes" { return 'A, "B"' }
            "csv_newline" { return "A`nB" }
            "nbsp_wrap" { return ([char]160).ToString() + "A" + ([char]160).ToString() }
            "tab_wrap" { return "`tA`t" }
            "control_1_A" { return ([char]1).ToString() + "A" }
            "zws_wrap" { return ([char]0x200B).ToString() + "A" + ([char]0x200B).ToString() }
            "precomposed_e_acute" { return ([char]0x00E9).ToString() }
            "combining_e_acute" { return "e" + ([char]0x0301).ToString() }
            "ascii_word" { return "Alpha" }
            default { throw "Unknown token '$token' in '$Expr'." }
        }
    }

    $literalMatch = [regex]::Match($raw, '^literal\((.*)\)$')
    if ($literalMatch.Success) {
        return $literalMatch.Groups[1].Value
    }

    throw "Unsupported value expression '$Expr'."
}

function Build-FormulaLiteralFromValue {
    param([string]$Value)

    $escaped = $Value.Replace('"', '""')
    return '="' + $escaped + '"'
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
    $textLen = $text.Length

    return [PSCustomObject]@{
        cell = $Cell
        text = $text
        text_len = $textLen
        formula2 = $formula2
        value2 = $value2
        value2_type = $value2Type
    }
}

function Serialize-Snapshots {
    param([object[]]$Snapshots)

    if ($null -eq $Snapshots -or $Snapshots.Count -eq 0) { return "" }
    return ([object[]]$Snapshots | ConvertTo-Json -Compress -Depth 4)
}

function Get-ScenarioStringField {
    param(
        [object]$Scenario,
        [string]$FieldName
    )

    if ($Scenario.PSObject.Properties.Name -contains $FieldName) {
        return [string]$Scenario.$FieldName
    }
    return ""
}

function Evaluate-ExpectedObservable {
    param(
        [string]$ExpectedObservable,
        [string]$ExecutionStatus,
        [string]$PrimaryValue2,
        [string]$PrimaryText,
        [string]$PrimaryTextLen,
        [string]$Notes
    )

    if ([string]::IsNullOrWhiteSpace($ExpectedObservable)) {
        return [PSCustomObject]@{
            specified = $false
            matched = $true
            detail = "expected_observable not specified"
        }
    }

    $clauses = @($ExpectedObservable -split '&&' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($clauses.Count -eq 0) {
        return [PSCustomObject]@{
            specified = $false
            matched = $true
            detail = "expected_observable empty after parse"
        }
    }

    $detailParts = @()
    $allMatched = $true

    foreach ($clause in $clauses) {
        $matched = $false

        if ($clause.StartsWith("primary_value2_eq:")) {
            $expected = $clause.Substring("primary_value2_eq:".Length)
            $matched = ($PrimaryValue2 -ceq $expected)
        }
        elseif ($clause.StartsWith("primary_text_eq:")) {
            $expected = $clause.Substring("primary_text_eq:".Length)
            $matched = ($PrimaryText -ceq $expected)
        }
        elseif ($clause.StartsWith("primary_text_len_eq:")) {
            $expected = $clause.Substring("primary_text_len_eq:".Length)
            $matched = ($PrimaryTextLen -ceq $expected)
        }
        elseif ($clause.StartsWith("execution_status_eq:")) {
            $expected = $clause.Substring("execution_status_eq:".Length)
            $matched = ($ExecutionStatus -ceq $expected)
        }
        elseif ($clause.StartsWith("notes_contains:")) {
            $expected = $clause.Substring("notes_contains:".Length)
            $matched = ([string]$Notes).Contains($expected)
        }
        else {
            $expected = $clause
            $matched = $false
            $detailParts += "unsupported_clause=$clause"
        }

        if (-not ($detailParts -contains "unsupported_clause=$clause")) {
            $detailParts += "$clause=>$matched"
        }
        if (-not $matched) {
            $allMatched = $false
        }
    }

    return [PSCustomObject]@{
        specified = $true
        matched = $allMatched
        detail = ($detailParts -join '; ')
    }
}

function Resolve-Expectation {
    param(
        [object]$Scenario,
        [string]$ExecutionStatus,
        [string]$PrimaryValue2,
        [string]$PrimaryText,
        [string]$PrimaryTextLen,
        [string]$Notes
    )

    $expectedStatus = Get-ScenarioStringField -Scenario $Scenario -FieldName "expected_status"
    $expectedObservable = Get-ScenarioStringField -Scenario $Scenario -FieldName "expected_observable"

    $statusSpecified = -not [string]::IsNullOrWhiteSpace($expectedStatus)
    $statusMatched = $true
    $statusDetail = "expected_status not specified"
    if ($statusSpecified) {
        $statusMatched = ($ExecutionStatus -ceq $expectedStatus)
        $statusDetail = "expected_status=$expectedStatus actual=$ExecutionStatus match=$statusMatched"
    }

    $observableEval = Evaluate-ExpectedObservable -ExpectedObservable $expectedObservable -ExecutionStatus $ExecutionStatus -PrimaryValue2 $PrimaryValue2 -PrimaryText $PrimaryText -PrimaryTextLen $PrimaryTextLen -Notes $Notes

    $expectationStatus = "not_specified"
    if ($statusSpecified -or $observableEval.specified) {
        if ($statusMatched -and $observableEval.matched) {
            $expectationStatus = "matched"
        }
        else {
            $expectationStatus = "mismatched"
        }
    }

    return [PSCustomObject]@{
        expected_status = $expectedStatus
        expected_observable = $expectedObservable
        expectation_status = $expectationStatus
        expectation_detail = "$statusDetail | $($observableEval.detail)"
    }
}

function Add-ResultRow {
    param(
        [System.Collections.Generic.List[object]]$Rows,
        [object]$Scenario,
        [string]$ExecutionStatus,
        [string]$ObservedClass,
        [string]$ExcelVersion,
        [string]$ExcelChannel,
        [string]$CompatVersion,
        [string]$ArtifactRef,
        [string]$PrimaryCell,
        [string]$PrimaryFormula2,
        [string]$PrimaryValue2,
        [string]$PrimaryText,
        [string]$PrimaryTextLen,
        [string]$ObservedCells,
        [string]$ComparisonBools,
        [string]$Notes,
        [string]$RunnerVersion,
        [string]$RunLabel,
        [string]$SourceManifest
    )

    $expectation = Resolve-Expectation -Scenario $Scenario -ExecutionStatus $ExecutionStatus -PrimaryValue2 $PrimaryValue2 -PrimaryText $PrimaryText -PrimaryTextLen $PrimaryTextLen -Notes $Notes

    $Rows.Add([PSCustomObject]@{
        scenario_id = [string]$Scenario.scenario_id
        lane = [string]$Scenario.lane
        scenario_kind = [string]$Scenario.scenario_kind
        mode = "excel-baseline"
        execution_status = $ExecutionStatus
        observed_class = $ObservedClass
        expected_status = $expectation.expected_status
        expected_observable = $expectation.expected_observable
        expectation_status = $expectation.expectation_status
        expectation_detail = $expectation.expectation_detail
        excel_version = $ExcelVersion
        excel_channel = $ExcelChannel
        compat_version = $CompatVersion
        locale_profile = "en-US"
        runner_version = $RunnerVersion
        run_label = $RunLabel
        source_manifest = $SourceManifest
        artifact_ref = $ArtifactRef
        primary_cell = $PrimaryCell
        primary_formula2 = $PrimaryFormula2
        primary_value2 = $PrimaryValue2
        primary_text = $PrimaryText
        primary_text_len = $PrimaryTextLen
        observed_cells = $ObservedCells
        comparison_bools = $ComparisonBools
        objective = [string]$Scenario.objective
        notes = $Notes
    })
}

function Get-FileSha256 {
    param([string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path)) { return "" }
    if (-not (Test-Path $Path)) { return "" }
    return (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-GitCommit {
    try {
        return ((git rev-parse HEAD) 2>$null | Select-Object -First 1).Trim()
    }
    catch {
        return ""
    }
}

function Get-GitDirty {
    try {
        $line = (git status --porcelain 2>$null | Select-Object -First 1)
        return -not [string]::IsNullOrWhiteSpace($line)
    }
    catch {
        return $false
    }
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

$runStartedUtc = (Get-Date).ToUniversalTime()
$runnerVersion = "string-excel-baseline-ps1/0.2.0"

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
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

$scenarios = Import-Csv -Path $manifestPath
if (-not $scenarios -or $scenarios.Count -eq 0) {
    throw "Manifest has no scenario rows: $manifestPath"
}

$excel = $null
$excelVersionFull = ""
$excelChannel = ""

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel

    $results = New-Object System.Collections.Generic.List[object]

    foreach ($scenario in $scenarios) {
        if ([string]$scenario.status -notin @("seed", "ready")) { continue }
        if ($Lanes -notcontains [string]$scenario.lane) { continue }

        $workbook = $null
        $worksheet = $null

        $compatVersion = ""
        $artifactRef = ""
        $primaryCell = [string]$scenario.op_primary_cell
        $primaryFormula2 = ""
        $primaryValue2 = ""
        $primaryText = ""
        $primaryTextLen = ""

        try {
            $scenarioDir = Join-Path $artifactRootPath ([string]$scenario.scenario_id)
            if (-not (Test-Path $scenarioDir)) {
                New-Item -ItemType Directory -Path $scenarioDir | Out-Null
            }

            $wbInfo = New-ScenarioWorkbook -Excel $excel -ScenarioId ([string]$scenario.scenario_id) -TemplatePath $templatePathResolved -ScenarioArtifactDir $scenarioDir
            $workbook = $wbInfo.Workbook
            $artifactRef = [string]$wbInfo.ArtifactRef
            $worksheet = $workbook.Worksheets.Item(1)
            $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

            $formulaAssignments = Parse-FormulaAssignments -Raw ([string]$scenario.formula_setup)
            $valueAssignments = Parse-ValueAssignments -Raw ([string]$scenario.value_setup)
            $scenarioKind = ([string]$scenario.scenario_kind).Trim()

            if ([string]::IsNullOrWhiteSpace($scenarioKind)) {
                $scenarioKind = "formula_case"
            }

            foreach ($valueSpec in $valueAssignments) {
                $resolvedValue = Resolve-ValueExpression -Expr $valueSpec.ValueExpr
                try {
                    $worksheet.Range($valueSpec.Cell).Value2 = $resolvedValue
                }
                catch {
                    throw "value set failed at $($valueSpec.Cell) with '$($valueSpec.ValueExpr)': $($_.Exception.Message)"
                }
            }

            if ($scenarioKind -eq "formula_literal_repeat") {
                if ($valueAssignments.Count -eq 0) {
                    throw "formula_literal_repeat requires value_setup assignment rows."
                }

                foreach ($valueSpec in $valueAssignments) {
                    $resolvedValue = Resolve-ValueExpression -Expr $valueSpec.ValueExpr
                    $literalFormula = Build-FormulaLiteralFromValue -Value $resolvedValue
                    try {
                        $worksheet.Range($valueSpec.Cell).Formula = $literalFormula
                    }
                    catch {
                        throw "formula literal set failed at $($valueSpec.Cell): $($_.Exception.Message)"
                    }
                }
            }
            elseif ($scenarioKind -eq "formula_case" -or $scenarioKind -eq "value_case" -or $scenarioKind -eq "mixed_case") {
                foreach ($formulaSpec in $formulaAssignments) {
                    try {
                        $worksheet.Range($formulaSpec.Cell).Formula = $formulaSpec.Formula
                    }
                    catch {
                        throw "formula set failed at $($formulaSpec.Cell) with '$($formulaSpec.Formula)': $($_.Exception.Message)"
                    }
                }
            }
            else {
                throw "Unsupported scenario_kind '$scenarioKind'."
            }

            if ([string]::IsNullOrWhiteSpace($primaryCell)) {
                if ($formulaAssignments.Count -gt 0) {
                    $primaryCell = $formulaAssignments[0].Cell
                }
                elseif ($valueAssignments.Count -gt 0) {
                    $primaryCell = $valueAssignments[0].Cell
                }
                else {
                    $primaryCell = "A1"
                }
            }

            $observeCells = Parse-ObserveCells -Raw ([string]$scenario.op_observe_cells) -PrimaryCell $primaryCell -FormulaAssignments $formulaAssignments -ValueAssignments $valueAssignments

            $action = [string]$scenario.op_action
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

                $xlsxPath = Join-Path $scenarioDir "$($scenario.scenario_id).xlsx"
                $workbook.SaveAs($xlsxPath, 51)
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

                $csvPath = Join-Path $scenarioDir "$($scenario.scenario_id).csv"
                $workbook.SaveAs($csvPath, 6)
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

                $notesExtra = "pre_csv=" + (Serialize-Snapshots -Snapshots $preSnapshots)
            }
            else {
                throw "Unsupported op_action '$action'."
            }

            $primarySnapshot = $postSnapshots | Where-Object { $_.cell -eq $primaryCell } | Select-Object -First 1
            if ($null -eq $primarySnapshot -and $postSnapshots.Count -gt 0) {
                $primarySnapshot = $postSnapshots[0]
            }

            if ($null -ne $primarySnapshot) {
                $primaryFormula2 = [string]$primarySnapshot.formula2
                $primaryValue2 = [string]$primarySnapshot.value2
                $primaryText = [string]$primarySnapshot.text
                $primaryTextLen = [string]$primarySnapshot.text_len
            }

            $comparisonBools = ""
            if ($primaryValue2 -in @("True", "False")) {
                $comparisonBools = "primary_is_boolean=$primaryValue2"
            }

            $combinedNotes = [string]$scenario.notes
            if (-not [string]::IsNullOrWhiteSpace($notesExtra)) {
                if (-not [string]::IsNullOrWhiteSpace($combinedNotes)) {
                    $combinedNotes = "$combinedNotes | $notesExtra"
                }
                else {
                    $combinedNotes = $notesExtra
                }
            }

            Add-ResultRow -Rows $results -Scenario $scenario -ExecutionStatus "observed" -ObservedClass "cell_snapshots" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion -ArtifactRef $artifactRef -PrimaryCell $primaryCell -PrimaryFormula2 $primaryFormula2 -PrimaryValue2 $primaryValue2 -PrimaryText $primaryText -PrimaryTextLen $primaryTextLen -ObservedCells (Serialize-Snapshots -Snapshots $postSnapshots) -ComparisonBools $comparisonBools -Notes $combinedNotes -RunnerVersion $runnerVersion -RunLabel $RunLabel -SourceManifest $manifestPath
        }
        catch {
            Add-ResultRow -Rows $results -Scenario $scenario -ExecutionStatus "failed" -ObservedClass "com_exception" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion -ArtifactRef $artifactRef -PrimaryCell $primaryCell -PrimaryFormula2 $primaryFormula2 -PrimaryValue2 $primaryValue2 -PrimaryText $primaryText -PrimaryTextLen $primaryTextLen -ObservedCells "" -ComparisonBools "" -Notes ([string]$_.Exception.Message) -RunnerVersion $runnerVersion -RunLabel $RunLabel -SourceManifest $manifestPath
        }
        finally {
            Close-WorkbookSafe -Workbook $workbook
            Release-ComObjectSafe -Obj $worksheet
            Release-ComObjectSafe -Obj $workbook
        }
    }

    $results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8

    $runFinishedUtc = (Get-Date).ToUniversalTime()
    $metaPath = "$outPath.run-metadata.json"
    $meta = [ordered]@{
        run_label = $RunLabel
        runner_version = $runnerVersion
        run_started_utc = $runStartedUtc.ToString("o")
        run_finished_utc = $runFinishedUtc.ToString("o")
        manifest_path = $manifestPath
        manifest_sha256 = Get-FileSha256 -Path $manifestPath
        workbook_template_path = $templatePathResolved
        workbook_template_sha256 = Get-FileSha256 -Path $templatePathResolved
        output_path = $outPath
        artifact_root = $artifactRootPath
        lanes = $Lanes
        manifest_total_rows = $scenarios.Count
        result_rows = $results.Count
        excel_version = $excelVersionFull
        excel_channel = $excelChannel
        locale_profile = "en-US"
        git_commit = Get-GitCommit
        git_dirty = Get-GitDirty
    }
    $meta | ConvertTo-Json -Depth 6 | Set-Content -Path $metaPath -Encoding UTF8

    Write-Host "String baseline run complete. Rows written: $($results.Count)"
    Write-Host "Output: $outPath"
    Write-Host "Metadata: $metaPath"
}
finally {
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }

    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

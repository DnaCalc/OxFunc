param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string[]]$Lanes = @("CO4-A", "CO4-B", "CO4-C", "CO4-D", "CO4-E"),

    [string]$ArtifactRoot = ".tmp/coercion-artifacts",

    [string]$RunLabel = "default",

    [switch]$IncludeSeed,

    [double]$ExternalWorkbookSeedValue = 11
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
    if ($Value -is [Array]) {
        return "[array]"
    }
    try {
        return [string]$Value
    }
    catch {
        if ($null -ne $Value) {
            return "[unprintable:" + $Value.GetType().FullName + "]"
        }
        return "[unprintable:null]"
    }
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

function Resolve-ValueExpression {
    param([string]$Expr)

    $raw = $Expr.Trim()

    if ($raw -eq "empty") {
        return [PSCustomObject]@{ Kind = "empty"; Value = $null }
    }

    $repeatMatch = [regex]::Match($raw, '^repeat\(([^,]+),(\d+)\)$')
    if ($repeatMatch.Success) {
        $token = $repeatMatch.Groups[1].Value.Trim()
        $count = [int]$repeatMatch.Groups[2].Value
        if ($count -lt 0) { throw "Negative repeat count in '$Expr'." }

        switch ($token) {
            "x" {
                return [PSCustomObject]@{ Kind = "scalar"; Value = ("x" * $count) }
            }
            "space" {
                return [PSCustomObject]@{ Kind = "scalar"; Value = (" " * $count) }
            }
            default {
                throw "Unknown repeat token '$token' in '$Expr'."
            }
        }
    }

    $quoted = [regex]::Match($raw, '^"(.*)"$')
    if ($quoted.Success) {
        $inner = $quoted.Groups[1].Value.Replace('""', '"')
        return [PSCustomObject]@{ Kind = "scalar"; Value = $inner }
    }

    if ($raw -eq "TRUE") {
        return [PSCustomObject]@{ Kind = "scalar"; Value = $true }
    }
    if ($raw -eq "FALSE") {
        return [PSCustomObject]@{ Kind = "scalar"; Value = $false }
    }

    $parsedNumber = 0.0
    if ([double]::TryParse(
        $raw,
        [System.Globalization.NumberStyles]::Float,
        [System.Globalization.CultureInfo]::InvariantCulture,
        [ref]$parsedNumber
    )) {
        return [PSCustomObject]@{ Kind = "scalar"; Value = $parsedNumber }
    }

    throw "Unsupported value expression '$Expr'."
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
    $textLen = $text.Length

    return [PSCustomObject]@{
        cell = $Cell
        text = $text
        text_len = $textLen
        formula2 = $formula2
        value2 = $value2
    }
}

function Extract-ExternalWorkbookNameFromFormula {
    param([string]$Formula)

    if ([string]::IsNullOrWhiteSpace($Formula)) { return "" }
    $m = [regex]::Match($Formula, '\[([^\]]+)\]')
    if ($m.Success) {
        return $m.Groups[1].Value
    }
    return ""
}

function New-ExternalWorkbookArtifact {
    param(
        [object]$Excel,
        [string]$ScenarioDir,
        [string]$WorkbookName,
        [double]$SeedValue
    )

    if ([string]::IsNullOrWhiteSpace($WorkbookName)) {
        $WorkbookName = "ClosedWorkbook.xlsx"
    }

    $path = Join-Path $ScenarioDir $WorkbookName
    $wb = $null
    $ws = $null

    try {
        $wb = $Excel.Workbooks.Add()
        $ws = $wb.Worksheets.Item(1)
        try { $ws.Name = "Sheet1" } catch {}
        $ws.Range("A1").Value2 = $SeedValue
        $wb.SaveAs($path, 51)
    }
    finally {
        Close-WorkbookSafe -Workbook $wb
        Release-ComObjectSafe -Obj $ws
        Release-ComObjectSafe -Obj $wb
    }

    return $path
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
        coercion_axis = [string]$Scenario.coercion_axis
        ref_resolution_axis = [string]$Scenario.ref_resolution_axis
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

$runStartedUtc = (Get-Date).ToUniversalTime()
$runnerVersion = "coercion-excel-baseline-ps1/0.1.0"

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
        if (-not $IncludeSeed -and [string]$scenario.status -eq "seed") { continue }
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

            $workbook = $excel.Workbooks.Add()
            $worksheet = $workbook.Worksheets.Item(1)
            $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

            $formulaAssignments = Parse-FormulaAssignments -Raw ([string]$scenario.formula_setup)
            $valueAssignments = Parse-ValueAssignments -Raw ([string]$scenario.value_setup)

            foreach ($valueSpec in $valueAssignments) {
                $resolved = Resolve-ValueExpression -Expr $valueSpec.ValueExpr
                if ($resolved.Kind -eq "empty") {
                    $worksheet.Range($valueSpec.Cell).ClearContents() | Out-Null
                }
                else {
                    if ($resolved.Value -is [string]) {
                        $worksheet.Range($valueSpec.Cell).Value = [string]$resolved.Value
                    }
                    else {
                        $worksheet.Range($valueSpec.Cell).Value2 = $resolved.Value
                    }
                }
            }

            foreach ($formulaSpec in $formulaAssignments) {
                $worksheet.Range($formulaSpec.Cell).Formula = $formulaSpec.Formula
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
            if ([string]::IsNullOrWhiteSpace($action)) { $action = "calculate" }

            $preSnapshots = @()
            $postSnapshots = @()
            $notesExtra = ""

            if ($action -eq "calculate") {
                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $postSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }
            }
            elseif ($action -eq "external_ref_open_state_compare") {
                if ($formulaAssignments.Count -eq 0) {
                    throw "external_ref_open_state_compare requires formula_setup entries."
                }

                $firstFormula = [string]$formulaAssignments[0].Formula
                $externalWorkbookName = Extract-ExternalWorkbookNameFromFormula -Formula $firstFormula
                $externalPath = New-ExternalWorkbookArtifact -Excel $excel -ScenarioDir $scenarioDir -WorkbookName $externalWorkbookName -SeedValue $ExternalWorkbookSeedValue
                $artifactRef = $externalPath

                $excel.CalculateFull()
                foreach ($cell in $observeCells) {
                    $preSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                }

                $externalWb = $null
                try {
                    $externalWb = $excel.Workbooks.Open($externalPath)
                    $excel.CalculateFull()
                    foreach ($cell in $observeCells) {
                        $postSnapshots += Get-CellSnapshot -Worksheet $worksheet -Cell $cell
                    }
                }
                finally {
                    Close-WorkbookSafe -Workbook $externalWb
                    Release-ComObjectSafe -Obj $externalWb
                }

                $preSerialized = Serialize-Snapshots -Snapshots $preSnapshots
                $postSerialized = Serialize-Snapshots -Snapshots $postSnapshots
                $notesExtra = "external_workbook_path=$externalPath | closed_state=$preSerialized | open_state=$postSerialized"
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
            if ($action -eq "external_ref_open_state_compare" -and $preSnapshots.Count -gt 0 -and $postSnapshots.Count -gt 0) {
                $prePrimary = $preSnapshots | Where-Object { $_.cell -eq $primaryCell } | Select-Object -First 1
                $postPrimary = $postSnapshots | Where-Object { $_.cell -eq $primaryCell } | Select-Object -First 1
                if ($null -ne $prePrimary -and $null -ne $postPrimary) {
                    $changed = ($prePrimary.value2 -cne $postPrimary.value2)
                    if ([string]::IsNullOrWhiteSpace($comparisonBools)) {
                        $comparisonBools = "primary_changed_closed_to_open=$changed"
                    }
                    else {
                        $comparisonBools = "$comparisonBools;primary_changed_closed_to_open=$changed"
                    }
                }
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
            $errMsg = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
            $stack = [string]$_.ScriptStackTrace
            if (-not [string]::IsNullOrWhiteSpace($stack)) {
                $errMsg = $errMsg + " | stack: " + ($stack -replace "`r|`n", " :: ")
            }
            Add-ResultRow -Rows $results -Scenario $scenario -ExecutionStatus "failed" -ObservedClass "com_exception" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion -ArtifactRef $artifactRef -PrimaryCell $primaryCell -PrimaryFormula2 $primaryFormula2 -PrimaryValue2 $primaryValue2 -PrimaryText $primaryText -PrimaryTextLen $primaryTextLen -ObservedCells "" -ComparisonBools "" -Notes $errMsg -RunnerVersion $runnerVersion -RunLabel $RunLabel -SourceManifest $manifestPath
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

    $expectationCounts = @{
        matched = ($results | Where-Object { $_.expectation_status -eq "matched" }).Count
        mismatched = ($results | Where-Object { $_.expectation_status -eq "mismatched" }).Count
        not_specified = ($results | Where-Object { $_.expectation_status -eq "not_specified" }).Count
    }

    $meta = [ordered]@{
        run_label = $RunLabel
        runner_version = $runnerVersion
        run_started_utc = $runStartedUtc.ToString("o")
        run_finished_utc = $runFinishedUtc.ToString("o")
        manifest_path = $manifestPath
        manifest_sha256 = Get-FileSha256 -Path $manifestPath
        output_path = $outPath
        artifact_root = $artifactRootPath
        lanes = $Lanes
        include_seed = [bool]$IncludeSeed
        manifest_total_rows = $scenarios.Count
        result_rows = $results.Count
        expectation_counts = $expectationCounts
        excel_version = $excelVersionFull
        excel_channel = $excelChannel
        locale_profile = "en-US"
        git_commit = Get-GitCommit
        git_dirty = Get-GitDirty
    }
    $meta | ConvertTo-Json -Depth 6 | Set-Content -Path $metaPath -Encoding UTF8

    Write-Host "Coercion baseline run complete. Rows written: $($results.Count)"
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

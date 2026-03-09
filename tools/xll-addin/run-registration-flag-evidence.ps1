param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [string]$XllPath = "",

    [switch]$BuildIfMissing,

    [string[]]$Lanes = @("W11-VOL", "W11-TS", "W11-MAC"),

    [string]$WorkbookTemplate = "",

    [string]$RunLabel = "default",

    [switch]$DisableFlagExperiments
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
    if ($Value -is [Array]) { return "[array]" }
    return [string]$Value
}

function Parse-Assignments {
    param([string]$Raw)

    $items = @()
    if ([string]::IsNullOrWhiteSpace($Raw)) { return $items }

    foreach ($entry in ($Raw -split '\|')) {
        $trimmed = $entry.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) { continue }
        $m = [regex]::Match($trimmed, "^\s*([A-Z]{1,3}[0-9]+)\s*:=\s*(.+)$")
        if (-not $m.Success) {
            throw "Invalid setup assignment '$trimmed' (expected CELL:=VALUE_OR_FORMULA)."
        }
        $items += [PSCustomObject]@{
            Cell = $m.Groups[1].Value
            Expr = $m.Groups[2].Value.Trim()
        }
    }
    return $items
}

function Apply-Assignment {
    param(
        [object]$Worksheet,
        [object]$Assignment
    )

    $expr = [string]$Assignment.Expr
    if ($expr.StartsWith("=")) {
        $Worksheet.Range($Assignment.Cell).Formula2 = $expr
        return
    }
    if ($expr -match '^".*"$') {
        $Worksheet.Range($Assignment.Cell).Value = $expr.Substring(1, $expr.Length - 2).Replace('""', '"')
        return
    }
    if ($expr -eq "TRUE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $true
        return
    }
    if ($expr -eq "FALSE") {
        $Worksheet.Range($Assignment.Cell).Value2 = $false
        return
    }

    $n = 0.0
    if ([double]::TryParse($expr, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$n)) {
        $Worksheet.Range($Assignment.Cell).Value2 = $n
        return
    }

    $Worksheet.Range($Assignment.Cell).Value = $expr
}

function Get-CellSnapshot {
    param(
        [object]$Worksheet,
        [string]$Cell
    )

    $range = $Worksheet.Range($Cell)
    return [PSCustomObject]@{
        cell = $Cell
        text = [string]$range.Text
        value2 = Convert-Value2ToString -Value $range.Value2
        formula2 = [string]$range.Formula2
    }
}

function Parse-BoolToken {
    param([string]$Raw)

    switch ($Raw.Trim().ToLowerInvariant()) {
        "true" { return $true }
        "false" { return $false }
        default { throw "Unsupported boolean token '$Raw'." }
    }
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

function Evaluate-ExpectedObservable {
    param(
        [string]$ExpectedObservable,
        [string]$ExecutionStatus,
        [object]$LhsPass1,
        [object]$RhsPass1,
        [object]$LhsPass2,
        [object]$RhsPass2,
        [bool]$LhsChanged,
        [bool]$RhsChanged,
        [bool]$EqualPass1,
        [bool]$EqualPass2,
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

        if ($clause.StartsWith("execution_status_eq:")) {
            $expected = $clause.Substring("execution_status_eq:".Length)
            $matched = ($ExecutionStatus -ceq $expected)
        }
        elseif ($clause.StartsWith("lhs_changed_eq:")) {
            $expected = Parse-BoolToken -Raw $clause.Substring("lhs_changed_eq:".Length)
            $matched = ($LhsChanged -eq $expected)
        }
        elseif ($clause.StartsWith("rhs_changed_eq:")) {
            $expected = Parse-BoolToken -Raw $clause.Substring("rhs_changed_eq:".Length)
            $matched = ($RhsChanged -eq $expected)
        }
        elseif ($clause.StartsWith("lhs_rhs_equal_pass1_eq:")) {
            $expected = Parse-BoolToken -Raw $clause.Substring("lhs_rhs_equal_pass1_eq:".Length)
            $matched = ($EqualPass1 -eq $expected)
        }
        elseif ($clause.StartsWith("lhs_rhs_equal_pass2_eq:")) {
            $expected = Parse-BoolToken -Raw $clause.Substring("lhs_rhs_equal_pass2_eq:".Length)
            $matched = ($EqualPass2 -eq $expected)
        }
        elseif ($clause.StartsWith("lhs_value2_pass1_eq:")) {
            $expected = $clause.Substring("lhs_value2_pass1_eq:".Length)
            $matched = ([string]$LhsPass1.value2 -ceq $expected)
        }
        elseif ($clause.StartsWith("rhs_value2_pass1_eq:")) {
            $expected = $clause.Substring("rhs_value2_pass1_eq:".Length)
            $matched = ([string]$RhsPass1.value2 -ceq $expected)
        }
        elseif ($clause.StartsWith("lhs_value2_pass2_eq:")) {
            $expected = $clause.Substring("lhs_value2_pass2_eq:".Length)
            $matched = ([string]$LhsPass2.value2 -ceq $expected)
        }
        elseif ($clause.StartsWith("rhs_value2_pass2_eq:")) {
            $expected = $clause.Substring("rhs_value2_pass2_eq:".Length)
            $matched = ([string]$RhsPass2.value2 -ceq $expected)
        }
        elseif ($clause.StartsWith("lhs_text_pass1_eq:")) {
            $expected = $clause.Substring("lhs_text_pass1_eq:".Length)
            $matched = ([string]$LhsPass1.text -ceq $expected)
        }
        elseif ($clause.StartsWith("rhs_text_pass1_eq:")) {
            $expected = $clause.Substring("rhs_text_pass1_eq:".Length)
            $matched = ([string]$RhsPass1.text -ceq $expected)
        }
        elseif ($clause.StartsWith("notes_contains:")) {
            $expected = $clause.Substring("notes_contains:".Length)
            $matched = ([string]$Notes).Contains($expected)
        }
        else {
            $detailParts += "unsupported_clause=$clause"
            $matched = $false
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
        [object]$LhsPass1,
        [object]$RhsPass1,
        [object]$LhsPass2,
        [object]$RhsPass2,
        [bool]$LhsChanged,
        [bool]$RhsChanged,
        [bool]$EqualPass1,
        [bool]$EqualPass2,
        [string]$Notes
    )

    $expectedStatus = [string]$Scenario.expected_status
    $expectedObservable = [string]$Scenario.expected_observable

    $statusSpecified = -not [string]::IsNullOrWhiteSpace($expectedStatus)
    $statusMatched = $true
    $statusDetail = "expected_status not specified"
    if ($statusSpecified) {
        $statusMatched = ($ExecutionStatus -ceq $expectedStatus)
        $statusDetail = "expected_status=$expectedStatus actual=$ExecutionStatus match=$statusMatched"
    }

    $observableEval = Evaluate-ExpectedObservable -ExpectedObservable $expectedObservable -ExecutionStatus $ExecutionStatus -LhsPass1 $LhsPass1 -RhsPass1 $RhsPass1 -LhsPass2 $LhsPass2 -RhsPass2 $RhsPass2 -LhsChanged $LhsChanged -RhsChanged $RhsChanged -EqualPass1 $EqualPass1 -EqualPass2 $EqualPass2 -Notes $Notes

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

function Open-ScenarioWorkbook {
    param(
        [object]$Excel,
        [string]$WorkbookTemplatePath
    )

    if ([string]::IsNullOrWhiteSpace($WorkbookTemplatePath)) {
        return $Excel.Workbooks.Add()
    }

    if (-not (Test-Path $WorkbookTemplatePath)) {
        throw "WorkbookTemplate not found: $WorkbookTemplatePath"
    }

    return $Excel.Workbooks.Open($WorkbookTemplatePath)
}

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

if ([string]::IsNullOrWhiteSpace($XllPath)) {
    $XllPath = Join-Path $PSScriptRoot "bin\OxFunc64.xll"
}
$xllPathFull = [System.IO.Path]::GetFullPath($XllPath)

if (-not (Test-Path $xllPathFull)) {
    if ($BuildIfMissing) {
        $buildScript = Join-Path $PSScriptRoot "build-oxfunc-xll.ps1"
        & $buildScript -Profile release
    }
    if (-not (Test-Path $xllPathFull)) {
        throw "XLL not found: $xllPathFull"
    }
}

$scenarios = Import-Csv -Path $manifestPath | Where-Object {
    $_.status -in @("ready", "seed") -and $Lanes -contains [string]$_.lane
}
if (-not $scenarios -or $scenarios.Count -eq 0) {
    throw "Manifest has no runnable scenario rows for requested lanes: $manifestPath"
}

$templatePath = ""
if (-not [string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
    $templatePath = (Resolve-Path -Path $WorkbookTemplate -ErrorAction Stop).Path
}

$runStartedUtc = (Get-Date).ToUniversalTime()
$runnerVersion = "xll-registration-flag-probe-ps1/0.1.0"

$previousFlagEnv = $env:OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS
if ($DisableFlagExperiments) {
    Remove-Item Env:OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS -ErrorAction SilentlyContinue
}
else {
    $env:OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS = "1"
}

$excel = $null
$workbook = $null
$worksheet = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    $registered = $excel.RegisterXLL($xllPathFull)
    if (-not $registered) {
        throw "RegisterXLL returned false for: $xllPathFull"
    }

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel

    $results = New-Object System.Collections.Generic.List[object]

    foreach ($scenario in $scenarios) {
        $lhsCell = "B1"
        $rhsCell = "H1"
        $executionStatus = "observed"
        $notes = [string]$scenario.notes
        $compatVersion = ""

        $lhsPass1 = $null
        $rhsPass1 = $null
        $lhsPass2 = $null
        $rhsPass2 = $null
        $lhsChanged = $false
        $rhsChanged = $false
        $equalPass1 = $false
        $equalPass2 = $false

        try {
            $workbook = Open-ScenarioWorkbook -Excel $excel -WorkbookTemplatePath $templatePath
            $worksheet = $workbook.Worksheets.Item(1)
            $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook
            $worksheet.Cells.Clear() | Out-Null

            $assignments = Parse-Assignments -Raw ([string]$scenario.setup_values)
            foreach ($assignment in $assignments) {
                Apply-Assignment -Worksheet $worksheet -Assignment $assignment
            }

            $lhsFormula = [string]$scenario.lhs_formula
            $rhsFormula = [string]$scenario.rhs_formula
            if (-not $lhsFormula.StartsWith("=")) { $lhsFormula = "=$lhsFormula" }
            if (-not $rhsFormula.StartsWith("=")) { $rhsFormula = "=$rhsFormula" }

            $worksheet.Range($lhsCell).Formula2 = $lhsFormula
            $worksheet.Range($rhsCell).Formula2 = $rhsFormula

            $action = [string]$scenario.op_action
            if ([string]::IsNullOrWhiteSpace($action)) { $action = "calculate_once" }
            $pauseMs = 0
            [void][int]::TryParse([string]$scenario.recalc_pause_ms, [ref]$pauseMs)

            switch ($action) {
                "calculate_once" {
                    $excel.CalculateFull()
                    $lhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell
                    $lhsPass2 = $lhsPass1
                    $rhsPass2 = $rhsPass1
                }
                "calculate_twice" {
                    $excel.CalculateFull()
                    $lhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell

                    if ($pauseMs -gt 0) {
                        Start-Sleep -Milliseconds $pauseMs
                    }

                    $excel.CalculateFull()
                    $lhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell
                }
                "calculate_twice_incremental" {
                    $excel.Calculate()
                    $lhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell

                    if ($pauseMs -gt 0) {
                        Start-Sleep -Milliseconds $pauseMs
                    }

                    $excel.Calculate()
                    $lhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell
                }
                "calculate_full_rebuild_twice" {
                    $excel.CalculateFullRebuild()
                    $lhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass1 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell

                    if ($pauseMs -gt 0) {
                        Start-Sleep -Milliseconds $pauseMs
                    }

                    $excel.CalculateFullRebuild()
                    $lhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $lhsCell
                    $rhsPass2 = Get-CellSnapshot -Worksheet $worksheet -Cell $rhsCell
                }
                default {
                    throw "Unsupported op_action '$action'."
                }
            }

            $lhsChanged = ([string]$lhsPass1.value2 -cne [string]$lhsPass2.value2) -or ([string]$lhsPass1.text -cne [string]$lhsPass2.text)
            $rhsChanged = ([string]$rhsPass1.value2 -cne [string]$rhsPass2.value2) -or ([string]$rhsPass1.text -cne [string]$rhsPass2.text)
            $equalPass1 = (([string]$lhsPass1.value2 -ceq [string]$rhsPass1.value2) -or ([string]$lhsPass1.text -ceq [string]$rhsPass1.text))
            $equalPass2 = (([string]$lhsPass2.value2 -ceq [string]$rhsPass2.value2) -or ([string]$lhsPass2.text -ceq [string]$rhsPass2.text))
        }
        catch {
            $executionStatus = "failed"
            $errMsg = [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
            $notes = if ([string]::IsNullOrWhiteSpace($notes)) { $errMsg } else { "$notes | $errMsg" }
            if ($null -eq $lhsPass1) {
                $lhsPass1 = [PSCustomObject]@{ text = ""; value2 = ""; formula2 = "" }
            }
            if ($null -eq $rhsPass1) {
                $rhsPass1 = [PSCustomObject]@{ text = ""; value2 = ""; formula2 = "" }
            }
            if ($null -eq $lhsPass2) {
                $lhsPass2 = $lhsPass1
            }
            if ($null -eq $rhsPass2) {
                $rhsPass2 = $rhsPass1
            }
        }
        finally {
            Close-WorkbookSafe -Workbook $workbook
            Release-ComObjectSafe -Obj $worksheet
            Release-ComObjectSafe -Obj $workbook
            $worksheet = $null
            $workbook = $null
        }

        $expectation = Resolve-Expectation -Scenario $scenario -ExecutionStatus $executionStatus -LhsPass1 $lhsPass1 -RhsPass1 $rhsPass1 -LhsPass2 $lhsPass2 -RhsPass2 $rhsPass2 -LhsChanged $lhsChanged -RhsChanged $rhsChanged -EqualPass1 $equalPass1 -EqualPass2 $equalPass2 -Notes $notes

        $results.Add([PSCustomObject]@{
            scenario_id = [string]$scenario.scenario_id
            lane = [string]$scenario.lane
            execution_status = $executionStatus
            expected_status = $expectation.expected_status
            expected_observable = $expectation.expected_observable
            expectation_status = $expectation.expectation_status
            expectation_detail = $expectation.expectation_detail
            lhs_formula = [string]$scenario.lhs_formula
            rhs_formula = [string]$scenario.rhs_formula
            lhs_value2_pass1 = [string]$lhsPass1.value2
            rhs_value2_pass1 = [string]$rhsPass1.value2
            lhs_text_pass1 = [string]$lhsPass1.text
            rhs_text_pass1 = [string]$rhsPass1.text
            lhs_value2_pass2 = [string]$lhsPass2.value2
            rhs_value2_pass2 = [string]$rhsPass2.value2
            lhs_text_pass2 = [string]$lhsPass2.text
            rhs_text_pass2 = [string]$rhsPass2.text
            lhs_changed = $lhsChanged
            rhs_changed = $rhsChanged
            lhs_rhs_equal_pass1 = $equalPass1
            lhs_rhs_equal_pass2 = $equalPass2
            excel_version = $excelVersionFull
            excel_channel = $excelChannel
            compat_version = $compatVersion
            run_label = $RunLabel
            xll_path = $xllPathFull
            flag_experiments_enabled = if ($DisableFlagExperiments) { "false" } else { "true" }
            notes = $notes
        })
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
        workbook_template = $templatePath
        lanes = $Lanes
        result_rows = $results.Count
        expectation_counts = $expectationCounts
        excel_version = $excelVersionFull
        excel_channel = $excelChannel
        locale_profile = "en-US"
        xll_path = $xllPathFull
        xll_sha256 = Get-FileSha256 -Path $xllPathFull
        flag_experiments_enabled = (-not $DisableFlagExperiments)
        git_commit = Get-GitCommit
        git_dirty = Get-GitDirty
    }
    $meta | ConvertTo-Json -Depth 6 | Set-Content -Path $metaPath -Encoding UTF8

    Write-Host "Registration flag probe run complete."
    Write-Host "Output: $outPath"
    Write-Host "Metadata: $metaPath"
}
finally {
    if ($null -ne $excel) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel

    if ($null -eq $previousFlagEnv) {
        Remove-Item Env:OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS -ErrorAction SilentlyContinue
    }
    else {
        $env:OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS = $previousFlagEnv
    }

    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

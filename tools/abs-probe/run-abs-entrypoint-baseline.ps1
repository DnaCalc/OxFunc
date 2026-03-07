param(
    [Parameter(Mandatory = $true)]
    [string]$Manifest,

    [Parameter(Mandatory = $true)]
    [string]$Out,

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

function Convert-ValueToString {
    param([object]$Value)
    if ($null -eq $Value) { return "" }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal]) {
        return ([string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:R}", $Value))
    }
    return [string]$Value
}

function Parse-WorksheetFunctionArg {
    param([string]$Expr)

    if ($Expr.StartsWith("number:")) {
        $raw = $Expr.Substring("number:".Length)
        $parsed = 0.0
        if (-not [double]::TryParse($raw, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$parsed)) {
            throw "Invalid numeric WorksheetFunction.Abs arg expression '$Expr'."
        }
        return $parsed
    }
    if ($Expr.StartsWith("text:")) {
        return $Expr.Substring("text:".Length)
    }
    if ($Expr.StartsWith("logical:")) {
        $raw = $Expr.Substring("logical:".Length).ToUpperInvariant()
        if ($raw -eq "TRUE") { return $true }
        if ($raw -eq "FALSE") { return $false }
        throw "Invalid logical WorksheetFunction.Abs arg expression '$Expr'."
    }

    throw "Unsupported WorksheetFunction.Abs arg expression '$Expr'."
}

function Evaluate-ExpectedObservable {
    param(
        [string]$ExpectedObservable,
        [string]$ExecutionStatus,
        [string]$ObservedValue,
        [string]$ObservedText,
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
        if ($clause.StartsWith("observed_value_eq:")) {
            $expected = $clause.Substring("observed_value_eq:".Length)
            $matched = ($ObservedValue -ceq $expected)
        }
        elseif ($clause.StartsWith("observed_text_eq:")) {
            $expected = $clause.Substring("observed_text_eq:".Length)
            $matched = ($ObservedText -ceq $expected)
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
        if (-not $matched) { $allMatched = $false }
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
        [string]$ObservedValue,
        [string]$ObservedText,
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

    $observableEval = Evaluate-ExpectedObservable -ExpectedObservable $expectedObservable -ExecutionStatus $ExecutionStatus -ObservedValue $ObservedValue -ObservedText $ObservedText -Notes $Notes

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

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$scenarios = Import-Csv -Path $manifestPath
if (-not $scenarios -or $scenarios.Count -eq 0) {
    throw "Manifest has no scenario rows: $manifestPath"
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

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)

    $excelVersion = [string]$excel.Version
    $excelBuild = ""
    try { $excelBuild = [string]$excel.Build } catch { $excelBuild = "" }
    $excelVersionFull = if ($excelBuild) { "$excelVersion (build $excelBuild)" } else { $excelVersion }
    $excelChannel = Get-ExcelChannel
    $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

    $rows = New-Object System.Collections.Generic.List[object]

    foreach ($scenario in $scenarios) {
        $scenarioId = [string]$scenario.scenario_id
        $mechanism = [string]$scenario.mechanism
        $inputExpr = [string]$scenario.input_expr

        $executionStatus = "observed"
        $observedType = ""
        $observedValue = ""
        $observedText = ""
        $notes = [string]$scenario.notes
        $observedFormula2 = ""

        try {
            switch ($mechanism) {
                "Range.Formula" {
                    $worksheet.Range("A1").ClearContents() | Out-Null
                    $worksheet.Range("A1").Formula = $inputExpr
                    $excel.CalculateFull()
                    try { $observedFormula2 = [string]$worksheet.Range("A1").Formula2 } catch { $observedFormula2 = [string]$worksheet.Range("A1").Formula }
                    $observedText = [string]$worksheet.Range("A1").Text
                    $raw = $worksheet.Range("A1").Value2
                    $observedValue = Convert-ValueToString -Value $raw
                    $observedType = if ($null -eq $raw) { "null" } else { $raw.GetType().Name }
                }
                "Application.Evaluate" {
                    $raw = $excel.Evaluate($inputExpr)
                    $observedValue = Convert-ValueToString -Value $raw
                    $observedType = if ($null -eq $raw) { "null" } else { $raw.GetType().Name }
                }
                "Worksheet.Evaluate" {
                    $raw = $worksheet.Evaluate($inputExpr)
                    $observedValue = Convert-ValueToString -Value $raw
                    $observedType = if ($null -eq $raw) { "null" } else { $raw.GetType().Name }
                }
                "WorksheetFunction.Abs" {
                    $arg = Parse-WorksheetFunctionArg -Expr $inputExpr
                    $raw = $excel.WorksheetFunction.Abs($arg)
                    $observedValue = Convert-ValueToString -Value $raw
                    $observedType = if ($null -eq $raw) { "null" } else { $raw.GetType().Name }
                }
                default {
                    throw "Unsupported mechanism '$mechanism'."
                }
            }
        }
        catch {
            $executionStatus = "failed"
            $observedType = "com_exception"
            $notes = [string]$notes + " | " + [string]$_.Exception.GetType().FullName + ": " + [string]$_.Exception.Message
        }

        $expectation = Resolve-Expectation -Scenario $scenario -ExecutionStatus $executionStatus -ObservedValue $observedValue -ObservedText $observedText -Notes $notes

        $rows.Add([PSCustomObject]@{
            scenario_id = $scenarioId
            mechanism = $mechanism
            input_expr = $inputExpr
            execution_status = $executionStatus
            observed_type = $observedType
            observed_formula2 = $observedFormula2
            observed_text = $observedText
            observed_value = $observedValue
            expected_status = $expectation.expected_status
            expected_observable = $expectation.expected_observable
            expectation_status = $expectation.expectation_status
            expectation_detail = $expectation.expectation_detail
            excel_version = $excelVersionFull
            excel_channel = $excelChannel
            compat_version = $compatVersion
            locale_profile = "en-US"
            run_label = $RunLabel
            runner_version = "abs-entrypoint-baseline-ps1/0.1.0"
            notes = $notes
        })
    }

    $rows | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "ABS entrypoint baseline run complete. Rows written: $($rows.Count)"
    Write-Host "Output: $outPath"
}
finally {
    Close-WorkbookSafe -Workbook $workbook
    Release-ComObjectSafe -Obj $worksheet
    Release-ComObjectSafe -Obj $workbook
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $RepoRoot,
    [string] $RunId,
    [int] $Seed = 8803,
    [switch] $KeepWorkbook
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RepoRoot = (Resolve-Path (Join-Path $scriptRoot "..\..")).Path
}
$RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ") + "-pmt-ppmt-pilot"
}

$runnerVersion = "smart-fuzzer-pmt-ppmt-pilot/0.1.0"
$runDir = Join-Path $RepoRoot ("smart-fuzzer\runs\" + $RunId)
$caseDir = Join-Path $runDir "cases"
$outcomeDir = Join-Path $runDir "outcomes"
$comparisonDir = Join-Path $runDir "comparisons"
$failureDir = Join-Path $runDir "failure_packets"
$logDir = Join-Path $runDir "logs"
New-Item -ItemType Directory -Force -Path $runDir, $caseDir, $outcomeDir, $comparisonDir, $failureDir, $logDir | Out-Null

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

function Get-Sha256Text {
    param([string] $Text)

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $sha.ComputeHash($bytes)
        return "sha256:" + ([System.BitConverter]::ToString($hash).Replace("-", "").ToLowerInvariant())
    }
    finally {
        $sha.Dispose()
    }
}

function Write-JsonFile {
    param(
        [string] $Path,
        [object] $Value,
        [int] $Depth = 16
    )

    $Value | ConvertTo-Json -Depth $Depth | Set-Content -LiteralPath $Path -Encoding UTF8
}

function Add-JsonLine {
    param(
        [string] $Path,
        [object] $Value,
        [int] $Depth = 16
    )

    $line = $Value | ConvertTo-Json -Compress -Depth $Depth
    for ($attempt = 1; $attempt -le 10; $attempt++) {
        try {
            $line | Add-Content -LiteralPath $Path -Encoding UTF8
            return
        }
        catch [System.IO.IOException] {
            if ($attempt -eq 10) {
                throw
            }
            Start-Sleep -Milliseconds (50 * $attempt)
        }
    }
}

function Get-F64BitsHex {
    param([double] $Value)

    $bits = [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($Value), 0)
    return ("0x{0:x16}" -f $bits)
}

function Convert-ExpressionToDouble {
    param([string] $Expression)

    $culture = [System.Globalization.CultureInfo]::InvariantCulture
    $parts = $Expression -split "/"
    if ($parts.Count -eq 2) {
        return [double]::Parse($parts[0].Trim(), $culture) / [double]::Parse($parts[1].Trim(), $culture)
    }
    if ($parts.Count -gt 2) {
        throw "Unsupported numeric expression: $Expression"
    }
    return [double]::Parse($Expression.Trim(), $culture)
}

function Release-ComObject {
    param([object] $Object)

    if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function Set-ExcelPropertyBestEffort {
    param(
        [object] $ExcelApplication,
        [string] $PropertyName,
        [object] $Value,
        [System.Collections.IList] $Warnings
    )

    try {
        $ExcelApplication.$PropertyName = $Value
    }
    catch {
        $Warnings.Add(([ordered]@{
            property = $PropertyName
            message = $_.Exception.Message
        })) | Out-Null
    }
}

function New-FormulaArray {
    param([object[]] $Cases)

    $array = New-Object "object[,]" $Cases.Count, 1
    for ($row = 0; $row -lt $Cases.Count; $row++) {
        $array[$row, 0] = [string] $Cases[$row].formula_text
    }
    return ,$array
}

function New-ErrorTypeFormulaArray {
    param([object[]] $Cases)

    $array = New-Object "object[,]" $Cases.Count, 1
    for ($row = 0; $row -lt $Cases.Count; $row++) {
        $excelRow = $row + 1
        $array[$row, 0] = "=IF(ISERROR(A$excelRow),ERROR.TYPE(A$excelRow),"""")"
    }
    return ,$array
}

function Get-ArrayCellValue {
    param(
        [object] $Values,
        [int] $RowIndex
    )

    if ($Values -is [System.Array]) {
        $lower0 = $Values.GetLowerBound(0)
        $lower1 = $Values.GetLowerBound(1)
        return $Values.GetValue($lower0 + $RowIndex, $lower1)
    }
    return $Values
}

function Convert-ExcelErrorTextToCode {
    param([string] $Text)

    switch ($Text) {
        "#NULL!" { return "Null" }
        "#DIV/0!" { return "Div0" }
        "#VALUE!" { return "Value" }
        "#REF!" { return "Ref" }
        "#NAME?" { return "Name" }
        "#NUM!" { return "Num" }
        "#N/A" { return "NA" }
        "#SPILL!" { return "Spill" }
        "#CALC!" { return "Calc" }
        default { return $null }
    }
}

function Convert-ExcelErrorTypeToCode {
    param([int] $ErrorType)

    switch ($ErrorType) {
        1 { return "Null" }
        2 { return "Div0" }
        3 { return "Value" }
        4 { return "Ref" }
        5 { return "Name" }
        6 { return "Num" }
        7 { return "NA" }
        8 { return "GettingData" }
        default { return "ExcelErrorType$ErrorType" }
    }
}

function New-NumberOutcome {
    param([double] $Value)

    $bitsHex = Get-F64BitsHex $Value
    return [ordered]@{
        kind = "number"
        value = $Value
        bits_hex = $bitsHex
        digest_payload = "number:$bitsHex"
    }
}

function New-ErrorOutcome {
    param([string] $Code)

    return [ordered]@{
        kind = "error"
        code = $Code
        digest_payload = "error:$Code"
    }
}

function Convert-ExcelCellOutcome {
    param(
        [object] $Value,
        [object] $ErrorType
    )

    if ($null -ne $ErrorType -and -not [string]::IsNullOrWhiteSpace([string] $ErrorType)) {
        return New-ErrorOutcome (Convert-ExcelErrorTypeToCode ([int] [double] $ErrorType))
    }

    if ($null -eq $Value) {
        return [ordered]@{
            kind = "blank"
            digest_payload = "blank:"
        }
    }

    if ($Value -is [bool]) {
        return [ordered]@{
            kind = "logical"
            value = [bool] $Value
            digest_payload = "logical:$([bool] $Value)"
        }
    }

    if ($Value -is [byte] -or $Value -is [int16] -or $Value -is [int32] -or $Value -is [int64] -or $Value -is [single] -or $Value -is [double] -or $Value -is [decimal]) {
        return New-NumberOutcome ([double] $Value)
    }

    $textValue = [string] $Value
    return [ordered]@{
        kind = "text"
        value = $textValue
        digest_payload = "text:$textValue"
    }
}

function Get-OutcomeDigest {
    param([object] $Outcome)

    return Get-Sha256Text ([string] $Outcome.digest_payload)
}

function Add-PilotCase {
    param(
        [System.Collections.IList] $Cases,
        [string] $CaseId,
        [string] $FunctionName,
        [string[]] $ArgExpressions,
        [string[]] $CoverageBuckets
    )

    $argValues = @()
    $argSources = @()
    foreach ($expr in $ArgExpressions) {
        $value = Convert-ExpressionToDouble $expr
        $argValues += $value
        $argSources += [ordered]@{
            expression = $expr
            value = $value
        }
    }

    $case = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.case.v0"
        case_id = $CaseId
        function_id = "FUNC.$FunctionName"
        function_name = $FunctionName
        generator_id = "pmt_ppmt_pilot.v0"
        seed = $Seed
        formula_text = ("={0}({1})" -f $FunctionName, ($ArgExpressions -join ","))
        args = $argValues
        arg_sources = $argSources
        coverage_buckets = $CoverageBuckets
    }
    $Cases.Add($case) | Out-Null
}

function New-PilotCases {
    $cases = New-Object System.Collections.ArrayList

    Add-PilotCase $cases "SFZ-PMT-0001" "PMT" @("0.05/12", "360", "200000") @("function:PMT", "arity:3", "rate:monthly_5pct", "known_witness:ftc_0377")
    Add-PilotCase $cases "SFZ-PMT-0002" "PMT" @("0", "12", "1200") @("function:PMT", "arity:3", "rate:zero")
    Add-PilotCase $cases "SFZ-PMT-0003" "PMT" @("0", "12", "1200", "100", "1") @("function:PMT", "arity:5", "rate:zero", "type:beginning")
    Add-PilotCase $cases "SFZ-PMT-0004" "PMT" @("0.08/12", "10", "10000") @("function:PMT", "arity:3", "w24_seed:pmt_sample")
    Add-PilotCase $cases "SFZ-PMT-0005" "PMT" @("0.05/12", "360", "200000", "0", "1") @("function:PMT", "arity:5", "type:beginning", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PMT-0006" "PMT" @("0.05/12", "360", "-200000") @("function:PMT", "arity:3", "pv:negative", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PMT-0007" "PMT" @("0.05/12", "359", "200000") @("function:PMT", "arity:3", "neighborhood:known_nper_minus_1")
    Add-PilotCase $cases "SFZ-PMT-0008" "PMT" @("0.05/12", "361", "200000") @("function:PMT", "arity:3", "neighborhood:known_nper_plus_1")
    Add-PilotCase $cases "SFZ-PMT-0009" "PMT" @("0.050000000000001/12", "360", "200000") @("function:PMT", "arity:3", "neighborhood:rate_plus")
    Add-PilotCase $cases "SFZ-PMT-0010" "PMT" @("0.049999999999999/12", "360", "200000") @("function:PMT", "arity:3", "neighborhood:rate_minus")
    Add-PilotCase $cases "SFZ-PMT-0011" "PMT" @("1E-9", "1000", "100000") @("function:PMT", "arity:3", "rate:tiny_positive")
    Add-PilotCase $cases "SFZ-PMT-0012" "PMT" @("-0.01/12", "12", "1000") @("function:PMT", "arity:3", "rate:negative")
    Add-PilotCase $cases "SFZ-PMT-0013" "PMT" @("1", "2", "100") @("function:PMT", "arity:3", "rate:large")
    Add-PilotCase $cases "SFZ-PMT-0014" "PMT" @("0.03/4", "16", "5000", "1000", "0") @("function:PMT", "arity:5", "fv:nonzero")

    Add-PilotCase $cases "SFZ-PPMT-0001" "PPMT" @("0.05/12", "1", "360", "200000") @("function:PPMT", "arity:4", "rate:monthly_5pct", "known_witness:ftc_0391")
    Add-PilotCase $cases "SFZ-PPMT-0002" "PPMT" @("0.10/12", "1", "36", "8000") @("function:PPMT", "arity:4", "w24_seed:ppmt_sample")
    Add-PilotCase $cases "SFZ-PPMT-0003" "PPMT" @("0", "1", "12", "1200") @("function:PPMT", "arity:4", "rate:zero")
    Add-PilotCase $cases "SFZ-PPMT-0004" "PPMT" @("0", "12", "12", "1200", "100", "1") @("function:PPMT", "arity:6", "rate:zero", "type:beginning")
    Add-PilotCase $cases "SFZ-PPMT-0005" "PPMT" @("0.05/12", "180", "360", "200000") @("function:PPMT", "arity:4", "period:middle", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0006" "PPMT" @("0.05/12", "360", "360", "200000") @("function:PPMT", "arity:4", "period:last", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0007" "PPMT" @("0.05/12", "1", "360", "200000", "0", "1") @("function:PPMT", "arity:6", "type:beginning", "rate:monthly_5pct")
    Add-PilotCase $cases "SFZ-PPMT-0008" "PPMT" @("0.05/12", "0", "360", "200000") @("function:PPMT", "arity:4", "period:zero_invalid")
    Add-PilotCase $cases "SFZ-PPMT-0009" "PPMT" @("0.05/12", "361", "360", "200000") @("function:PPMT", "arity:4", "period:past_nper_invalid")
    Add-PilotCase $cases "SFZ-PPMT-0010" "PPMT" @("0.050000000000001/12", "1", "360", "200000") @("function:PPMT", "arity:4", "neighborhood:rate_plus")
    Add-PilotCase $cases "SFZ-PPMT-0011" "PPMT" @("0.049999999999999/12", "1", "360", "200000") @("function:PPMT", "arity:4", "neighborhood:rate_minus")
    Add-PilotCase $cases "SFZ-PPMT-0012" "PPMT" @("1E-9", "1", "1000", "100000") @("function:PPMT", "arity:4", "rate:tiny_positive")
    Add-PilotCase $cases "SFZ-PPMT-0013" "PPMT" @("-0.01/12", "1", "12", "1000") @("function:PPMT", "arity:4", "rate:negative")
    Add-PilotCase $cases "SFZ-PPMT-0014" "PPMT" @("0.03/4", "4", "16", "5000", "1000", "0") @("function:PPMT", "arity:6", "fv:nonzero")

    return @($cases)
}

function Invoke-ExcelEvaluation {
    param(
        [object[]] $Cases,
        [string] $WorkbookPath
    )

    $excel = $null
    $workbook = $null
    $worksheet = $null
    $range = $null
    $errorRange = $null
    $excelProcessId = $null
    $warnings = New-Object System.Collections.ArrayList
    $environment = [ordered]@{
        excel_available = $false
        excel_version = $null
        excel_build = $null
        workbook_compatibility = "unknown"
        excel_setting_warnings = @()
    }

    try {
        $excelProcessIdsBefore = @(Get-Process EXCEL -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
        $excel = New-Object -ComObject Excel.Application
        $excelProcessIdsAfter = @(Get-Process EXCEL -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
        $newExcelProcessIds = @($excelProcessIdsAfter | Where-Object { $excelProcessIdsBefore -notcontains $_ })
        if ($newExcelProcessIds.Count -eq 1) {
            $excelProcessId = [int] $newExcelProcessIds[0]
        }

        Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "Visible" -Value $false -Warnings $warnings
        Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "DisplayAlerts" -Value $false -Warnings $warnings
        Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "ScreenUpdating" -Value $false -Warnings $warnings
        Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "EnableEvents" -Value $false -Warnings $warnings
        Set-ExcelPropertyBestEffort -ExcelApplication $excel -PropertyName "Calculation" -Value -4135 -Warnings $warnings

        $environment.excel_available = $true
        $environment.excel_version = [string] $excel.Version
        try { $environment.excel_build = [string] $excel.Build } catch { $environment.excel_build = $null }
        if ($warnings.Count -gt 0) {
            $environment.excel_setting_warnings = @($warnings)
        }

        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)
        try { $environment.workbook_compatibility = [string] $workbook.CompatibilityVersion } catch { $environment.workbook_compatibility = "unknown" }
        try { $worksheet.Columns.Item(1).ColumnWidth = 32 } catch {}

        $formulas = New-FormulaArray $Cases
        $errorFormulas = New-ErrorTypeFormulaArray $Cases
        $anchor = $worksheet.Range("A1")
        $range = $anchor.Resize($Cases.Count, 1)
        Release-ComObject $anchor
        $errorAnchor = $worksheet.Range("B1")
        $errorRange = $errorAnchor.Resize($Cases.Count, 1)
        Release-ComObject $errorAnchor
        $range.Formula2 = $formulas
        $errorRange.Formula2 = $errorFormulas
        $calcAnchor = $worksheet.Range("A1")
        $calcRange = $calcAnchor.Resize($Cases.Count, 2)
        Release-ComObject $calcAnchor
        [void] $calcRange.Calculate()
        Release-ComObject $calcRange
        $values = $range.Value2
        $errorValues = $errorRange.Value2

        $outcomes = New-Object System.Collections.ArrayList
        for ($row = 0; $row -lt $Cases.Count; $row++) {
            $value = Get-ArrayCellValue -Values $values -RowIndex $row
            $errorType = Get-ArrayCellValue -Values $errorValues -RowIndex $row
            $outcome = Convert-ExcelCellOutcome -Value $value -ErrorType $errorType
            $outcomes.Add(([ordered]@{
                schema_version = "oxfunc.smart_fuzzer.excel_outcome.v0"
                case_id = [string] $Cases[$row].case_id
                function_id = [string] $Cases[$row].function_id
                evaluator_id = "excel.com.range_formula2_value2/0.1.0"
                execution_status = "ok"
                formula_text = [string] $Cases[$row].formula_text
                excel_error_type = $errorType
                outcome = $outcome
            })) | Out-Null
        }

        if ($KeepWorkbook) {
            [void] $workbook.SaveAs($WorkbookPath)
        }

        return [ordered]@{
            blocked = $false
            blocker = $null
            environment = $environment
            outcomes = @($outcomes)
        }
    }
    catch {
        $environment["blocker"] = $_.Exception.Message
        return [ordered]@{
            blocked = $true
            blocker = $_.Exception.Message
            environment = $environment
            outcomes = @()
        }
    }
    finally {
        Release-ComObject $range
        Release-ComObject $errorRange
        if ($null -ne $workbook -and -not $KeepWorkbook) {
            try { $workbook.Close($false) } catch {}
        }
        if ($null -ne $excel) {
            try { $excel.Quit() } catch {}
        }
        Release-ComObject $worksheet
        Release-ComObject $workbook
        Release-ComObject $excel
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
        if ($null -ne $excelProcessId) {
            $createdExcelProcess = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
            if ($null -ne $createdExcelProcess) {
                try { $createdExcelProcess.WaitForExit(2000) | Out-Null } catch {}
                $createdExcelProcess = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
                if ($null -ne $createdExcelProcess) {
                    Stop-Process -Id $excelProcessId -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }
}

$casesPath = Join-Path $caseDir "pmt-ppmt-cases.jsonl"
$localOutcomesPath = Join-Path $outcomeDir "local.jsonl"
$excelOutcomesPath = Join-Path $outcomeDir "excel.jsonl"
$comparisonsPath = Join-Path $comparisonDir "comparisons.jsonl"
$telemetryPath = Join-Path $runDir "telemetry.jsonl"
foreach ($path in @($casesPath, $localOutcomesPath, $excelOutcomesPath, $comparisonsPath, $telemetryPath)) {
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path
    }
}

$gitRevision = Get-GitValue @("rev-parse", "HEAD")
$gitStatus = Get-GitValue @("status", "--short")
$createdUtc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$cases = New-PilotCases
foreach ($case in $cases) {
    Add-JsonLine -Path $casesPath -Value $case
}

$localWatch = [System.Diagnostics.Stopwatch]::StartNew()
$localEvaluatorManifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
& cargo run --quiet --manifest-path $localEvaluatorManifest -- --cases $casesPath --out $localOutcomesPath
if ($LASTEXITCODE -ne 0) {
    throw "Local PMT/PPMT evaluator failed with exit code $LASTEXITCODE"
}
$localWatch.Stop()

$workbookPath = Join-Path $logDir "pmt-ppmt-pilot-workbook.xlsx"
$excelWatch = [System.Diagnostics.Stopwatch]::StartNew()
$excelResult = Invoke-ExcelEvaluation -Cases $cases -WorkbookPath $workbookPath
$excelWatch.Stop()

$localById = @{}
foreach ($line in Get-Content -LiteralPath $localOutcomesPath) {
    if ([string]::IsNullOrWhiteSpace($line)) {
        continue
    }
    $outcome = $line | ConvertFrom-Json
    $localById[[string] $outcome.case_id] = $outcome
}

$excelById = @{}
foreach ($outcome in $excelResult.outcomes) {
    $excelById[[string] $outcome.case_id] = $outcome
    Add-JsonLine -Path $excelOutcomesPath -Value $outcome
}

$matches = 0
$mismatches = 0
$blocked = 0
$failurePackets = New-Object System.Collections.ArrayList
$comparisonWatch = [System.Diagnostics.Stopwatch]::StartNew()
foreach ($case in $cases) {
    $caseId = [string] $case.case_id
    $local = $localById[$caseId]
    $excelOutcomeRecord = $excelById[$caseId]
    $comparisonResult = "blocked"
    $absDelta = $null
    $withinAbs1e9 = $null
    $localKind = $null
    $excelKind = $null
    $localBits = $null
    $excelBits = $null
    $localDigest = $null
    $excelDigest = $null

    if ($null -eq $local -or $null -eq $excelOutcomeRecord) {
        $blocked += 1
    }
    else {
        $localKind = [string] $local.outcome.kind
        $excelKind = [string] $excelOutcomeRecord.outcome.kind
        $localDigest = Get-OutcomeDigest $local.outcome
        $excelDigest = Get-OutcomeDigest $excelOutcomeRecord.outcome
        if ($localKind -eq "number" -and $excelKind -eq "number") {
            $localBits = [string] $local.outcome.bits_hex
            $excelBits = [string] $excelOutcomeRecord.outcome.bits_hex
            $absDelta = [Math]::Abs(([double] $local.outcome.value) - ([double] $excelOutcomeRecord.outcome.value))
            $withinAbs1e9 = $absDelta -le 1E-9
            if ($localBits -eq $excelBits) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_numeric_bits"
                $mismatches += 1
            }
        }
        elseif ($localKind -eq "error" -and $excelKind -eq "error") {
            if ([string] $local.outcome.code -eq [string] $excelOutcomeRecord.outcome.code) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_error_code"
                $mismatches += 1
            }
        }
        else {
            if ($local.outcome.digest_payload -eq $excelOutcomeRecord.outcome.digest_payload) {
                $comparisonResult = "match"
                $matches += 1
            }
            else {
                $comparisonResult = "mismatch_kind_or_payload"
                $mismatches += 1
            }
        }
    }

    $comparison = [ordered]@{
        schema_version = "oxfunc.smart_fuzzer.comparison.v0"
        case_id = $caseId
        function_id = [string] $case.function_id
        formula_text = [string] $case.formula_text
        comparison_result = $comparisonResult
        local_kind = $localKind
        excel_kind = $excelKind
        local_bits_hex = $localBits
        excel_bits_hex = $excelBits
        abs_delta = $absDelta
        within_abs_1e_9 = $withinAbs1e9
        coverage_buckets = $case.coverage_buckets
    }
    Add-JsonLine -Path $comparisonsPath -Value $comparison
    Add-JsonLine -Path $telemetryPath -Value ([ordered]@{
        schema_version = "oxfunc.smart_fuzzer.telemetry.v0"
        case_id = $caseId
        run_id = $RunId
        function_id = [string] $case.function_id
        generator_id = "pmt_ppmt_pilot.v0"
        seed = $Seed
        invocation_digest = Get-Sha256Text ([string] $case.formula_text)
        coverage_buckets = $case.coverage_buckets
        local_outcome_digest = $localDigest
        excel_outcome_digest = $excelDigest
        comparison_result = $comparisonResult
    })

    if ($comparisonResult -like "mismatch*") {
        $packetPath = Join-Path $failureDir ($caseId + ".json")
        Write-JsonFile -Path $packetPath -Value ([ordered]@{
            schema_version = "oxfunc.smart_fuzzer.failure_packet.v0"
            run_id = $RunId
            case = $case
            local_outcome = $local
            excel_outcome = $excelOutcomeRecord
            comparison = $comparison
        })
        $failurePackets.Add($packetPath.Replace($RepoRoot + "\", "")) | Out-Null
    }
}
$comparisonWatch.Stop()

$byFunction = @{}
$byCoverageBucket = @{}
foreach ($case in $cases) {
    $functionId = [string] $case.function_id
    if (-not $byFunction.ContainsKey($functionId)) {
        $byFunction[$functionId] = 0
    }
    $byFunction[$functionId] += 1
    foreach ($bucket in $case.coverage_buckets) {
        $bucketKey = [string] $bucket
        if (-not $byCoverageBucket.ContainsKey($bucketKey)) {
            $byCoverageBucket[$bucketKey] = 0
        }
        $byCoverageBucket[$bucketKey] += 1
    }
}

$manifest = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.run_manifest.v0"
    run_id = $RunId
    created_utc = $createdUtc
    git_revision = $gitRevision
    worktree_dirty = -not [string]::IsNullOrWhiteSpace($gitStatus)
    runner = [ordered]@{
        runner_id = "smart-fuzzer-pmt-ppmt-pilot"
        runner_version = $runnerVersion
        command_line = @([System.Environment]::CommandLine)
    }
    scope = [ordered]@{
        function_ids = @("FUNC.PMT", "FUNC.PPMT")
        generator_ids = @("pmt_ppmt_pilot.v0")
        excel_budget_cases = $cases.Count
        local_budget_cases = $cases.Count
    }
    environment = [ordered]@{
        host_os = [System.Environment]::OSVersion.VersionString
        rust_profile = "cargo run default dev profile for helper"
        excel = $excelResult.environment
        locale_profile = "en-US"
    }
    inputs = [ordered]@{
        metadata_snapshot_refs = @("docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv")
        source_index_digest = $null
        seed = $Seed
        generator_policy = "fixed PMT/PPMT witness, neighborhood, arity, rate, period, fv/type, and invalid-period rows"
    }
}

$manifestPath = Join-Path $runDir "manifest.json"
Write-JsonFile -Path $manifestPath -Value $manifest
$manifestHash = Get-Sha256Text (Get-Content -LiteralPath $manifestPath -Raw)

$rollup = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.rollup.v0"
    run_id = $RunId
    manifest_hash = $manifestHash
    case_counts = [ordered]@{
        generated = $cases.Count
        local_evaluated = $localById.Count
        excel_evaluated = $excelById.Count
        matches = $matches
        mismatches = $mismatches
        unstable = 0
        blocked = $blocked
        invalid_generator_output = 0
    }
    by_function = $byFunction
    by_generator = @{ "pmt_ppmt_pilot.v0" = $cases.Count }
    by_coverage_bucket = $byCoverageBucket
    throughput = [ordered]@{
        local_cases_per_second = if ($localWatch.Elapsed.TotalSeconds -gt 0) { $localById.Count / $localWatch.Elapsed.TotalSeconds } else { $null }
        excel_cases_per_second = if ($excelWatch.Elapsed.TotalSeconds -gt 0) { $excelById.Count / $excelWatch.Elapsed.TotalSeconds } else { $null }
        comparison_cases_per_second = if ($comparisonWatch.Elapsed.TotalSeconds -gt 0) { $cases.Count / $comparisonWatch.Elapsed.TotalSeconds } else { $null }
        local_wall_seconds = $localWatch.Elapsed.TotalSeconds
        excel_wall_seconds = $excelWatch.Elapsed.TotalSeconds
        comparison_wall_seconds = $comparisonWatch.Elapsed.TotalSeconds
    }
    promotion_candidates = @($failurePackets)
    workbook_path = if ($KeepWorkbook) { $workbookPath.Replace($RepoRoot + "\", "") } else { $null }
}
if ($excelResult.blocked) {
    $rollup["blocker"] = $excelResult.blocker
}

Write-JsonFile -Path (Join-Path $runDir "rollup.json") -Value $rollup
Write-JsonFile -Path (Join-Path $logDir "pmt-ppmt-pilot-summary.json") -Value ([ordered]@{
    run_id = $RunId
    manifest_hash = $manifestHash
    total_cases = $cases.Count
    matches = $matches
    mismatches = $mismatches
    blocked = $blocked
    promotion_candidates = @($failurePackets)
})

Write-Host "Run: $RunId"
Write-Host "Run directory: $runDir"
Write-Host "Manifest hash: $manifestHash"
Write-Host ("Cases: {0}; matches: {1}; mismatches: {2}; blocked: {3}" -f $cases.Count, $matches, $mismatches, $blocked)
if ($failurePackets.Count -gt 0) {
    Write-Host "Failure packets:"
    foreach ($packet in $failurePackets) {
        Write-Host ("  {0}" -f $packet)
    }
}

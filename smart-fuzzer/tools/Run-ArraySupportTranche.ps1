param(
    [string]$RunId = "",
    [string]$TranchePath = "smart-fuzzer/cache/array-support-first-tranche-v0.json",
    [string]$CaseSetPath = "",
    [string]$CaseSetTrancheId = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptPath "..\..")
$RepoRoot = $RepoRoot.Path
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($RunId)) {
    $RunId = "w090-array-tranche-a-" + (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ")
}

$RunRoot = Join-Path $RepoRoot "smart-fuzzer\runs\$RunId"
if (Test-Path $RunRoot) {
    throw "run directory already exists: $RunRoot"
}

$CaseDir = Join-Path $RunRoot "cases"
$OutcomeDir = Join-Path $RunRoot "outcomes"
$ComparisonDir = Join-Path $RunRoot "comparisons"
$FailureDir = Join-Path $RunRoot "failure_packets"
foreach ($path in @($RunRoot, $CaseDir, $OutcomeDir, $ComparisonDir, $FailureDir)) {
    [void](New-Item -ItemType Directory -Path $path -Force)
}

$CasesPath = Join-Path $CaseDir "cases.jsonl"
$ExcelOutcomesPath = Join-Path $OutcomeDir "excel.jsonl"
$LocalOutcomesPath = Join-Path $OutcomeDir "local.jsonl"
$ComparisonsPath = Join-Path $ComparisonDir "comparisons.jsonl"
$RollupPath = Join-Path $RunRoot "rollup.json"
$ManifestPath = Join-Path $RunRoot "manifest.json"
$RoadmapPath = Join-Path $RunRoot "roadmap_trace.md"

function ConvertTo-JsonLine {
    param([Parameter(Mandatory = $true)] $InputObject)
    return ($InputObject | ConvertTo-Json -Depth 80 -Compress)
}

function Add-JsonLine {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$InputObject
    )
    Add-Content -Path $Path -Encoding UTF8 -Value (ConvertTo-JsonLine $InputObject)
}

function New-JsonArray {
    param([object[]]$Items)
    $list = New-Object 'System.Collections.Generic.List[object]'
    foreach ($item in $Items) {
        [void]$list.Add($item)
    }
    return $list
}

function New-NumberValue {
    param([double]$Value)
    return [ordered]@{ kind = "number"; value = $Value }
}

function New-TextValue {
    param([string]$Value)
    return [ordered]@{ kind = "text"; value = $Value }
}

function New-LogicalValue {
    param([bool]$Value)
    return [ordered]@{ kind = "logical"; value = $Value }
}

function New-ErrorValue {
    param([string]$Code)
    return [ordered]@{ kind = "error"; code = $Code }
}

function New-Row {
    param([object[]]$Cells)
    return New-JsonArray $Cells
}

function New-ArrayValue {
    param([object[]]$Rows)
    return [ordered]@{ kind = "array"; rows = (New-JsonArray $Rows) }
}

function New-HNumberArray {
    param([double[]]$Values)
    $row = New-Object 'System.Collections.Generic.List[object]'
    foreach ($value in $Values) {
        [void]$row.Add((New-NumberValue $value))
    }
    $rows = New-Object 'System.Collections.Generic.List[object]'
    [void]$rows.Add($row)
    return [ordered]@{ kind = "array"; rows = $rows }
}

function New-VNumberArray {
    param([double[]]$Values)
    $rows = New-Object 'System.Collections.Generic.List[object]'
    foreach ($value in $Values) {
        $row = New-Object 'System.Collections.Generic.List[object]'
        [void]$row.Add((New-NumberValue $value))
        [void]$rows.Add($row)
    }
    return [ordered]@{ kind = "array"; rows = $rows }
}

function Get-GitRevision {
    try {
        return (& git rev-parse HEAD).Trim()
    } catch {
        return "git_revision_unavailable"
    }
}

function Get-GitStatusShort {
    try {
        return ((& git status --short) -join "`n")
    } catch {
        return "git_status_unavailable"
    }
}

function New-CaseArgMap {
    $map = @{}
    $map["=ROUND({1.234,2.345},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.234, 2.345)), (New-NumberValue 1))
    $map["=ROUND(1.234,{0,1})"] = New-JsonArray @((New-NumberValue 1.234), (New-HNumberArray -Values @(0, 1)))
    $map["=ROUND({1.234;2.345},{0;1})"] = New-JsonArray @((New-VNumberArray -Values @(1.234, 2.345)), (New-VNumberArray -Values @(0, 1)))
    $map["=ROUNDDOWN({1.234,2.345},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.234, 2.345)), (New-NumberValue 1))
    $map["=ROUNDDOWN(1.234,{0,1})"] = New-JsonArray @((New-NumberValue 1.234), (New-HNumberArray -Values @(0, 1)))
    $map["=ROUNDUP({1.234,2.345},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.234, 2.345)), (New-NumberValue 1))
    $map["=ROUNDUP(1.234,{0,1})"] = New-JsonArray @((New-NumberValue 1.234), (New-HNumberArray -Values @(0, 1)))
    $map["=TRUNC({1.234,-2.345},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.234, -2.345)), (New-NumberValue 1))
    $map["=TRUNC(1.234,{0,1})"] = New-JsonArray @((New-NumberValue 1.234), (New-HNumberArray -Values @(0, 1)))
    $map["=TRUNC({1.234;2.345})"] = New-JsonArray @((New-VNumberArray -Values @(1.234, 2.345)))
    $map["=CEILING({1.2,2.3},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.2, 2.3)), (New-NumberValue 1))
    $map["=CEILING(1.2,{0.5,1})"] = New-JsonArray @((New-NumberValue 1.2), (New-HNumberArray -Values @(0.5, 1)))
    $map["=CEILING.MATH({-1.2,1.2})"] = New-JsonArray @((New-HNumberArray -Values @(-1.2, 1.2)))
    $map["=CEILING.MATH(-1.2,{1,2})"] = New-JsonArray @((New-NumberValue -1.2), (New-HNumberArray -Values @(1, 2)))
    $map["=CEILING.MATH(-1.2,1,{0,1})"] = New-JsonArray @((New-NumberValue -1.2), (New-NumberValue 1), (New-HNumberArray -Values @(0, 1)))
    $map["=CEILING.PRECISE({-1.2,1.2})"] = New-JsonArray @((New-HNumberArray -Values @(-1.2, 1.2)))
    $map["=CEILING.PRECISE(-1.2,{1,2})"] = New-JsonArray @((New-NumberValue -1.2), (New-HNumberArray -Values @(1, 2)))
    $map["=FLOOR({1.2,2.3},1)"] = New-JsonArray @((New-HNumberArray -Values @(1.2, 2.3)), (New-NumberValue 1))
    $map["=FLOOR(1.2,{0.5,1})"] = New-JsonArray @((New-NumberValue 1.2), (New-HNumberArray -Values @(0.5, 1)))
    $map["=FLOOR.MATH({-1.2,1.2})"] = New-JsonArray @((New-HNumberArray -Values @(-1.2, 1.2)))
    $map["=FLOOR.MATH(-1.2,{1,2})"] = New-JsonArray @((New-NumberValue -1.2), (New-HNumberArray -Values @(1, 2)))
    $map["=FLOOR.MATH(-1.2,1,{0,1})"] = New-JsonArray @((New-NumberValue -1.2), (New-NumberValue 1), (New-HNumberArray -Values @(0, 1)))
    $map["=FLOOR.PRECISE({-1.2,1.2})"] = New-JsonArray @((New-HNumberArray -Values @(-1.2, 1.2)))
    $map["=FLOOR.PRECISE(-1.2,{1,2})"] = New-JsonArray @((New-NumberValue -1.2), (New-HNumberArray -Values @(1, 2)))
    $map["=ISO.CEILING({-1.2,1.2})"] = New-JsonArray @((New-HNumberArray -Values @(-1.2, 1.2)))
    $map["=ISO.CEILING(-1.2,{1,2})"] = New-JsonArray @((New-NumberValue -1.2), (New-HNumberArray -Values @(1, 2)))
    $map["=ATAN2({1,0},1)"] = New-JsonArray @((New-HNumberArray -Values @(1, 0)), (New-NumberValue 1))
    $map["=ATAN2(1,{0,1})"] = New-JsonArray @((New-NumberValue 1), (New-HNumberArray -Values @(0, 1)))
    $map["=ATAN2({0,1},{0,1})"] = New-JsonArray @((New-HNumberArray -Values @(0, 1)), (New-HNumberArray -Values @(0, 1)))
    $map["=BASE({15,16},16)"] = New-JsonArray @((New-HNumberArray -Values @(15, 16)), (New-NumberValue 16))
    $map["=BASE(15,{2,16})"] = New-JsonArray @((New-NumberValue 15), (New-HNumberArray -Values @(2, 16)))
    $map["=BASE(15,16,{2,4})"] = New-JsonArray @((New-NumberValue 15), (New-NumberValue 16), (New-HNumberArray -Values @(2, 4)))
    $map["=MROUND({1.3,2.7},0.5)"] = New-JsonArray @((New-HNumberArray -Values @(1.3, 2.7)), (New-NumberValue 0.5))
    $map["=MROUND(1.3,{0.5,1})"] = New-JsonArray @((New-NumberValue 1.3), (New-HNumberArray -Values @(0.5, 1)))
    return $map
}

function New-ArraySweepCases {
    param([Parameter(Mandatory = $true)]$Tranche)

    $argMap = New-CaseArgMap
    $cases = New-Object 'System.Collections.Generic.List[object]'
    $caseIndex = 1
    foreach ($surface in @($Tranche.surfaces)) {
        foreach ($seed in @($surface.formula_seeds)) {
            $formula = [string]$seed.formula
            if (-not $argMap.ContainsKey($formula)) {
                throw "no local argument mapping for formula seed: $formula"
            }
            $rawArgs = $argMap[$formula]
            if ($rawArgs -is [System.Collections.Specialized.OrderedDictionary] -or $rawArgs -is [hashtable]) {
                $typedArgs = [object[]]@($rawArgs)
            } elseif ($rawArgs -is [System.Collections.IEnumerable] -and -not ($rawArgs -is [string])) {
                $typedArgList = New-Object 'System.Collections.Generic.List[object]'
                foreach ($arg in $rawArgs) {
                    [void]$typedArgList.Add($arg)
                }
                $typedArgs = [object[]]$typedArgList.ToArray()
            } else {
                $typedArgs = [object[]]@($rawArgs)
            }
            $caseId = "w090-tranche-a-{0:d4}-{1}-{2}" -f $caseIndex, (($surface.canonical_surface_name -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant()), (($seed.case_tag -replace '[^A-Za-z0-9]+', '-').ToLowerInvariant())
            $cases.Add([ordered]@{
                schema_version = "oxfunc.smart_fuzzer.array_case.v0"
                run_id = $RunId
                tranche_id = [string]$Tranche.tranche_id
                case_id = $caseId
                function_id = [string]$surface.surface_id
                canonical_surface_name = [string]$surface.canonical_surface_name
                case_tag = [string]$seed.case_tag
                axis = [string]$seed.axis
                expected_probe_class = [string]$seed.expected_probe_class
                formula_text = $formula
                args = $typedArgs
            }) | Out-Null
            $caseIndex += 1
        }
    }
    return $cases
}

function Read-ArraySweepCaseSet {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [string]$SelectedTrancheId = ""
    )

    $resolved = Resolve-Path (Join-Path $RepoRoot $Path)
    $raw = Get-Content $resolved -Raw -Encoding UTF8
    $parsed = $raw | ConvertFrom-Json

    $selectedTranche = $null
    $selectedCaseIds = $null
    if (-not [string]::IsNullOrWhiteSpace($SelectedTrancheId)) {
        foreach ($tranche in @($parsed.tranches)) {
            if ([string]$tranche.tranche_id -eq $SelectedTrancheId) {
                $selectedTranche = $tranche
                break
            }
        }
        if ($null -eq $selectedTranche) {
            throw "case set does not contain tranche_id: $SelectedTrancheId"
        }
        $selectedCaseIds = @{}
        foreach ($caseId in @($selectedTranche.case_ids)) {
            $selectedCaseIds[[string]$caseId] = $true
        }
    }

    $cases = New-Object 'System.Collections.Generic.List[object]'
    foreach ($case in @($parsed.cases)) {
        if ($null -ne $selectedCaseIds -and -not $selectedCaseIds.ContainsKey([string]$case.case_id)) {
            continue
        }
        [void]$cases.Add($case)
    }
    if ($cases.Count -eq 0) {
        throw "case set tranche produced zero cases: $SelectedTrancheId"
    }
    $surfaces = @($cases | ForEach-Object {
        if ($_.PSObject.Properties.Name -contains "canonical_surface_name") {
            [string]$_.canonical_surface_name
        } else {
            [string]$_.function_id
        }
    } | Sort-Object -Unique)
    $effectiveTrancheId = if ($null -ne $selectedTranche) {
        [string]$selectedTranche.tranche_id
    } elseif ([string]::IsNullOrWhiteSpace([string]$parsed.tranche_id)) {
        "external-case-set"
    } else {
        [string]$parsed.tranche_id
    }
    return [ordered]@{
        tranche_id = $effectiveTrancheId
        surfaces = $surfaces
        cases = $cases
        selected_tranche = $selectedTranche
        metadata = $parsed
    }
}

function Get-DoubleBitsHex {
    param([double]$Value)
    $bytes = [BitConverter]::GetBytes($Value)
    $bits = [BitConverter]::ToUInt64($bytes, 0)
    return ("0x{0:x16}" -f $bits)
}

function New-NumberOutcome {
    param([double]$Value)
    $bits = Get-DoubleBitsHex $Value
    return [ordered]@{ kind = "number"; value = $Value; bits_hex = $bits; digest_payload = "number:$bits" }
}

function New-TextOutcome {
    param([string]$Value)
    return [ordered]@{ kind = "text"; value = $Value; digest_payload = "text:$Value" }
}

function New-LogicalOutcome {
    param([bool]$Value)
    $literal = if ($Value) { "true" } else { "false" }
    return [ordered]@{ kind = "logical"; value = $Value; digest_payload = "logical:$literal" }
}

function New-ErrorOutcome {
    param([string]$Code)
    return [ordered]@{ kind = "error"; code = $Code; digest_payload = "error:$Code" }
}

function New-EmptyCellOutcome {
    return [ordered]@{ kind = "empty_cell"; digest_payload = "empty_cell" }
}

function New-HarnessErrorOutcome {
    param([string]$Message)
    return [ordered]@{ kind = "harness_error"; message = $Message; digest_payload = "harness_error:$Message" }
}

function Get-CaseProperty {
    param(
        [Parameter(Mandatory = $true)]$Case,
        [Parameter(Mandatory = $true)][string]$Name
    )
    if ($Case.PSObject.Properties.Name -contains $Name) {
        return $Case.$Name
    }
    return $null
}

function Get-CaseStringProperty {
    param(
        [Parameter(Mandatory = $true)]$Case,
        [Parameter(Mandatory = $true)][string]$Name,
        [string]$Default = ""
    )
    $value = Get-CaseProperty $Case $Name
    if ($null -eq $value) {
        return $Default
    }
    return [string]$value
}

function Get-CaseArrayProperty {
    param(
        [Parameter(Mandatory = $true)]$Case,
        [Parameter(Mandatory = $true)][string]$Name
    )
    $value = Get-CaseProperty $Case $Name
    if ($null -eq $value) {
        return @()
    }
    if ($value -is [pscustomobject] -and @($value.PSObject.Properties).Count -eq 0) {
        return @()
    }
    if ($value -is [System.Array]) {
        return @($value)
    }
    return @($value)
}

function Get-ValueKind {
    param([Parameter(Mandatory = $true)]$Value)
    if ($Value.PSObject.Properties.Name -contains "kind") {
        return [string]$Value.kind
    }
    throw "fixture/input value is missing kind"
}

function Get-ExcelErrorFormula {
    param([string]$Code)
    switch ($Code) {
        "Null" { return "=#NULL!" }
        "Div0" { return "=1/0" }
        "Value" { return "=VALUE(""x"")" }
        "Ref" { return "=INDEX(A1,2,2)" }
        "Name" { return "=not_a_real_name" }
        "Num" { return "=SQRT(-1)" }
        "NA" { return "=NA()" }
        "Spill" { return "=SEQUENCE(0)" }
        "Calc" { return "=FILTER({1},{FALSE})" }
        default { throw "unsupported Excel fixture error code: $Code" }
    }
}

function Set-ExcelCellFromTypedValue {
    param(
        [Parameter(Mandatory = $true)]$Cell,
        [Parameter(Mandatory = $true)]$Value
    )
    switch (Get-ValueKind $Value) {
        "number" { $Cell.Value2 = [double]$Value.value }
        "text" { $Cell.Value2 = [string]$Value.value }
        "logical" { $Cell.Value2 = [bool]$Value.value }
        "empty_cell" { $Cell.ClearContents() | Out-Null }
        "error" { $Cell.Formula2 = Get-ExcelErrorFormula ([string]$Value.code) }
        default { throw "unsupported scalar fixture value kind: $($Value.kind)" }
    }
}

function Set-ExcelRangeFromTypedArray {
    param(
        [Parameter(Mandatory = $true)]$Range,
        [Parameter(Mandatory = $true)]$Rows
    )
    $rowValues = @($Rows)
    if ($rowValues.Count -eq 0) {
        throw "array fixture has no rows"
    }
    $firstRow = @($rowValues[0])
    $rowCount = $rowValues.Count
    $colCount = $firstRow.Count
    if ($colCount -eq 0) {
        throw "array fixture has no columns"
    }
    if ([int]$Range.Rows.Count -ne $rowCount -or [int]$Range.Columns.Count -ne $colCount) {
        throw "array fixture shape ${rowCount}x${colCount} does not match target $($Range.Address($false, $false))"
    }
    for ($r = 1; $r -le $rowCount; $r += 1) {
        $cells = @($rowValues[$r - 1])
        if ($cells.Count -ne $colCount) {
            throw "array fixture row has inconsistent column count"
        }
        for ($c = 1; $c -le $colCount; $c += 1) {
            Set-ExcelCellFromTypedValue $Range.Cells.Item($r, $c) $cells[$c - 1]
        }
    }
}

function Set-ExcelFixture {
    param(
        [Parameter(Mandatory = $true)]$Worksheet,
        [Parameter(Mandatory = $true)]$Fixture
    )
    $target = [string]$Fixture.target
    $range = $Worksheet.Range($target)
    $value = $Fixture.value
    if ((Get-ValueKind $value) -eq "array") {
        Set-ExcelRangeFromTypedArray $range @($value.rows)
    } else {
        Set-ExcelCellFromTypedValue $range.Cells.Item(1, 1) $value
    }
}

function Get-OutcomeDigest {
    param($Outcome)
    return [string]$Outcome.digest_payload
}

function New-ArrayOutcome {
    param(
        [int]$Rows,
        [int]$Cols,
        [object[]]$Cells
    )
    $digests = New-Object 'System.Collections.Generic.List[string]'
    foreach ($row in $Cells) {
        foreach ($cell in $row) {
            [void]$digests.Add((Get-OutcomeDigest $cell))
        }
    }
    return [ordered]@{
        kind = "array"
        rows = $Rows
        cols = $Cols
        cells = (New-JsonArray $Cells)
        digest_payload = "array:${Rows}x${Cols}:[$(($digests.ToArray()) -join '|')]"
    }
}

function Convert-ExcelErrorTypeToCode {
    param(
        [int]$ErrorType,
        [string]$Text
    )
    switch ($Text) {
        "#SPILL!" { return "Spill" }
        "#CALC!" { return "Calc" }
        "#FIELD!" { return "Field" }
        "#BLOCKED!" { return "Blocked" }
        "#CONNECT!" { return "Connect" }
    }
    switch ($ErrorType) {
        1 { return "Null" }
        2 { return "Div0" }
        3 { return "Value" }
        4 { return "Ref" }
        5 { return "Name" }
        6 { return "Num" }
        7 { return "NA" }
        8 { return "GettingData" }
        9 { return "Spill" }
        14 { return "Calc" }
        16 { return "Field" }
        17 { return "Blocked" }
        18 { return "Connect" }
        default { return "ExcelErrorType:$ErrorType" }
    }
}

function Convert-ExcelCellToOutcome {
    param(
        [Parameter(Mandatory = $true)]$Worksheet,
        [Parameter(Mandatory = $true)]$Cell
    )
    $address = $Cell.Address($false, $false)
    $isError = $false
    try {
        $isError = [bool]$Worksheet.Evaluate("ISERROR($address)")
    } catch {
        $isError = $false
    }
    if ($isError) {
        $errorType = [int]$Worksheet.Evaluate("ERROR.TYPE($address)")
        return New-ErrorOutcome (Convert-ExcelErrorTypeToCode $errorType ([string]$Cell.Text))
    }

    $value = $Cell.Value2
    if ($null -eq $value) {
        return New-EmptyCellOutcome
    }
    if ($value -is [bool]) {
        return New-LogicalOutcome ([bool]$value)
    }
    if ($value -is [byte] -or $value -is [int16] -or $value -is [int32] -or $value -is [int64] -or $value -is [single] -or $value -is [double] -or $value -is [decimal]) {
        return New-NumberOutcome ([double]$value)
    }
    return New-TextOutcome ([string]$value)
}

function Invoke-ExcelArrayEvaluation {
    param(
        [Parameter(Mandatory = $true)]$Cases,
        [Parameter(Mandatory = $true)][string]$OutputPath
    )

    $excel = $null
    $workbook = $null
    $environment = [ordered]@{}
    try {
        $excel = New-Object -ComObject Excel.Application
        $excel.Visible = $false
        $excel.DisplayAlerts = $false
        $excel.ScreenUpdating = $false
        $excel.EnableEvents = $false
        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)
        $environment = [ordered]@{
            available = $true
            version = [string]$excel.Version
            build = [string]$excel.Build
            calculation = [string]$excel.Calculation
            workbook_compatibility_version = try { [string]$workbook.CompatibilityVersion } catch { "unavailable" }
        }

        foreach ($case in $Cases) {
            try {
                $worksheet.Cells.Clear() | Out-Null
                $formulaCell = $null
                $fixtures = @(Get-CaseArrayProperty $case "cell_fixture")
                foreach ($fixture in $fixtures) {
                    Set-ExcelFixture $worksheet $fixture
                }
                $formulaCell = Get-CaseStringProperty $case "formula_cell"
                if ([string]::IsNullOrWhiteSpace($formulaCell)) {
                    $formulaCell = if ($fixtures.Count -gt 0) { "J10" } else { "A1" }
                }
                $anchor = $worksheet.Range($formulaCell)
                $anchor.Formula2 = [string]$case.formula_text
                $anchor.Calculate() | Out-Null
                try {
                    $spillCandidate = $anchor.SpillingToRange
                    $null = $spillCandidate.Rows.Count
                    $null = $spillCandidate.Columns.Count
                    $spill = $spillCandidate
                } catch {
                    $spill = $anchor
                }

                $rows = [int]$spill.Rows.Count
                $cols = [int]$spill.Columns.Count
                $cellRows = New-Object 'System.Collections.Generic.List[object]'
                for ($r = 1; $r -le $rows; $r += 1) {
                    $rowOutcomes = New-Object 'System.Collections.Generic.List[object]'
                    for ($c = 1; $c -le $cols; $c += 1) {
                        $cell = $spill.Cells.Item($r, $c)
                        [void]$rowOutcomes.Add((Convert-ExcelCellToOutcome $worksheet $cell))
                    }
                    [void]$cellRows.Add($rowOutcomes)
                }

                if ($rows -eq 1 -and $cols -eq 1) {
                    $outcome = $cellRows[0][0]
                } else {
                    $outcome = New-ArrayOutcome $rows $cols $cellRows.ToArray()
                }

                Add-JsonLine $OutputPath ([ordered]@{
                    schema_version = "oxfunc.smart_fuzzer.array_outcome.v0"
                    run_id = $RunId
                    case_id = [string]$case.case_id
                    function_id = [string]$case.function_id
                    formula_text = [string]$case.formula_text
                    evaluator_id = "excel.com.dynamic_array_spill_capture/0.1.0"
                    execution_status = "ok"
                    formula_cell = $formulaCell
                    spill_address = [string]$spill.Address($false, $false)
                    outcome = $outcome
                })
            } catch {
                Add-JsonLine $OutputPath ([ordered]@{
                    schema_version = "oxfunc.smart_fuzzer.array_outcome.v0"
                    run_id = $RunId
                    case_id = [string]$case.case_id
                    function_id = [string]$case.function_id
                    formula_text = [string]$case.formula_text
                    evaluator_id = "excel.com.dynamic_array_spill_capture/0.1.0"
                    execution_status = "excel_case_harness_error"
                    formula_cell = if ($null -eq $formulaCell) { $null } else { $formulaCell }
                    spill_address = $null
                    outcome = (New-HarnessErrorOutcome $_.Exception.Message)
                })
            }
        }
    } catch {
        $environment = [ordered]@{ available = $false; blocker = $_.Exception.Message }
        foreach ($case in $Cases) {
            Add-JsonLine $OutputPath ([ordered]@{
                schema_version = "oxfunc.smart_fuzzer.array_outcome.v0"
                run_id = $RunId
                case_id = [string]$case.case_id
                function_id = [string]$case.function_id
                formula_text = [string]$case.formula_text
                evaluator_id = "excel.com.dynamic_array_spill_capture/0.1.0"
                execution_status = "excel_harness_blocked"
                formula_cell = $null
                spill_address = $null
                outcome = (New-HarnessErrorOutcome $_.Exception.Message)
            })
        }
    } finally {
        if ($null -ne $workbook) {
            $workbook.Close($false) | Out-Null
            [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($workbook)
        }
        if ($null -ne $excel) {
            $excel.Quit() | Out-Null
            [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel)
        }
        [GC]::Collect()
        [GC]::WaitForPendingFinalizers()
    }
    return $environment
}

function Read-JsonLinesByCase {
    param([string]$Path)
    $map = @{}
    foreach ($line in Get-Content $Path -Encoding UTF8) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        $record = $line | ConvertFrom-Json
        $map[[string]$record.case_id] = $record
    }
    return $map
}

function Get-RecordDigest {
    param($Record)
    if ($null -eq $Record -or $null -eq $Record.outcome) {
        return ""
    }
    return [string]$Record.outcome.digest_payload
}

function Compare-ArrayOutcomes {
    param(
        [Parameter(Mandatory = $true)]$Cases,
        [Parameter(Mandatory = $true)][string]$LocalPath,
        [Parameter(Mandatory = $true)][string]$ExcelPath,
        [Parameter(Mandatory = $true)][string]$OutputPath,
        [Parameter(Mandatory = $true)][string]$FailurePacketDir
    )

    $localByCase = Read-JsonLinesByCase $LocalPath
    $excelByCase = Read-JsonLinesByCase $ExcelPath
    $rollup = [ordered]@{
        total_cases = 0
        by_classification = [ordered]@{}
        by_function = [ordered]@{}
        mismatch_case_ids = New-Object 'System.Collections.Generic.List[string]'
        axis_witness_pairs = [ordered]@{
            total_pairs = 0
            differentiated_pairs = 0
            not_differentiated_pairs = 0
            blocked_or_incomplete_pairs = 0
            by_axis_tag = [ordered]@{}
            pairs = New-Object 'System.Collections.Generic.List[object]'
        }
    }

    foreach ($case in $Cases) {
        $caseId = [string]$case.case_id
        $local = $localByCase[$caseId]
        $excel = $excelByCase[$caseId]
        $localDigest = Get-RecordDigest $local
        $excelDigest = Get-RecordDigest $excel
        $classification = "unexpected_mismatch"
        if ($null -eq $local) {
            $classification = "local_harness_missing"
        } elseif ($null -eq $excel) {
            $classification = "excel_harness_missing"
        } elseif ([string]$local.execution_status -ne "ok") {
            $classification = "local_harness_blocked"
        } elseif ([string]$excel.execution_status -ne "ok") {
            $classification = "excel_harness_blocked"
        } elseif ($localDigest -eq $excelDigest) {
            $classification = "exact_typed_bit_match"
        } elseif ((Get-CaseArrayProperty $case "known_deviation_tags") -contains "expected_known_financial_exactness_drift") {
            $classification = "known_expected_deviation"
        }

        $comparison = [ordered]@{
            schema_version = "oxfunc.smart_fuzzer.array_comparison.v0"
            run_id = $RunId
            case_id = $caseId
            function_id = [string]$case.function_id
            canonical_surface_name = [string]$case.canonical_surface_name
            case_tag = [string]$case.case_tag
            axis = [string]$case.axis
            axis_pair_id = Get-CaseStringProperty $case "axis_pair_id"
            axis_role = Get-CaseStringProperty $case "axis_role"
            axis_group = Get-CaseStringProperty $case "axis_group"
            axis_tag = Get-CaseStringProperty $case "axis_tag"
            formula_text = [string]$case.formula_text
            classification = $classification
            local_execution_status = if ($null -eq $local) { "missing" } else { [string]$local.execution_status }
            excel_execution_status = if ($null -eq $excel) { "missing" } else { [string]$excel.execution_status }
            local_digest = $localDigest
            excel_digest = $excelDigest
        }
        Add-JsonLine $OutputPath $comparison

        $rollup.total_cases += 1
        if (-not $rollup.by_classification.Contains($classification)) {
            $rollup.by_classification[$classification] = 0
        }
        $rollup.by_classification[$classification] += 1
        $fn = [string]$case.function_id
        if (-not $rollup.by_function.Contains($fn)) {
            $rollup.by_function[$fn] = [ordered]@{}
        }
        if (-not $rollup.by_function[$fn].Contains($classification)) {
            $rollup.by_function[$fn][$classification] = 0
        }
        $rollup.by_function[$fn][$classification] += 1

        if ($classification -ne "exact_typed_bit_match") {
            [void]$rollup.mismatch_case_ids.Add($caseId)
            $packetPath = Join-Path $FailurePacketDir "$caseId.json"
            [ordered]@{
                schema_version = "oxfunc.smart_fuzzer.array_failure_packet.v0"
                run_id = $RunId
                case = $case
                comparison = $comparison
                local = $local
                excel = $excel
            } | ConvertTo-Json -Depth 100 | Set-Content -Path $packetPath -Encoding UTF8
        }
    }

    $pairGroups = @($Cases | Where-Object {
        -not [string]::IsNullOrWhiteSpace((Get-CaseStringProperty $_ "axis_pair_id"))
    } | Group-Object { Get-CaseStringProperty $_ "axis_pair_id" })
    foreach ($pairGroup in $pairGroups) {
        $pairCases = @($pairGroup.Group)
        $control = @($pairCases | Where-Object { (Get-CaseStringProperty $_ "axis_role") -eq "control" } | Select-Object -First 1)
        $variant = @($pairCases | Where-Object { (Get-CaseStringProperty $_ "axis_role") -eq "variant" } | Select-Object -First 1)
        $axisGroup = if ($pairCases.Count -gt 0) { Get-CaseStringProperty $pairCases[0] "axis_group" } else { "" }
        $axisTag = if ($pairCases.Count -gt 0) { Get-CaseStringProperty $pairCases[0] "axis_tag" } else { "" }
        if ([string]::IsNullOrWhiteSpace($axisTag)) { $axisTag = "(unknown)" }
        if (-not $rollup.axis_witness_pairs.by_axis_tag.Contains($axisTag)) {
            $rollup.axis_witness_pairs.by_axis_tag[$axisTag] = [ordered]@{
                total = 0
                differentiated = 0
                not_differentiated = 0
                blocked_or_incomplete = 0
            }
        }

        $pairClass = "blocked_or_incomplete"
        $controlDigest = ""
        $variantDigest = ""
        if ($control.Count -eq 1 -and $variant.Count -eq 1) {
            $controlRecord = $excelByCase[[string]$control[0].case_id]
            $variantRecord = $excelByCase[[string]$variant[0].case_id]
            if ($null -ne $controlRecord -and $null -ne $variantRecord -and
                [string]$controlRecord.execution_status -eq "ok" -and
                [string]$variantRecord.execution_status -eq "ok") {
                $controlDigest = Get-RecordDigest $controlRecord
                $variantDigest = Get-RecordDigest $variantRecord
                $pairClass = if ($controlDigest -ne $variantDigest) {
                    "differentiated"
                } else {
                    "not_differentiated"
                }
            }
        }

        $rollup.axis_witness_pairs.total_pairs += 1
        $rollup.axis_witness_pairs.by_axis_tag[$axisTag].total += 1
        switch ($pairClass) {
            "differentiated" {
                $rollup.axis_witness_pairs.differentiated_pairs += 1
                $rollup.axis_witness_pairs.by_axis_tag[$axisTag].differentiated += 1
            }
            "not_differentiated" {
                $rollup.axis_witness_pairs.not_differentiated_pairs += 1
                $rollup.axis_witness_pairs.by_axis_tag[$axisTag].not_differentiated += 1
            }
            default {
                $rollup.axis_witness_pairs.blocked_or_incomplete_pairs += 1
                $rollup.axis_witness_pairs.by_axis_tag[$axisTag].blocked_or_incomplete += 1
            }
        }
        [void]$rollup.axis_witness_pairs.pairs.Add([ordered]@{
            axis_pair_id = $pairGroup.Name
            axis_group = $axisGroup
            axis_tag = $axisTag
            classification = $pairClass
            control_case_id = if ($control.Count -eq 1) { [string]$control[0].case_id } else { $null }
            variant_case_id = if ($variant.Count -eq 1) { [string]$variant[0].case_id } else { $null }
            control_excel_digest = $controlDigest
            variant_excel_digest = $variantDigest
        })
    }
    return $rollup
}

function Write-RoadmapTrace {
    param(
        [Parameter(Mandatory = $true)]$Tranche,
        [Parameter(Mandatory = $true)]$Cases,
        [Parameter(Mandatory = $true)]$Rollup,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $lines = New-Object 'System.Collections.Generic.List[object]'
    [void]$lines.Add("# Array Support Roadmap Trace")
    [void]$lines.Add("")
    [void]$lines.Add("- Run ID: $RunId")
    [void]$lines.Add("- Tranche: $($Tranche.tranche_id)")
    [void]$lines.Add("- Comparison policy: exact typed equality with bit-exact numeric digests; no tolerance.")
    $surfaceCount = 0
    foreach ($surface in $Tranche.surfaces) {
        $surfaceCount += 1
    }
    $caseCount = 0
    foreach ($case in $Cases) {
        $caseCount += 1
    }
    [void]$lines.Add("- Surfaces executed: $surfaceCount")
    [void]$lines.Add("- Formula cases executed: $caseCount")
    [void]$lines.Add("")
    [void]$lines.Add("## Axes touched")
    [void]$lines.Add("")
    $categories = @($Cases | Where-Object { $_.PSObject.Properties.Name -contains "category" } | Select-Object -ExpandProperty category -Unique | Sort-Object)
    if ($categories.Count -gt 0) {
        [void]$lines.Add("- Function family/category: $(($categories | ForEach-Object { [string]$_ }) -join ', ').")
    } else {
        [void]$lines.Add("- Function family/category: tranche-defined surface set.")
    }
    $riskBands = @($Cases | Where-Object { $_.PSObject.Properties.Name -contains "risk_band" } | Select-Object -ExpandProperty risk_band -Unique | Sort-Object)
    if ($riskBands.Count -gt 0) {
        [void]$lines.Add("- Risk bands: $(($riskBands | ForEach-Object { [string]$_ }) -join ', ').")
    }
    $axes = @($Cases | Select-Object -ExpandProperty axis -Unique | Sort-Object)
    if ($axes.Count -gt 0) {
        [void]$lines.Add("- Array argument axes: $(($axes | ForEach-Object { [string]$_ }) -join ', ').")
    }
    $axisTags = @($Cases | Where-Object { $_.PSObject.Properties.Name -contains "axis_tag" } | Select-Object -ExpandProperty axis_tag -Unique | Sort-Object)
    if ($axisTags.Count -gt 0) {
        [void]$lines.Add("- Invocation-space axis tags: $(($axisTags | ForEach-Object { [string]$_ }) -join ', ').")
    }
    [void]$lines.Add("- Shape/value pattern: inline array literals supplied by the case set, with exact Excel Formula2 spill capture.")
    $sources = @($Cases | Where-Object { $_.PSObject.Properties.Name -contains "source_manifest" } | Select-Object -ExpandProperty source_manifest -Unique | Sort-Object)
    if ($sources.Count -gt 0) {
        [void]$lines.Add("- Scenario seed sources: $(($sources | ForEach-Object { [string]$_ }) -join ', ').")
    }
    [void]$lines.Add("")
    [void]$lines.Add("## Rollup")
    [void]$lines.Add("")
    foreach ($key in $Rollup.by_classification.Keys) {
        [void]$lines.Add("- ${key}: $($Rollup.by_classification[$key])")
    }
    if ($Rollup.PSObject.Properties.Name -contains "axis_witness_pairs") {
        [void]$lines.Add("- axis witness pairs: total=$($Rollup.axis_witness_pairs.total_pairs), differentiated=$($Rollup.axis_witness_pairs.differentiated_pairs), not_differentiated=$($Rollup.axis_witness_pairs.not_differentiated_pairs), blocked_or_incomplete=$($Rollup.axis_witness_pairs.blocked_or_incomplete_pairs)")
    }
    [void]$lines.Add("")
    [void]$lines.Add("## Function highlights")
    [void]$lines.Add("")
    foreach ($fn in $Rollup.by_function.Keys) {
        $parts = New-Object 'System.Collections.Generic.List[string]'
        foreach ($class in $Rollup.by_function[$fn].Keys) {
            [void]$parts.Add("${class}=$($Rollup.by_function[$fn][$class])")
        }
        [void]$lines.Add("- ${fn}: $(($parts.ToArray()) -join ', ')")
    }
    [void]$lines.Add("")
    [void]$lines.Add("## Failure packet policy")
    [void]$lines.Add("")
    [void]$lines.Add("Pass rows remain aggregate telemetry in comparisons.jsonl and rollup.json. Full per-case packets were written only for non-pass classifications.")
    $lines | Set-Content -Path $Path -Encoding UTF8
}

if ([string]::IsNullOrWhiteSpace($CaseSetPath)) {
    $ResolvedTranchePath = Resolve-Path (Join-Path $RepoRoot $TranchePath)
    $Tranche = Get-Content $ResolvedTranchePath -Raw -Encoding UTF8 | ConvertFrom-Json
    $Cases = New-ArraySweepCases $Tranche
} else {
    $caseSet = Read-ArraySweepCaseSet $CaseSetPath $CaseSetTrancheId
    $Tranche = [pscustomobject]@{
        tranche_id = $caseSet.tranche_id
        surfaces = $caseSet.surfaces
    }
    $Cases = $caseSet.cases
}
foreach ($case in $Cases) {
    if ($case.PSObject.Properties.Name -contains "run_id") {
        $case.run_id = $RunId
    } else {
        $case | Add-Member -NotePropertyName run_id -NotePropertyValue $RunId -Force
    }
    if (-not ($case.PSObject.Properties.Name -contains "tranche_id") -or [string]::IsNullOrWhiteSpace([string]$case.tranche_id)) {
        $case | Add-Member -NotePropertyName tranche_id -NotePropertyValue ([string]$Tranche.tranche_id) -Force
    }
    Add-JsonLine $CasesPath $case
}

$localHelperManifest = Join-Path $RepoRoot "smart-fuzzer\tools\pmt_ppmt_local_eval\Cargo.toml"
& cargo run --manifest-path $localHelperManifest --bin array_tranche_local_eval -- --cases $CasesPath --out $LocalOutcomesPath
if ($LASTEXITCODE -ne 0) {
    throw "local array_tranche_local_eval failed with exit code $LASTEXITCODE"
}

$ExcelEnvironment = Invoke-ExcelArrayEvaluation $Cases $ExcelOutcomesPath
$ComparisonRollup = Compare-ArrayOutcomes $Cases $LocalOutcomesPath $ExcelOutcomesPath $ComparisonsPath $FailureDir

$Rollup = [pscustomobject]@{
    schema_version = "oxfunc.smart_fuzzer.array_rollup.v0"
    run_id = $RunId
    tranche_id = [string]$Tranche.tranche_id
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    total_cases = $ComparisonRollup.total_cases
    by_classification = $ComparisonRollup.by_classification
    by_function = $ComparisonRollup.by_function
    mismatch_case_ids = $ComparisonRollup.mismatch_case_ids
    axis_witness_pairs = $ComparisonRollup.axis_witness_pairs
    excel_environment = $ExcelEnvironment
    comparison_policy = "exact_typed_bit_match_no_tolerance"
    failure_packet_dir = "smart-fuzzer/runs/$RunId/failure_packets"
}
$Rollup | ConvertTo-Json -Depth 100 | Set-Content -Path $RollupPath -Encoding UTF8

$Manifest = [ordered]@{
    schema_version = "oxfunc.smart_fuzzer.array_run_manifest.v0"
    run_id = $RunId
    runner = "smart-fuzzer/tools/Run-ArraySupportTranche.ps1"
    runner_version = "0.1.0"
    generated_utc = (Get-Date).ToUniversalTime().ToString("o")
    git_revision = Get-GitRevision
    git_status_short = Get-GitStatusShort
    tranche_path = if ([string]::IsNullOrWhiteSpace($CaseSetPath)) { $TranchePath } else { $CaseSetPath }
    case_set_tranche_id = $CaseSetTrancheId
    tranche_id = [string]$Tranche.tranche_id
    artifacts = [ordered]@{
        cases = "smart-fuzzer/runs/$RunId/cases/cases.jsonl"
        local_outcomes = "smart-fuzzer/runs/$RunId/outcomes/local.jsonl"
        excel_outcomes = "smart-fuzzer/runs/$RunId/outcomes/excel.jsonl"
        comparisons = "smart-fuzzer/runs/$RunId/comparisons/comparisons.jsonl"
        rollup = "smart-fuzzer/runs/$RunId/rollup.json"
        roadmap_trace = "smart-fuzzer/runs/$RunId/roadmap_trace.md"
        failure_packets = "smart-fuzzer/runs/$RunId/failure_packets"
    }
}
$Manifest | ConvertTo-Json -Depth 100 | Set-Content -Path $ManifestPath -Encoding UTF8
Write-RoadmapTrace -Tranche $Tranche -Cases $Cases -Rollup $Rollup -Path $RoadmapPath

Write-Host "array tranche run finished"
Write-Host "run_id=$RunId"
Write-Host "rollup=$RollupPath"
Write-Host "comparisons=$ComparisonsPath"

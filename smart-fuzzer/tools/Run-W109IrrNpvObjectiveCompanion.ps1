[CmdletBinding(PositionalBinding = $false)]
param(
    [Parameter(Mandatory)] [string] $Batch,
    [Parameter(Mandatory)] [string] $Out,
    [Parameter(Mandatory)] [string] $ExpectedBatchSha256,
    [switch] $ValidateOnly
)

# Clean-room, discovery-only worksheet-NPV companion for the frozen W109 IRR
# exact-graph battery.  The runner always computes live (no oracle cache),
# writes every binary64 argument through one Range.Value2 matrix, and invokes
# NPV only through R1C1 cell references.  It captures three related surfaces:
#
#   raw              = NPV(rate,c1..cn)
#   direct_composed  = NPV(rate,c1..cn)+c0 in one formula
#   cell_composed    = raw_result_cell+c0 in a separate formula
#
# The distinction decides whether an IRR objective discrepancy belongs inside
# worksheet NPV, in the caller's composition, or at the helper publication
# boundary.  This script must only be launched under the serialized Excel lane.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:RunnerVersion = 'w109-irr-npv-objective-companion-v1'
$script:ExpectedSchema = 'w109.irr.npv_objective_companion.batch.v1'
$script:ExpectedFunction = 'NPV'
$script:ExpectedRowId = 'irr-npv-objective-companion-discovery-20260809'
$script:ExpectedSourceRowId = 'irr-exact-graph-discovery-20260809'
$script:ExpectedSourceSha256 = '93E340A1A571799519DA9D38B26996C8BBA439B7BF646C9185D3966874B55A98'
$script:ExpectedSourcePath = 'smart-fuzzer/work/w109/G6-solvers/batch-irr-exact-graph-discovery-20260809.json'
$script:ExpectedProbeCount = 900
$script:ExpectedSourceCount = 300
$script:ExpectedHBits = '0x3f50624de0000000'
$script:XlCalculationManual = -4135

function Get-BitsHex([double] $Value) {
    $bits = [BitConverter]::ToUInt64([BitConverter]::GetBytes($Value), 0)
    return ('0x{0:x16}' -f $bits)
}

function From-BitsHex([string] $Raw) {
    if ($Raw -notmatch '^0x[0-9a-fA-F]{16}$') {
        throw "Invalid binary64 hex string: $Raw"
    }
    $bits = [uint64]::Parse($Raw.Substring(2), [Globalization.NumberStyles]::HexNumber)
    return [BitConverter]::ToDouble([BitConverter]::GetBytes($bits), 0)
}

function Assert-Equal([string] $Label, [object] $Actual, [object] $Expected) {
    if ($Actual -ne $Expected) {
        throw "$Label mismatch: actual='$Actual' expected='$Expected'"
    }
}

function Release-Com([object] $Object) {
    if ($null -ne $Object -and [Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function Get-ExcelBitness([object] $Excel) {
    $operatingSystem = $(try { [string] $Excel.OperatingSystem } catch { '' })
    if ($operatingSystem -match '(?i)64[ -]?bit') { return '64-bit' }
    if ($operatingSystem -match '(?i)32[ -]?bit') { return '32-bit' }
    return 'unknown'
}

function Get-Range(
    [object] $Worksheet,
    [object] $Cells,
    [int] $FirstRow,
    [int] $FirstColumn,
    [int] $LastRow,
    [int] $LastColumn
) {
    $topLeft = $null
    $bottomRight = $null
    try {
        $topLeft = $Cells.Item($FirstRow, $FirstColumn)
        $bottomRight = $Cells.Item($LastRow, $LastColumn)
        return ,$Worksheet.Range($topLeft, $bottomRight)
    }
    finally {
        Release-Com $bottomRight
        Release-Com $topLeft
    }
}

function Array-Value([object] $Values, [int] $Row, [int] $Column) {
    if ($Values -is [Array]) {
        return $Values.GetValue(
            $Values.GetLowerBound(0) + $Row,
            $Values.GetLowerBound(1) + $Column
        )
    }
    if ($Row -ne 0 -or $Column -ne 0) {
        throw 'Collapsed scalar Value2 requested with nonzero index'
    }
    return $Values
}

function New-ResultRecord([object] $Value, [object] $ErrorType) {
    $errorText = if ($null -eq $ErrorType) { '' } else { [string] $ErrorType }
    if (-not [string]::IsNullOrWhiteSpace($errorText)) {
        $errorNumber = [int] ([double] $ErrorType)
        $errorNames = @{
            1 = '#NULL!'
            2 = '#DIV/0!'
            3 = '#VALUE!'
            4 = '#REF!'
            5 = '#NAME?'
            6 = '#NUM!'
            7 = '#N/A'
            8 = '#GETTING_DATA'
        }
        if (-not $errorNames.ContainsKey($errorNumber)) {
            throw "Unexpected ERROR.TYPE result $errorNumber"
        }
        return [ordered]@{
            kind = 'error'
            error_type = $errorNumber
            error = [string] $errorNames[$errorNumber]
            value2_code = $(if ($null -eq $Value) { $null } else { [double] $Value })
        }
    }

    if ($null -eq $Value -or $Value -is [DBNull]) {
        return [ordered]@{ kind = 'blank' }
    }
    if ($Value -is [bool]) {
        return [ordered]@{ kind = 'boolean'; value = [bool] $Value }
    }
    if ($Value -is [string]) {
        return [ordered]@{ kind = 'text'; value = [string] $Value }
    }
    if ($Value -is [ValueType]) {
        $number = [double] $Value
        return [ordered]@{ kind = 'number'; bits = Get-BitsHex $number }
    }
    throw "Unsupported Excel Value2 result type $($Value.GetType().FullName)"
}

$batchItem = Get-Item -LiteralPath $Batch -ErrorAction Stop
$actualBatchHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $batchItem.FullName).Hash.ToUpperInvariant()
$expectedHash = $ExpectedBatchSha256.ToUpperInvariant()
if ($expectedHash -notmatch '^[0-9A-F]{64}$') {
    throw 'ExpectedBatchSha256 must be exactly 64 hexadecimal digits'
}
Assert-Equal 'frozen batch SHA256' $actualBatchHash $expectedHash

$document = Get-Content -LiteralPath $batchItem.FullName -Raw | ConvertFrom-Json
Assert-Equal 'schema_version' ([string] $document.schema_version) $script:ExpectedSchema
Assert-Equal 'function' ([string] $document.function) $script:ExpectedFunction
Assert-Equal 'row_id' ([string] $document.row_id) $script:ExpectedRowId
Assert-Equal 'source function' ([string] $document.source_discovery.function) 'IRR'
Assert-Equal 'source row_id' ([string] $document.source_discovery.row_id) $script:ExpectedSourceRowId
Assert-Equal 'source probe_count' ([int] $document.source_discovery.probe_count) $script:ExpectedSourceCount
Assert-Equal 'source SHA256' ([string] $document.source_discovery.sha256) $script:ExpectedSourceSha256
Assert-Equal 'source path' ([string] $document.source_discovery.path) $script:ExpectedSourcePath
Assert-Equal 'point h bits' ([string] $document.point_specification.h_magnitude_bits) $script:ExpectedHBits
Assert-Equal 'capture input plumbing' ([string] $document.capture_contract.input_plumbing) 'exact binary64 Range.Value2 matrix and R1C1 cell references'

# Reopen the declared 300-row discovery source, verify its frozen hash, and
# build an exact bit-keyed alignment map.  The held-out path is never resolved
# or read by this runner.
$runnerDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $runnerDirectory)
$sourcePath = [IO.Path]::GetFullPath((Join-Path $repositoryRoot ([string] $document.source_discovery.path)))
$sourceItem = Get-Item -LiteralPath $sourcePath -ErrorAction Stop
$actualSourceHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourceItem.FullName).Hash.ToUpperInvariant()
Assert-Equal 'on-disk source SHA256' $actualSourceHash $script:ExpectedSourceSha256
$sourceDocument = Get-Content -LiteralPath $sourceItem.FullName -Raw | ConvertFrom-Json
Assert-Equal 'on-disk source function' ([string] $sourceDocument.function) 'IRR'
Assert-Equal 'on-disk source row_id' ([string] $sourceDocument.row_id) $script:ExpectedSourceRowId
$sourceProbes = @($sourceDocument.probes)
Assert-Equal 'on-disk source probe count' $sourceProbes.Count $script:ExpectedSourceCount
$sourceById = @{}
foreach ($sourceWrapper in $sourceProbes) {
    $sourceProbe = $sourceWrapper.probe
    $sourceId = [string] $sourceProbe.id
    if ([string]::IsNullOrWhiteSpace($sourceId) -or $sourceById.ContainsKey($sourceId)) {
        throw "Missing or duplicate source IRR id '$sourceId'"
    }
    $sourceArgs = @($sourceProbe.args)
    Assert-Equal "$sourceId source arity" $sourceArgs.Count 2
    $sourceCashflowBits = @($sourceArgs[0] | ForEach-Object { [string] $_ })
    if ($sourceCashflowBits.Count -lt 2 -or $sourceCashflowBits.Count -gt 8) {
        throw "$sourceId has unexpected source cash-flow length $($sourceCashflowBits.Count)"
    }
    $sourceById[$sourceId] = [ordered]@{
        c0_bits = $sourceCashflowBits[0]
        tail_bits = [string[]] $sourceCashflowBits[1..($sourceCashflowBits.Count - 1)]
        guess_bits = [string] $sourceArgs[1]
    }
}
Assert-Equal 'unique on-disk source id count' $sourceById.Count $script:ExpectedSourceCount

$probes = @($document.probes)
Assert-Equal 'probe count' $probes.Count $script:ExpectedProbeCount

$ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$sourcePointSets = @{}
$pointCounts = @{ base = 0; v_h_neg = 0; v_h_pos = 0 }
$rows = [Collections.Generic.List[object]]::new()

for ($index = 0; $index -lt $probes.Count; $index++) {
    $probe = $probes[$index].probe
    $id = [string] $probe.id
    $sourceId = [string] $probe.source_irr_id
    $pointClass = [string] $probe.point_class
    if ([string]::IsNullOrWhiteSpace($id) -or -not $ids.Add($id)) {
        throw "Missing or duplicate probe id '$id'"
    }
    if ($pointClass -notin @('base', 'v_h_neg', 'v_h_pos')) {
        throw "$id has unexpected point_class '$pointClass'"
    }
    $pointCounts[$pointClass]++
    if (-not $sourcePointSets.ContainsKey($sourceId)) {
        $sourcePointSets[$sourceId] = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    }
    if (-not $sourcePointSets[$sourceId].Add($pointClass)) {
        throw "$sourceId repeats point_class '$pointClass'"
    }

    $c0Bits = [string] $probe.c0_bits
    $rateBits = [string] $probe.rate_bits
    $guessBits = [string] $probe.guess_bits
    $v0Bits = [string] $probe.v0_bits
    $hBits = [string] $probe.h_bits
    $evaluationVBits = [string] $probe.evaluation_v_bits
    $derivedRateBits = [string] $probe.derived_rate_bits
    $tailBits = @($probe.tail_bits | ForEach-Object { [string] $_ })
    if ($tailBits.Count -lt 1 -or $tailBits.Count -gt 7) {
        throw "$id has unexpected NPV tail length $($tailBits.Count)"
    }
    if (-not $sourceById.ContainsKey($sourceId)) {
        throw "$id names unknown source IRR id '$sourceId'"
    }
    $sourceRow = $sourceById[$sourceId]
    Assert-Equal "$id source c0 alignment" $c0Bits ([string] $sourceRow.c0_bits)
    Assert-Equal "$id source tail length alignment" $tailBits.Count @($sourceRow.tail_bits).Count
    Assert-Equal "$id source tail bits alignment" ($tailBits -join '|') (@($sourceRow.tail_bits) -join '|')
    Assert-Equal "$id source guess alignment" $guessBits ([string] $sourceRow.guess_bits)

    $c0 = From-BitsHex $c0Bits
    $rate = From-BitsHex $rateBits
    $guess = From-BitsHex $guessBits
    $h = From-BitsHex $hBits
    $tail = @($tailBits | ForEach-Object { From-BitsHex $_ })
    if (-not [double]::IsFinite($c0) -or -not [double]::IsFinite($rate) -or
        -not [double]::IsFinite($guess) -or -not [double]::IsFinite($h) -or
        @($tail | Where-Object { -not [double]::IsFinite($_) }).Count -ne 0) {
        throw "$id contains a non-finite worksheet argument"
    }
    if ($rate -le -1.0 -or $guess -le -1.0) {
        throw "$id contains a rate outside the frozen finite domain"
    }

    $expectedHBits = switch ($pointClass) {
        'base' { '0x0000000000000000' }
        'v_h_neg' { '0xbf50624de0000000' }
        'v_h_pos' { '0x3f50624de0000000' }
    }
    Assert-Equal "$id h bits" $hBits $expectedHBits

    # Independently replay every derived-bit field before Excel is created.
    $v0 = 1.0 / (1.0 + $guess)
    $evaluationV = $v0 + $h
    $derivedRate = (1.0 / $evaluationV) - 1.0
    Assert-Equal "$id v0 bits" $v0Bits (Get-BitsHex $v0)
    Assert-Equal "$id evaluation_v bits" $evaluationVBits (Get-BitsHex $evaluationV)
    Assert-Equal "$id derived_rate bits" $derivedRateBits (Get-BitsHex $derivedRate)
    Assert-Equal "$id rate bits" $rateBits $(if ($pointClass -eq 'base') { $guessBits } else { $derivedRateBits })

    [void] $rows.Add([pscustomobject]@{
        original_index = $index
        id = $id
        source_irr_id = $sourceId
        point_class = $pointClass
        c0_bits = $c0Bits
        c0 = $c0
        rate_bits = $rateBits
        rate = $rate
        tail_bits = [string[]] $tailBits
        tail = [double[]] $tail
        tail_count = $tailBits.Count
        guess_bits = $guessBits
        v0_bits = $v0Bits
        h_bits = $hBits
        evaluation_v_bits = $evaluationVBits
        derived_rate_bits = $derivedRateBits
    })
}

Assert-Equal 'unique id count' $ids.Count $script:ExpectedProbeCount
Assert-Equal 'source id count' $sourcePointSets.Count $script:ExpectedSourceCount
foreach ($entry in $sourcePointSets.GetEnumerator()) {
    Assert-Equal "$($entry.Key) point count" $entry.Value.Count 3
    foreach ($pointClass in @('base', 'v_h_neg', 'v_h_pos')) {
        if (-not $entry.Value.Contains($pointClass)) {
            throw "$($entry.Key) is missing point_class '$pointClass'"
        }
    }
}
foreach ($pointClass in @('base', 'v_h_neg', 'v_h_pos')) {
    Assert-Equal "$pointClass count" ([int] $pointCounts[$pointClass]) $script:ExpectedSourceCount
}
if (Test-Path -LiteralPath $Out) {
    throw "Refusing to overwrite existing capture: $Out"
}

# Sorting by tail length makes each NPV arity a contiguous shared-formula fill.
$executionRows = @($rows | Sort-Object tail_count, original_index)
$inputMatrix = New-Object 'object[,]' $executionRows.Count, 9
for ($rowIndex = 0; $rowIndex -lt $executionRows.Count; $rowIndex++) {
    $row = $executionRows[$rowIndex]
    $inputMatrix[$rowIndex, 0] = $row.c0
    $inputMatrix[$rowIndex, 1] = $row.rate
    for ($tailIndex = 0; $tailIndex -lt $row.tail_count; $tailIndex++) {
        $inputMatrix[$rowIndex, (2 + $tailIndex)] = $row.tail[$tailIndex]
    }
}

if ($ValidateOnly) {
    [ordered]@{
        validation = 'offline_only'
        batch_path = $batchItem.FullName
        batch_bytes = $batchItem.Length
        batch_sha256 = $actualBatchHash
        source_path = $sourceItem.FullName
        source_bytes = $sourceItem.Length
        source_sha256 = $actualSourceHash
        probe_count = $rows.Count
        unique_id_count = $ids.Count
        source_id_count = $sourcePointSets.Count
        point_counts = $pointCounts
        tail_lengths = @($rows.tail_count | Sort-Object -Unique)
        heldout_opened = $false
        excel_launched = $false
    } | ConvertTo-Json -Depth 6
    return
}

$before = @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue).Count
if ($before -ne 0) {
    throw "Serialized COM lane is not free: EXCEL_PROCESS_COUNT_BEFORE=$before"
}

$excel = $null
$workbook = $null
$worksheet = $null
$cells = $null
$inputRange = $null
$resultRange = $null
$errorRange = $null
$environment = $null
$resultValues = $null
$errorValues = $null
$argumentReadback = $null
$formulaTemplates = [Collections.Generic.List[object]]::new()
$captureError = $null

try {
    try {
        $excel = New-Object -ComObject Excel.Application
        $excel.Visible = $false
        $excel.DisplayAlerts = $false
        $excel.ScreenUpdating = $false
        $excel.EnableEvents = $false
        $workbook = $excel.Workbooks.Add()
        $workbook.PrecisionAsDisplayed = $false
        $excel.Calculation = $script:XlCalculationManual
        $worksheet = $workbook.Worksheets.Item(1)
        $cells = $worksheet.Cells

        $environment = [ordered]@{
            excel_version = [string] $excel.Version
            excel_build = $(try { [string] $excel.Build } catch { $null })
            excel_bitness = Get-ExcelBitness $excel
            workbook_compatibility = $(try { [string] $workbook.CompatibilityVersion } catch { 'unknown' })
            excel_operating_system = $(try { [string] $excel.OperatingSystem } catch { 'unknown' })
            excel_input_plumbing = 'cell_value2_matrix_multioutput_r1c1'
            calculation = 'manual_then_single_application_calculate'
        }
        Assert-Equal 'Excel version' $environment.excel_version '16.0'
        Assert-Equal 'Excel build' $environment.excel_build '20228'
        Assert-Equal 'Excel bitness' $environment.excel_bitness '64-bit'
        Assert-Equal 'Workbook CompatibilityVersion' $environment.workbook_compatibility '2'

        $inputRange = Get-Range $worksheet $cells 1 1 $executionRows.Count 9
        $inputRange.Value2 = $inputMatrix
        $argumentReadback = $inputRange.Value2

        # Verify all populated argument cells bit-for-bit before formula entry.
        for ($rowIndex = 0; $rowIndex -lt $executionRows.Count; $rowIndex++) {
            $row = $executionRows[$rowIndex]
            Assert-Equal "$($row.id) c0 Value2 readback" (Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex 0))) $row.c0_bits
            Assert-Equal "$($row.id) rate Value2 readback" (Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex 1))) $row.rate_bits
            for ($tailIndex = 0; $tailIndex -lt $row.tail_count; $tailIndex++) {
                Assert-Equal "$($row.id) tail[$tailIndex] Value2 readback" (Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex (2 + $tailIndex)))) $row.tail_bits[$tailIndex]
            }
        }

        # Columns A:I are inputs, J is intentionally blank, K:M are the three
        # captured objectives, and N:P are their ERROR.TYPE companions.
        foreach ($tailCount in 1..7) {
            $matching = @(
                for ($rowIndex = 0; $rowIndex -lt $executionRows.Count; $rowIndex++) {
                    if ($executionRows[$rowIndex].tail_count -eq $tailCount) { $rowIndex }
                }
            )
            if ($matching.Count -eq 0) { continue }
            $firstRow = $matching[0] + 1
            $lastRow = $matching[-1] + 1
            Assert-Equal "tail length $tailCount contiguous count" ($lastRow - $firstRow + 1) $matching.Count

            # K: rate is B (RC[-9]); tail begins C (RC[-8]).
            $rawTailEndOffset = $tailCount - 9
            $rawFormula = "=NPV(RC[-9],RC[-8]:RC[$rawTailEndOffset])"
            # L: rate is B (RC[-10]); tail begins C (RC[-9]); c0 is A (RC[-11]).
            $directTailEndOffset = $tailCount - 10
            $directFormula = "=NPV(RC[-10],RC[-9]:RC[$directTailEndOffset])+RC[-11]"
            [void] $formulaTemplates.Add([ordered]@{
                tail_count = $tailCount
                first_row = $firstRow
                last_row = $lastRow
                raw_formula_r1c1 = $rawFormula
                direct_composed_formula_r1c1 = $directFormula
            })

            $formulaRange = $null
            try {
                $formulaRange = Get-Range $worksheet $cells $firstRow 11 $lastRow 11
                $formulaRange.Formula2R1C1 = $rawFormula
            }
            finally {
                Release-Com $formulaRange
            }
            try {
                $formulaRange = Get-Range $worksheet $cells $firstRow 12 $lastRow 12
                $formulaRange.Formula2R1C1 = $directFormula
            }
            finally {
                Release-Com $formulaRange
                $formulaRange = $null
            }
        }

        $formulaRange = $null
        try {
            # M: raw result K is RC[-2], c0 A is RC[-12].
            $formulaRange = Get-Range $worksheet $cells 1 13 $executionRows.Count 13
            $formulaRange.Formula2R1C1 = '=RC[-2]+RC[-12]'
        }
        finally {
            Release-Com $formulaRange
        }
        try {
            # N:P each point three columns left to its own result in K:M.
            $formulaRange = Get-Range $worksheet $cells 1 14 $executionRows.Count 16
            $formulaRange.Formula2R1C1 = '=IF(ISERROR(RC[-3]),ERROR.TYPE(RC[-3]),"")'
        }
        finally {
            Release-Com $formulaRange
            $formulaRange = $null
        }

        [void] $excel.Calculate()
        $resultRange = Get-Range $worksheet $cells 1 11 $executionRows.Count 13
        $errorRange = Get-Range $worksheet $cells 1 14 $executionRows.Count 16
        $resultValues = $resultRange.Value2
        $errorValues = $errorRange.Value2
    }
    catch {
        $captureError = $_
    }
}
finally {
    if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
    if ($null -ne $excel) { try { $excel.Quit() } catch {} }
    Release-Com $errorRange
    Release-Com $resultRange
    Release-Com $inputRange
    Release-Com $cells
    Release-Com $worksheet
    Release-Com $workbook
    Release-Com $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$after = -1
for ($poll = 0; $poll -lt 20; $poll++) {
    $after = @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue).Count
    if ($after -eq 0) { break }
    Start-Sleep -Milliseconds 200
}
if ($after -ne 0) {
    throw "Excel did not exit cleanly: EXCEL_PROCESS_COUNT_AFTER=$after"
}
if ($null -ne $captureError) {
    throw $captureError
}

$capturedById = @{}
for ($rowIndex = 0; $rowIndex -lt $executionRows.Count; $rowIndex++) {
    $row = $executionRows[$rowIndex]
    $capturedById[$row.id] = [ordered]@{
        probe = [ordered]@{
            id = $row.id
            source_irr_id = $row.source_irr_id
            point_class = $row.point_class
            c0_bits = $row.c0_bits
            tail_bits = $row.tail_bits
            rate_bits = $row.rate_bits
            guess_bits = $row.guess_bits
            v0_bits = $row.v0_bits
            h_bits = $row.h_bits
            evaluation_v_bits = $row.evaluation_v_bits
            derived_rate_bits = $row.derived_rate_bits
        }
        argument_value2_readback = [ordered]@{
            c0_bits = Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex 0))
            rate_bits = Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex 1))
            tail_bits = @(
                for ($tailIndex = 0; $tailIndex -lt $row.tail_count; $tailIndex++) {
                    Get-BitsHex ([double] (Array-Value $argumentReadback $rowIndex (2 + $tailIndex)))
                }
            )
        }
        raw_npv = New-ResultRecord (Array-Value $resultValues $rowIndex 0) (Array-Value $errorValues $rowIndex 0)
        direct_composed = New-ResultRecord (Array-Value $resultValues $rowIndex 1) (Array-Value $errorValues $rowIndex 1)
        cell_composed = New-ResultRecord (Array-Value $resultValues $rowIndex 2) (Array-Value $errorValues $rowIndex 2)
    }
}

$records = @(
    foreach ($row in $rows) {
        if (-not $capturedById.ContainsKey($row.id)) {
            throw "Missing captured result for $($row.id)"
        }
        $capturedById[$row.id]
    }
)
Assert-Equal 'captured result count' $records.Count $script:ExpectedProbeCount

$runnerItem = Get-Item -LiteralPath $MyInvocation.MyCommand.Path
$artifact = [ordered]@{
    schema_version = 'w109.irr.npv_objective_companion.answers.v1'
    function = $script:ExpectedFunction
    row_id = $script:ExpectedRowId
    capture_provenance = [ordered]@{
        schema_version = 'w109-capture-provenance-v1'
        captured_utc = [DateTime]::UtcNow.ToString('o')
        excel_process_count_before = $before
        excel_process_count_after = $after
        environment = $environment
        oracle_cache = [ordered]@{ mode = 'no_cache'; hits = 0; misses = 0 }
        batch = [ordered]@{
            path = $batchItem.FullName
            bytes = $batchItem.Length
            sha256 = $actualBatchHash
            probe_count = $script:ExpectedProbeCount
            unique_id_count = $ids.Count
        }
        source_discovery = $document.source_discovery
        alignment = [ordered]@{
            verified_before_excel = $true
            source_id_count = $sourcePointSets.Count
            points_per_source = 3
            point_counts = $pointCounts
            argument_value2_readback_verified = $true
        }
        runner = [ordered]@{
            name = $runnerItem.Name
            version = $script:RunnerVersion
            path = $runnerItem.FullName
            sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $runnerItem.FullName).Hash.ToUpperInvariant()
            powershell_version = $PSVersionTable.PSVersion.ToString()
        }
        formulas = [ordered]@{
            formula_style = 'Formula2R1C1 shared fills; arguments only by cell reference'
            raw_and_direct_by_tail_count = $formulaTemplates
            cell_composed_formula_r1c1 = '=RC[-2]+RC[-12]'
            error_type_formula_r1c1 = '=IF(ISERROR(RC[-3]),ERROR.TYPE(RC[-3]),"")'
            calculate_calls = 1
        }
    }
    probes = $records
}

$parent = Split-Path -Parent $Out
if (-not [string]::IsNullOrWhiteSpace($parent)) {
    [void] (New-Item -ItemType Directory -Force -Path $parent)
}
$artifact | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $Out -Encoding utf8
$artifact | ConvertTo-Json -Depth 5

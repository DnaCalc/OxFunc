# CellRefBatch.psm1
#
# Shared cell-ref Excel comparator plumbing for OxFunc smart-fuzzer
# runners. Numeric inputs are written to worksheet cells via
# Range.Value2 (bit-exact f64 round-trip) and the formula in column A
# references those cells. This eliminates the
# expected_formula_literal_encoding_drift class.
#
# See smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md for the full
# rule and the empirical witness chain.
#
# Owning workset: W097.
#
# Public API:
#   Invoke-ExcelCellRefBatch -Candidates <object[]>
#     Each candidate is an object/hashtable with:
#       function_name : string   (Excel function name, no leading "=")
#       args          : object[] (scalar f64s, or @{kind="matrix"; values=2D-array})
#       expected      : optional (passed through untouched, for client use)
#     Returns [ordered] @{ blocked; blocker?; outcomes; environment }.
#
#   Get-F64BitsHex -Value <double>
#     Returns "0x{16-hex-digits}" for the f64 bit pattern.
#
#   ConvertTo-ExcelOutcome -Value <object> -ErrorType <object>
#     Maps a (Range.Value2, ERROR.TYPE) pair to a typed outcome
#     [ordered] @{ kind; ...; digest_payload }.
#
#   Get-UlpDistance -A <double> -B <double>
#     Approximate ULP distance for two finite f64s.
#
# Private helpers are not exported.

Set-StrictMode -Version Latest

function Get-F64BitsHex {
    param([Parameter(Mandatory)] [double] $Value)
    $bits = [System.BitConverter]::ToUInt64([System.BitConverter]::GetBytes($Value), 0)
    return ("0x{0:x16}" -f $bits)
}

function _Release-ComObject {
    param([object] $Object)
    if ($null -ne $Object -and [System.Runtime.InteropServices.Marshal]::IsComObject($Object)) {
        [void] [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Object)
    }
}

function _Get-ArrayCellValue {
    param([object] $Values, [int] $RowIndex, [int] $ColIndex = 0)
    if ($Values -is [System.Array]) {
        $lower0 = $Values.GetLowerBound(0)
        $lower1 = $Values.GetLowerBound(1)
        return $Values.GetValue($lower0 + $RowIndex, $lower1 + $ColIndex)
    }
    return $Values
}

function _Convert-ExcelErrorTypeToCode {
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

function ConvertTo-ExcelOutcome {
    param([object] $Value, [object] $ErrorType)
    if ($null -ne $ErrorType -and -not [string]::IsNullOrWhiteSpace([string] $ErrorType)) {
        $code = _Convert-ExcelErrorTypeToCode ([int] [double] $ErrorType)
        return [ordered]@{ kind = "error"; code = $code; digest_payload = "error:$code" }
    }
    if ($Value -is [double] -or $Value -is [single] -or $Value -is [decimal] -or $Value -is [int]) {
        $value = [double] $Value
        $bits = Get-F64BitsHex $value
        return [ordered]@{ kind = "number"; value = $value; bits_hex = $bits; digest_payload = "number:$bits" }
    }
    if ($Value -is [bool]) {
        return [ordered]@{ kind = "logical"; value = [bool] $Value; digest_payload = "logical:$Value" }
    }
    $text = [string] $Value
    return [ordered]@{ kind = "text"; value = $text; digest_payload = "text:$text" }
}

function Get-UlpDistance {
    param([Parameter(Mandatory)] [double] $A, [Parameter(Mandatory)] [double] $B)
    if ([double]::IsNaN($A) -or [double]::IsNaN($B)) { return [double]::PositiveInfinity }
    if ([double]::IsInfinity($A) -or [double]::IsInfinity($B)) { return [double]::PositiveInfinity }
    $aBits = [System.BitConverter]::DoubleToInt64Bits($A)
    $bBits = [System.BitConverter]::DoubleToInt64Bits($B)
    if ($aBits -lt 0) { $aBits = [int64]9223372036854775807 - $aBits }
    if ($bBits -lt 0) { $bBits = [int64]9223372036854775807 - $bBits }
    $diff = $aBits - $bBits
    if ($diff -lt 0) { $diff = -$diff }
    return [double] $diff
}

function _Get-CandidateColumnLetter {
    param([int] $ZeroBasedColIndex)
    if ($ZeroBasedColIndex -lt 26) {
        return [char]([byte][char]'A' + $ZeroBasedColIndex)
    }
    $hi = [int]([Math]::Floor($ZeroBasedColIndex / 26)) - 1
    $lo = $ZeroBasedColIndex % 26
    return ([char]([byte][char]'A' + $hi)) + ([char]([byte][char]'A' + $lo))
}

function _Get-ScalarArgs {
    # Returns the candidate's argument list as a flat object[] of doubles
    # for scalar candidates. For matrix candidates returns $null (caller
    # must take the matrix branch). Wraps in @(...) to guarantee a
    # collection even for single-arg candidates.
    param([object] $Candidate)
    $argList = @($Candidate.args)
    foreach ($a in $argList) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
            return $null
        }
    }
    return ,$argList
}

function _Get-MaxScalarArity {
    param([object[]] $Candidates)
    $maxArity = 0
    foreach ($c in $Candidates) {
        $argList = _Get-ScalarArgs $c
        if ($null -eq $argList) { continue }
        $n = @($argList).Count
        if ($n -gt $maxArity) { $maxArity = $n }
    }
    return [Math]::Max(1, $maxArity)
}

function _New-ScalarArgValueArray {
    param([object[]] $Candidates, [int] $MaxArity, [int] $StartRow)
    $array = New-Object "object[,]" $Candidates.Count, $MaxArity
    for ($row = 0; $row -lt $Candidates.Count; $row++) {
        $argList = _Get-ScalarArgs $Candidates[$row]
        if ($null -eq $argList) {
            for ($col = 0; $col -lt $MaxArity; $col++) { $array[$row, $col] = $null }
            continue
        }
        $argList = @($argList)
        for ($col = 0; $col -lt $MaxArity; $col++) {
            if ($col -lt $argList.Count) {
                $a = $argList[$col]
                if ($a -is [bool]) {
                    $array[$row, $col] = [bool] $a
                } else {
                    $array[$row, $col] = [double] $a
                }
            } else {
                $array[$row, $col] = $null
            }
        }
    }
    return ,$array
}

function _New-ScalarFormulaArray {
    param([object[]] $Candidates, [int] $ArgStartColZeroIndexed)
    $array = New-Object "object[,]" $Candidates.Count, 1
    for ($row = 0; $row -lt $Candidates.Count; $row++) {
        $cand = $Candidates[$row]
        $argList = _Get-ScalarArgs $cand
        if ($null -eq $argList) { $array[$row, 0] = $null; continue }
        $argList = @($argList)
        $excelRow = $row + 1
        $name = [string] $cand.function_name
        $refs = @()
        for ($col = 0; $col -lt $argList.Count; $col++) {
            $letter = _Get-CandidateColumnLetter ($ArgStartColZeroIndexed + $col)
            $refs += "$letter$excelRow"
        }
        $array[$row, 0] = "=$name($($refs -join ','))"
    }
    return ,$array
}

function _New-ErrorTypeFormulaArray {
    param([int] $Count)
    $array = New-Object "object[,]" $Count, 1
    for ($row = 0; $row -lt $Count; $row++) {
        $excelRow = $row + 1
        $array[$row, 0] = "=IF(ISERROR(A$excelRow),ERROR.TYPE(A$excelRow),"""")"
    }
    return ,$array
}

# Range placement for matrix candidates. Each matrix-bearing candidate
# gets a dedicated cell block whose top-left is fixed for that candidate;
# the formula references the block. Matrix blocks live to the right of
# the per-row scalar arg cells, starting at column M (column index 12).
# This keeps scalar plumbing for non-matrix candidates undisturbed.
$script:MatrixStartColZeroIndexed = 12

function _Has-MatrixArg {
    param([object] $Candidate)
    foreach ($a in @($Candidate.args)) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") { return $true }
    }
    return $false
}

function _New-MatrixFormula {
    # Builds an Excel formula for a matrix-bearing candidate. Each
    # matrix arg is referenced as a range; scalar args are referenced
    # as a single cell. The function call is wrapped in INDEX(...,r,c)
    # so the cell answer is always a scalar — this keeps array-returning
    # functions like MINVERSE from spilling into adjacent helper cells.
    # The caller can pin (r, c) via $Candidate.result_index = @(r, c)
    # (1-based); default is (1, 1).
    param([object] $Candidate, [int] $ExcelRow)
    $name = [string] $Candidate.function_name
    $refs = @()
    $colCursor = $script:MatrixStartColZeroIndexed
    foreach ($a in @($Candidate.args)) {
        if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
            $rows = [int]$a.rows
            $cols = [int]$a.cols
            $topLeft = (_Get-CandidateColumnLetter $colCursor) + $ExcelRow
            $bottomRight = (_Get-CandidateColumnLetter ($colCursor + $cols - 1)) + ($ExcelRow + $rows - 1)
            $refs += "$topLeft`:$bottomRight"
            $colCursor += $cols + 1   # leave a gap column between matrices
        } else {
            # Inline scalar passed alongside a matrix: write into the
            # matrix lane as a 1x1 block.
            $topLeft = (_Get-CandidateColumnLetter $colCursor) + $ExcelRow
            $refs += $topLeft
            $colCursor += 2
        }
    }
    $resultIndex = @(1, 1)
    $hasResultIndex = $false
    if ($Candidate -is [System.Collections.IDictionary]) {
        $hasResultIndex = $Candidate.Contains("result_index")
    } else {
        $hasResultIndex = ($Candidate.PSObject.Properties.Name -contains "result_index")
    }
    if ($hasResultIndex -and $null -ne $Candidate.result_index) {
        $ri = @($Candidate.result_index)
        if ($ri.Count -ge 2) { $resultIndex = @([int]$ri[0], [int]$ri[1]) }
    }
    $callExpr = "$name($($refs -join ','))"
    return "=INDEX($callExpr,$($resultIndex[0]),$($resultIndex[1]))"
}

function _Write-MatrixCells {
    param([object] $Worksheet, [object[]] $Candidates, [int] $RowOffset = 0)
    # Each matrix-bearing candidate occupies multiple Excel rows. We map
    # candidate row index -> Excel row block by allocating per-candidate
    # row blocks based on max matrix row count among that candidate's
    # matrix args. The function returns a parallel array of (excelRow,
    # excelRowSpan) per candidate so the formula and outcome array can
    # use the right anchors.
    $blocks = @()
    $cursor = 1 + $RowOffset
    for ($r = 0; $r -lt $Candidates.Count; $r++) {
        $cand = $Candidates[$r]
        $rowSpan = 1
        foreach ($a in @($cand.args)) {
            if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
                if ([int]$a.rows -gt $rowSpan) { $rowSpan = [int]$a.rows }
            }
        }
        $blocks += [PSCustomObject]@{ excelRow = $cursor; rowSpan = $rowSpan }
        # Now actually write each matrix arg's cells via Value2.
        $colCursor = $script:MatrixStartColZeroIndexed
        foreach ($a in @($cand.args)) {
            if ($a -is [System.Collections.IDictionary] -and $a.Contains("kind") -and $a.kind -eq "matrix") {
                $rows = [int]$a.rows
                $cols = [int]$a.cols
                $vals = $a.values
                $array = New-Object "object[,]" $rows, $cols
                # Tolerate three input shapes for $vals:
                #   (i)   jagged @(@(...),@(...))
                #   (ii)  flat @(...) of length rows*cols (row-major)
                #   (iii) 2D object[,] of shape [rows,cols]
                if ($vals -is [System.Array] -and $vals.Rank -eq 2) {
                    for ($i = 0; $i -lt $rows; $i++) {
                        for ($j = 0; $j -lt $cols; $j++) {
                            $array[$i, $j] = [double] $vals[$i, $j]
                        }
                    }
                } elseif ($vals.Count -eq $rows -and ($vals[0] -is [System.Array] -or $vals[0] -is [System.Collections.IList])) {
                    for ($i = 0; $i -lt $rows; $i++) {
                        $rowVals = @($vals[$i])
                        for ($j = 0; $j -lt $cols; $j++) {
                            $array[$i, $j] = [double] $rowVals[$j]
                        }
                    }
                } else {
                    # Flat row-major.
                    $flat = @($vals)
                    for ($i = 0; $i -lt $rows; $i++) {
                        for ($j = 0; $j -lt $cols; $j++) {
                            $array[$i, $j] = [double] $flat[$i * $cols + $j]
                        }
                    }
                }
                $anchor = $Worksheet.Range((_Get-CandidateColumnLetter $colCursor) + $cursor)
                $range = $anchor.Resize($rows, $cols)
                _Release-ComObject $anchor
                $range.Value2 = $array
                _Release-ComObject $range
                $colCursor += $cols + 1
            } else {
                # Scalar passed alongside a matrix: write 1x1
                $anchor = $Worksheet.Range((_Get-CandidateColumnLetter $colCursor) + $cursor)
                $anchor.Value2 = [double] $a
                _Release-ComObject $anchor
                $colCursor += 2
            }
        }
        $cursor += $rowSpan + 1   # leave a gap row between candidate blocks
    }
    return ,$blocks
}

function Invoke-ExcelCellRefBatch {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)] [object[]] $Candidates
    )
    if ($Candidates.Count -eq 0) {
        return [ordered]@{ blocked = $false; outcomes = @(); environment = @{ excel_input_plumbing = "cell_value2" } }
    }
    $hasMatrix = $false
    foreach ($c in $Candidates) { if (_Has-MatrixArg $c) { $hasMatrix = $true; break } }
    if ($hasMatrix) {
        return _Invoke-ExcelCellRefMatrixBatch -Candidates $Candidates
    } else {
        return _Invoke-ExcelCellRefScalarBatch -Candidates $Candidates
    }
}

function _Invoke-ExcelCellRefScalarBatch {
    param([object[]] $Candidates)
    $excel = $null; $workbook = $null; $worksheet = $null
    $formulaRange = $null; $errorRange = $null; $argRange = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        try { $excel.Visible = $false } catch {}
        try { $excel.DisplayAlerts = $false } catch {}
        try { $excel.ScreenUpdating = $false } catch {}
        try { $excel.EnableEvents = $false } catch {}
        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)
        try { $worksheet.Columns.Item(1).ColumnWidth = 36 } catch {}

        $maxArity = _Get-MaxScalarArity $Candidates
        $argStartColZeroIndexed = 2  # A=formula, B=errorType, C..=args

        $argAnchor = $worksheet.Range((_Get-CandidateColumnLetter $argStartColZeroIndexed) + "1")
        $argRange = $argAnchor.Resize($Candidates.Count, $maxArity)
        _Release-ComObject $argAnchor
        $argRange.Value2 = _New-ScalarArgValueArray $Candidates $maxArity 1

        $formulaAnchor = $worksheet.Range("A1")
        $formulaRange = $formulaAnchor.Resize($Candidates.Count, 1)
        _Release-ComObject $formulaAnchor
        $errorAnchor = $worksheet.Range("B1")
        $errorRange = $errorAnchor.Resize($Candidates.Count, 1)
        _Release-ComObject $errorAnchor
        $formulaRange.Formula2 = _New-ScalarFormulaArray $Candidates $argStartColZeroIndexed
        $errorRange.Formula2 = _New-ErrorTypeFormulaArray $Candidates.Count

        $calcAnchor = $worksheet.Range("A1")
        $calcRange = $calcAnchor.Resize($Candidates.Count, 2)
        _Release-ComObject $calcAnchor
        [void] $calcRange.Calculate()
        _Release-ComObject $calcRange

        $values = $formulaRange.Value2
        $errorValues = $errorRange.Value2
        $outcomes = New-Object System.Collections.ArrayList
        for ($row = 0; $row -lt $Candidates.Count; $row++) {
            $value = _Get-ArrayCellValue $values $row 0
            $errorType = _Get-ArrayCellValue $errorValues $row 0
            $outcomes.Add((ConvertTo-ExcelOutcome $value $errorType)) | Out-Null
        }
        return [ordered]@{
            blocked = $false
            outcomes = @($outcomes)
            environment = [ordered]@{
                excel_version = [string] $excel.Version
                excel_build = $(try { [string] $excel.Build } catch { $null })
                workbook_compatibility = $(try { [string] $workbook.CompatibilityVersion } catch { "unknown" })
                excel_input_plumbing = "cell_value2"
            }
        }
    }
    catch {
        return [ordered]@{ blocked = $true; blocker = $_.Exception.Message; outcomes = @(); environment = @{ excel_input_plumbing = "cell_value2" } }
    }
    finally {
        _Release-ComObject $formulaRange
        _Release-ComObject $errorRange
        _Release-ComObject $argRange
        if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
        if ($null -ne $excel) { try { $excel.Quit() } catch {} }
        _Release-ComObject $worksheet
        _Release-ComObject $workbook
        _Release-ComObject $excel
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
    }
}

function _Invoke-ExcelCellRefMatrixBatch {
    # Matrix-aware batch. Each candidate occupies a per-candidate Excel
    # row block (row span = max matrix rows). The formula for candidate
    # k goes in cell A<excelRowOfBlock>, the error-type companion goes
    # in B<excelRowOfBlock>. Matrix arg cells are written via Value2.
    param([object[]] $Candidates)
    $excel = $null; $workbook = $null; $worksheet = $null
    try {
        $excel = New-Object -ComObject Excel.Application
        try { $excel.Visible = $false } catch {}
        try { $excel.DisplayAlerts = $false } catch {}
        try { $excel.ScreenUpdating = $false } catch {}
        try { $excel.EnableEvents = $false } catch {}
        $workbook = $excel.Workbooks.Add()
        $worksheet = $workbook.Worksheets.Item(1)

        $blocks = _Write-MatrixCells -Worksheet $worksheet -Candidates $Candidates -RowOffset 0

        $outcomes = New-Object System.Collections.ArrayList
        for ($r = 0; $r -lt $Candidates.Count; $r++) {
            $excelRow = $blocks[$r].excelRow
            $cand = $Candidates[$r]
            $formulaText = _New-MatrixFormula -Candidate $cand -ExcelRow $excelRow
            $cellA = $worksheet.Range("A$excelRow")
            $cellB = $worksheet.Range("B$excelRow")
            $cellA.Formula2 = $formulaText
            $cellB.Formula2 = "=IF(ISERROR(A$excelRow),ERROR.TYPE(A$excelRow),"""")"
            [void] $cellA.Calculate()
            [void] $cellB.Calculate()
            $value = $cellA.Value2
            $errorType = $cellB.Value2
            # If the result is an array, take the [0,0] element for
            # bit-exact scalar comparison; clients that need full array
            # semantics can extend the module later.
            if ($value -is [System.Array]) { $value = _Get-ArrayCellValue $value 0 0 }
            $outcomes.Add((ConvertTo-ExcelOutcome $value $errorType)) | Out-Null
            _Release-ComObject $cellA
            _Release-ComObject $cellB
        }
        return [ordered]@{
            blocked = $false
            outcomes = @($outcomes)
            environment = [ordered]@{
                excel_version = [string] $excel.Version
                excel_build = $(try { [string] $excel.Build } catch { $null })
                workbook_compatibility = $(try { [string] $workbook.CompatibilityVersion } catch { "unknown" })
                excel_input_plumbing = "cell_value2_matrix"
            }
        }
    }
    catch {
        return [ordered]@{ blocked = $true; blocker = $_.Exception.Message; outcomes = @(); environment = @{ excel_input_plumbing = "cell_value2_matrix" } }
    }
    finally {
        if ($null -ne $workbook) { try { $workbook.Close($false) } catch {} }
        if ($null -ne $excel) { try { $excel.Quit() } catch {} }
        _Release-ComObject $worksheet
        _Release-ComObject $workbook
        _Release-ComObject $excel
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
    }
}

Export-ModuleMember -Function `
    Invoke-ExcelCellRefBatch, `
    Get-F64BitsHex, `
    ConvertTo-ExcelOutcome, `
    Get-UlpDistance

param(
    [string]$Out = ".tmp/formula-admission-results.csv",
    [string]$ArtifactRoot = "C:/Temp/oxfunc_formula_admission_artifacts"
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.IO.Compression

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

function Add-Row {
    param(
        [System.Collections.Generic.List[object]]$Rows,
        [string]$ScenarioId,
        [string]$Mechanism,
        [string]$InputExpr,
        [string]$Status,
        [string]$ObservedType,
        [string]$ObservedFormula2,
        [string]$ObservedText,
        [string]$ObservedValue,
        [string]$Notes,
        [string]$ExcelVersion,
        [string]$ExcelChannel,
        [string]$CompatVersion
    )

    $Rows.Add([PSCustomObject]@{
        scenario_id = $ScenarioId
        mechanism = $Mechanism
        input_expr = $InputExpr
        status = $Status
        observed_type = $ObservedType
        observed_formula2 = $ObservedFormula2
        observed_text = $ObservedText
        observed_value = $ObservedValue
        excel_version = $ExcelVersion
        excel_channel = $ExcelChannel
        compat_version = $CompatVersion
        locale_profile = "en-US"
        runner_version = "formula-admission-baseline-ps1/0.1.0"
        notes = $Notes
    })
}

function Mutate-WorkbookFormula {
    param(
        [string]$SourcePath,
        [string]$TargetPath,
        [string]$OldFormulaBody,
        [string]$NewFormulaBody
    )

    Copy-Item -Path $SourcePath -Destination $TargetPath -Force

    $fs = [System.IO.File]::Open($TargetPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::ReadWrite)
    try {
        $zip = [System.IO.Compression.ZipArchive]::new($fs, [System.IO.Compression.ZipArchiveMode]::Update, $false)
        try {
            $entry = $zip.GetEntry("xl/worksheets/sheet1.xml")
            if ($null -eq $entry) { throw "sheet1.xml missing in $TargetPath" }

            $entryStream = $entry.Open()
            $reader = [System.IO.StreamReader]::new($entryStream)
            $xml = $reader.ReadToEnd()
            $reader.Close()
            $entryStream.Close()

            $oldToken = "<f>$OldFormulaBody</f>"
            $newToken = "<f>$NewFormulaBody</f>"
            if (-not $xml.Contains($oldToken)) {
                throw "Formula token '$oldToken' not found in workbook XML."
            }
            $xml = $xml.Replace($oldToken, $newToken)

            $entry.Delete()
            $newEntry = $zip.CreateEntry("xl/worksheets/sheet1.xml")
            $newStream = $newEntry.Open()
            $writer = [System.IO.StreamWriter]::new($newStream, [System.Text.UTF8Encoding]::new($false))
            $writer.Write($xml)
            $writer.Flush()
            $writer.Close()
            $newStream.Close()
        }
        finally {
            $zip.Dispose()
        }
    }
    finally {
        $fs.Close()
    }
}

$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Parent $outPath
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$artifactRootPath = [System.IO.Path]::GetFullPath($ArtifactRoot)
if (-not (Test-Path $artifactRootPath)) {
    New-Item -ItemType Directory -Path $artifactRootPath | Out-Null
}

$excel = $null
$workbook = $null
$worksheet = $null
$fileProbeWorkbook = $null
$fileProbeSheet = $null

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

    $rows = New-Object System.Collections.Generic.List[object]

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)
    $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

    $entryCases = @(
        @{ scenario = "FA-ENTRY-001"; method = "Range.Formula"; cell = "A1"; expr = "=PI()" },
        @{ scenario = "FA-ENTRY-002"; method = "Range.Formula"; cell = "A2"; expr = "=PI(123)" },
        @{ scenario = "FA-ENTRY-003"; method = "Range.Formula2"; cell = "A3"; expr = "=PI(123)" },
        @{ scenario = "FA-ENTRY-004"; method = "Range.Formula"; cell = "A4"; expr = "=1+" },
        @{ scenario = "FA-ENTRY-005"; method = "Range.Formula"; cell = "A5"; expr = "=ASIN(2)" },
        @{ scenario = "FA-ENTRY-006"; method = "Range.Formula"; cell = "A6"; expr = "=SIN()" }
    )

    foreach ($case in $entryCases) {
        $setOk = $true
        $setErr = ""
        try {
            if ($case.method -eq "Range.Formula2") {
                $worksheet.Range($case.cell).Formula2 = $case.expr
            }
            else {
                $worksheet.Range($case.cell).Formula = $case.expr
            }
        }
        catch {
            $setOk = $false
            $setErr = [string]$_.Exception.Message
        }

        if ($setOk) {
            $excel.CalculateFull()
            $formula2 = ""
            $text = ""
            $value2 = ""
            try { $formula2 = [string]$worksheet.Range($case.cell).Formula2 } catch { $formula2 = [string]$worksheet.Range($case.cell).Formula }
            try { $text = [string]$worksheet.Range($case.cell).Text } catch { $text = "" }
            try { $value2 = Convert-ValueToString -Value $worksheet.Range($case.cell).Value2 } catch { $value2 = "" }

            Add-Row -Rows $rows -ScenarioId $case.scenario -Mechanism $case.method -InputExpr $case.expr -Status "admitted" -ObservedType "cell" -ObservedFormula2 $formula2 -ObservedText $text -ObservedValue $value2 -Notes "formula accepted and materialized in cell" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
        }
        else {
            Add-Row -Rows $rows -ScenarioId $case.scenario -Mechanism $case.method -InputExpr $case.expr -Status "rejected_on_set" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes $setErr -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
        }
    }

    $evalCases = @(
        @{ scenario = "FA-EVAL-001"; expr = "=PI()" },
        @{ scenario = "FA-EVAL-002"; expr = "=PI(123)" },
        @{ scenario = "FA-EVAL-003"; expr = "=1+" },
        @{ scenario = "FA-EVAL-004"; expr = "=ASIN(2)" },
        @{ scenario = "FA-EVAL-005"; expr = "=SIN()" }
    )

    foreach ($case in $evalCases) {
        foreach ($mechanism in @("Application.Evaluate", "Worksheet.Evaluate")) {
            $ok = $true
            $err = ""
            $valueRaw = $null
            try {
                if ($mechanism -eq "Application.Evaluate") {
                    $valueRaw = $excel.Evaluate($case.expr)
                }
                else {
                    $valueRaw = $worksheet.Evaluate($case.expr)
                }
            }
            catch {
                $ok = $false
                $err = [string]$_.Exception.Message
            }

            if ($ok) {
                $typeName = if ($null -eq $valueRaw) { "null" } else { $valueRaw.GetType().Name }
                $value = Convert-ValueToString -Value $valueRaw
                Add-Row -Rows $rows -ScenarioId "$($case.scenario)-$($mechanism.Replace('.', '-'))" -Mechanism $mechanism -InputExpr $case.expr -Status "returned" -ObservedType $typeName -ObservedFormula2 "" -ObservedText "" -ObservedValue $value -Notes "evaluate returned a marshaled value" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
            }
            else {
                Add-Row -Rows $rows -ScenarioId "$($case.scenario)-$($mechanism.Replace('.', '-'))" -Mechanism $mechanism -InputExpr $case.expr -Status "threw" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes $err -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
            }
        }

        $macroExpr = $case.expr.TrimStart('=')
        $x4Ok = $true
        $x4Err = ""
        $x4Raw = $null
        try {
            $x4Raw = $excel.ExecuteExcel4Macro($macroExpr)
        }
        catch {
            $x4Ok = $false
            $x4Err = [string]$_.Exception.Message
        }

        if ($x4Ok) {
            $x4Type = if ($null -eq $x4Raw) { "null" } else { $x4Raw.GetType().Name }
            $x4Value = Convert-ValueToString -Value $x4Raw
            Add-Row -Rows $rows -ScenarioId "$($case.scenario)-ExecuteExcel4Macro" -Mechanism "ExecuteExcel4Macro" -InputExpr $macroExpr -Status "returned" -ObservedType $x4Type -ObservedFormula2 "" -ObservedText "" -ObservedValue $x4Value -Notes "XLM evaluate-style call returned a marshaled value" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
        }
        else {
            Add-Row -Rows $rows -ScenarioId "$($case.scenario)-ExecuteExcel4Macro" -Mechanism "ExecuteExcel4Macro" -InputExpr $macroExpr -Status "threw" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes $x4Err -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
        }
    }

    try {
        $pi0 = $excel.WorksheetFunction.Pi()
        Add-Row -Rows $rows -ScenarioId "FA-WF-001" -Mechanism "WorksheetFunction.Pi" -InputExpr "Pi()" -Status "returned" -ObservedType ($pi0.GetType().Name) -ObservedFormula2 "" -ObservedText "" -ObservedValue (Convert-ValueToString -Value $pi0) -Notes "direct worksheet-function COM entrypoint" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
    }
    catch {
        Add-Row -Rows $rows -ScenarioId "FA-WF-001" -Mechanism "WorksheetFunction.Pi" -InputExpr "Pi()" -Status "threw" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes ([string]$_.Exception.Message) -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
    }

    try {
        $pi1 = $excel.WorksheetFunction.Pi(123)
        Add-Row -Rows $rows -ScenarioId "FA-WF-002" -Mechanism "WorksheetFunction.Pi" -InputExpr "Pi(123)" -Status "returned" -ObservedType ($pi1.GetType().Name) -ObservedFormula2 "" -ObservedText "" -ObservedValue (Convert-ValueToString -Value $pi1) -Notes "unexpected extra-arg acceptance" -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
    }
    catch {
        Add-Row -Rows $rows -ScenarioId "FA-WF-002" -Mechanism "WorksheetFunction.Pi" -InputExpr "Pi(123)" -Status "threw" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes ([string]$_.Exception.Message) -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatVersion
    }

    $seedPath = Join-Path $artifactRootPath "formula_admission_seed.xlsx"
    $mutValidPath = Join-Path $artifactRootPath "formula_admission_mutated_asin2.xlsx"
    $mutInvalidPath = Join-Path $artifactRootPath "formula_admission_mutated_pi_123.xlsx"

    $fileProbeBook = $excel.Workbooks.Add()
    $fileProbeSheet = $fileProbeBook.Worksheets.Item(1)
    $fileProbeSheet.Range("A1").Formula = "=PI()"
    $fileProbeBook.SaveAs($seedPath, 51)
    $fileProbeBook.Close($false) | Out-Null
    Release-ComObjectSafe -Obj $fileProbeSheet
    Release-ComObjectSafe -Obj $fileProbeBook
    $fileProbeSheet = $null
    $fileProbeBook = $null

    Mutate-WorkbookFormula -SourcePath $seedPath -TargetPath $mutValidPath -OldFormulaBody "PI()" -NewFormulaBody "ASIN(2)"
    Mutate-WorkbookFormula -SourcePath $seedPath -TargetPath $mutInvalidPath -OldFormulaBody "PI()" -NewFormulaBody "PI(123)"

    foreach ($openCase in @(
        @{ scenario = "FA-FILE-001"; path = $mutValidPath; expectation = "valid_formula_in_xml" },
        @{ scenario = "FA-FILE-002"; path = $mutInvalidPath; expectation = "admission_invalid_formula_in_xml" }
    )) {
        $opened = $null
        $sheet = $null
        $compatForRow = ""
        try {
            $opened = $excel.Workbooks.Open($openCase.path)
            $compatForRow = Get-CompatibilityDescriptor -Workbook $opened
            $sheet = $opened.Worksheets.Item(1)
            $formula2 = ""
            $textPre = ""
            $valuePre = ""
            $textPost = ""
            $valuePost = ""
            try { $formula2 = [string]$sheet.Range("A1").Formula2 } catch { $formula2 = [string]$sheet.Range("A1").Formula }
            try { $textPre = [string]$sheet.Range("A1").Text } catch { $textPre = "" }
            try { $valuePre = Convert-ValueToString -Value $sheet.Range("A1").Value2 } catch { $valuePre = "" }
            $excel.CalculateFull()
            try { $textPost = [string]$sheet.Range("A1").Text } catch { $textPost = "" }
            try { $valuePost = Convert-ValueToString -Value $sheet.Range("A1").Value2 } catch { $valuePost = "" }

            $notes = "$($openCase.path); pre_open_value=$valuePre; pre_open_text=$textPre"
            Add-Row -Rows $rows -ScenarioId $openCase.scenario -Mechanism "Workbook.Open(mutated_xml_formula)" -InputExpr $openCase.expectation -Status "opened" -ObservedType "cell" -ObservedFormula2 $formula2 -ObservedText $textPost -ObservedValue $valuePost -Notes $notes -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatForRow
        }
        catch {
            Add-Row -Rows $rows -ScenarioId $openCase.scenario -Mechanism "Workbook.Open(mutated_xml_formula)" -InputExpr $openCase.expectation -Status "open_failed" -ObservedType "com_exception" -ObservedFormula2 "" -ObservedText "" -ObservedValue "" -Notes ([string]$_.Exception.Message + " | path=" + $openCase.path) -ExcelVersion $excelVersionFull -ExcelChannel $excelChannel -CompatVersion $compatForRow
        }
        finally {
            Close-WorkbookSafe -Workbook $opened
            Release-ComObjectSafe -Obj $sheet
            Release-ComObjectSafe -Obj $opened
        }
    }

    $rows | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
    Write-Host "Formula admission baseline run complete. Rows written: $($rows.Count)"
    Write-Host "Output: $outPath"
}
finally {
    Close-WorkbookSafe -Workbook $workbook
    Release-ComObjectSafe -Obj $worksheet
    Release-ComObjectSafe -Obj $workbook
    Close-WorkbookSafe -Workbook $fileProbeWorkbook
    Release-ComObjectSafe -Obj $fileProbeSheet
    Release-ComObjectSafe -Obj $fileProbeWorkbook
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

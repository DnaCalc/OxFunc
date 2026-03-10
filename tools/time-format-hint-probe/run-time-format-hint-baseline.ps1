param(
    [string]$Manifest = "docs/function-lane/TIME_FORMAT_HINT_SCENARIO_MANIFEST_SEED.csv",
    [Parameter(Mandatory = $true)]
    [string]$Out,
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
    try { return [string]$Value } catch { return "[unprintable]" }
}

$manifestPath = (Resolve-Path -Path $Manifest -ErrorAction Stop).Path
$rows = Import-Csv -Path $manifestPath
$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false

$excelVersion = ""
try { $excelVersion = [string]$excel.Version } catch { $excelVersion = "" }
$excelChannel = Get-ExcelChannel

$results = New-Object System.Collections.Generic.List[object]

try {
    foreach ($row in $rows) {
        $workbook = $null
        $worksheet = $null
        $range = $null
        try {
            if ([string]::IsNullOrWhiteSpace($WorkbookTemplate)) {
                $workbook = $excel.Workbooks.Add()
            }
            else {
                $templatePath = (Resolve-Path -Path $WorkbookTemplate -ErrorAction Stop).Path
                $tempCopy = Join-Path ([System.IO.Path]::GetTempPath()) ("tfmt-" + [guid]::NewGuid().ToString("N") + [System.IO.Path]::GetExtension($templatePath))
                Copy-Item -Path $templatePath -Destination $tempCopy -Force
                $workbook = $excel.Workbooks.Open($tempCopy)
            }

            $worksheet = $workbook.Worksheets.Item(1)
            $range = $worksheet.Range("A1")
            $compatVersion = Get-CompatibilityDescriptor -Workbook $workbook

            $range.NumberFormat = [string]$row.initial_number_format
            $beforeFormat = [string]$range.NumberFormat
            $range.Formula = [string]$row.formula
            $afterFormulaFormat = [string]$range.NumberFormat
            $excel.CalculateFullRebuild()
            Start-Sleep -Milliseconds 200
            $afterCalcFormat = [string]$range.NumberFormat
            $value2 = Convert-Value2ToString -Value $range.Value2
            $text = [string]$range.Text

            $matched = (
                $afterFormulaFormat -ceq [string]$row.expected_after_formula_number_format
            ) -and (
                $afterCalcFormat -ceq [string]$row.expected_after_calc_number_format
            )

            $results.Add([PSCustomObject]@{
                scenario_id = [string]$row.scenario_id
                lane = [string]$row.lane
                execution_status = "observed"
                expectation_status = if ($matched) { "matched" } else { "mismatched" }
                expectation_detail = "after_formula=$afterFormulaFormat expected_after_formula=$($row.expected_after_formula_number_format); after_calc=$afterCalcFormat expected_after_calc=$($row.expected_after_calc_number_format)"
                function_name = [string]$row.function_name
                initial_number_format = [string]$row.initial_number_format
                before_number_format = $beforeFormat
                after_formula_number_format = $afterFormulaFormat
                after_calc_number_format = $afterCalcFormat
                primary_value2 = $value2
                primary_text = $text
                excel_version = $excelVersion
                excel_channel = $excelChannel
                compat_version = $compatVersion
                locale_profile = "en-US"
                run_label = $RunLabel
                source_manifest = $manifestPath
                notes = [string]$row.notes
            })
        }
        catch {
            $results.Add([PSCustomObject]@{
                scenario_id = [string]$row.scenario_id
                lane = [string]$row.lane
                execution_status = "failed"
                expectation_status = "mismatched"
                expectation_detail = [string]$_.Exception.Message
                function_name = [string]$row.function_name
                initial_number_format = [string]$row.initial_number_format
                before_number_format = ""
                after_formula_number_format = ""
                after_calc_number_format = ""
                primary_value2 = ""
                primary_text = ""
                excel_version = $excelVersion
                excel_channel = $excelChannel
                compat_version = ""
                locale_profile = "en-US"
                run_label = $RunLabel
                source_manifest = $manifestPath
                notes = [string]$row.notes
            })
        }
        finally {
            Close-WorkbookSafe -Workbook $workbook
            Release-ComObjectSafe -Obj $range
            Release-ComObjectSafe -Obj $worksheet
            Release-ComObjectSafe -Obj $workbook
        }
    }
}
finally {
    try { $excel.Quit() } catch {}
    Release-ComObjectSafe -Obj $excel
    [gc]::Collect()
    [gc]::WaitForPendingFinalizers()
}

$results | Export-Csv -Path $outPath -NoTypeInformation -Encoding UTF8
Write-Host "Time format-hint baseline complete."
Write-Host "Results: $outPath"

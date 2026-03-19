param(
    [string]$FoundationCatalog = "..\Foundation\research\runs\20260228-130325-excel-compat-spec-index-pass-01\outputs\function_catalog_full.csv",
    [string]$SeedCsv = "docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv",
    [string]$ProbeOut = "docs/function-lane/W28_FUNCTION_NAME_EXISTENCE_PROBE_RESULTS.csv",
    [string]$CatalogOut = "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv"
)

$ErrorActionPreference = "Stop"

function Get-EnglishSeedRow {
    param(
        [array]$SeedRows,
        [string]$Name
    )

    $row = $SeedRows | Where-Object { $_.locale_tag -eq 'en-US' -and $_.localized_name -eq $Name } | Select-Object -First 1
    if (-not $row) {
        throw "Missing en-US support seed row for '$Name'."
    }
    return $row
}

if (-not (Test-Path $FoundationCatalog)) {
    throw "Foundation catalog not found: $FoundationCatalog"
}

if (-not (Test-Path $SeedCsv)) {
    throw "W28 localization seed not found: $SeedCsv"
}

$probeCases = @(
    @{ name = "BETA.INV"; formula = "=BETA.INV(0.5,2,3)" },
    @{ name = "BETA.INVn"; formula = "=BETA.INVn(0.5,2,3)" },
    @{ name = "BETAINV"; formula = "=BETAINV(0.5,2,3,0,1)" },
    @{ name = "IMAGINARY"; formula = "=IMAGINARY(""3+4i"")" },
    @{ name = "ISBLANK"; formula = "=ISBLANK(A20)" },
    @{ name = "ISERR"; formula = "=ISERR(1/0)" },
    @{ name = "ISERROR"; formula = "=ISERROR(1/0)" },
    @{ name = "ISLOGICAL"; formula = "=ISLOGICAL(TRUE)" },
    @{ name = "ISNA"; formula = "=ISNA(NA())" },
    @{ name = "ISNONTEXT"; formula = "=ISNONTEXT(1)" },
    @{ name = "ISNUMBER"; formula = "=ISNUMBER(1)" },
    @{ name = "ISODD"; formula = "=ISODD(3)" },
    @{ name = "ISREF"; formula = "=ISREF(A1)" },
    @{ name = "ISTEXT"; formula = "=ISTEXT(""x"")" },
    @{ name = "FORECAST"; formula = "=FORECAST(3,{1,2},{2,4})" },
    @{ name = "FORECAST.LINEAR"; formula = "=FORECAST.LINEAR(3,{1,2},{2,4})" }
)

$excel = $null
$wb = $null
$ws = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $wb = $excel.Workbooks.Add()
    $ws = $wb.Worksheets.Item(1)

    $probeRows = foreach ($case in $probeCases) {
        $cell = $ws.Cells.Item(1, 1)
        $null = $cell.Clear()
        $cell.Formula = $case.formula
        $excel.CalculateFull()

        [pscustomobject]@{
            function_name = $case.name
            entered_formula = $case.formula
            stored_formula = [string]$cell.Formula
            displayed_text = [string]$cell.Text
            value2 = [string]$cell.Value2
        }
    }

    $probeRows | Export-Csv -NoTypeInformation $ProbeOut
}
finally {
    if ($wb) { $wb.Close($false) }
    if ($excel) { $excel.Quit() }
    if ($ws) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ws) }
    if ($wb) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($wb) }
    if ($excel) { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($excel) }
}

$foundationRows = Import-Csv $FoundationCatalog
$seedRows = Import-Csv $SeedCsv

$platformNote = "Union target across Desktop/Mac/Web. Track exceptions per-function as discovered."

$newRows = @()

$forecastSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "FORECAST"
$isblankSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISBLANK"
$iserrSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISERR"
$iserrorSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISERROR"
$islogicalSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISLOGICAL"
$isnaSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISNA"
$isnontextSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISNONTEXT"
$isnumberSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISNUMBER"
$isoddSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISODD"
$isrefSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISREF"
$istextSeed = Get-EnglishSeedRow -SeedRows $seedRows -Name "ISTEXT"

$newRows += [pscustomobject]@{
    function_name = "FORECAST"
    function_url = "https://support.microsoft.com$($forecastSeed.localized_function_url)"
    category = "Statistical functions"
    version_marker = ""
    tier = "2"
    tier_label = "baseline_context"
    interesting = "false"
    reason_codes = ""
    context_note = ""
    platform_note = $platformNote
    description = $forecastSeed.localized_description
}

$newRows += [pscustomobject]@{
    function_name = "FORECAST.LINEAR"
    function_url = "https://support.microsoft.com$($forecastSeed.localized_function_url)"
    category = "Statistical functions"
    version_marker = ""
    tier = "1"
    tier_label = "regular_pure_or_low_risk"
    interesting = "false"
    reason_codes = ""
    context_note = ""
    platform_note = $platformNote
    description = "Returns a value along a linear trend"
}

foreach ($item in @(
    @{ name = "ISBLANK"; seed = $isblankSeed },
    @{ name = "ISERR"; seed = $iserrSeed },
    @{ name = "ISERROR"; seed = $iserrorSeed },
    @{ name = "ISLOGICAL"; seed = $islogicalSeed },
    @{ name = "ISNA"; seed = $isnaSeed },
    @{ name = "ISNONTEXT"; seed = $isnontextSeed },
    @{ name = "ISNUMBER"; seed = $isnumberSeed },
    @{ name = "ISODD"; seed = $isoddSeed },
    @{ name = "ISREF"; seed = $isrefSeed },
    @{ name = "ISTEXT"; seed = $istextSeed }
)) {
    $newRows += [pscustomobject]@{
        function_name = $item.name
        function_url = "https://support.microsoft.com$($item.seed.localized_function_url)"
        category = "Information functions"
        version_marker = ""
        tier = "1"
        tier_label = "regular_pure_or_low_risk"
        interesting = "false"
        reason_codes = ""
        context_note = ""
        platform_note = $platformNote
        description = $item.seed.localized_description
    }
}

$filteredFoundation = $foundationRows | Where-Object { $_.function_name -ne "RANDARRA" }

$catalogRows =
    @($filteredFoundation) +
    @($newRows | Where-Object {
        $name = $_.function_name
        -not ($filteredFoundation | Where-Object { $_.function_name -eq $name })
    })

$catalogRows |
    Sort-Object function_name |
    Export-Csv -NoTypeInformation $CatalogOut

$probeSummary = Import-Csv $ProbeOut
$catalogCount = (Import-Csv $CatalogOut | Measure-Object).Count

Write-Output "Probe rows: $($probeSummary.Count)"
Write-Output "Local canonical catalog count: $catalogCount"

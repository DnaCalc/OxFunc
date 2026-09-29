[CmdletBinding()]
param([string]$Out = "smart-fuzzer/cache/w111-locale-profile-20260929.json")

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if (@(Get-Process -Name EXCEL -ErrorAction SilentlyContinue).Count -ne 0) {
    throw "An Excel process already exists; the root oracle owner must run this only after its queue releases Excel."
}
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../../..")).Path
$outPath = if ([IO.Path]::IsPathRooted($Out)) { $Out } else { Join-Path $repoRoot $Out }
[void][IO.Directory]::CreateDirectory((Split-Path -Parent $outPath))
$excel = $null; $books = $null; $book = $null; $sheets = $null; $sheet = $null; $inputCell = $null; $resultCell = $null
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false; $excel.DisplayAlerts = $false
    $books = $excel.Workbooks; $book = $books.Add()
    $book.PrecisionAsDisplayed = $false
    $sheets = $book.Worksheets; $sheet = $sheets.Item(1)
    # Public enumeration: https://learn.microsoft.com/en-us/office/vba/api/excel.xlapplicationinternational
    $indices = [ordered]@{ country_code=1;country_setting=2;decimal_separator=3;thousands_separator=4;list_separator=5;
        column_separator=14;row_separator=15;date_separator=17;time_separator=18;year_code=19;month_code=20;
        day_code=21;hour_code=22;minute_code=23;second_code=24;currency_code=25;general_format_name=26;
        currency_digits=27;currency_negative=28;noncurrency_digits=29;date_order=32;clock_24_hour=33;
        non_english_functions=34;currency_space_before=36;currency_before=37;currency_minus_sign=38;
        currency_trailing_zeros=39;currency_leading_zeros=40;month_leading_zero=41;day_leading_zero=42;
        four_digit_years=43;long_date_mdy=44;time_leading_zero=45 }
    $international = [ordered]@{}
    foreach($entry in $indices.GetEnumerator()) {
        try { $international[$entry.Key] = $excel.International([int]$entry.Value) }
        catch { $international[$entry.Key] = [ordered]@{unavailable=$_.Exception.Message} }
    }
    $culture = [Globalization.CultureInfo]::CurrentCulture
    $windows = [ordered]@{name=$culture.Name;lcid=$culture.LCID;
        decimal_separator=$culture.NumberFormat.NumberDecimalSeparator;thousands_separator=$culture.NumberFormat.NumberGroupSeparator;
        currency_symbol=$culture.NumberFormat.CurrencySymbol;currency_positive_pattern=$culture.NumberFormat.CurrencyPositivePattern;
        currency_negative_pattern=$culture.NumberFormat.CurrencyNegativePattern;short_date_pattern=$culture.DateTimeFormat.ShortDatePattern;
        date_separator=$culture.DateTimeFormat.DateSeparator;time_separator=$culture.DateTimeFormat.TimeSeparator;
        two_digit_year_max=$culture.DateTimeFormat.Calendar.TwoDigitYearMax}
    $inputCell = $sheet.Range("A1"); $resultCell = $sheet.Range("C1")
    $inputCell.Value2 = [double]1234.5
    $sentinels = @()
    foreach($formula in @('=DOLLAR(A1)','=FIXED(A1)','=TEXT(A1,"0.00")','=DATEVALUE("1/2/2026")','=TIMEVALUE("13:30")','=VALUE("1,234.5")')) {
        $resultCell.Formula2 = $formula; $resultCell.Calculate()
        $value = $resultCell.Value2
        $sentinels += [ordered]@{formula=$formula;value2=$value;clr_type=$(if($null -eq $value){"null"}else{$value.GetType().FullName});
            bits_hex=$(if($value -is [double]){"0x{0:x16}" -f [BitConverter]::DoubleToInt64Bits($value)}else{$null})}
    }
    $sourceHashes = [ordered]@{}
    foreach($relative in @('crates/oxfunc_core/src/locale_format.rs','../OxFml/crates/oxfml_core/src/format/engine.rs','../OxFml/crates/oxfml_core/src/format/number.rs','../OxFml/crates/oxfml_core/src/format/datetime.rs','../OxFml/crates/oxfml_core/src/format/general.rs')) {
        $sourceHashes[$relative] = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $repoRoot $relative)).Hash.ToLowerInvariant()
    }
    $record = [ordered]@{schema_version='w111.live_locale_profile.v1';captured_utc=[DateTime]::UtcNow.ToString('o');
        excel_version=[string]$excel.Version;excel_build=[string]$excel.Build;operating_system=[string]$excel.OperatingSystem;
        channel='unavailable_not_queried';compatibility_version=$(try{[string]$book.CompatibilityVersion}catch{'unavailable'});
        date_system=$(if($book.Date1904){'1904'}else{'1900'});precision_as_displayed=[bool]$book.PrecisionAsDisplayed;
        use_system_separators=[bool]$excel.UseSystemSeparators;application_decimal_separator=[string]$excel.DecimalSeparator;
        application_thousands_separator=[string]$excel.ThousandsSeparator;international=$international;windows_culture=$windows;
        sentinel_input_value2=1234.5;sentinels=$sentinels;provider_source_sha256=$sourceHashes;
        profile_admission='No automatic profile selection; compare recorded regional fields with an explicit FormatProfile before local replay.';
        script_sha256=(Get-FileHash -Algorithm SHA256 -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()}
    $record | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $outPath -Encoding utf8NoBOM
    Write-Output $outPath
} finally {
    if($null -ne $book) { $book.Close($false) }
    if($null -ne $excel) { $excel.Quit() }
    foreach($object in @($resultCell,$inputCell,$sheet,$sheets,$book,$books,$excel)) {
        if($null -ne $object -and [Runtime.InteropServices.Marshal]::IsComObject($object)) {
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($object)
        }
    }
    [GC]::Collect(); [GC]::WaitForPendingFinalizers()
}

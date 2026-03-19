[CmdletBinding()]
param(
    [string]$CanonicalUrl = "https://support.microsoft.com/en-us/office/excel-functions-alphabetical-b3944572-255d-4efb-bb96-c6d90033e188",
    [string]$FoundationCatalogPath = "..\Foundation\research\runs\20260228-130325-excel-compat-spec-index-pass-01\outputs\function_catalog_full.csv",
    [string]$LocaleSeedOut = ".tmp/w28-support-function-localization-locales.csv",
    [string]$LibrarySeedOut = "docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv",
    [string]$ReconciliationOut = "docs/function-lane/W28_SUPPORT_FUNCTION_CATALOG_RECONCILIATION.csv",
    [string]$SummaryOut = ".tmp/w28-support-function-localization-summary.json"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Convert-HtmlText {
    param([string]$Text)
    if ($null -eq $Text) { return "" }
    Add-Type -AssemblyName System.Web
    $decoded = [System.Web.HttpUtility]::HtmlDecode($Text)
    $decoded = $decoded -replace '<[^>]+>', ' '
    $decoded = $decoded -replace '\s+', ' '
    return $decoded.Trim()
}

function Get-ArticleGuid {
    param([string]$Url)
    $match = [regex]::Match($Url, '([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})$')
    if (-not $match.Success) {
        throw "Could not extract article GUID from '$Url'."
    }
    $match.Groups[1].Value.ToLowerInvariant()
}

function Get-SupportPageContent {
    param([string]$Url)
    (Invoke-WebRequest -UseBasicParsing $Url).Content
}

function Get-AlternateLocales {
    param(
        [string]$Html,
        [string]$CanonicalArticleGuid
    )

    $pattern = '<link[^>]+rel="alternate"[^>]+hreflang="([^"]+)"[^>]+href="([^"]+)"'
    $matches = [regex]::Matches($Html, $pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    $rows = @()
    foreach ($m in $matches) {
        $locale = $m.Groups[1].Value
        $url = $m.Groups[2].Value
        if ($locale -eq 'x-default') { continue }
        if ($url -notmatch '/office/') { continue }
        $rows += [pscustomobject]@{
            locale_tag = $locale
            article_guid = $CanonicalArticleGuid
            localized_url = $url
            discovery_source = "support_hreflang_harvest"
            status_note = "harvested"
        }
    }
    $rows | Sort-Object locale_tag -Unique
}

function Get-FunctionRowsFromPage {
    param(
        [string]$Html,
        [string]$LocaleTag,
        [string]$PageUrl
    )

    $rowPattern = '(?s)<tr>\s*<td>\s*<p[^>]*>\s*<a href="([^"]+/office/[^"]+-(?:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}))" class="ocpArticleLink"[^>]*>(.*?)</a>\s*(?:<br[^>]*>\s*\(([^<]+)\))?.*?</td>\s*<td>\s*<p>\s*<b class="ocpRunInHead">(.*?)</b>(.*?)</p>'
    $matches = [regex]::Matches($Html, $rowPattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    $rows = New-Object System.Collections.Generic.List[object]
    foreach ($m in $matches) {
        $articleUrl = $m.Groups[1].Value
        $localizedName = Convert-HtmlText $m.Groups[2].Value
        $versionMarker = Convert-HtmlText $m.Groups[3].Value
        $categoryText = Convert-HtmlText $m.Groups[4].Value
        $description = Convert-HtmlText $m.Groups[5].Value
        if ([string]::IsNullOrWhiteSpace($localizedName)) { continue }
        $rows.Add([pscustomobject]@{
            locale_tag = $LocaleTag
            list_article_guid = (Get-ArticleGuid $PageUrl)
            function_article_guid = (Get-ArticleGuid $articleUrl)
            localized_name = $localizedName
            localized_function_url = $articleUrl
            version_marker = $versionMarker
            localized_category = $categoryText.TrimEnd(':')
            localized_description = $description
        })
    }
    $rows
}

function Ensure-ParentDirectory {
    param([string]$Path)
    $parent = Split-Path -Parent $Path
    if ($parent -and -not (Test-Path $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
}

$canonicalHtml = Get-SupportPageContent -Url $CanonicalUrl
$canonicalGuid = Get-ArticleGuid $CanonicalUrl
$locales = Get-AlternateLocales -Html $canonicalHtml -CanonicalArticleGuid $canonicalGuid

$harvestRows = New-Object System.Collections.Generic.List[object]
foreach ($localeRow in $locales) {
    $pageHtml = Get-SupportPageContent -Url $localeRow.localized_url
    foreach ($functionRow in (Get-FunctionRowsFromPage -Html $pageHtml -LocaleTag $localeRow.locale_tag -PageUrl $localeRow.localized_url)) {
        $harvestRows.Add($functionRow)
    }
}

$englishRows = $harvestRows | Where-Object { $_.locale_tag -eq 'en-US' }
$foundationCatalog = Import-Csv $FoundationCatalogPath
$foundationNames = $foundationCatalog.function_name | Sort-Object -Unique
$supportNames = $englishRows.localized_name | Sort-Object -Unique

$supportSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
foreach ($name in $supportNames) { [void]$supportSet.Add($name) }
$foundationSet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
foreach ($name in $foundationNames) { [void]$foundationSet.Add($name) }

$reconciliation = New-Object System.Collections.Generic.List[object]
foreach ($name in $supportNames) {
    $class = if ($foundationSet.Contains($name)) { 'in_both' } else { 'support_only' }
    $reconciliation.Add([pscustomobject]@{
        reconciliation_class = $class
        function_name = $name
    })
}
foreach ($name in $foundationNames) {
    if (-not $supportSet.Contains($name)) {
        $reconciliation.Add([pscustomobject]@{
            reconciliation_class = 'foundation_only'
            function_name = $name
        })
    }
}
$reconciliation = $reconciliation | Sort-Object reconciliation_class, function_name

$supportOnlyCount = ($reconciliation | Where-Object reconciliation_class -eq 'support_only' | Measure-Object).Count
$foundationOnlyCount = ($reconciliation | Where-Object reconciliation_class -eq 'foundation_only' | Measure-Object).Count

$summary = [pscustomobject]@{
    captured_at_utc = [DateTime]::UtcNow.ToString("o")
    canonical_url = $CanonicalUrl
    article_guid = $canonicalGuid
    locale_count = ($locales | Measure-Object).Count
    harvested_row_count = ($harvestRows | Measure-Object).Count
    english_row_count = ($englishRows | Measure-Object).Count
    support_unique_name_count = ($supportNames | Measure-Object).Count
    foundation_catalog_count = ($foundationNames | Measure-Object).Count
    support_only_count = $supportOnlyCount
    foundation_only_count = $foundationOnlyCount
}

Ensure-ParentDirectory $LocaleSeedOut
Ensure-ParentDirectory $LibrarySeedOut
Ensure-ParentDirectory $ReconciliationOut
Ensure-ParentDirectory $SummaryOut

$locales | Export-Csv -Path $LocaleSeedOut -NoTypeInformation -Encoding UTF8
$harvestRows | Sort-Object locale_tag, localized_name, function_article_guid |
    Export-Csv -Path $LibrarySeedOut -NoTypeInformation -Encoding UTF8
$reconciliation | Export-Csv -Path $ReconciliationOut -NoTypeInformation -Encoding UTF8
$summary | ConvertTo-Json -Depth 4 | Set-Content -Path $SummaryOut -Encoding UTF8

Write-Output "Locales: $($summary.locale_count)"
Write-Output "Harvested rows: $($summary.harvested_row_count)"
Write-Output "English unique names: $($summary.support_unique_name_count)"
Write-Output "Foundation catalog names: $($summary.foundation_catalog_count)"
Write-Output "Support-only names: $supportOnlyCount"
Write-Output "Foundation-only names: $foundationOnlyCount"

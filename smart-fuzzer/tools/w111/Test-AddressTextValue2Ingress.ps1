[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Batch,
    [Parameter(Mandatory)][string]$Out,
    [int]$ChunkSize = 20000
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$source = Get-Content -LiteralPath $Batch -Raw | ConvertFrom-Json
$texts = [Collections.Generic.List[string]]::new()
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($probe in $source.probes) {
    foreach ($arg in $probe.probe.args) {
        if ($arg -is [string] -and $arg -notmatch '^0x[0-9a-fA-F]{16}$' -and $seen.Add($arg)) {
            $texts.Add($arg)
        }
    }
}
$excel = $null
$workbook = $null
$sheet = $null
$failures = [Collections.Generic.List[object]]::new()
$profile = $null
$started = [DateTime]::UtcNow.ToString('o')
try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $workbook = $excel.Workbooks.Add()
    $sheet = $workbook.Worksheets.Item(1)
    $profile = [ordered]@{
        excel_version = [string]$excel.Version
        excel_build = [string]$excel.Build
        excel_operating_system = [string]$excel.OperatingSystem
        workbook_compatibility = $(try { [string]$workbook.CompatibilityVersion } catch { 'unknown' })
        powershell_version = $PSVersionTable.PSVersion.ToString()
        method = 'raw String in object[,] Range.Value2, General format, ordinal String readback'
    }
    for ($start = 0; $start -lt $texts.Count; $start += $ChunkSize) {
        $count = [Math]::Min($ChunkSize, $texts.Count - $start)
        $range = $sheet.Range("A1:A$count")
        try {
            $range.ClearContents() | Out-Null
            $range.NumberFormat = 'General'
            $values = New-Object 'object[,]' $count, 1
            for ($i = 0; $i -lt $count; $i++) { $values[$i, 0] = $texts[$start + $i] }
            $range.Value2 = $values
            $observed = $range.Value2
            for ($i = 0; $i -lt $count; $i++) {
                $got = if ($count -eq 1) { $observed } else { $observed.GetValue($i + 1, 1) }
                $expected = $texts[$start + $i]
                if ($got -isnot [string] -or -not [string]::Equals($got, $expected, [StringComparison]::Ordinal)) {
                    $failures.Add([ordered]@{
                        unique_text_index = $start + $i
                        expected = $expected
                        expected_utf16_hex = [Convert]::ToHexString([Text.Encoding]::Unicode.GetBytes($expected))
                        actual_type = $(if ($null -eq $got) { 'null' } else { $got.GetType().Name })
                        actual = $got
                        actual_utf16_hex = $(if ($got -is [string]) { [Convert]::ToHexString([Text.Encoding]::Unicode.GetBytes($got)) } else { $null })
                    })
                }
            }
        } finally {
            if ($null -ne $range) { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($range) }
        }
    }
} finally {
    if ($null -ne $workbook) { $workbook.Close($false) }
    if ($null -ne $excel) { $excel.Quit() }
    foreach ($obj in @($sheet, $workbook, $excel)) {
        if ($null -ne $obj -and [Runtime.InteropServices.Marshal]::IsComObject($obj)) {
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($obj)
        }
    }
}
$result = [ordered]@{
    schema_version = 'w111-address-text-value2-ingress-v1'
    started_utc = $started
    captured_utc = [DateTime]::UtcNow.ToString('o')
    source_batch = $Batch
    source_sha256 = (Get-FileHash -LiteralPath $Batch -Algorithm SHA256).Hash
    runner_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
    profile = $profile
    unique_text_count = $texts.Count
    exact_ordinal_count = $texts.Count - $failures.Count
    failure_count = $failures.Count
    failures = $failures.ToArray()
}
$result | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $Out -Encoding utf8
Write-Output "Text Value2 ingress: $($result.exact_ordinal_count)/$($texts.Count) exact, failures=$($failures.Count), output=$Out"

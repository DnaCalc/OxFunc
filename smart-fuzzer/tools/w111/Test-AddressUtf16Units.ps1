[CmdletBinding()]
param([Parameter(Mandatory)][string]$Out)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$unitCases = [Collections.Generic.List[object]]::new()
foreach ($unit in 0xD800..0xDFFF) {
    $unitCases.Add([int[]]@($unit, 122))
    $unitCases.Add([int[]]@(122, $unit, 122))
}
foreach ($units in @([int[]]@(0), [int[]]@(122,0,122), [int[]]@(0xD800,0xDC00),
                    [int[]]@(0xDBFF,0xDFFF), [int[]]@(0xD800,0xD800), [int[]]@(0xDC00,0xDC00))) {
    $unitCases.Add($units)
}
function Get-UnitHex([string]$Text) {
    return (($Text.ToCharArray() | ForEach-Object { '{0:X4}' -f [int]$_ }) -join '')
}
$excel=$null; $book=$null; $sheet=$null; $inputRange=$null; $resultRange=$null
$rows=[Collections.Generic.List[object]]::new()
$profile=$null
try {
    $excel=New-Object -ComObject Excel.Application
    $excel.Visible=$false; $excel.DisplayAlerts=$false
    $book=$excel.Workbooks.Add(); $sheet=$book.Worksheets.Item(1)
    $profile=[ordered]@{excel_version=[string]$excel.Version;excel_build=[string]$excel.Build;
        workbook_compatibility=$(try{[string]$book.CompatibilityVersion}catch{'unknown'});
        excel_operating_system=[string]$excel.OperatingSystem}
    $count=$unitCases.Count
    $inputRange=$sheet.Range("A1:A$count"); $inputRange.NumberFormat='General'
    $data=New-Object 'object[,]' $count,1
    for($i=0;$i -lt $count;$i++) {
        $data[$i,0]=-join ([char[]]$unitCases[$i])
    }
    $inputRange.Value2=$data
    $readback=$inputRange.Value2
    $resultRange=$sheet.Range("B1:B$count")
    $resultRange.Formula2R1C1='=ADDRESS(1,1,1,1,RC[-1])'
    $excel.Calculate()
    $answers=$resultRange.Value2
    for($i=0;$i -lt $count;$i++) {
        $inputHex=Get-UnitHex $data[$i,0]
        $got=$readback.GetValue($i+1,1); $answer=$answers.GetValue($i+1,1)
        $rows.Add([ordered]@{id=('address-utf16-{0:d4}' -f $i);input_utf16_hex=$inputHex;
            ingress_exact=($got -is [string] -and [string]::Equals($got,$data[$i,0],[StringComparison]::Ordinal));
            readback_utf16_hex=$(if($got -is [string]){Get-UnitHex $got}else{$null});
            result_type=$(if($null -eq $answer){'null'}else{$answer.GetType().Name});
            result_utf16_hex=$(if($answer -is [string]){Get-UnitHex $answer}else{$null});
            result_scalar=$(if($answer -isnot [string]){$answer}else{$null})})
    }
} finally {
    if($null -ne $book){$book.Close($false)}
    if($null -ne $excel){$excel.Quit()}
    foreach($obj in @($resultRange,$inputRange,$sheet,$book,$excel)){
        if($null -ne $obj -and [Runtime.InteropServices.Marshal]::IsComObject($obj)){
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($obj)
        }
    }
}
[ordered]@{schema_version='w111-address-utf16-unit-observations-v1';captured_utc=[DateTime]::UtcNow.ToString('o');
    runner_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash;
    profile=$profile;witnesses=$rows.ToArray()} | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $Out -Encoding utf8
Write-Output "ADDRESS UTF16 units: $($rows.Count) rows -> $Out"

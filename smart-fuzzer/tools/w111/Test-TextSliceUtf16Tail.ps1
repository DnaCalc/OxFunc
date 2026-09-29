[CmdletBinding()]
param([Parameter(Mandatory)][string]$Out)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# Raw units stay numeric in the evidence; JSON never transports an isolated
# surrogate as a string. This is a public COM black-box capture only.
function Get-UnitHex([string]$Text) {
    return (($Text.ToCharArray() | ForEach-Object { '{0:X4}' -f [int]$_ }) -join '')
}
$sources=[Collections.Generic.List[object]]::new()
$sources.Add([int[]]@(65))
$sources.Add([int[]]@(65,65))
$sources.Add([int[]]@(65,65,65))
$sources.Add([int[]]@(65,65,55296))
$sources.Add([int[]]@(65,65,56319,66))
$sources.Add([int[]]@(65,65,56320))
$sources.Add([int[]]@(65,65,56320,57343,57343,55296,55296,57343,55296))
$sources.Add([int[]]@(65,66,65,57343,57343,66))
$sources.Add([int[]]@(65,55296))
$sources.Add([int[]]@(65,55296,65))
$sources.Add([int[]]@(65,55296,55296))
$sources.Add([int[]]@(65,55296,55296,56319))
$sources.Add([int[]]@(65,55296,55296,56319,55296))
$sources.Add([int[]]@(65,55296,56320))
$sources.Add([int[]]@(65,56320))
$sources.Add([int[]]@(65,56320,65))
$sources.Add([int[]]@(65,56320,66,56319,56320,66,57343))
$sources.Add([int[]]@(65,56320,55296))
$sources.Add([int[]]@(65,56320,56320))
$sources.Add([int[]]@(65,56320,57343,65,55296,56320,66))
$sources.Add([int[]]@(65,57343,56320,55296))
$sources.Add([int[]]@(65,57343,57343,56320,56320))
$sources.Add([int[]]@(66,65,56320,57343,65,57343,66,56319,57343))
$sources.Add([int[]]@(66,66,65,56320,57343))
$sources.Add([int[]]@(66,66,66,66,66,56320))
$sources.Add([int[]]@(66,66,56319,65,55296,57343,56319,56320))
$sources.Add([int[]]@(66,55296,66,57343,55296,56320,56320,56319))
$sources.Add([int[]]@(66,56320,57343,65,55296))
$sources.Add([int[]]@(66,57343,55296,55296))
$sources.Add([int[]]@(66,57343,55296,57343,55296,66))
$sources.Add([int[]]@(66,57343,56319,65,56320,65,56319,55296,65))
$sources.Add([int[]]@(66,57343,56320,57343,65,66,57343,57343,55296))
$sources.Add([int[]]@(55296))
$sources.Add([int[]]@(55296,65))
$sources.Add([int[]]@(55296,65,65))
$sources.Add([int[]]@(55296,65,55296))
$sources.Add([int[]]@(55296,65,56320))
$sources.Add([int[]]@(55296,65,57343,65))
$sources.Add([int[]]@(55296,65,57343,66,56320,57343,56320,65,56319))
$sources.Add([int[]]@(55296,66,57343,57343,66,57343,57343,55296,55296))
$sources.Add([int[]]@(55296,55296))
$sources.Add([int[]]@(55296,55296,65))
$sources.Add([int[]]@(55296,55296,55296))
$sources.Add([int[]]@(55296,55296,56320))
$sources.Add([int[]]@(55296,55296,57343,56320,56319,55296,65,57343,55296))
$sources.Add([int[]]@(55296,56319,65,65,55296,57343,56320,66,66))
$sources.Add([int[]]@(55296,56319,66,56319,56319,65,65,56320,57343))
$sources.Add([int[]]@(55296,56320))
$sources.Add([int[]]@(55296,56320,65))
$sources.Add([int[]]@(55296,56320,66,55296,56319,57343,57343,56319,56319))
$sources.Add([int[]]@(55296,56320,55296))
$sources.Add([int[]]@(55296,56320,56320))
$sources.Add([int[]]@(55296,56320,57343,66,66,56319,56320))
$sources.Add([int[]]@(55296,57343,66,66,66))
$sources.Add([int[]]@(55296,57343,66,55296,56319,56320))
$sources.Add([int[]]@(55296,57343,56320,66,57343))
$sources.Add([int[]]@(55296,57343,56320,57343,56319,66,65,56319))
$sources.Add([int[]]@(56319,65,56319,56319,55296,57343,55296))
$sources.Add([int[]]@(56319,66,56319,55296,66,66,55296,55296,57343))
$sources.Add([int[]]@(56319,66,56320,57343))
$sources.Add([int[]]@(56319,56319,55296,56320,56319,57343,57343,57343,56320))
$sources.Add([int[]]@(56319,56319,55296,57343,66))
$sources.Add([int[]]@(56319,56319,56320,56319,56320))
$sources.Add([int[]]@(56319,56320,65,57343,65,55296))
$sources.Add([int[]]@(56319,56320,56320,66))
$sources.Add([int[]]@(56319,56320,57343,65,55296,57343,56320))
$sources.Add([int[]]@(56319,56320,57343,66,56320,65))
$sources.Add([int[]]@(56320))
$sources.Add([int[]]@(56320,65))
$sources.Add([int[]]@(56320,65,65))
$sources.Add([int[]]@(56320,65,65,65,57343))
$sources.Add([int[]]@(56320,65,55296))
$sources.Add([int[]]@(56320,65,56320))
$sources.Add([int[]]@(56320,55296))
$sources.Add([int[]]@(56320,55296,65))
$sources.Add([int[]]@(56320,55296,66,57343,66,65))
$sources.Add([int[]]@(56320,55296,55296))
$sources.Add([int[]]@(56320,55296,56320))
$sources.Add([int[]]@(56320,55296,56320,66,57343,65,66,55296))
$sources.Add([int[]]@(56320,56319,56319,57343))
$sources.Add([int[]]@(56320,56320))
$sources.Add([int[]]@(56320,56320,65))
$sources.Add([int[]]@(56320,56320,66,57343,55296,66,56319))
$sources.Add([int[]]@(56320,56320,55296))
$sources.Add([int[]]@(56320,56320,56319,56319))
$sources.Add([int[]]@(56320,56320,56320))
$sources.Add([int[]]@(57343,65,65,66,66,55296))
$sources.Add([int[]]@(57343,65,56319,65,65))
$sources.Add([int[]]@(57343,55296,65,65,66,56319,66))
$sources.Add([int[]]@(57343,55296,56319,56320,65,57343,56320,56320,56320))
$sources.Add([int[]]@(57343,57343,65,65,55296,56320,57343))
$sources.Add([int[]]@(57343,57343,56319,55296,65,55296))
$sources.Add([int[]]@(57343,57343,56319,56320,56320,56320,57343))
$sources.Add([int[]]@(57343,57343,56319,57343))
$sources.Add([int[]]@(57343,57343,56320,56320))
$sources.Add([int[]]@(57343,57343,57343,56319,55296,56319))
$probes=[Collections.Generic.List[object]]::new()
foreach($units in $sources) {
    foreach($fn in @('LEFT','RIGHT','LEFTB','RIGHTB')) {
        foreach($count in @(-0.5,0,1,2,3,4,5,6,8,32767)) {
            $probes.Add([ordered]@{function=$fn;units=$units;count=$count;start=1;formula="=$fn(RC[-3],RC[-2])"})
        }
    }
    foreach($fn in @('LEN','LENB')) {
        $probes.Add([ordered]@{function=$fn;units=$units;count=0;start=1;formula="=$fn(RC[-3])"})
    }
    foreach($fn in @('MID','MIDB')) {
        foreach($start in 0..8) { foreach($count in @(0,1,2,3,8)) {
            $probes.Add([ordered]@{function=$fn;units=$units;count=$count;start=$start;formula="=$fn(RC[-3],RC[-1],RC[-2])"})
        }}
    }
}
$excel=$null;$book=$null;$sheet=$null;$inputs=$null;$textRange=$null;$results=$null
$rows=[Collections.Generic.List[object]]::new();$profile=$null
try {
    $excel=New-Object -ComObject Excel.Application
    $excel.Visible=$false;$excel.DisplayAlerts=$false
    $book=$excel.Workbooks.Add();$sheet=$book.Worksheets.Item(1)
    $profile=[ordered]@{excel_version=[string]$excel.Version;excel_build=[string]$excel.Build;
        workbook_compatibility=$(try{[string]$book.CompatibilityVersion}catch{'unknown'});
        excel_operating_system=[string]$excel.OperatingSystem}
    $count=$probes.Count;$inputs=$sheet.Range("A1:C$count");$textRange=$sheet.Range("A1:A$count");$textRange.NumberFormat='@'
    $data=New-Object 'object[,]' $count,3;$formulas=New-Object 'object[,]' $count,1
    for($i=0;$i -lt $count;$i++) {
        $probe=$probes[$i];$data[$i,0]=-join ([char[]]$probe.units)
        $data[$i,1]=[double]$probe.count;$data[$i,2]=[double]$probe.start;$formulas[$i,0]=$probe.formula
    }
    $inputs.Value2=$data;$readback=$inputs.Value2
    $results=$sheet.Range("D1:D$count");$results.Formula2R1C1=$formulas;$excel.Calculate();$answers=$results.Value2
    for($i=0;$i -lt $count;$i++) {
        $probe=$probes[$i];$got=$readback.GetValue($i+1,1);$answer=$answers.GetValue($i+1,1)
        $rows.Add([ordered]@{id=('text-utf16-{0:d4}' -f $i);function=$probe.function;count=$probe.count;start=$probe.start;
            input_utf16_hex=(Get-UnitHex $data[$i,0]);
            ingress_exact=($got -is [string] -and [string]::Equals($got,$data[$i,0],[StringComparison]::Ordinal));
            readback_utf16_hex=$(if($got -is [string]){Get-UnitHex $got}else{$null});
            result_type=$(if($null -eq $answer){'null'}else{$answer.GetType().Name});
            result_utf16_hex=$(if($answer -is [string]){Get-UnitHex $answer}else{$null});
            result_scalar=$(if($answer -isnot [string]){$answer}else{$null});
            result_bits=$(if($answer -is [double]){'{0:x16}' -f [BitConverter]::DoubleToUInt64Bits($answer)}else{$null})})
    }
} finally {
    if($null -ne $book){$book.Close($false)}
    if($null -ne $excel){$excel.Quit()}
    foreach($obj in @($results,$textRange,$inputs,$sheet,$book,$excel)) {
        if($null -ne $obj -and [Runtime.InteropServices.Marshal]::IsComObject($obj)) {
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($obj)
        }
    }
}
[ordered]@{schema_version='w111-text-utf16-tail-observations-v1';captured_utc=[DateTime]::UtcNow.ToString('o');
    runner_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash;
    profile=$profile;witnesses=$rows.ToArray()} | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $Out -Encoding utf8
Write-Output "Text UTF16 units: $($rows.Count) rows -> $Out"

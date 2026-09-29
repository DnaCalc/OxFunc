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
$sources.Add([int[]]@(9415,55799,55403,57191,57132,18132,56410,55507,55725,56502,55473))
$sources.Add([int[]]@(14087,56868,19608,56916,55666,55847,12730,3657))
$sources.Add([int[]]@(14805,56162,55724,55850,8548,38242,56525,55760,56977))
$sources.Add([int[]]@(18321,56105,15415,56617))
$sources.Add([int[]]@(19031,56678))
$sources.Add([int[]]@(19750,55424,56975,55610,56438,31274,8540,55143,52129,14550,57086,43855,45410,40645))
$sources.Add([int[]]@(21687,57038,55401,56030,56365,55630,55911,41820,57033,55897,56033))
$sources.Add([int[]]@(24982,57050,56700,57311,38891,50500,18182,42448,56785,31362,25687))
$sources.Add([int[]]@(32804,15552,55368,56851,55426,55615,32003,55487,55513,56879,56841,57324,56388,20014))
$sources.Add([int[]]@(34506,55937,56358,57208,55895,56658,55984,56619,55778,55380,56658,19504,8792))
$sources.Add([int[]]@(37590,50657,17575,30554))
$sources.Add([int[]]@(47866,56257,55534,56739,57152,56268,56815,56554,56180,44649,57031,57168,55504,56658,55370,56066))
$sources.Add([int[]]@(47948,55730,57132,55801,16973))
$sources.Add([int[]]@(48704,23970,30257,57028,56977,55472,38215,56915))
$sources.Add([int[]]@(49212,57063))
$sources.Add([int[]]@(50240,48581,56196,56989,38279))
$sources.Add([int[]]@(55317))
$sources.Add([int[]]@(55324,15791,41557,28857,44863,55765,10277,35758,41994,18721,55453,26721,57122,57103,56108))
$sources.Add([int[]]@(55364,40109,33614,23604,5068,56437))
$sources.Add([int[]]@(55431,56413,16007,57083,15438,56170,56266,51086,57124,56158,22254))
$sources.Add([int[]]@(55448,56555,56071,56474,55883,57129,57241,55738,55521))
$sources.Add([int[]]@(55449,12019,57207,56945,53888,56863,56189,22197,56205))
$sources.Add([int[]]@(55493,56604,56756))
$sources.Add([int[]]@(55763,57132,56477,46263,55789,56260,56074,57092,56587,56273))
$sources.Add([int[]]@(55812,57286,55756,56142))
$sources.Add([int[]]@(55908))
$sources.Add([int[]]@(55917,41935,56453))
$sources.Add([int[]]@(55929,56730,55933,27562,55686,54874,55572,25525,4308,55970,56978,57074,4206,55965,55899,56325,56223))
$sources.Add([int[]]@(55931,55612,51421,11838,48140))
$sources.Add([int[]]@(55994,56162,56940,57107,57250,45352,46676,57129,56779,56773,56007,55438,56044,9626))
$sources.Add([int[]]@(56009,57275,29071,57064,55639,56207,55524,55331,30536,18493,56775,55483))
$sources.Add([int[]]@(56103,57183,56856))
$sources.Add([int[]]@(56116,28324))
$sources.Add([int[]]@(56222,56314,24473,56301,37750))
$sources.Add([int[]]@(56294,56437,57153,4684,55312,29814,38695,24382))
$sources.Add([int[]]@(56362,56087,36153,57128,56237,13926,2754,56535,57065,57092,55606,55478,56126,55393,56188))
$sources.Add([int[]]@(56383,57014,51599,57112,57035,57086,6594))
$sources.Add([int[]]@(56428,56279,56647,5063,31935,50691,36535,55385,56185,56741,55728,55674,56658,33303))
$sources.Add([int[]]@(56459,55849,43960,42465,56346,57323,56835,35120,56428,56240,170,17749,56690,55334))
$sources.Add([int[]]@(56600,57213,16684,56010,57075,27885,56252,25188,55333,55422,56087,54737,28901,29861))
$sources.Add([int[]]@(56657,56392,56632,56451,51255,55651,56353,57022,55018,29958,56265,57189,56162,22637))
$sources.Add([int[]]@(56734,56247,42721,55934,56576,22722,50770,18935,57018,56235,55718,55429,55453))
$sources.Add([int[]]@(56792,56105,56111,5792,6879,56876,57133,25363))
$sources.Add([int[]]@(56856,55760,55682,57217,49329,56128,57176,56776,56633,27701,55483,55703,56827))
$sources.Add([int[]]@(57002,2246,57239,32056,56873,56957,30758,44250))
$sources.Add([int[]]@(57032,55806,56312,31088,19998,55999,9399,56384,56943,56086))
$sources.Add([int[]]@(57135,56869,57120,35502,13965,57226,56145,55452,56708,57276,9525,8472,57204))
$sources.Add([int[]]@(57144,56090,55784,6150,57055,34958,56357,57092,55856,55689,56421,56836,57134,55477,55692,39949))
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
$longUnits=[Collections.Generic.List[int]]::new()
for($i=0;$i -lt 16383;$i++){$longUnits.Add(0xD83D);$longUnits.Add(0xDE00)}
$longUnits.Add(0xD800)
foreach($spec in @(@('LEN',0,1),@('LENB',0,1),@('LEFT',16384,1),@('RIGHT',1,1),@('MID',1,16384),@('MIDB',1,32767))) {
    $fn=[string]$spec[0];$formula=if($fn -in @('LEN','LENB')){"=$fn(RC[-3])"}elseif($fn -in @('MID','MIDB')){"=$fn(RC[-3],RC[-1],RC[-2])"}else{"=$fn(RC[-3],RC[-2])"}
    $probes.Add([ordered]@{function=$fn;units=$longUnits.ToArray();count=$spec[1];start=$spec[2];formula=$formula})
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
[ordered]@{schema_version='w111-text-utf16-heldout-observations-v1';captured_utc=[DateTime]::UtcNow.ToString('o');
    runner_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash;
    profile=$profile;witnesses=$rows.ToArray()} | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $Out -Encoding utf8
Write-Output "Text UTF16 units: $($rows.Count) rows -> $Out"

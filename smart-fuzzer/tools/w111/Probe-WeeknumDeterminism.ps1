[CmdletBinding()]
param([Parameter(Mandatory)][string]$Out)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if(Test-Path -LiteralPath $Out){throw "Refusing to replace retained observations: $Out"}
Import-Module (Join-Path $PSScriptRoot '../CellRefBatch.psm1') -Force
$records=[Collections.Generic.List[object]]::new()
$environments=[Collections.Generic.List[object]]::new()
function Release-LocalCom([object]$value){
    if($null -ne $value -and [Runtime.InteropServices.Marshal]::IsComObject($value)){
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($value)
    }
}
for($session=0;$session -lt 2;$session++){
    $app=$book=$sheet=$first=$second=$result=$errorType=$null
    try{
        $app=New-Object -ComObject Excel.Application
        $app.Visible=$false;$app.DisplayAlerts=$false;$app.EnableEvents=$false
        $book=$app.Workbooks.Add();$book.PrecisionAsDisplayed=$false;$book.Date1904=$false
        $app.Calculation=-4135
        $sheet=$book.Worksheets.Item(1)
        $first=$sheet.Range('A1');$second=$sheet.Range('B1')
        $result=$sheet.Range('C1');$errorType=$sheet.Range('D1')
        $first.NumberFormat='General';$second.NumberFormat='General'
        $result.Formula2='=WEEKNUM(A1,B1)'
        $errorType.Formula2='=IFERROR(ERROR.TYPE(C1),0)'
        $environments.Add([ordered]@{session=$session;version=[string]$app.Version;build=[string]$app.Build;
            compatibility_version=[string]$book.CompatibilityVersion;date1904=[bool]$book.Date1904;
            precision_as_displayed=[bool]$book.PrecisionAsDisplayed;operating_system=[string]$app.OperatingSystem;
            channel='unverified';input_transport='Value2, exact per-observation readback'})
        foreach($threaded in @($true,$false)){
            $app.MultiThreadedCalculation.Enabled=$threaded
            for($pass=0;$pass -lt 3;$pass++){
                foreach($serial in @(0,1,59,60,43831,2958465)){
                    foreach($selector in @(-1,0,3,4,10,18,20,22,100,1,2,11,17,21)){
                        $first.Value2=[double]$serial;$second.Value2=[double]$selector
                        $a=Get-F64BitsHex ([double]$first.Value2);$b=Get-F64BitsHex ([double]$second.Value2)
                        if($a -ne (Get-F64BitsHex ([double]$serial)) -or $b -ne (Get-F64BitsHex ([double]$selector))){throw 'Numeric fixture readback changed'}
                        if([bool]$first.HasFormula -or [bool]$second.HasFormula){throw 'Fixture unexpectedly has a formula'}
                        switch($pass){0 {$sheet.Calculate()} 1 {$app.CalculateFull()} 2 {$app.CalculateFullRebuild()}}
                        $code=[double]$errorType.Value2
                        $outcome=ConvertTo-ExcelOutcome -Value $result.Value2 -ErrorType $(if($code -eq 0){$null}else{$code})
                        $records.Add([ordered]@{session=$session;threaded=[bool]$app.MultiThreadedCalculation.Enabled;
                            calculation_mode=[int]$app.Calculation;pass=$pass;args=@($a,$b);
                            formula_readback=[string]$result.Formula2;error_formula_readback=[string]$errorType.Formula2;
                            outcome=$outcome})
                    }
                }
            }
        }
    }finally{
        if($null -ne $book){$book.Close($false)}
        if($null -ne $app){$app.Quit()}
        foreach($item in @($errorType,$result,$second,$first,$sheet,$book,$app)){Release-LocalCom $item}
        [GC]::Collect();[GC]::WaitForPendingFinalizers()
    }
}
$parent=Split-Path -Parent $Out
if($parent -and !(Test-Path -LiteralPath $parent)){[void](New-Item -ItemType Directory -Path $parent -Force)}
[ordered]@{schema_version='w111.weeknum_direct_controls.v1';captured_utc=[DateTime]::UtcNow.ToString('o');
    script_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash;
    environments=$environments.ToArray();observations=$records.ToArray()} |
    ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $Out -Encoding utf8
Write-Output "Retained $($records.Count) direct WEEKNUM observations."

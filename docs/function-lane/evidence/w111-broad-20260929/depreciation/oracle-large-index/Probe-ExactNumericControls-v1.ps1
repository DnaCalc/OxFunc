[CmdletBinding()]
param([Parameter(Mandatory)][string]$Batch,[Parameter(Mandatory)][string]$Out,[int]$CalculateTimeoutSeconds=30)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if(Test-Path -LiteralPath $Out){throw "Refusing to overwrite retained capture $Out"}
Import-Module (Join-Path $PSScriptRoot '../CellRefBatch.psm1') -Force
Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
public sealed class OwnedExcelWatchdog : IDisposable {
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    readonly int pid; readonly long started; readonly Timer timer;
    public volatile bool Triggered;
    public OwnedExcelWatchdog(int id) {
        pid=id; started=Process.GetProcessById(id).StartTime.ToUniversalTime().Ticks;
        timer=new Timer(_=>{
            try {var p=Process.GetProcessById(pid);if(p.StartTime.ToUniversalTime().Ticks==started){Triggered=true;p.Kill();}}
            catch(ArgumentException) {} catch(InvalidOperationException) {}
        },null,Timeout.Infinite,Timeout.Infinite);
    }
    public void Arm(int milliseconds){timer.Change(milliseconds,Timeout.Infinite);}
    public void Disarm(){timer.Change(Timeout.Infinite,Timeout.Infinite);}
    public void Dispose(){timer.Dispose();}
}
'@
function Release-LocalCom([object]$value){if($null -ne $value -and [Runtime.InteropServices.Marshal]::IsComObject($value)){[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($value)}}
$data=Get-Content -LiteralPath $Batch -Raw | ConvertFrom-Json -Depth 30
if($data.function -notmatch '^[A-Z0-9.]+$'){throw 'Unexpected function identifier'}
$priorPids=@(Get-Process EXCEL -ErrorAction SilentlyContinue | ForEach-Object Id)
$app=$book=$sheet=$result=$errorCell=$watchdog=$null
$witnesses=[Collections.Generic.List[object]]::new()
$controls=[Collections.Generic.List[object]]::new()
$failure=$null;$environment=$null
$journal="$Out.partial-observations.jsonl"
try{
    $app=New-Object -ComObject Excel.Application
    $app.Visible=$false;$app.DisplayAlerts=$false;$app.EnableEvents=$false
    $ownedPid=[uint32]0
    [void][OwnedExcelWatchdog]::GetWindowThreadProcessId([IntPtr]$app.Hwnd,[ref]$ownedPid)
    if($ownedPid -eq 0 -or $priorPids -contains $ownedPid){throw 'Cannot establish a newly created Excel process; aborting owned-session probe'}
    $watchdog=[OwnedExcelWatchdog]::new([int]$ownedPid)
    $book=$app.Workbooks.Add();$book.PrecisionAsDisplayed=$false;$book.Date1904=$false
    if($app.Workbooks.Count -ne 1){throw 'Unexpected workbook ownership state'}
    $app.Calculation=-4135;$sheet=$book.Worksheets.Item(1)
    $result=$sheet.Range('I1');$errorCell=$sheet.Range('J1')
    $errorCell.Formula2='=IFERROR(ERROR.TYPE(I1),0)'
    $environment=[ordered]@{version=[string]$app.Version;build=[string]$app.Build;operating_system=[string]$app.OperatingSystem;
        compatibility_version=[string]$book.CompatibilityVersion;date1904=[bool]$book.Date1904;channel='unverified';owned_process_id=$ownedPid;
        precision_as_displayed=[bool]$book.PrecisionAsDisplayed;input_transport='Value2 with per-cell exact bit and HasFormula readback';calculation_timeout_seconds=$CalculateTimeoutSeconds}
    foreach($entry in $data.probes){
        $probe=$entry.probe;$targets=[Collections.Generic.List[string]]::new();$readback=[Collections.Generic.List[string]]::new()
        for($i=0;$i -lt $probe.args.Count;$i++){
            $target=([char](65+$i)).ToString()+'1';$targets.Add($target)
            $cell=$sheet.Range($target)
            try{
                $bits=[Convert]::ToUInt64(([string]$probe.args[$i]).Substring(2),16)
                $number=[BitConverter]::UInt64BitsToDouble($bits)
                $cell.NumberFormat='General';$cell.Value2=$number
                $actual=Get-F64BitsHex ([double]$cell.Value2)
                if($actual -cne ([string]$probe.args[$i]).ToLowerInvariant() -or [bool]$cell.HasFormula){throw "Exact fixture readback failed for $($probe.id)"}
                $readback.Add($actual)
            }finally{Release-LocalCom $cell}
        }
        $formula='='+$data.function+'('+($targets -join ',')+')'
        $result.Formula2=$formula
        if([string]$result.Formula2 -cne $formula){throw 'Formula readback changed'}
        $watchdog.Arm($CalculateTimeoutSeconds*1000)
        try{$sheet.Calculate()}finally{$watchdog.Disarm()}
        $code=[double]$errorCell.Value2
        $outcome=ConvertTo-ExcelOutcome -Value $result.Value2 -ErrorType $(if($code -eq 0){$null}else{$code})
        $expected=if($outcome.kind -eq 'number'){$outcome.bits_hex}else{'error:'+$outcome.code}
        $record=[ordered]@{id=[string]$probe.id;args=@($probe.args);expected_bits=$expected}
        $control=[ordered]@{id=[string]$probe.id;args=@($probe.args);readback=$readback.ToArray();formula_readback=[string]$result.Formula2;outcome=$outcome}
        $witnesses.Add($record);$controls.Add($control)
        $control | ConvertTo-Json -Depth 10 -Compress | Add-Content -LiteralPath $journal -Encoding utf8
    }
}catch{$failure=$_.Exception.Message}
finally{
    if($null -ne $watchdog){$watchdog.Dispose()}
    if($null -ne $book){try{$book.Close($false)}catch{}}
    if($null -ne $app){try{$app.Quit()}catch{}}
    foreach($item in @($errorCell,$result,$sheet,$book,$app)){Release-LocalCom $item}
    [GC]::Collect();[GC]::WaitForPendingFinalizers()
}
[ordered]@{function=$data.function;witnesses=$witnesses.ToArray();controls=$controls.ToArray();capture_failure=$failure;
    capture_provenance=[ordered]@{captured_utc=[DateTime]::UtcNow.ToString('o');environment=$environment;script_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash;input_sha256=(Get-FileHash -LiteralPath $Batch -Algorithm SHA256).Hash}} |
    ConvertTo-Json -Depth 15 | Set-Content -LiteralPath $Out -Encoding utf8
if($failure){throw "Retained $($witnesses.Count) observations before failure: $failure"}
Write-Output "Retained $($witnesses.Count) exact-readback numeric controls."

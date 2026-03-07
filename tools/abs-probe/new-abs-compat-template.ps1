param(
    [Parameter(Mandatory = $true)]
    [string]$Out
)

$ErrorActionPreference = "Stop"

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

$outPath = [System.IO.Path]::GetFullPath($Out)
$outDir = Split-Path -Path $outPath -Parent
if ($outDir -and -not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

$excel = $null
$workbook = $null
$worksheet = $null

try {
    $excel = New-Object -ComObject Excel.Application
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.ScreenUpdating = $false
    $excel.EnableEvents = $false

    $workbook = $excel.Workbooks.Add()
    $worksheet = $workbook.Worksheets.Item(1)
    $worksheet.Range("A1").Value2 = "abs-compat-template"

    # Use legacy XLS as compatibility-template seed.
    $workbook.SaveAs($outPath, 56)
    Write-Host "ABS compatibility template created: $outPath"
}
finally {
    Close-WorkbookSafe -Workbook $workbook
    Release-ComObjectSafe -Obj $worksheet
    Release-ComObjectSafe -Obj $workbook
    if ($excel -ne $null) {
        try { $excel.Quit() } catch {}
    }
    Release-ComObjectSafe -Obj $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

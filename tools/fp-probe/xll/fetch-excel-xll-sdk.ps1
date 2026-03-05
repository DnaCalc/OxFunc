param(
    [string]$OutRoot = ".tmp/excelxllsdk_extracted"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "..\..\..")
$outRootPath = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $OutRoot))
if (-not (Test-Path $outRootPath)) {
    New-Item -ItemType Directory -Path $outRootPath | Out-Null
}

$msiUrl = "https://download.microsoft.com/download/d/a/c/dac9c321-279c-4625-b381-bcb044eef7b2/excelxllsdk.msi"
$msiPath = Join-Path $outRootPath "excelxllsdk.msi"

Write-Host "Downloading Excel XLL SDK MSI..."
Invoke-WebRequest -Uri $msiUrl -OutFile $msiPath

Write-Host "Extracting MSI..."
Start-Process msiexec.exe -ArgumentList @("/a", "`"$msiPath`"", "/qn", "TARGETDIR=`"$outRootPath`"") -Wait -NoNewWindow

$sdkDir = Join-Path $outRootPath "2013 Office System Developer Resources\Excel2013XLLSDK"
if (-not (Test-Path (Join-Path $sdkDir "INCLUDE\XLCALL.H"))) {
    throw "SDK extraction failed: missing INCLUDE\\XLCALL.H under $sdkDir"
}

Write-Host "Excel XLL SDK ready:"
Write-Host $sdkDir
Write-Host "Set EXCEL_XLL_SDK_DIR to this path (or rely on crate default resolution)."

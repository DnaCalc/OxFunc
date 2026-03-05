param(
    [string]$OutRoot = ".tmp/excelxllsdk_extracted",

    [switch]$SkipHashCheck,

    [switch]$UpdateLock
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -Path (Join-Path $PSScriptRoot "..\..\..")
$outRootPath = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $OutRoot))
if (-not (Test-Path $outRootPath)) {
    New-Item -ItemType Directory -Path $outRootPath | Out-Null
}

$lockPath = Join-Path $PSScriptRoot "EXCEL_XLL_SDK_LOCK.json"
if (-not (Test-Path $lockPath)) {
    throw "Missing lock file: $lockPath"
}
$lock = Get-Content $lockPath -Raw | ConvertFrom-Json

$msiUrl = [string]$lock.source_url
$msiPath = Join-Path $outRootPath "excelxllsdk.msi"

Write-Host "Downloading Excel XLL SDK MSI..."
Invoke-WebRequest -Uri $msiUrl -OutFile $msiPath

if (-not $SkipHashCheck) {
    Write-Host "Hash verification enabled."
}
$actualHash = (Get-FileHash -Path $msiPath -Algorithm SHA256).Hash.ToUpperInvariant()
$expectedHash = ([string]$lock.sha256).ToUpperInvariant()
if ($actualHash -ne $expectedHash) {
    if ($UpdateLock) {
        Write-Warning "SDK MSI hash drift detected. Updating lock: $expectedHash -> $actualHash"
        $lock.sha256 = $actualHash
        $lock | ConvertTo-Json -Depth 8 | Set-Content -Path $lockPath -Encoding utf8
    }
    elseif ($SkipHashCheck) {
        Write-Warning "SDK MSI hash mismatch ignored by -SkipHashCheck. expected=$expectedHash actual=$actualHash"
    }
    else {
        throw "SDK MSI hash mismatch. expected=$expectedHash actual=$actualHash"
    }
}
elseif ($UpdateLock) {
    Write-Host "SDK lock hash already current."
}

Write-Host "Extracting MSI..."
Start-Process msiexec.exe -ArgumentList @("/a", "`"$msiPath`"", "/qn", "TARGETDIR=`"$outRootPath`"") -Wait -NoNewWindow

$sdkSubPath = [string]$lock.sdk_subpath
$sdkSubPathWin = $sdkSubPath -replace '/', '\'
$sdkDir = Join-Path $outRootPath $sdkSubPathWin
foreach ($required in $lock.required_files) {
    $requiredWin = ([string]$required) -replace '/', '\'
    $requiredPath = Join-Path $sdkDir $requiredWin
    if (-not (Test-Path $requiredPath)) {
        throw "SDK extraction failed: missing required file $requiredPath"
    }
}

Write-Host "Excel XLL SDK ready:"
Write-Host $sdkDir
Write-Host "Set EXCEL_XLL_SDK_DIR to this path (or rely on crate default resolution)."

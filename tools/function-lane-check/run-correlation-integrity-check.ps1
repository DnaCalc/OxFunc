param(
    [string]$CorrelationLedger = "docs/function-lane/FUNCTION_SLICE_CORRELATION_LEDGER.csv",
    [string]$EvidenceRegistry = "docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md",
    [string]$OutReport = ".tmp/function-slice-correlation-integrity-report.csv"
)

$ErrorActionPreference = "Stop"

function Split-Ids {
    param([string]$Raw)
    if ([string]::IsNullOrWhiteSpace($Raw)) { return @() }
    return @($Raw -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function Resolve-LeanModulePath {
    param([string]$LeanModule)
    $relative = ($LeanModule -replace '\.', '/') + ".lean"
    return [System.IO.Path]::GetFullPath((Join-Path "formal/lean" $relative))
}

function Resolve-RustModulePath {
    param([string]$RustModule)
    $prefix = "oxfunc_core::"
    if (-not $RustModule.StartsWith($prefix)) { return "" }
    $relative = ($RustModule.Substring($prefix.Length) -replace '::', '\') + ".rs"
    return [System.IO.Path]::GetFullPath((Join-Path "crates/oxfunc_core/src" $relative))
}

function Add-Issue {
    param(
        [System.Collections.Generic.List[object]]$Issues,
        [string]$Severity,
        [string]$FunctionId,
        [string]$Field,
        [string]$Message
    )
    $Issues.Add([PSCustomObject]@{
        severity = $Severity
        function_id = $FunctionId
        field = $Field
        message = $Message
    })
}

$correlationPath = (Resolve-Path -Path $CorrelationLedger -ErrorAction Stop).Path
$registryPath = (Resolve-Path -Path $EvidenceRegistry -ErrorAction Stop).Path

$rows = Import-Csv -Path $correlationPath
if (-not $rows -or $rows.Count -eq 0) {
    throw "Correlation ledger has no rows: $correlationPath"
}

$registryContent = Get-Content -Path $registryPath -Raw
$registryMatches = [regex]::Matches($registryContent, '\|\s+`([^`]+)`\s+\|')
$knownEvidenceIds = New-Object System.Collections.Generic.HashSet[string]
foreach ($m in $registryMatches) {
    [void]$knownEvidenceIds.Add($m.Groups[1].Value)
}

$issues = New-Object System.Collections.Generic.List[object]

foreach ($row in $rows) {
    $functionId = [string]$row.function_id
    $leanModule = [string]$row.lean_module
    $rustModule = [string]$row.rust_module

    $leanPath = Resolve-LeanModulePath -LeanModule $leanModule
    if (-not (Test-Path $leanPath)) {
        Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "lean_module" -Message "Lean module path does not exist: $leanPath"
    }
    else {
        $leanContent = Get-Content -Path $leanPath -Raw
        foreach ($theoremId in (Split-Ids -Raw ([string]$row.lean_theorem_ids))) {
            if ($theoremId.StartsWith("TBD-")) { continue }
            $pattern = "(theorem|lemma|def)\s+$([regex]::Escape($theoremId))\b"
            if (-not ([regex]::IsMatch($leanContent, $pattern))) {
                Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "lean_theorem_ids" -Message "Theorem/lemma/def id not found in module '$leanModule': $theoremId"
            }
        }
    }

    $rustPath = Resolve-RustModulePath -RustModule $rustModule
    if ([string]::IsNullOrWhiteSpace($rustPath)) {
        Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "rust_module" -Message "Unsupported rust_module prefix: $rustModule"
    }
    elseif (-not (Test-Path $rustPath)) {
        Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "rust_module" -Message "Rust module path does not exist: $rustPath"
    }
    else {
        $rustContent = Get-Content -Path $rustPath -Raw
        foreach ($testId in (Split-Ids -Raw ([string]$row.rust_test_ids))) {
            if ($testId.StartsWith("TBD-")) { continue }
            $pattern = "fn\s+$([regex]::Escape($testId))\s*\("
            if (-not ([regex]::IsMatch($rustContent, $pattern))) {
                Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "rust_test_ids" -Message "Rust test function id not found in module '$rustModule': $testId"
            }
        }
    }

    foreach ($evidenceId in (Split-Ids -Raw ([string]$row.evidence_ids))) {
        if ($evidenceId.StartsWith("TBD-")) { continue }
        if (-not $knownEvidenceIds.Contains($evidenceId)) {
            Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "evidence_ids" -Message "Evidence id not present in registry: $evidenceId"
        }
    }

    if ([string]::IsNullOrWhiteSpace([string]$row.excel_version_scope)) {
        Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "excel_version_scope" -Message "Excel version scope is empty."
    }
    if ([string]::IsNullOrWhiteSpace([string]$row.compatibility_version_scope)) {
        Add-Issue -Issues $issues -Severity "error" -FunctionId $functionId -Field "compatibility_version_scope" -Message "Compatibility version scope is empty."
    }
}

$outReportPath = [System.IO.Path]::GetFullPath($OutReport)
$outReportDir = Split-Path -Path $outReportPath -Parent
if ($outReportDir -and -not (Test-Path $outReportDir)) {
    New-Item -ItemType Directory -Path $outReportDir | Out-Null
}
$issues | Export-Csv -Path $outReportPath -NoTypeInformation -Encoding UTF8

if ($issues.Count -gt 0) {
    Write-Host "Correlation integrity check: FAILED ($($issues.Count) issue(s))."
    Write-Host "Report: $outReportPath"
    exit 1
}

Write-Host "Correlation integrity check: PASS."
Write-Host "Report: $outReportPath"

param(
    [string]$HeaderPath = ".tmp/excelxllsdk_extracted/2013 Office System Developer Resources/Excel2013XLLSDK/INCLUDE/XLCALL.H",
    [string]$CatalogPath = "docs/function-lane/FUNCTION_CATALOG_CURRENT_BASELINE_LOCAL.csv",
    [string]$OutCsv = "docs/function-lane/XLCALL_CODE_CATALOG.csv"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\")
Set-Location $repoRoot

$resolvedHeader = Resolve-Path $HeaderPath
$headerRef = $resolvedHeader.Path.Replace((Resolve-Path ".").Path + "\", "").Replace("\", "/")
$catalog = Import-Csv $CatalogPath
$catalogNames = @{}
foreach ($row in $catalog) {
    $catalogNames[$row.function_name] = $true
}

function Get-XlcallCategory {
    param([string]$Symbol)
    $auxiliarySymbols = @(
        "xlFree", "xlStack", "xlCoerce", "xlSet", "xlSheetId", "xlSheetNm", "xlAbort",
        "xlGetInst", "xlGetHwnd", "xlGetName", "xlEnableXLMsgs", "xlDisableXLMsgs",
        "xlDefineBinaryName", "xlGetBinaryName", "xlGetFmlaInfo", "xlGetMouseInfo",
        "xlAsyncReturn", "xlEventRegister", "xlRunningOnCluster", "xlGetInstPtr"
    )
    if ($Symbol -eq "xlUDF") { return "udf_sentinel" }
    if ($Symbol.StartsWith("xlf")) { return "built_in_function" }
    if ($Symbol.StartsWith("xlc")) { return "command" }
    if ($auxiliarySymbols -contains $Symbol) { return "auxiliary_function" }
    return "other"
}

function Get-OxfuncCandidateName {
    param([string]$Symbol)

    switch ($Symbol) {
        "xlfRegisterId" { return "REGISTER.ID" }
        default {
            if (-not $Symbol.StartsWith("xlf")) { return "" }
            return ($Symbol.Substring(3).ToUpperInvariant() -replace "_", ".")
        }
    }
}

function Get-NumericCode {
    param([string]$Expression)

    $trimmed = $Expression.Trim()
    if ($trimmed -match "^\((?<num>\d+)\s*\|\s*(?<flag>xl\w+)\)$") {
        return [ordered]@{
            numeric_code = [int]$Matches.num
            flag_kind = $Matches.flag
        }
    }
    if ($trimmed -match "^(?<num>\d+)$") {
        return [ordered]@{
            numeric_code = [int]$Matches.num
            flag_kind = ""
        }
    }
    return [ordered]@{
        numeric_code = ""
        flag_kind = ""
    }
}

$rows = New-Object System.Collections.Generic.List[object]
foreach ($line in Get-Content $resolvedHeader) {
    if ($line -notmatch '^#define\s+(?<symbol>xl[A-Za-z0-9_]+)\s+(?<expr>.+?)\s*(?:/\*.*)?$') {
        continue
    }

    $symbol = $Matches.symbol
    $expr = $Matches.expr.Trim()
    $category = Get-XlcallCategory $symbol
    if ($category -eq "other") {
        continue
    }

    $code = Get-NumericCode $expr
    $candidateName = Get-OxfuncCandidateName $symbol
    $stableId = ""
    $matchStatus = ""
    $note = ""

    switch ($category) {
        "built_in_function" {
            if ($candidateName -and $catalogNames.ContainsKey($candidateName)) {
                $stableId = "FUNC.$candidateName"
                $matchStatus = "matched_current_catalog"
            } else {
                $matchStatus = "not_in_current_catalog"
                $note = "No exact current-baseline OxFunc catalog row matched the XLCALL built-in symbol."
            }
            if ($symbol -in @("xlfRegister", "xlfCall", "xlfRegisterId")) {
                $note = "Registration/invocation seam row; see W046."
            }
        }
        "command" {
            $matchStatus = "command_outside_function_catalog"
            $note = "Excel C API command code; not a worksheet function catalog row in OxFunc."
        }
        "auxiliary_function" {
            $matchStatus = "auxiliary_c_api_only"
            $note = "Excel C API auxiliary callback/special function; not a worksheet function catalog row in OxFunc."
        }
        "udf_sentinel" {
            $matchStatus = "udf_dispatch_sentinel"
            $note = "Excel C API user-defined function sentinel, not a built-in worksheet function row."
        }
    }

    $rows.Add([pscustomobject][ordered]@{
        source_header_ref = $headerRef
        xlcall_category = $category
        xlcall_symbol = $symbol
        xlcall_numeric_code = $code.numeric_code
        xlcall_flag_kind = $code.flag_kind
        xlcall_code_expression = $expr
        oxfunc_surface_stable_id = $stableId
        oxfunc_canonical_surface_name = $candidateName
        oxfunc_match_status = $matchStatus
        note = $note
    })
}

$rows |
    Sort-Object xlcall_category, xlcall_numeric_code, xlcall_symbol |
    Export-Csv -NoTypeInformation -Encoding UTF8 $OutCsv

$summary = $rows | Group-Object xlcall_category | ForEach-Object { "{0}={1}" -f $_.Name, $_.Count }
Write-Host ("wrote {0} rows: {1}" -f $rows.Count, ($summary -join ", "))

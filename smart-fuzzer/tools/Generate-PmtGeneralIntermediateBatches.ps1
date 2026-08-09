[CmdletBinding(PositionalBinding = $false)]
param(
    [ValidateSet("Exp", "Ln")]
    [string] $Mode = "Exp",
    [string] $Corpus = "smart-fuzzer/work/w109/G6-solvers/em_consolidated.csv",
    [string] $ExpBatch = "smart-fuzzer/work/w109/G6-solvers/batch-pmt-general-exp-20260809.json",
    [string] $ExpAnswers = "smart-fuzzer/work/w109/G6-solvers/answers-pmt-general-exp-20260809.json",
    [string] $LnBatch = "smart-fuzzer/work/w109/G6-solvers/batch-pmt-general-ln-20260809.json",
    [string] $Meta = "smart-fuzzer/work/w109/G6-solvers/meta-pmt-general-intermediates-20260809.csv"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-RepositoryPath {
    param([Parameter(Mandatory)] [string] $Path)
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    $repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    return [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $Path))
}

function Write-JsonUtf8NoBom {
    param(
        [Parameter(Mandatory)] [object] $Value,
        [Parameter(Mandatory)] [string] $Path
    )
    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    $json = $Value | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($Path, $json + [Environment]::NewLine, [System.Text.UTF8Encoding]::new($false))
}

$corpusPath = Resolve-RepositoryPath $Corpus
$expBatchPath = Resolve-RepositoryPath $ExpBatch
$expAnswersPath = Resolve-RepositoryPath $ExpAnswers
$lnBatchPath = Resolve-RepositoryPath $LnBatch
$metaPath = Resolve-RepositoryPath $Meta

$rows = @(Import-Csv -LiteralPath $corpusPath | Where-Object { $_.src -eq "gen" })
if ($rows.Count -ne 90) {
    throw "Expected 90 general-rate PMT rows, found $($rows.Count)"
}

if ($Mode -eq "Exp") {
    $metaRows = [System.Collections.Generic.List[object]]::new()
    $probes = [System.Collections.Generic.List[object]]::new()
    for ($index = 0; $index -lt $rows.Count; $index++) {
        $row = $rows[$index]
        $id = "pmt-gen-exp-{0:D4}" -f $index
        $tauBits = "0x$($row.tau_bits.ToLowerInvariant())"
        $probes.Add([ordered]@{
            probe = [ordered]@{
                id = $id
                args = @($tauBits)
                label = "general-rate PMT tau EXP; source row $index"
            }
        })
        $metaRows.Add([pscustomobject][ordered]@{
            index = $index
            exp_id = $id
            ln_id = "pmt-gen-ln-{0:D4}" -f $index
            r_bits = "0x$($row.r_bits.ToLowerInvariant())"
            n = $row.n
            tau_bits = $tauBits
            em_pinned = "0x$($row.em_pinned.ToLowerInvariant())"
            kahan = "0x$($row.kahan.ToLowerInvariant())"
        })
    }
    $batch = [ordered]@{
        function = "EXP"
        row_id = "pmt-general-intermediate-exp-20260809"
        selection = "all 90 pre-existing general-rate rows from em_consolidated.csv; no answer-based selection"
        probes = @($probes)
    }
    Write-JsonUtf8NoBom -Value $batch -Path $expBatchPath
    $metaRows | Export-Csv -LiteralPath $metaPath -NoTypeInformation -Encoding utf8NoBOM
    Write-Output "Wrote EXP batch: $expBatchPath ($($probes.Count) rows)"
    Write-Output "Wrote metadata: $metaPath"
    return
}

if (-not (Test-Path -LiteralPath $expAnswersPath)) {
    throw "EXP answers not found: $expAnswersPath"
}
$answers = Get-Content -LiteralPath $expAnswersPath -Raw | ConvertFrom-Json -Depth 20
if ($answers.function -ne "EXP") {
    throw "Expected EXP answers, found '$($answers.function)'"
}
$witnesses = @($answers.witnesses)
if ($witnesses.Count -ne $rows.Count) {
    throw "Expected $($rows.Count) EXP witnesses, found $($witnesses.Count)"
}

$lnProbes = [System.Collections.Generic.List[object]]::new()
for ($index = 0; $index -lt $rows.Count; $index++) {
    $witness = $witnesses[$index]
    $expectedExpId = "pmt-gen-exp-{0:D4}" -f $index
    $expectedTau = "0x$($rows[$index].tau_bits.ToLowerInvariant())"
    if ($witness.id -ne $expectedExpId) {
        throw "EXP witness id mismatch at ${index}: '$($witness.id)' != '$expectedExpId'"
    }
    if (@($witness.args).Count -ne 1 -or $witness.args[0] -ne $expectedTau) {
        throw "EXP witness argument mismatch at $index"
    }
    $expBits = [string]$witness.expected_bits
    if ($expBits -notmatch '^0x[0-9a-fA-F]{16}$') {
        throw "EXP witness $expectedExpId is not numeric bits: '$expBits'"
    }
    $lnProbes.Add([ordered]@{
        probe = [ordered]@{
            id = "pmt-gen-ln-{0:D4}" -f $index
            args = @($expBits.ToLowerInvariant())
            label = "LN of captured EXP for $expectedExpId"
        }
    })
}

$lnBatchObject = [ordered]@{
    function = "LN"
    row_id = "pmt-general-intermediate-ln-20260809"
    selection = "LN inputs are the exact live EXP outputs from the frozen 90-row general-rate batch"
    source_exp_answers = Split-Path -Leaf $expAnswersPath
    probes = @($lnProbes)
}
Write-JsonUtf8NoBom -Value $lnBatchObject -Path $lnBatchPath
Write-Output "Wrote LN batch: $lnBatchPath ($($lnProbes.Count) rows)"

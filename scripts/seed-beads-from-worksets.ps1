param(
    [switch]$ForceReinit
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

Push-Location (Join-Path $PSScriptRoot "..")
try {
    if ($ForceReinit -and (Test-Path ".beads")) {
        & br init --prefix oxf --force | Out-Null
    }
    elseif (-not (Test-Path ".beads")) {
        & br init --prefix oxf | Out-Null
    }
    else {
        $hasIssues = $false
        if (Test-Path ".beads/issues.jsonl") {
            $issueLine = Get-Content ".beads/issues.jsonl" | Where-Object { $_.Trim() -ne "" } | Select-Object -First 1
            $hasIssues = [bool]$issueLine
        }

        if ($hasIssues) {
            throw "seed-beads-from-worksets: .beads already contains issues. Re-run with -ForceReinit only if you want to overwrite the existing graph."
        }

        & br init --prefix oxf | Out-Null
    }

    function New-Issue {
        param(
            [Parameter(Mandatory = $true)][string]$Title,
            [Parameter(Mandatory = $true)][string]$Type,
            [Parameter(Mandatory = $true)][string]$Priority,
            [Parameter(Mandatory = $true)][string]$Labels,
            [Parameter(Mandatory = $true)][string]$Description,
            [string]$Parent,
            [string[]]$Deps = @(),
            [string]$Acceptance = ""
        )

        $createArgs = @(
            "create",
            "--silent",
            "--title", $Title,
            "--type", $Type,
            "--priority", $Priority,
            "--labels", $Labels,
            "--description", $Description
        )

        if ($Parent) {
            $createArgs += @("--parent", $Parent)
        }

        $id = (& br @createArgs).Trim()
        if (-not $id) {
            throw "seed-beads-from-worksets: failed to create issue '$Title'"
        }

        if ($Acceptance) {
            & br update $id --acceptance-criteria $Acceptance | Out-Null
        }

        foreach ($dep in $Deps) {
            if ($dep) {
                & br dep add $id $dep | Out-Null
            }
        }

        return $id
    }

    $ids = @{}

    $ids.w070_archive = New-Issue `
        -Title "W070 Active-tree reduction and archive-wave execution" `
        -Type "epic" `
        -Priority "P1" `
        -Labels "W070,migration" `
        -Description "Run scope: execute the first active-tree reduction waves from the W070 triage register without breaking current truth surfaces." `
        -Acceptance "Evidence: at least one archive/removal wave is explicitly staged from the triage register and the next ready reduction set is visible in the bead graph."

    $ids.w070_bridge = New-Issue `
        -Title "W070 Transitional tracker narrowing and truth-surface cleanup" `
        -Type "epic" `
        -Priority "P1" `
        -Labels "W070,migration" `
        -Description "Run scope: narrow or retire bridge tracker surfaces that are now superseded by the live bead workspace and post-park doctrine." `
        -Acceptance "Evidence: the intended fate of CURRENT_BLOCKERS, docs/worksets/README, and the feature register is reflected in code or docs without restoring ad hoc execution notes."

    $ids.w070_next = New-Issue `
        -Title "W070 First post-migration workset rollout" `
        -Type "epic" `
        -Priority "P1" `
        -Labels "W070,migration" `
        -Description "Run scope: prepare the first real post-migration execution lane so OxFunc can leave migration-only work and execute W069 under beads." `
        -Acceptance "Evidence: W069 has explicit child epics and a believable ready path under the live bead graph."

    $ids.w070_t1 = New-Issue `
        -Title "Draft the first Phase E archive/removal wave from the W070 triage register" `
        -Type "task" `
        -Priority "P1" `
        -Labels "W070,migration,archive" `
        -Parent $ids.w070_archive `
        -Description "Run scope: turn the triage register into the first explicit archive/removal wave with named file families, keep-vs-remove rationale, and risk notes for current truth surfaces." `
        -Acceptance "Evidence: W070 or a linked live register names the first archive wave explicitly and the selected files can be removed without narrative memory."

    $ids.w070_t2 = New-Issue `
        -Title "Narrow CURRENT_BLOCKERS and workset-index bridge posture after bead bootstrap" `
        -Type "task" `
        -Priority "P1" `
        -Labels "W070,migration,cleanup" `
        -Parent $ids.w070_bridge `
        -Deps @($ids.w070_t1) `
        -Description "Run scope: narrow the remaining bridge tracker posture now that .beads is live, keeping only what still has a real current role." `
        -Acceptance "Evidence: active doctrine and surviving tracker files agree that ordinary blockers and execution truth live in .beads, and any retained prose tracker is explicitly exceptional."

    $ids.w070_t3 = New-Issue `
        -Title "Roll out W069 child epics and first execution beads" `
        -Type "task" `
        -Priority "P1" `
        -Labels "W070,migration,W069" `
        -Parent $ids.w070_next `
        -Deps @($ids.w070_t1, $ids.w070_t2) `
        -Description "Run scope: convert the W069 plan into explicit epics and first execution beads so the first post-migration workset can start without ad hoc planning notes." `
        -Acceptance "Evidence: W069 execution epics and first child beads exist in the graph with dependencies and closure evidence."

    $ids.w044_bridge = New-Issue `
        -Title "W044 V1 export integrity and V2 bridge narrowing" `
        -Type "epic" `
        -Priority "P2" `
        -Labels "W044,export" `
        -Description "Run scope: keep the V1 export honest while narrowing the retained bridge into the later V2 witness direction." `
        -Acceptance "Evidence: the retained V1 bridge work is explicit and does not duplicate W069 execution state."

    $ids.w049_runtime = New-Issue `
        -Title "W049 Runtime provider and snapshot model narrowing" `
        -Type "epic" `
        -Priority "P2" `
        -Labels "W049,runtime-model" `
        -Description "Run scope: narrow the runtime provider and snapshot model toward the witness-bearing post-park direction without reopening the closed seam-freeze packet." `
        -Acceptance "Evidence: the runtime model follow-on work is explicit in the graph and tied to concrete surviving truth surfaces."

    $ids.w069_schema = New-Issue `
        -Title "W069 SemanticWitnessEntry schema and stability tiers" `
        -Type "epic" `
        -Priority "P1" `
        -Labels "W069,witness" `
        -Description "Run scope: define the V2 witness entry shape, field tiers, and stability expectations before broad generator or family rollout work begins." `
        -Acceptance "Evidence: the schema and stability tiers exist in the active truth surfaces and later W069 beads can depend on them."

    $ids.w069_pipeline = New-Issue `
        -Title "W069 Witness generation and export bridge pipeline" `
        -Type "epic" `
        -Priority "P2" `
        -Labels "W069,witness" `
        -Description "Run scope: map the current V1 export and W049 runtime model into a concrete V2 generation path." `
        -Acceptance "Evidence: a concrete generation path is defined and tied to retained V1/W049 truth surfaces."

    $ids.w069_seed = New-Issue `
        -Title "W069 First seeded function-family witness rollout" `
        -Type "epic" `
        -Priority "P2" `
        -Labels "W069,witness" `
        -Description "Run scope: choose the first function-family slice for V2 witness output and tie it to concrete evidence and formal refs." `
        -Acceptance "Evidence: the first family slice is explicit and later implementation beads can land without reopening the whole W069 design."

    $ids.w069_t1 = New-Issue `
        -Title "Define the SemanticWitnessEntry schema and stability tiers in active contract docs" `
        -Type "task" `
        -Priority "P1" `
        -Labels "W069,witness" `
        -Parent $ids.w069_schema `
        -Deps @($ids.w070_t3) `
        -Description "Run scope: define the V2 witness entry fields, provenance categories, and stability tiers in the live OxFunc contract surfaces." `
        -Acceptance "Evidence: the schema lives in active docs and downstream-facing surfaces can cite it directly."

    $ids.w069_t2 = New-Issue `
        -Title "Map V1 export and W049 runtime model into a witness-bearing V2 generator shape" `
        -Type "task" `
        -Priority "P2" `
        -Labels "W069,witness,W044,W049" `
        -Parent $ids.w069_pipeline `
        -Deps @($ids.w069_t1) `
        -Description "Run scope: define how current V1 rows and runtime provider fields project into V2 witness generation without duplicating current-truth ownership." `
        -Acceptance "Evidence: the bridge path from V1 and W049 into V2 is explicit and reviewable."

    $ids.w069_t3 = New-Issue `
        -Title "Choose the first seeded family and map its help, evidence, and formal refs" `
        -Type "task" `
        -Priority "P2" `
        -Labels "W069,witness" `
        -Parent $ids.w069_seed `
        -Deps @($ids.w069_t2) `
        -Description "Run scope: pick the first family slice for V2 output and map the help, evidence, and formal-reference surfaces it will consume." `
        -Acceptance "Evidence: the first seeded family is explicit and the source surfaces for its witness payload are named."

    $ids.w044_t1 = New-Issue `
        -Title "Reconcile retained W044 V1 surfaces with the V2 bridge plan" `
        -Type "task" `
        -Priority "P2" `
        -Labels "W044,export,W069" `
        -Parent $ids.w044_bridge `
        -Deps @($ids.w069_t1) `
        -Description "Run scope: narrow the retained W044 readme, policy, and bridge language so they support W069 without pretending V1 remains the long-term execution center." `
        -Acceptance "Evidence: W044 surfaces explicitly describe their retained role relative to W069 and no stale V1-as-destination wording remains."

    $ids.w049_t1 = New-Issue `
        -Title "Narrow the W049 runtime provider and snapshot model toward V2 witness consumption" `
        -Type "task" `
        -Priority "P2" `
        -Labels "W049,runtime-model,W069" `
        -Parent $ids.w049_runtime `
        -Deps @($ids.w069_t1) `
        -Description "Run scope: express the retained W049 runtime model in the narrower post-park direction that can support V2 witness consumption." `
        -Acceptance "Evidence: W049 active truth surfaces explicitly describe the retained runtime model role relative to W069."

    Write-Host "seed-beads-from-worksets: created $($ids.Count) issues"
}
finally {
    Pop-Location
}

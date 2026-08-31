# Copy live firehorse ERFC campaign artifacts into the GitHub snapshot dir.
# Does not commit. From repo root: pwsh -File smart-fuzzer/tools/calc_graph_racer/snapshot-erfc-campaign.ps1
$ErrorActionPreference = "Stop"
$dest = "docs/function-lane/w109-erfc-campaign"
$src = "dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
$files = @(
    "STATUS.md",
    "REGION_MAP.md",
    "status.json",
    "checkpoint.json",
    "leaders.jsonl",
    "pin-hits.jsonl",
    "R0.md",
    "R0c.md",
    "R0aabb.md",
    "R1base.md"
)
$paths = $files | ForEach-Object { "$src/$_" }
scp -o BatchMode=yes @paths $dest/
Write-Host "copied $($files.Count) files to $dest"

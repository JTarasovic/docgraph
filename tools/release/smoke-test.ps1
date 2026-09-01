param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("windows-x86_64", "linux-x86_64")]
    [string] $Target,

    [Parameter(Mandatory = $true)]
    [string] $Version,

    [Parameter(Mandatory = $true)]
    [string] $ArchivePath
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$versionNumber = $Version.TrimStart("v")
$archive = [IO.Path]::GetFullPath($ArchivePath)
if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
    throw "Release archive does not exist: $archive"
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$scratch = Join-Path ([IO.Path]::GetTempPath()) "docgraph-release-smoke-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $scratch | Out-Null

try {
    if ($Target -eq "windows-x86_64") {
        Expand-Archive -LiteralPath $archive -DestinationPath $scratch
        $executableName = "docgraph.exe"
    } else {
        tar -xzf $archive -C $scratch
        $executableName = "docgraph"
    }
    $executable = Get-ChildItem -LiteralPath $scratch -Filter $executableName -File -Recurse |
        Select-Object -First 1 -ExpandProperty FullName
    if (-not $executable) {
        throw "Archive does not contain $executableName."
    }
    $runtimeName = "docgraph-logic-runtime"
    $runtime = Join-Path (Split-Path -Parent $executable) $runtimeName
    if (-not (Test-Path -LiteralPath $runtime -PathType Leaf)) {
        throw "Archive does not place $runtimeName beside $executableName."
    }
    foreach ($required in @("LICENSE", "README.md", "THIRD_PARTY_LICENSES", "skills/docgraph/skill.toml")) {
        if (-not (Test-Path -LiteralPath (Join-Path (Split-Path -Parent $executable) $required))) {
            throw "Archive is missing $required."
        }
    }

    $reportedVersion = (& $executable --version).Trim()
    if ($reportedVersion -ne "docgraph $versionNumber") {
        throw "Packaged CLI version '$reportedVersion' does not match '$versionNumber'."
    }
    & $executable --help *> $null

    $fixture = Join-Path $repository "fixtures\synthetic"
    $workspace = Join-Path $scratch "workspace"
    Copy-Item -LiteralPath $fixture -Destination $workspace -Recurse
    Push-Location $workspace
    try {
        $missingSkillDetected = $false
        try {
            & $executable instructions check *> $null
        } catch {
            $missingSkillDetected = $true
        }
        if (-not $missingSkillDetected) {
            throw "A workspace without the portable skill unexpectedly passed instructions check."
        }
        & $executable instructions sync --dry-run *> $null
        if (Test-Path -LiteralPath (Join-Path $workspace "skills/docgraph/SKILL.md")) {
            throw "Instruction dry-run wrote the portable skill."
        }
        & $executable instructions sync *> $null
        & $executable instructions check *> $null
        & $executable validate *> $null
        & $executable query scalar_values *> $null
        & $executable search "florp" *> $null
    } finally {
        Pop-Location
    }
    Write-Output "release smoke test passed: $Target $versionNumber"
} finally {
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}

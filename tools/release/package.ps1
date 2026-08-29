param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("windows-x86_64", "linux-x86_64")]
    [string] $Target,

    [Parameter(Mandatory = $true)]
    [string] $Version,

    [Parameter(Mandatory = $true)]
    [string] $CliPath,

    [Parameter(Mandatory = $true)]
    [string] $LogicRuntimePath,

    [Parameter(Mandatory = $true)]
    [string] $LogicLicensesPath,

    [string] $OutputDirectory = (Join-Path $PSScriptRoot "..\..\target\release-artifacts")
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$versionNumber = $Version.TrimStart("v")
if ($versionNumber -notmatch '^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') {
    throw "Release version must be a semantic version, found '$Version'."
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$cli = [IO.Path]::GetFullPath($CliPath)
$logicRuntime = [IO.Path]::GetFullPath($LogicRuntimePath)
$logicLicenses = [IO.Path]::GetFullPath($LogicLicensesPath)
$output = [IO.Path]::GetFullPath($OutputDirectory)
$readme = Join-Path $repository "README.md"
$license = Join-Path $repository "LICENSE"

foreach ($required in @($cli, $logicRuntime, $logicLicenses, $readme, $license)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Required release input does not exist: $required"
    }
}

$reportedVersion = (& $cli --version).Trim()
if ($reportedVersion -ne "docgraph $versionNumber") {
    throw "CLI version '$reportedVersion' does not match release version '$versionNumber'."
}

$archiveBase = "docgraph-v$versionNumber-$Target"
$stagingParent = Join-Path $repository "target\release-staging"
$staging = Join-Path $stagingParent $archiveBase
$resolvedStagingParent = [IO.Path]::GetFullPath($stagingParent)
$resolvedStaging = [IO.Path]::GetFullPath($staging)
if (-not $resolvedStaging.StartsWith($resolvedStagingParent + [IO.Path]::DirectorySeparatorChar)) {
    throw "Unsafe release staging path: $resolvedStaging"
}

if (Test-Path -LiteralPath $resolvedStaging) {
    Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $resolvedStaging | Out-Null

try {
    $cliName = if ($Target -eq "windows-x86_64") { "docgraph.exe" } else { "docgraph" }
    $runtimeName = if ($Target -eq "windows-x86_64") { "docgraph-logic-runtime.exe" } else { "docgraph-logic-runtime" }
    Copy-Item -LiteralPath $cli -Destination (Join-Path $resolvedStaging $cliName)
    Copy-Item -LiteralPath $logicRuntime -Destination (Join-Path $resolvedStaging $runtimeName)
    Copy-Item -LiteralPath $readme, $license -Destination $resolvedStaging
    $thirdParty = Join-Path $resolvedStaging "THIRD_PARTY_LICENSES\souffle"
    New-Item -ItemType Directory -Force -Path $thirdParty | Out-Null
    Copy-Item -LiteralPath $logicLicenses -Destination $thirdParty -Recurse

    if ($Target -eq "linux-x86_64") {
        chmod +x (Join-Path $resolvedStaging $cliName) (Join-Path $resolvedStaging $runtimeName)
    }

    New-Item -ItemType Directory -Force -Path $output | Out-Null
    $extension = if ($Target -eq "windows-x86_64") { ".zip" } else { ".tar.gz" }
    $archive = Join-Path $output "$archiveBase$extension"
    $checksum = "$archive.sha256"
    foreach ($generated in @($archive, $checksum)) {
        if (Test-Path -LiteralPath $generated) {
            Remove-Item -LiteralPath $generated -Force
        }
    }

    if ($Target -eq "windows-x86_64") {
        Compress-Archive -LiteralPath $resolvedStaging -DestinationPath $archive
    } else {
        tar -czf $archive -C $resolvedStagingParent $archiveBase
    }

    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
    Set-Content -NoNewline -LiteralPath $checksum -Value "$hash  $([IO.Path]::GetFileName($archive))`n"
    [pscustomobject]@{
        Archive = $archive
        Checksum = $checksum
        Sha256 = $hash
    }
} finally {
    if (Test-Path -LiteralPath $resolvedStaging) {
        Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
    }
}

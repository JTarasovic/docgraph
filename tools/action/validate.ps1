param(
    [Parameter(Mandatory = $true)]
    [string] $Version,

    [string] $WorkingDirectory = ".",

    [string] $Changes = ""
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$versionNumber = $Version.TrimStart("v")
if ($versionNumber -notmatch '^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') {
    throw "docgraph version must be an exact semantic version, found '$Version'."
}
$tag = "v$versionNumber"

$architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($architecture -ne "X64") {
    throw "docgraph release artifacts do not support runner architecture '$architecture'."
}
if ($IsWindows) {
    $target = "x86_64-pc-windows-msvc"
    $extension = ".zip"
    $executableName = "docgraph.exe"
} elseif ($IsLinux) {
    $target = "x86_64-unknown-linux-gnu"
    $extension = ".tar.gz"
    $executableName = "docgraph"
} else {
    throw "docgraph release artifacts support only Windows and Linux x86-64 runners."
}

$runtimeNames = @("docgraph-logic-runtime")
if ($IsWindows) {
    $runtimeNames += "docgraph-logic-runtime.exe"
}
$archiveCandidates = @(
    "docgraph-cli-$target$extension"
    "docgraph-$tag-$(if ($IsWindows) { 'windows' } else { 'linux' })-x86_64$extension"
)
$runnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$installation = Join-Path $runnerTemp "docgraph-action-$versionNumber-$target-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $installation | Out-Null

$apiHeaders = @{
    Accept = "application/vnd.github+json"
    "X-GitHub-Api-Version" = "2022-11-28"
}
if ($env:DOCGRAPH_ACTION_TOKEN) {
    $apiHeaders.Authorization = "Bearer $env:DOCGRAPH_ACTION_TOKEN"
}
$release = Invoke-RestMethod `
    -Uri "https://api.github.com/repos/JTarasovic/docgraph/releases/tags/$tag" `
    -Headers $apiHeaders

$archiveName = $null
$archiveAsset = $null
$checksumAsset = $null
foreach ($candidate in $archiveCandidates) {
    $candidateArchive = $release.assets | Where-Object name -EQ $candidate | Select-Object -First 1
    $candidateChecksum = $release.assets | Where-Object name -EQ "$candidate.sha256" | Select-Object -First 1
    if ($candidateArchive -and $candidateChecksum) {
        $archiveName = $candidate
        $archiveAsset = $candidateArchive
        $checksumAsset = $candidateChecksum
        break
    }
}
if (-not $archiveAsset -or -not $checksumAsset) {
    throw "Release $tag has no supported archive/checksum pair for $target. Expected one of: $($archiveCandidates -join ', ')."
}

$archive = Join-Path $installation $archiveName
$checksum = "$archive.sha256"
$assetHeaders = $apiHeaders.Clone()
$assetHeaders.Accept = "application/octet-stream"
Invoke-WebRequest -Uri $archiveAsset.url -Headers $assetHeaders -OutFile $archive
Invoke-WebRequest -Uri $checksumAsset.url -Headers $assetHeaders -OutFile $checksum

$checksumMatch = [regex]::Match(
    (Get-Content -Raw -LiteralPath $checksum).Trim(),
    '^(?<hash>[0-9a-fA-F]{64})\s+\*?(?<file>.+)$'
)
if (-not $checksumMatch.Success) {
    throw "Release checksum has an invalid format."
}
$checksumFile = [IO.Path]::GetFileName($checksumMatch.Groups["file"].Value.Trim())
if ($checksumFile -ne $archiveName) {
    throw "Release checksum names '$checksumFile' instead of '$archiveName'."
}
$expectedHash = $checksumMatch.Groups["hash"].Value.ToLowerInvariant()
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
if ($actualHash -ne $expectedHash) {
    throw "Release checksum mismatch: expected $expectedHash, got $actualHash."
}

if ($IsWindows) {
    Expand-Archive -LiteralPath $archive -DestinationPath $installation
} else {
    tar -xzf $archive -C $installation
}
$executable = Get-ChildItem -LiteralPath $installation -Filter $executableName -File -Recurse |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $executable) {
    throw "Release archive does not contain $executableName."
}
$executableDirectory = Split-Path -Parent $executable
$runtime = $runtimeNames |
    ForEach-Object { Join-Path $executableDirectory $_ } |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    Select-Object -First 1
if (-not $runtime) {
    throw "Release archive does not place a supported logic runtime beside $executableName."
}
if ($IsLinux) {
    chmod +x $executable $runtime
}

# The release runtime is part of the action's verified installation. Point the
# CLI at it explicitly so a caller's development-only override cannot select a
# missing or incompatible runtime from the checkout being validated.
$env:DOCGRAPH_LOGIC_RUNTIME = $runtime

$reportedVersion = (& $executable --version).Trim()
if ($reportedVersion -ne "docgraph $versionNumber") {
    throw "Installed CLI version '$reportedVersion' does not match '$versionNumber'."
}

if ($env:GITHUB_PATH) {
    Add-Content -LiteralPath $env:GITHUB_PATH -Value $executableDirectory
}
if ($env:GITHUB_OUTPUT) {
    Add-Content -LiteralPath $env:GITHUB_OUTPUT -Value "version=$versionNumber"
    Add-Content -LiteralPath $env:GITHUB_OUTPUT -Value "executable=$executable"
}

$workspace = if ($env:GITHUB_WORKSPACE) { $env:GITHUB_WORKSPACE } else { (Get-Location).Path }
$repository = [IO.Path]::GetFullPath((Join-Path $workspace $WorkingDirectory))
if (-not (Test-Path -LiteralPath $repository -PathType Container)) {
    throw "Validation working directory does not exist: $repository"
}
$arguments = @("validate")
if (-not [string]::IsNullOrWhiteSpace($Changes)) {
    $arguments += "--changes", $Changes
}

Push-Location $repository
try {
    & $executable @arguments
} finally {
    Pop-Location
}
Write-Output "docgraph $versionNumber validated $repository"

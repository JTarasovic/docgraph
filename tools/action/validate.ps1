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
    $target = "windows-x86_64"
    $extension = ".zip"
    $executableName = "docgraph.exe"
    $runtimeName = "docgraph-logic-runtime.exe"
} elseif ($IsLinux) {
    $target = "linux-x86_64"
    $extension = ".tar.gz"
    $executableName = "docgraph"
    $runtimeName = "docgraph-logic-runtime"
} else {
    throw "docgraph release artifacts support only Windows and Linux x86-64 runners."
}

$archiveName = "docgraph-$tag-$target$extension"
$releaseBase = "https://github.com/JTarasovic/docgraph/releases/download/$tag"
$runnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$installation = Join-Path $runnerTemp "docgraph-action-$versionNumber-$target-$([guid]::NewGuid())"
$archive = Join-Path $installation $archiveName
$checksum = "$archive.sha256"
New-Item -ItemType Directory -Path $installation | Out-Null

if ($env:DOCGRAPH_ACTION_TOKEN) {
    $apiHeaders = @{
        Authorization = "Bearer $env:DOCGRAPH_ACTION_TOKEN"
        Accept = "application/vnd.github+json"
        "X-GitHub-Api-Version" = "2022-11-28"
    }
    $release = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/JTarasovic/docgraph/releases/tags/$tag" `
        -Headers $apiHeaders
    $archiveAsset = $release.assets | Where-Object name -EQ $archiveName | Select-Object -First 1
    $checksumAsset = $release.assets | Where-Object name -EQ "$archiveName.sha256" | Select-Object -First 1
    if (-not $archiveAsset -or -not $checksumAsset) {
        throw "Release $tag does not contain $archiveName and its checksum."
    }
    $assetHeaders = $apiHeaders.Clone()
    $assetHeaders.Accept = "application/octet-stream"
    Invoke-WebRequest -Uri $archiveAsset.url -Headers $assetHeaders -OutFile $archive
    Invoke-WebRequest -Uri $checksumAsset.url -Headers $assetHeaders -OutFile $checksum
} else {
    Invoke-WebRequest -Uri "$releaseBase/$archiveName" -OutFile $archive
    Invoke-WebRequest -Uri "$releaseBase/$archiveName.sha256" -OutFile $checksum
}

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
$runtime = Join-Path $executableDirectory $runtimeName
if (-not (Test-Path -LiteralPath $runtime -PathType Leaf)) {
    throw "Release archive does not place $runtimeName beside $executableName."
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

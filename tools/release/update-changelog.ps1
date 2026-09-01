param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string] $Version
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

$versionNumber = $Version.TrimStart("v")
if ($versionNumber -notmatch '^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') {
    throw "Release version must be an exact semantic version, found '$Version'."
}

$repository = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$previousTag = (& git -C $repository describe `
    --tags `
    --match "v[0-9]*.[0-9]*.[0-9]*" `
    --abbrev=0 `
    HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $previousTag -notmatch '^v\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') {
    throw "Could not determine the previous stable release tag."
}

$metadata = (& cargo metadata `
    --manifest-path (Join-Path $repository "Cargo.toml") `
    --no-deps `
    --format-version 1) | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) {
    throw "Could not read Cargo workspace metadata."
}
$checkedInVersion = $metadata.packages |
    Where-Object name -EQ "docgraph-cli" |
    Select-Object -First 1 -ExpandProperty version
if (-not $checkedInVersion) {
    throw "Cargo metadata does not contain docgraph-cli."
}

$cliffArguments = @(
    "--config", (Join-Path $repository "cliff.toml"),
    "--repository", $repository,
    "--offline",
    "--tag", "v$versionNumber"
)
if ($checkedInVersion -eq $versionNumber) {
    # cargo-release writes the intended version before running hooks only in
    # execute mode. Its dry run keeps the old version in the file, so render the
    # proposal to stdout without modifying the changelog.
    $cliffArguments += "--prepend", (Join-Path $repository "CHANGELOG.md")
} else {
    $cliffArguments += "--strip", "header"
}

& git-cliff @cliffArguments "$previousTag..HEAD"
if ($LASTEXITCODE -ne 0) {
    throw "git-cliff failed to update CHANGELOG.md."
}

if ($checkedInVersion -eq $versionNumber) {
    Write-Output "Prepared changelog entries for $previousTag..v$versionNumber"
} else {
    Write-Output "Previewed changelog entries for $previousTag..v$versionNumber"
}

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true

function Get-TomlString {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Document,

        [Parameter(Mandatory = $true)]
        [string] $Section,

        [Parameter(Mandatory = $true)]
        [string] $Key
    )

    $sectionPattern = "(?ms)^\[$([regex]::Escape($Section))\]\s*\r?\n(?<body>.*?)(?=^\[|\z)"
    $sectionMatch = [regex]::Match($Document, $sectionPattern)
    if (-not $sectionMatch.Success) {
        throw "Missing [$Section] in tools/logic-runtime/sources.toml."
    }
    $keyPattern = "(?m)^$([regex]::Escape($Key))\s*=\s*`"(?<value>[^`"]+)`"\s*$"
    $keyMatch = [regex]::Match($sectionMatch.Groups["body"].Value, $keyPattern)
    if (-not $keyMatch.Success) {
        throw "Missing $Key in [$Section] in tools/logic-runtime/sources.toml."
    }
    $keyMatch.Groups["value"].Value
}

$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$targetRoot = [IO.Path]::GetFullPath((Join-Path $repository "target"))
$staging = [IO.Path]::GetFullPath((Join-Path $targetRoot "release-inputs"))
if (-not $staging.StartsWith($targetRoot + [IO.Path]::DirectorySeparatorChar)) {
    throw "Unsafe dist staging path: $staging"
}

$platform = if ($IsWindows) {
    "windows-x86_64"
} elseif ($IsLinux) {
    "linux-x86_64"
} else {
    throw "Dist inputs support only Windows and Linux x86-64 hosts."
}
$architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($architecture -ne "X64") {
    throw "Dist inputs do not support host architecture '$architecture'."
}

$sourcesPath = Join-Path $repository "tools\logic-runtime\sources.toml"
$sources = Get-Content -Raw -LiteralPath $sourcesPath
$section = "artifact.$platform"
$runtimeName = Get-TomlString -Document $sources -Section $section -Key "name"
$release = Get-TomlString -Document $sources -Section $section -Key "release"
$url = Get-TomlString -Document $sources -Section $section -Key "url"
$archiveSha256 = Get-TomlString -Document $sources -Section $section -Key "archive_sha256"
$binarySha256 = Get-TomlString -Document $sources -Section $section -Key "binary_sha256"

$scratch = Join-Path ([IO.Path]::GetTempPath()) "docgraph-dist-inputs-$([guid]::NewGuid())"
$archiveExtension = if ($url.EndsWith(".zip", [StringComparison]::OrdinalIgnoreCase)) {
    ".zip"
} else {
    ".tar.gz"
}
$archive = Join-Path $scratch "runtime$archiveExtension"
$extracted = Join-Path $scratch "extracted"
New-Item -ItemType Directory -Path $scratch, $extracted | Out-Null

try {
    $assetName = [IO.Path]::GetFileName(([uri] $url).AbsolutePath)
    gh release download $release `
        --repo JTarasovic/docgraph `
        --pattern $assetName `
        --output $archive
    $actualArchiveHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
    if ($actualArchiveHash -ne $archiveSha256.ToLowerInvariant()) {
        throw "Logic runtime archive checksum mismatch: expected $archiveSha256, got $actualArchiveHash"
    }

    if ($archiveExtension -eq ".zip") {
        Expand-Archive -LiteralPath $archive -DestinationPath $extracted
    } else {
        tar -xzf $archive -C $extracted
    }

    $runtime = Get-ChildItem -LiteralPath $extracted -Recurse -File |
        Where-Object Name -EQ $runtimeName |
        Select-Object -First 1
    if (-not $runtime) {
        throw "Downloaded logic runtime archive does not contain $runtimeName."
    }
    $actualBinaryHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $runtime.FullName).Hash.ToLowerInvariant()
    if ($actualBinaryHash -ne $binarySha256.ToLowerInvariant()) {
        throw "Logic runtime binary checksum mismatch: expected $binarySha256, got $actualBinaryHash"
    }
    $licenses = Join-Path $runtime.Directory.FullName "licenses"
    if (-not (Test-Path -LiteralPath $licenses -PathType Container)) {
        throw "Downloaded logic runtime archive does not contain its licenses directory."
    }

    if (Test-Path -LiteralPath $staging) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
    $skills = Join-Path $staging "skills"
    $thirdParty = Join-Path $staging "THIRD_PARTY_LICENSES\souffle"
    New-Item -ItemType Directory -Force -Path $staging, $skills, $thirdParty | Out-Null
    Copy-Item -LiteralPath $runtime.FullName -Destination (Join-Path $staging "docgraph-logic-runtime")
    Copy-Item -LiteralPath (Join-Path $repository "skills\docgraph") -Destination $skills -Recurse
    Get-ChildItem -LiteralPath $licenses | Copy-Item -Destination $thirdParty -Recurse
    if ($IsLinux) {
        chmod +x (Join-Path $staging "docgraph-logic-runtime")
    }
} finally {
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}

Write-Output "staged dist inputs for $platform at $staging"

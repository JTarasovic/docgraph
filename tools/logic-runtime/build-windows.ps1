param(
    [string] $CacheDirectory = (Join-Path $PSScriptRoot "..\..\.tools\logic-runtime\windows"),
    [string] $OutputDirectory = (Join-Path $PSScriptRoot "..\..\target\logic-runtime\windows-x86_64")
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$souffleRevision = "a1303be3c0166400dee3d1f36f0d96abe03e6901"
$vcpkgRevision = "cd61e1e26a038e82d6550a3ebbe0fbbfe7da78e3"
$winFlexBisonVersion = "2.5.25"
$winFlexBisonUrl = "https://github.com/lexxmark/winflexbison/releases/download/v$winFlexBisonVersion/win_flex_bison-$winFlexBisonVersion.zip"
$winFlexBisonSha256 = "8D324B62BE33604B2C45AD1DD34AB93D722534448F55A16CA7292DE32B6AC135"

if (-not $IsWindows) {
    throw "The pinned native runtime build currently supports Windows only."
}

$cache = [System.IO.Path]::GetFullPath($CacheDirectory)
$output = [System.IO.Path]::GetFullPath($OutputDirectory)
$source = Join-Path $cache "souffle"
$vcpkg = Join-Path $cache "vcpkg"
$installed = Join-Path $cache "vcpkg-installed"
$archive = Join-Path $cache "winflexbison.zip"
$winFlexBison = Join-Path $cache "winflexbison"
$build = Join-Path $cache "build"
$artifact = Join-Path $output "docgraph-logic-runtime.exe"
$stamp = Join-Path $output "build-inputs.sha256"

New-Item -ItemType Directory -Force -Path $cache, $output | Out-Null
$inputHashes = @(
    (Get-FileHash -Algorithm SHA256 -LiteralPath $PSCommandPath).Hash,
    (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $PSScriptRoot "sources.toml")).Hash,
    (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $PSScriptRoot "static-sqlite-windows.patch")).Hash
) -join "`n"
$inputHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($inputHashes)))
if ((Test-Path -LiteralPath $artifact) -and
    (Test-Path -LiteralPath $stamp) -and
    ((Get-Content -Raw -LiteralPath $stamp).Trim() -eq $inputHash)) {
    Get-FileHash -Algorithm SHA256 -LiteralPath $artifact
    exit 0
}

if (-not (Test-Path -LiteralPath (Join-Path $source ".git"))) {
    git clone --filter=blob:none https://github.com/souffle-lang/souffle.git $source
}
git -C $source fetch --depth 1 origin $souffleRevision
git -C $source checkout --detach $souffleRevision
$staticSqlitePatch = Join-Path $PSScriptRoot "static-sqlite-windows.patch"
$sourceDiff = git -C $source diff -- src/CMakeLists.txt
if ([string]::IsNullOrWhiteSpace(($sourceDiff -join "`n"))) {
    git -C $source apply $staticSqlitePatch
} elseif (($sourceDiff -join "`n") -notmatch "if \(EXISTS.*SQLite3_LIBRARY_DIR") {
    throw "The cached Souffle source contains an unexpected modification."
}

if (-not (Test-Path -LiteralPath (Join-Path $vcpkg ".git"))) {
    git clone --filter=blob:none https://github.com/microsoft/vcpkg.git $vcpkg
}
git -C $vcpkg fetch --depth 1 origin $vcpkgRevision
git -C $vcpkg checkout --detach $vcpkgRevision
& (Join-Path $vcpkg "bootstrap-vcpkg.bat") -disableMetrics
& (Join-Path $vcpkg "vcpkg.exe") install sqlite3:x64-windows-static --x-install-root=$installed

if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest -Uri $winFlexBisonUrl -OutFile $archive
}
$actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash
if ($actualHash -ne $winFlexBisonSha256) {
    throw "winflexbison checksum mismatch: expected $winFlexBisonSha256, found $actualHash"
}
if (-not (Test-Path -LiteralPath (Join-Path $winFlexBison "win_flex.exe"))) {
    Expand-Archive -LiteralPath $archive -DestinationPath $winFlexBison
}

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path -LiteralPath $vswhere)) {
    throw "Visual Studio Installer's vswhere.exe was not found."
}
$visualStudio = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $visualStudio) {
    throw "Visual Studio with the C++ toolchain was not found."
}
$cmake = Join-Path $visualStudio "Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
$ninja = Join-Path $visualStudio "Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe"
$vcvars = Join-Path $visualStudio "VC\Auxiliary\Build\vcvars64.bat"
foreach ($line in (& cmd.exe /d /s /c "`"$vcvars`" >nul && set")) {
    if ($line -match "^([^=]+)=(.*)$") {
        Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
    }
}

& $cmake -S $source -B $build -G Ninja `
    "-DCMAKE_MAKE_PROGRAM=$ninja" `
    "-DCMAKE_TOOLCHAIN_FILE=$(Join-Path $vcpkg 'scripts\buildsystems\vcpkg.cmake')" `
    "-DVCPKG_INSTALLED_DIR=$installed" `
    -DVCPKG_MANIFEST_MODE=OFF `
    -DVCPKG_TARGET_TRIPLET=x64-windows-static `
    -DCMAKE_BUILD_TYPE=Release `
    -DCMAKE_CXX_FLAGS=/bigobj `
    -DSOUFFLE_DOMAIN_64BIT=ON `
    -DSOUFFLE_USE_SQLITE=ON `
    -DSOUFFLE_USE_CURSES=OFF `
    -DSOUFFLE_USE_ZLIB=OFF `
    -DSOUFFLE_USE_LIBFFI=OFF `
    -DSOUFFLE_USE_OPENMP=OFF `
    -DSOUFFLE_ENABLE_TESTING=OFF `
    -DSOUFFLE_BASH_COMPLETION=OFF `
    -DSOUFFLE_GIT=OFF `
    "-DFLEX_EXECUTABLE=$(Join-Path $winFlexBison 'win_flex.exe')" `
    "-DBISON_EXECUTABLE=$(Join-Path $winFlexBison 'win_bison.exe')"
& $cmake --build $build --target souffle -j4

Copy-Item -Force -LiteralPath (Join-Path $build "src\souffle.exe") -Destination $artifact
Copy-Item -Force -Recurse -LiteralPath (Join-Path $source "licenses") -Destination $output
Set-Content -NoNewline -LiteralPath $stamp -Value $inputHash
Get-FileHash -Algorithm SHA256 -LiteralPath $artifact

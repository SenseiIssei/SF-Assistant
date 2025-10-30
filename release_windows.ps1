$ErrorActionPreference = 'Stop'

function Get-VersionFromCargoToml {
    $line = Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (-not $line) { throw "Could not extract version from Cargo.toml" }
    $m = [regex]::Match($line.Line, 'version\s*=\s*"([^"]+)"')
    if (-not $m.Success) { throw "Could not parse version from Cargo.toml" }
    return $m.Groups[1].Value
}

$version = Get-VersionFromCargoToml
$crateName = 'sf-assistant'
$productName = 'SFAssistant'
$target = 'x86_64-pc-windows-msvc'

Write-Host "Building $productName $version for $target..."
& cargo build --release --quiet --target $target

$binPath = Join-Path -Path "target/$target/release" -ChildPath "$crateName.exe"
if (-not (Test-Path $binPath)) { throw "Binary not found: $binPath" }

$workdir = "${productName}_v${version}_${target}"
if (Test-Path $workdir) { Remove-Item -Recurse -Force $workdir }
New-Item -ItemType Directory -Path $workdir | Out-Null

Copy-Item $binPath "$workdir/$productName.exe"
Copy-Item LICENSE "$workdir/LICENSE.txt"
Copy-Item README.md "$workdir/README.md"
if (Test-Path 'THIRD_PARTY_LICENSES.txt') { Copy-Item 'THIRD_PARTY_LICENSES.txt' "$workdir/THIRD_PARTY_LICENSES.txt" }
if (Test-Path 'THIRD_PARTY_NOTICES.txt') { Copy-Item 'THIRD_PARTY_NOTICES.txt' "$workdir/THIRD_PARTY_NOTICES.txt" }

$dist = 'dist'
if (-not (Test-Path $dist)) { New-Item -ItemType Directory -Path $dist | Out-Null }
$outfile = "$workdir.zip"

if (Get-Command zip -ErrorAction SilentlyContinue) {
    & zip -rq $outfile $workdir
} else {
    Compress-Archive -Path $workdir -DestinationPath $outfile -Force
}

$checksumFile = Join-Path $dist "$outfile.sha256"
$hashOutput = & certutil -hashfile $outfile SHA256
$hashLine = ($hashOutput | Select-Object -Index 1).Trim()
Set-Content -Path $checksumFile -Value "$hashLine  $outfile"

Move-Item $outfile (Join-Path $dist $outfile) -Force
Remove-Item -Recurse -Force $workdir

Write-Host "Done. Artifacts and checksums are in ./$dist"
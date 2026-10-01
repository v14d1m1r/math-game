# Run on Windows with Rust and the Visual Studio C++ build tools installed.
$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    rustup target add x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw "Installing the Windows target failed." }
    cargo build --locked --release --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw "Building the game failed." }

    $package = Join-Path $projectRoot "dist/mala-matematika-windows-x64"
    New-Item -ItemType Directory -Force -Path $package | Out-Null
    Copy-Item "target/x86_64-pc-windows-msvc/release/math-game.exe" "$package/mala-matematika.exe" -Force
    Copy-Item "assets/fonts/LICENSE-Liberation.txt" "$package/LICENSE-Liberation.txt" -Force
    Copy-Item "WINDOWS-README.txt" "$package/README.txt" -Force
    Compress-Archive -Path $package -DestinationPath "dist/mala-matematika-windows-x64.zip" -Force
    Write-Host "Ready: $package/mala-matematika.exe"
} finally {
    Pop-Location
}

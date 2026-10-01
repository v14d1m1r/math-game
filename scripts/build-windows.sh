#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v zip >/dev/null || { echo "Potreban je zip." >&2; exit 1; }
cargo xwin build --locked --release --target x86_64-pc-windows-msvc
package="dist/mala-matematika-windows-x64"
mkdir -p "$package"
install -m 644 target/x86_64-pc-windows-msvc/release/math-game.exe "$package/mala-matematika.exe"
install -m 644 assets/fonts/LICENSE-Liberation.txt "$package/LICENSE-Liberation.txt"
install -m 644 WINDOWS-README.txt "$package/README.txt"
cd dist
zip -q mala-matematika-windows-x64.zip \
    mala-matematika-windows-x64/mala-matematika.exe \
    mala-matematika-windows-x64/LICENSE-Liberation.txt \
    mala-matematika-windows-x64/README.txt
echo "Spremno: $package/mala-matematika.exe"

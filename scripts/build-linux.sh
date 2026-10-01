#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
    echo "Ova skripta pravi paket na Linux x64 racunaru." >&2
    exit 1
fi

cargo build --locked --release
package="dist/mala-matematika-linux-x64"
mkdir -p "$package"
install -m 755 target/release/math-game "$package/mala-matematika"
install -m 644 assets/fonts/LICENSE-Liberation.txt "$package/LICENSE-Liberation.txt"
install -m 644 LINUX-README.txt "$package/README.txt"
tar -czf dist/mala-matematika-linux-x64.tar.gz -C dist mala-matematika-linux-x64
echo "Spremno: $package/mala-matematika"

#!/usr/bin/env bash

set -euo pipefail

nix-shell --run "make -j '$(nproc --all)'"

workspace=$(mktemp -d)

function cleanup {
    rm -rf "${workspace}"
}

trap cleanup EXIT

chromejson="${workspace}"/out.json
tracyfile="${workspace}"/out.tracy

echo "[+] Instantiating derivation"
NIX_SHOW_TRACE=1 ./outputs/out/bin/nix-instantiate $@ > "${chromejson}"

echo "[+] Converting chrome profile to tracy profile"
nix-shell -p tracy --run "import-chrome '${chromejson}' '${tracyfile}'"
nix-shell -p tracy --run "tracy '${tracyfile}'"

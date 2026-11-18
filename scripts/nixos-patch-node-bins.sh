#!/usr/bin/env bash
# NixOS without nix-ld cannot run prebuilt dynamically linked executables from
# npm (e.g. Vite+'s type-aware linter `tsrs`): point them at Nix's dynamic
# linker. Runs as the root `postinstall`; does nothing elsewhere.
set -euo pipefail

[ -e /etc/NIXOS ] || exit 0
[ -n "${NIX_LD:-}" ] && exit 0 # nix-ld is set up: nothing to do

if [ -n "${NIX_CC:-}" ] && [ -f "$NIX_CC/nix-support/dynamic-linker" ]; then
  interp=$(cat "$NIX_CC/nix-support/dynamic-linker")
else
  echo "nixos-patch-node-bins: not in the nix dev shell, skipping (run: nix develop -c pnpm install)" >&2
  exit 0
fi

patchelf=$(command -v patchelf || true)
if [ -z "$patchelf" ]; then
  echo "nixos-patch-node-bins: patchelf not found, skipping" >&2
  exit 0
fi

root=$(cd "$(dirname "$0")/.." && pwd)
find "$root/node_modules/.pnpm" -type f -perm -u+x ! -name '*.node' -size +100k 2>/dev/null |
  while read -r f; do
    current=$("$patchelf" --print-interpreter "$f" 2>/dev/null) || continue
    case "$current" in
      /nix/store/*) ;;
      *)
        "$patchelf" --set-interpreter "$interp" "$f"
        echo "nixos-patch-node-bins: patched ${f#"$root/"}"
        ;;
    esac
  done

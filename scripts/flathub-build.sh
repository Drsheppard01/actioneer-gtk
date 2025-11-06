#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 [flathub-build args...]" >&2
  exit 1
fi

flatpak run --command=sh org.flatpak.Builder -c 'flathub-build --disable-rofiles-fuse "$@"' -- "$@"

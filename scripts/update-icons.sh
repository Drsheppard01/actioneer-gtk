#!/usr/bin/env bash
set -euo pipefail

# update-icons.sh
# Mirror the repository hicolor icons into the user's local icon theme,
# ensure index.theme exists, rebuild the icon cache, refresh the desktop DB,
# and list the installed actioneer icons.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$REPO_ROOT/icons/icons/hicolor/"
DST="$HOME/.local/share/icons/hicolor"
DESKTOP_SRC="$REPO_ROOT/data/me.spaceinbox.actioneer.desktop"
DESKTOP_DST="$HOME/.local/share/applications/me.spaceinbox.actioneer.desktop"

echo "Source icons: $SRC"
echo "Destination: $DST"

# Mirror icons
mkdir -p "$DST"
# Copy contents of SRC into DST (mirror), preserving the hicolor/* layout
rsync -av --delete "$SRC" "$DST" >/dev/null

# Ensure index.theme exists locally (copy from system if available)
if [ ! -f "$DST/index.theme" ]; then
  if [ -f "/usr/share/icons/hicolor/index.theme" ]; then
    cp /usr/share/icons/hicolor/index.theme "$DST/"
    echo "Copied system index.theme to $DST/index.theme"
  else
    echo "Warning: /usr/share/icons/hicolor/index.theme not found. Icon cache tools may warn." >&2
  fi
fi

# Rebuild icon cache
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache "$DST" >/dev/null
  echo "Rebuilt icon cache at $DST"
else
  echo "gtk-update-icon-cache not found; please install libgtk-4-dev or the relevant package." >&2
fi

# Install desktop file
mkdir -p "$(dirname "$DESKTOP_DST")"
cp "$DESKTOP_SRC" "$DESKTOP_DST"
update-desktop-database "$(dirname "$DESKTOP_DST")" >/dev/null || true

# List installed icons
echo "Installed actioneer icons:"
ls -l "$DST"/*/apps/actioneer.* 2>/dev/null || true

echo "Done. You may need to log out or restart GNOME Shell to see changes in the overview." 

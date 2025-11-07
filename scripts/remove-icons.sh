#!/usr/bin/env bash
set -euo pipefail

# remove-icons.sh
# Revert what update-icons.sh does: backup+remove local actioneer icons and desktop file.
# Use --system to also remove system-wide files (requires sudo).

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DST="$HOME/.local/share/icons/hicolor"
DESKTOP_DST="$HOME/.local/share/applications/me.spaceinbox.actioneer.desktop"
SYSTEM_ICON_DST="/usr/share/icons/hicolor"
SYSTEM_DESKTOP_DST="/usr/share/applications/me.spaceinbox.actioneer.desktop"

BACKUP_DIR="$HOME/actioneer-cleanup-backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

usage(){
  cat <<EOF
Usage: $0 [--system] [--yes]

Removes the Actioneer icons and the user desktop file installed by scripts/update-icons.sh.
By default this operates on user-local locations only (~/.local/share/icons and ~/.local/share/applications).
Pass --system to also remove system-wide files under /usr/share (this requires sudo).
Pass --yes to skip the interactive confirmation.
Backups of removed files are placed under: $BACKUP_DIR
EOF
}

DO_SYSTEM=0
AUTO_YES=0
while [[ ${#} -gt 0 ]]; do
  case "$1" in
    --system) DO_SYSTEM=1; shift;;
    --yes) AUTO_YES=1; shift;;
    -h|--help) usage; exit 0;;
    *) echo "Unknown arg: $1"; usage; exit 2;;
  esac
done

echo "Backup directory: $BACKUP_DIR"

confirm(){
  if [ "$AUTO_YES" -eq 1 ]; then
    return 0
  fi
  read -r -p "$1 [y/N]: " ans
  case "${ans,,}" in
    y|yes) return 0;;
    *) return 1;;
  esac
}

echo "Scanning for Actioneer icon files in: $DST"
mapfile -t ICON_PATHS < <(find "$DST" -type f -iname '*actioneer*' 2>/dev/null || true)

if [ ${#ICON_PATHS[@]} -eq 0 ]; then
  echo "No user-local actioneer icons found under $DST"
else
  echo "Found ${#ICON_PATHS[@]} user-local icon files:" 
  for p in "${ICON_PATHS[@]}"; do echo "  $p"; done
  if confirm "Backup and remove these user-local icon files?"; then
    mkdir -p "$BACKUP_DIR/icons"
    for p in "${ICON_PATHS[@]}"; do
      dest="$BACKUP_DIR/icons$(dirname "$p" | sed "s|$DST||")"
      mkdir -p "$dest"
      mv -v "$p" "$dest/" || true
    done
    echo "Removed icon files (backed up to $BACKUP_DIR/icons)."
    # prune empty directories under DST
    find "$DST" -type d -empty -delete || true
  else
    echo "Skipped removing user-local icons."
  fi
fi

if [ -f "$DESKTOP_DST" ]; then
  echo "Found user desktop file: $DESKTOP_DST"
  if confirm "Backup and remove $DESKTOP_DST?"; then
    mkdir -p "$BACKUP_DIR/desktop"
    mv -v "$DESKTOP_DST" "$BACKUP_DIR/desktop/" || true
    echo "Moved desktop file to $BACKUP_DIR/desktop/"
  else
    echo "Skipped removing user desktop file."
  fi
else
  echo "No user desktop file at $DESKTOP_DST"
fi

if [ "$DO_SYSTEM" -eq 1 ]; then
  echo "--system specified: checking system locations (requires sudo)"
  if [ -f "$SYSTEM_DESKTOP_DST" ]; then
    echo "Found system desktop file: $SYSTEM_DESKTOP_DST"
    if confirm "Backup and remove $SYSTEM_DESKTOP_DST (sudo)?"; then
      sudo mkdir -p "$BACKUP_DIR/desktop-system"
      sudo cp -v "$SYSTEM_DESKTOP_DST" "$BACKUP_DIR/desktop-system/" || true
      sudo rm -fv "$SYSTEM_DESKTOP_DST" || true
      echo "Removed system desktop file (backup in $BACKUP_DIR/desktop-system)."
    else
      echo "Skipped removing system desktop file."
    fi
  else
    echo "No system desktop file at $SYSTEM_DESKTOP_DST"
  fi

  echo "Searching for system-wide actioneer icon files under $SYSTEM_ICON_DST"
  mapfile -t SYS_ICON_PATHS < <(sudo find "$SYSTEM_ICON_DST" -type f -iname '*actioneer*' 2>/dev/null || true)
  if [ ${#SYS_ICON_PATHS[@]} -eq 0 ]; then
    echo "No system-wide actioneer icons found."
  else
    echo "Found ${#SYS_ICON_PATHS[@]} system icon files:" 
    for p in "${SYS_ICON_PATHS[@]}"; do echo "  $p"; done
    if confirm "Backup and remove these system icon files (sudo)?"; then
      sudo mkdir -p "$BACKUP_DIR/icons-system"
      for p in "${SYS_ICON_PATHS[@]}"; do
        dest_dir="$BACKUP_DIR/icons-system$(dirname "$p" | sed "s|$SYSTEM_ICON_DST||")"
        sudo mkdir -p "$dest_dir"
        sudo mv -v "$p" "$dest_dir/" || true
      done
      echo "Removed system icon files (backed up to $BACKUP_DIR/icons-system)."
      # attempt to remove empty dirs (best-effort)
      for d in $(sudo find "$SYSTEM_ICON_DST" -type d -empty 2>/dev/null || true); do
        sudo rmdir "$d" 2>/dev/null || true
      done
    else
      echo "Skipped removing system icon files."
    fi
  fi
fi

# Rebuild icon cache and update desktop DB where appropriate
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  echo "Rebuilding icon cache for user icons: $DST"
  gtk-update-icon-cache "$DST" >/dev/null 2>&1 || true
else
  echo "gtk-update-icon-cache not found; please rebuild icon cache manually if needed."
fi

if [ "$DO_SYSTEM" -eq 1 ]; then
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    echo "Rebuilding icon cache for system icons: $SYSTEM_ICON_DST (sudo)"
    sudo gtk-update-icon-cache "$SYSTEM_ICON_DST" >/dev/null 2>&1 || true
  fi
  if command -v update-desktop-database >/dev/null 2>&1; then
    echo "Updating system desktop database (sudo)"
    sudo update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
  fi
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  echo "Updating user desktop database"
  update-desktop-database "$(dirname "$DESKTOP_DST")" >/dev/null 2>&1 || true
fi

echo "Done. Backups (if any) are under: $BACKUP_DIR"

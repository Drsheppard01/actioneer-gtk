This folder contains generated application icons for the "actioneer" app in the hicolor icon theme layout.

Generated items:
- icons/hicolor/scalable/apps/actioneer.svg (source SVG)
- icons/hicolor/16x16/apps/actioneer.png
- icons/hicolor/22x22/apps/actioneer.png
- icons/hicolor/24x24/apps/actioneer.png
- icons/hicolor/32x32/apps/actioneer.png
- icons/hicolor/48x48/apps/actioneer.png
- icons/hicolor/64x64/apps/actioneer.png
- icons/hicolor/96x96/apps/actioneer.png
- icons/hicolor/128x128/apps/actioneer.png
- icons/hicolor/192x192/apps/actioneer.png
- icons/hicolor/256x256/apps/actioneer.png
- icons/hicolor/384x384/apps/actioneer.png
- icons/hicolor/512x512/apps/actioneer.png

Installation (system-wide):
1. Copy the `icons/hicolor` directory to `/usr/share/icons/` (requires root):

   sudo cp -r icons/hicolor /usr/share/icons/
   sudo gtk-update-icon-cache /usr/share/icons/hicolor

User-only install:
1. Copy `icons/hicolor` into `~/.local/share/icons/`:

   mkdir -p ~/.local/share/icons
   cp -r icons/hicolor ~/.local/share/icons/
   gtk-update-icon-cache ~/.local/share/icons/hicolor

Notes:
- The icon name used is `actioneer`. If you need a different icon name, rename the files in each `apps/` folder and the scalable SVG accordingly.
- GNOME/GTK will pick the correct size automatically from the icon theme. If you have issues, try logging out/in or running `update-icon-caches` depending on your distribution.

Generated with ImageMagick from AppIcon.svg

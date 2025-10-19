# Using the local Libadwaita 1.8 documentation (for automated agents)

Purpose
- This repository contains a curated set of Libadwaita 1.8 reference files under `docs/libadwaita/`. Automated agents and developers should use these local files as the canonical UI design and implementation reference when modifying the GTK/libadwaita UI in this project.

Rules for agents
1. Always consult `docs/libadwaita/` before using or adding Libadwaita widgets or style classes. These files summarize the recommended components, adaptive layout patterns, style classes and CSS variables for Libadwaita 1.8.
2. Do NOT use deprecated APIs or style classes referenced in `docs/libadwaita/*` — prefer the recommended replacements (deprecated items are called out in the summaries).
3. When implementing or refactoring UI, prefer the adaptive patterns described: AdwToolbarView + AdwHeaderBar, AdwNavigationSplitView / AdwOverlaySplitView for sidebars, AdwDialog for adaptive dialogs, and AdwPreferencesDialog / AdwPreferencesGroup for settings pages.
4. For color and theming, use CSS variables from `css-variables.md` and AdwStyleManager when programmatic access is required.
5. If detailed API or method-level information is required (e.g., exact method names/parameters), reference the upstream pages linked at the top of each file in `docs/libadwaita/`.

Commit & CI guidance
- Adding or updating files under `docs/libadwaita/` should be accompanied by a short note in the PR describing which upstream pages were summarized or updated.

Contact
- If any doc is missing or ambiguous, open an issue so maintainers can expand the local summary.

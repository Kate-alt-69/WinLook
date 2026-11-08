# WLT — WinLook Template

## Purpose
WLT is the portable export/import format for selected WinLook settings and assets. One .wl.t file is one Template. A ZIP containing multiple .wl.t files is a Template Group; each member imports independently.

## Proposed container and structure
A .wl.t is proposed as a TAR archive compressed with gzip (tar.gz semantics) using the .wl.t extension.

    Midnight.wl.t
      manifest.json
      profile.json
      rmskin/
        Clock/
        SystemMonitor/
      wl/
        taskbar/
          configuration.json
          themes/
        &FONT_FILE&/
          Inter.ttf
          JetBrainsMono.ttf
        wallpaper/
          Midnight.wl.wp
          Interactive.wl.i.wp
        icons/
        cursors/
        sounds/
        desktop/
        wlwm/
        display/

The literal &FONT_FILE& directory token is intentionally preserved from the user's layout proposal.

## Directory semantics
- rmskin/: imported Rainmeter package folders and their retained structure.
- wl/taskbar/: taskbar configuration/theme resources.
- wl/&FONT_FILE&/: bundled fonts; check redistribution license and request consent before installation.
- wl/wallpaper/: .wl.wp and .wl.i.wp assets.
- wl/icons/, cursors/, sounds/: associated assets and mappings.
- wl/desktop/: portable desktop settings only; not machine-local Group membership.
- wl/wlwm/: WLWM settings/layouts/rules.
- wl/display/: portable display preferences without assuming stable physical monitor IDs.

## Manifest
Proposed fields: format ID/schema version, template ID/name/description/author, creation/modification date, minimum WinLook version, supported OS/capabilities, included/optional components, dependencies, assets with sizes and SHA-256, publisher/license/attribution, permissions, previews and migration notes. Exact schema remains open.

## Wallpaper formats
.wl.wp is a normal wallpaper asset/package. .wl.i.wp is an interactive wallpaper archive containing the wallpaper and supporting resources. Preserve source quality; do not recompress lossily by default.

## Local versus Fancy desktop Groups
Local Groups refer to machine-specific desktop items and are never exported in WLT. Fancy Groups are a separate portable/shareable concept requiring their own explicit item-reference/conflict rules.

## Template Group ZIP
A ZIP containing multiple .wl.t files is only a collection container. Validate and import each Template independently; a damaged member should not corrupt others.

## Security
Reject absolute/parent-traversal paths, escaping links, duplicate normalized paths, file/count/depth/ratio bombs and invalid hashes. Stage extraction, validate manifest and permissions, do not execute arbitrary native code during import, and roll back partial apply operations.

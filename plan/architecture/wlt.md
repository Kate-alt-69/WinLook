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

## Detailed manifest and archive contract (proposal)

The archive remains a proposal: one .wl.t file contains one Template; a ZIP containing multiple .wl.t files is a Template Group whose members are validated/imported independently. TAR+gzip semantics and extension handling need a prototype and cross-platform tool review.

### Manifest sections

| Section | Required information |
|---|---|
| Format | Format ID, schema version and minimum reader version |
| Identity | Stable template ID, name, description, author/publisher and timestamps |
| Compatibility | OS family/version range, required capabilities and optional components |
| Components | Component IDs, scope, apply order, dependencies and conflict keys |
| Assets | Normalized archive path, media type, byte length, SHA-256 and metadata |
| Permissions | Requested filesystem/network/runtime capabilities and reason |
| Provenance | License, attribution, publisher identity and optional signature metadata |
| Migration | Previous schema support, migration notes and unsupported-field behaviour |
| Presentation | Preview, category, tags and display metadata |

### Semantic rules

- Component IDs remain stable across compatible template versions.
- Asset hashes cover exact archived bytes.
- Required components reject incompatible import; optional components may be skipped with a reason.
- Manifest is data, never executable instructions.
- Apply order is explicit and dependency-checked; dependency cycles reject the affected template.
- Unknown required fields reject import. Unknown optional fields are preserved where possible.
- Missing/invalid signatures are never silently treated as a trusted publisher.

### Import state machine

1. **Selected:** user chooses local file or catalogue entry.
2. **Staged:** archive is copied to controlled temporary area; original remains untouched.
3. **Structure checked:** archive type, member count, paths, depth, duplicates, links and size/ratio limits are checked.
4. **Manifest validated:** schema, identifiers, component declarations and migration path are checked.
5. **Assets verified:** hashes, media types, permissions and licences are inspected.
6. **Compatibility resolved:** Manager maps components to capability report.
7. **Previewed:** user sees apply/skip/reject components and machine-wide permission requests.
8. **Approved:** user confirms exact operations and scope.
9. **Snapshotted:** Restoration records current state before each mutation.
10. **Applied:** components run in dependency-safe order.
11. **Verified:** observable result is checked and journal entries are committed.
12. **Completed or rolled back:** failure restores completed changes; incomplete restoration is reported.

Manager must not execute arbitrary native code during extraction, preview or ordinary import. Interactive scripts start only in the restricted runtime and after separate approval of declared capabilities.

### Archive safety limits

Numeric limits require sizing research, but the schema must support caps for total compressed/uncompressed bytes, individual file size, file count, directory depth, compression ratio, decompression time, manifest size/nesting, path length, normalized-name collisions, preview dimensions and decoder resource cost.

Reject absolute paths, parent traversal, device paths, alternate data stream tricks, escaping links/reparse points and duplicate normalized paths. Revalidate the destination at write time to prevent path races. Extract into staging; never write directly into active system directories from an untrusted archive.

### Template Group behaviour

A Template Group ZIP is a collection, not one merged configuration. Validate each .wl.t independently, show member-level results, and let the user choose which valid members to import. One invalid member must not alter another member's result.

### Export rules

- Export only components explicitly selected by the user.
- Exclude machine-local Groups, monitor instance IDs, session state, cache and diagnostics by default.
- Never include secrets, tokens, passwords or arbitrary private paths.
- Warn when asset redistribution rights are unclear.
- Recalculate hashes from exported bytes.
- Do not claim portability to an OS merely because the archive can be unpacked there.

### Open decisions

- Final archive format and extension handling.
- Canonical manifest and migration policy.
- Signature/trust-root model and unsigned local import policy.
- Numeric archive limits and allowed media formats.
- Asset deduplication.
- Conflict resolution and ownership when multiple templates change the same setting.


# wl_Lwlde

C89 companion to the portable WinLook Desktop Engine scene core.

## Scope

- Caller-owned, bounded monitor topology storage (maximum 16 monitors).
- Geometry validation and Fill/Fit/Stretch/Center source-to-monitor placement.
- One scene-owned wl_Lanimate frame clock shared by all active tracks.
- Visibility-aware frame requests; Unknown stays eligible, fully occluded scenes
  hold their last frame, static scenes do not poll a frame clock.
- Normal 60 FPS, Optimize/Power Saver 30 FPS and Suspend policies.
- A bounded, allocation-free per-application Game Mode resolver using
  host-supplied executable SHA-256 fingerprints, explicit profile precedence,
  and optional OS Game Mode/fullscreen fallbacks.
- An allocation-free caller-fed resource controller for CPU/GPU/RAM/VRAM
  percentages, configurable consecutive enter/recovery windows and separate
  Optimize/Suspend states. A peak domain is used instead of an average.
- A scene pressure setter that turns Optimize into 30 FPS and Suspend into
  no animation frame requests.
- Non-owning references to existing wl_Lcpuv and wl_Lgpuv governors.
- Capability checks that keep interactive content and Groups unavailable until
  a platform adapter explicitly proves support and crash-safe restoration.
- Bounded virtual desktop Group records over caller-supplied IDs for apps,
  files and folders; replacement validates all references before updating the
  caller-owned record array.
- Adaptive app-icon metadata plus a bounded RGBA compositor. It combines
  caller-decoded foreground/background layers, applies the safe zone and
  Circle/RoundedSquare/Squircle mask into a caller-owned output buffer.
  Source artwork, shortcuts and executables are never edited.
- A type-aware C89 context-action query for files, folders, applications and
  Group tiles. Its output buffer is caller-owned; native Shell verbs remain
  the host's responsibility.
- Immutable provider metadata for Wallpaper Engine Workshop plus free/open
  catalogues (Openverse, Wikimedia Commons and NASA). The C89 API exposes
  provider descriptions, URLs, access/pricing class and item-level rights notes;
  platform UI handles browser launching and file picking.

No allocation occurs in the topology storage, placement, condition update,
resource controller update or scene tick paths. Resource telemetry is supplied
by the platform host; this library does not poll Windows or create threads. The caller supplies monitor storage and serializes governor
access as required by CPUv/GPUv. The archive links against wl_Lanimate,
wl_Lcpuv and wl_Lgpuv; the root CMake aggregator wires those targets.

## Game Mode resolution

`wl_Lwlde_game_mode_resolve()` evaluates at most 64 unique executable
fingerprint profiles. An exact user profile wins first, including a deliberate
Normal override; an enabled system Game Mode signal is next, followed by an
enabled fullscreen fallback. Non-foreground applications always resolve to
Normal. The host supplies the SHA-256 digest and detection signals: the C89
core does not enumerate processes, inspect files, hash binaries or poll.

`wl_Lwlde_scene_set_game_mode()` applies the resolved mode to the existing
scene clock policy. Windows-specific foreground/fullscreen detection and
persistent profile editing remain adapter/Manager work.


## Desktop groups, adaptive app icons and menus

The C89 desktop model uses caller-owned bounded Group storage and references
to host-provided desktop item IDs. Each Group can contain applications, files
and folders. Replacement validates IDs, membership and duplicates before
committing the new collection. Member ID strings remain owned by the caller
and must outlive the Group records.

Adaptive app icons reference separate foreground/background assets and an
optional monochrome asset, plus a Circle/RoundedSquare/Squircle mask and a
48–72% safe-zone value. `wl_Lwlde_adaptive_app_icon_render_rgba()` composites
caller-decoded RGBA layers into a caller-owned 16–512 px output buffer. It
uses centered background cover, foreground safe-zone contain, premultiplied-
alpha bilinear sampling and a 2×2 antialiased mask. It performs no allocation;
the caller can reuse the output buffer and cache the tile until its
source/settings change. The source and output pixel buffers must not overlap.
`wl_Lwlde_desktop_context_actions()` returns the same type-aware action set
as the Rust core without allocating.

The portable APIs do not themselves enumerate Explorer or draw/hide desktop
icons. Explorer integration is mandatory BGW/WLDE runtime infrastructure
where Explorer owns the desktop; per-OS adapters use supported Shell APIs and
version-specific hooks only where needed. Original desktop icons must be
restored after BGW failure, Explorer restart, feature recovery and uninstall.

## Not implemented here

The Store metadata API is intentionally a provider directory, not a catalogue
 search/download client. It never bypasses Steam subscriptions or redistributes
 creator packages, and it does not assume that provider-level `Free` status
 overrides an individual asset's license.

This library does not create a desktop window, decode source icon/media files,
inspect Explorer, enumerate live desktop items, hide native desktop icons,
draw desktop/Group tiles or open Group popups. It can compose app-icon RGBA
bitmaps from caller-supplied decoded layers, but native Shell icon integration,
the desktop presenter and crash-safe restoration are still required. It also
does not run untrusted widget code. It is a C89 source API; that alone does not prove a
binary can run on Windows 95 or any other legacy target.

Run the paired C tests from the repository root with
python build.py verify c --sanitizers, or use CMake through
C/library/CMakeLists.txt.


The Rust mirror exposes both `render_adaptive_app_icon()` (allocating
convenience) and `render_adaptive_app_icon_into()` (caller-buffer,
allocation-free). Both produce the same packed RGBA8 tile as this C API.


## Group lookup and background context menu

The caller-owned Group set has read-only
`wl_Lwlde_desktop_groups_find()` and
`wl_Lwlde_desktop_group_contains()` helpers. To edit a collection, construct
a complete candidate array and call `wl_Lwlde_desktop_groups_replace()`;
the existing set is retained when validation fails. Member-ID storage remains
caller-owned and the C API does not allocate.

The context target `WL_LWDE_CONTEXT_DESKTOP_BACKGROUND` exposes the WLDE
`Create Group` and `Open Desktop Settings` actions. Native Shell entries
such as View, Sort, Refresh, New, wallpaper, Display Settings, Open and
Properties remain owned by Windows.


### Themed monochrome mode

`wl_Lwlde_adaptive_app_icon_render_monochrome_rgba()` renders the optional
monochrome layer using the source alpha as a glyph mask and a caller-supplied
RGB tint. It writes directly into caller-owned RGBA storage and performs no
allocation or source mutation.


## Normalized desktop layout

The C89 API provides `wl_Lwlde_desktop_layout` with caller-owned fixed storage.
Each entry is an Item or Group target, a current monitor ID, x/y basis-point
anchors (0–10,000) against that monitor's work area, and a stable z-order hint.

`wl_Lwlde_desktop_layout_replace()` validates the complete replacement before
copying it into the active array. Every ungrouped inventory item and each Group
must occur exactly once; a Group member cannot also be a standalone tile.
Unknown IDs/monitors, duplicated targets, invalid coordinates, incomplete
layouts and malformed caller-supplied snapshots are rejected transactionally.
The portable maximum is 4,096 inventory items plus 64 Groups (4,160 placements).

The layout only describes presentation intent. It does not move files, change
shortcuts, modify Explorer or make a Group visible. The native presenter and
crash-safe restoration path are still required.

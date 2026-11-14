# WLDE — WinLook Desktop Engine (WALDE)

## Ownership
WLDE is one integrated subsystem. Wallpaper runtime, local wallpaper library and the community Store/catalogue client all belong inside WLDE; this is not a separate Wallpaper Engine product.

WLDE owns wallpaper playback/rendering, wallpaper asset formats, library/Store, widgets, Rainmeter compatibility, desktop icon presentation/virtual Groups, visibility-aware rendering, resource adaptation, Game Mode and per-monitor wallpaper placement.

## DWM and desktop integration
Vista+: DWM remains the compositor. Windows 8+: composition is always on. Win95-XP: USER/GDI/Shell adapter. Desktop-layer placement/parenting techniques are version-specific and must not be assumed stable public APIs. Never modify Explorer.exe on disk or disable/replace DWM.

Sources: [DWM overview](https://learn.microsoft.com/en-us/windows/win32/dwm/composition-ovw), [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/desktop-window-manager-is-always-on).

## Visibility-aware rendering
WLDE should avoid drawing wallpaper regions that are fully occluded when visibility can be determined reliably. Distinguish exposed, fully occluded, translucent/contributing and unknown regions. Observe window/display state, maintain per-monitor regions, update dirty rectangles and clip rendering. If accurate occlusion tracking costs more than it saves or the OS doesn't provide reliable data, use a conservative fallback.

Video decoding is not the same as final composition: clipping a draw rectangle does not necessarily let a decoder skip processing the full compressed frame. Profile decode, upload, shader and composition costs separately.

## Shared animation engine and frame scheduling

Wallpaper, widget and desktop-transition animation must use the portable
WinLook animation cores rather than implementing a separate per-widget timing
system:

- Rust hosts use `wl_animate` (also re-exported by `wl_ui::animate`).
- C89 hosts use `wl_Lanimate`.
- One host-owned `FrameClock` / `wl_Lanimate_frame_clock` is shared by the
  whole visible WLDE scene. Its frame delta advances all active animation
  tracks; no animation creates its own timer, worker thread or message pump.
- The default target is 60 FPS with a 100 ms maximum delta after stalls. Hosts
  pause frame requests while the scene is hidden, fully occluded, suspended by
  Game Mode or has no active animated content, then resume from a fresh time
  baseline.
- Simple transforms and opacity should remain on the OS compositor/native
  presentation path when the backend supports it. The animation libraries
  calculate values; they do not upload textures, draw frames or guarantee
  GPU acceleration.
- Resource governor policies may lower the host target rate, stop requesting
  frames, or pause animation playback. Media decoding and GPU composition are
  measured separately from property interpolation.

This gives wl_ui, the WinLook Manager and future WLDE renderers the same timing
and easing semantics across Windows and non-Windows hosts. It does not imply
that a WLDE renderer exists in this branch yet; the engine's platform-specific
presentation and resource-governor integrations remain separate work.

## Resource-aware policy
Monitor CPU, GPU engine usage, RAM, VRAM, WLDE's own cost, power state, full-screen/game state independently. The user's intended threshold is to stop animated playback/rendering when system resource pressure exceeds 90% under a sustained policy. Exact interpretation per resource, sampling window and recovery hysteresis remain to be finalized.

Quality policy: preserve the user's selected source image/video fidelity. Under sustained pressure, reduce the amount and frequency of WinLook-owned work: defer nonessential updates, stop optional interactive/effect work, pause background or occluded animation and media decoding, hold the last-known-good frame, release recoverable decoder/GPU caches, and suspend WLDE when needed. Never silently downsample or re-encode the user's source assets. Foreground apps/games take priority.

## Portable resource-pressure controller

Rust wl_wlde::ResourceGovernor and C89
wl_Lwlde_resource_controller accept host-supplied CPU, GPU, RAM and VRAM
utilization percentages. Values outside 0..=100 are rejected. The controller
uses the maximum of the four domains rather than an average, so a single
saturated resource cannot disappear into a blended score.

The policy is explicit: recovery threshold < Optimize threshold < Suspend
threshold <= 100, with non-zero consecutive windows for entering and recovering.
The controller is allocation-free, creates no sampler/thread and never claims
that CPUv/GPUv admission tickets are OS telemetry. The returned pressure maps
to the common scene policy: Normal keeps configured cadence, Optimize requests
30 FPS, Suspend stops animated frame requests. A caller feeds each telemetry
sample and applies the returned pressure to the scene.

The 90% severe-pressure figure remains a design trigger from the plan, not a
hardcoded default; platform sampling cadence, sensor quality, WLDE-owned cost
attribution and Win10/Win11 providers remain open. Tests prove only the
portable state machine, not the quality of OS telemetry.

## Game Mode
Per-app profiles: Normal, Optimize, Suspend. Optimize reduces target frame cadence and nonessential background work, suspends optional interactive work, and preserves configured source-media fidelity. Suspend stops animation and unnecessary renderer/decoder work where feasible. Detect through several signals, not executable name alone. Examples include MW4, Roblox and Minecraft but are not hardcoded special cases. Resume smoothly after game exit/focus loss.

## Desktop Groups
- Disabled by default.
- Groups are virtual presentation objects, never physical folders.
- Original desktop shortcuts/Shell items remain at original locations and keep their targets.
- WLDE displays a custom Group icon in place of member icon representations.
- Clicking opens a borderless popup expanding around the Group icon in available screen space; popup shows original items, activation delegated to Windows/Shell.
- Crash/disable/uninstall restores original icon visibility.
- Local Groups are machine-specific and excluded from WLT.
- Fancy Groups are a separate portable concept not yet specified.

## Wallpaper formats
- .wl.wp: normal wallpaper asset/package; preserve source bytes/quality and avoid forced lossy re-encoding.
- .wl.i.wp: interactive wallpaper archive/package containing media, scripts, images, audio/shaders, manifest, permissions and preview.
- Interactive content must run in a restricted runtime with explicit capabilities.
- Archive compression cannot recover detail already lost in original lossy encoding.

## Store/community
The Store is integrated into WLDE. Support user-selected/community-operated catalogs, GitHub releases, self-hosted indexes, independent websites and local import. Avoid mandatory dependence on one central WinLook-operated server. Catalog entries should expose publisher, package hash/signature, license, OS/capability compatibility, permissions and resource hints. Local wallpapers work offline.

## Rainmeter
Parsing .rmskin is not full runtime compatibility. Research measures, meters, variables, bangs/actions, plugins, manifest variations and licensing. See [Rainmeter documentation](https://docs.rainmeter.net/) and [source](https://github.com/rainmeter/rainmeter).

## Detailed runtime design

The following are logical responsibilities; they do not require one executable per row.

| Logical component | Responsibility |
|---|---|
| Wallpaper library | Local asset index, metadata, preview and source tracking |
| Wallpaper host | Decode, schedule and present selected wallpaper |
| Interactive content host | Restricted runtime for interactive wallpaper content |
| Widget host | Layout, input routing and isolated widget execution |
| Desktop presenter | Desktop-owned visual surfaces and presentation state |
| Group presenter | Virtual Group icon and popup presentation |
| Resource governor | Sampling, quality policy and suspend/resume |
| Catalogue client | User-selected indexes, download and package verification |

WLDE owns desktop presentation and wallpaper/widget playback. WLDM supplies monitor information. WLWM may supply window visibility/occlusion hints. Wallpaper and widget hosting must remain useful when WLWM is absent.

## Wallpaper lifecycle

1. Discover a local asset or catalogue entry.
2. Validate metadata, file sizes, hashes, declared permissions and compatibility.
3. Resolve target monitor(s), scale mode and crop policy.
4. Decode or initialize in a bounded worker.
5. Present a static/first frame.
6. Enter scheduled playback only when visibility and resource policy permit.
7. Respond to pause, display change, lock/session transition, Game Mode and power changes.
8. Release decoder, GPU and GDI resources on stop/device loss.
9. Fall back to a static frame or native Windows background when initialization/recovery fails.

Preserve original media bytes. Generated previews/caches are disposable and must not replace the source asset.

## Render scheduling and resource governor

Track resource domains independently: WLDE CPU cost, overall CPU pressure, GPU engine pressure, RAM, VRAM, decoder workload, power state, foreground/full-screen state and display topology. A single aggregate system-use number can hide a saturated GPU or memory bottleneck.

The intended severe-pressure threshold is 90%. Sampling window, sustained duration, per-resource aggregation and recovery hysteresis remain open. Until ratified, 90% is a design trigger, not a hardcoded implementation constant.

Proposed quality ladder:
1. Normal configured quality and frame rate.
2. Reduce target frame rate and defer nonessential background updates.
3. Stop optional effects, audio-reactive work and expensive widget updates without changing source-media fidelity.
4. Pause background/occluded animation and media decoding.
5. Freeze a last-known-good static frame and release recoverable decoder/GPU caches where safe.
6. Suspend the render/decode pipeline and use a lightweight/native fallback.

Distinguish a brief spike from sustained pressure. Recovery needs hysteresis to avoid oscillation. The user may choose a stricter profile, but not a setting that prevents preserving system responsiveness.

## Game Mode contract

- **Normal:** respect wallpaper/widget settings and normal resource policy.
- **Optimize:** reduce render rate and nonessential background updates, suspend optional interactive work and preserve configured source-media fidelity.
- **Suspend:** stop animation, decoding and interactive processing where feasible; keep the minimum safe static/native presentation.

Detection combines foreground ownership, fullscreen state, user-selected application rules and system signals. Executable names alone are insufficient. User override must be possible. Return to Normal predictably when the trigger ends.

### Portable per-application resolver

The paired Rust `resolve_game_mode()` and C89
`wl_Lwlde_game_mode_resolve()` APIs accept a host-supplied executable
SHA-256 fingerprint, foreground/fullscreen flags, an optional OS Game Mode
signal and up to 64 unique per-application profiles. A matching user profile
wins first, including an explicit Normal override. If no profile matches, an
enabled OS Game Mode fallback wins over an enabled fullscreen fallback;
otherwise the result is Normal. Background applications always resolve to
Normal.

The resolver allocates nothing and does not enumerate processes, inspect/hash
executable files, install hooks or poll Windows. The host is responsible for
obtaining a trustworthy fingerprint and the detection signals, persisting
profiles and applying the returned mode to the scene. Windows-specific
detection, profile storage and Manager editing remain open.

## Desktop Groups lifecycle and recovery

Groups stay disabled by default. They are virtual presentation objects and never filesystem folders.

1. Read current desktop item inventory without changing it.
2. Resolve selected members to stable-enough references and record original visibility/position/presentation state.
3. Verify that the OS adapter can restore native items after a crash.
4. Create the Group visual proxy only after the recovery prerequisite passes.
5. On activation, open a borderless popup in available work-area space; delegate activation to Windows/Shell.
6. On disable, renderer stop, session shutdown or uninstall, restore original presentation.
7. Reconcile missing/renamed/moved items without deleting or recreating user files.

The mechanism for suppressing duplicate native icon presentation remains unresolved and is a release blocker on any OS where reliable restoration cannot be demonstrated. Do not substitute filesystem folders or move original shortcuts.

Local Groups are machine-specific and excluded from WLT. Fancy Groups remain a separate, unapproved portable feature.

## Adaptive application icons and context menus

WLDE application tiles use an adaptive-icon model inspired by Android launchers:
separate foreground and background artwork, an optional monochrome layer for
themed surfaces, a selectable shape mask and a configurable safe-zone scale.
The default safe zone is 61% of the tile. Native icon assets are referenced,
not rewritten; the Shell adapter is responsible for obtaining the correct app
identity and artwork.

The portable app-icon renderer accepts decoded RGBA layers from the platform
adapter, center-crops the background, contains the foreground inside the
configured safe zone, composites in premultiplied-alpha space and applies a
2x2 antialiased mask. Output sizes are bounded to 16–512 pixels and the
caller caches the result; source artwork and shortcut resources remain
unchanged. The C89 equivalent writes into caller-owned storage and allocates
nothing.

The portable context-menu contract is type-aware. Applications expose Open,
Add/Remove from Group, Customize Icon and Properties; files/folders expose
Open, Add/Remove from Group and Properties; Groups expose Open Group, Add
Members, Rename, Ungroup and Properties. The platform host must delegate
native Open/Properties verbs to the Shell and preserve existing Shell menu
behaviour. This is a model of WLDE additions, not an Explorer replacement.

The first Rust/C89 slice validates virtual Group membership and adaptive icon
metadata. The next slice now composes decoded RGBA foreground/background layers
into cached app-icon bitmaps with safe-zone scaling and Circle/RoundedSquare/
Squircle masks in both portable cores. It still does not enumerate the live
desktop, extract/decode Shell icon assets, persist references, render desktop
tiles or Group popups, intercept right-click or suppress native desktop icon
representations. Groups stay disabled until the OS adapter proves crash-safe
restoration.

## Required Explorer Desktop Bridge

The Explorer Desktop Bridge is a mandatory BGW/WLDE runtime subsystem for
Windows sessions where Explorer owns the desktop. It applies across supported
Windows generations, not only Windows 10 and 11. BGW discovers which process
actually owns the current desktop view, records its OS/build/architecture and
selects the matching native adapter. The same logical contract is shared
across generations; APIs, event paths and hook mechanics are version-specific.

The bridge supplies desktop item identity/inventory, application icon
presentation, virtual Group proxy coordination and context-menu event routing.
WLDE owns the visuals and Group configuration. Windows Shell keeps ownership
of filesystem operations, native activation, file associations, permissions,
drag/drop and item semantics. DWM continues to compose the desktop; WinLook
does not replace DWM.

Use documented Shell COM extension points where they satisfy a required
operation. Where the desktop view needs interception or presentation beyond
those public interfaces, the generation-specific adapter may install a
controlled runtime hook inside the actual Explorer desktop process. This is
part of BGW/WLDE's required integration, not a per-feature user opt-in.
Hooks must be narrowly scoped to the desktop host, version/architecture
validated, short-running, observed by BGW and recoverable. Do not patch,
replace or redistribute Explorer.exe on disk, and do not attach to unrelated
Explorer folder-window processes.

Heavy rendering, media decode, file enumeration, network requests, Group
persistence and policy decisions remain in BGW/WLDE outside Explorer's UI
callbacks. The in-process side passes bounded events and immutable item
references across authenticated per-user IPC. If an adapter is unknown or
fails verification, BGW keeps native desktop icons visible and disables only
the unsupported WLDE presentation capability. This fail-safe is a recovery
state, not an optional user configuration.

Crash/restart handling is required: the bridge detects Explorer process
replacement, reconciles desktop inventory and reattaches without duplicating,
deleting or recreating original Shell items. Before any native icon
representation is suppressed, WLDE must persist enough restoration state to
show the original items after BGW/Explorer crash, session exit, disable,
upgrade and uninstall.

## Widgets, interactive content and Rainmeter

Widgets and interactive wallpapers are untrusted content. They need explicit resource, input, filesystem, network and process capabilities. Installation does not grant broad access automatically.

Rainmeter support must have explicit levels:
- package recognition/import only;
- parsing and preview of a documented subset;
- runtime support for documented measure/meter/action subset;
- optional plugin support after licensing and isolation review.

The supported subset is open. Parsing an RMSKIN archive is not equivalent to executing Rainmeter skins/plugins. Preserve attribution/licensing and do not bundle third-party plugins without permission.

## Multi-monitor and visibility contract

- Consume current WLDM topology rather than caching monitor geometry indefinitely.
- Define monitor arrival/removal, primary changes, docking, remote sessions and topology behaviour.
- Persist user profile intent separately from physical monitor identifiers.
- Treat unknown occlusion as visible/contributing unless conservative rendering is known safe.
- Measure decode, CPU copy, GPU upload, composition and presentation separately; clipping final drawing does not guarantee lower decode cost.

## Modern Win32 monitor snapshot adapter

The modern Rust adapter lives in `rust/library/wl_wlde_win32`; the portable
`wl_wlde` core and C89 `wl_Lwlde` remain platform-neutral. The adapter uses
`EnumDisplayDevicesW` to select desktop-attached, non-mirroring GDI display
devices, then `EnumDisplayMonitors` and `GetMonitorInfoW(MONITORINFOEXW)`
to collect the GDI device name, virtual-screen monitor rectangle, work-area
rectangle and primary flag. It rejects more than 16 monitor records and
returns an owned validated snapshot; no HMONITOR is retained.

`refresh_topology` constructs the complete replacement before assigning it,
so a failed enumeration never destroys the last known-good topology.
`should_refresh_topology` identifies `WM_DISPLAYCHANGE` and
`WM_DEVICECHANGE/DBT_DEVNODES_CHANGED`. The host owns message dispatch and
decides when to coalesce/refresh. No timer, background polling or hook is
created by the adapter.

GDI device names are only current-session snapshot keys. They are not stable
physical monitor identifiers and must not be used as durable profile IDs.
Per-monitor DPI, orientation, display color profile, session/remote-session
behaviour and real Win10/11 hot-plug verification remain open. The C89 core
continues to accept adapter-supplied snapshots; a C host-specific enumerator is
a separate platform task.



### Initial BGW Explorer host monitor

The independent `winlook-bgw` Cargo target now hosts the first process monitor.
It opens the actual desktop-owning Explorer process with query/synchronization
rights only, verifies the process creation time, and waits on the process
handle instead of polling continuously. BGW revalidates the Shell desktop
window owner every five seconds and rediscovers after process exit or owner
replacement; if no compatible Explorer desktop is present, discovery retries
every two seconds.

This is only the BGW lifecycle foundation. It does not yet load a COM extension
or in-process hook, enumerate PIDLs, suppress native icons, draw Groups, store
restore state, render wallpaper, register Windows startup or communicate with
the Manager. The Explorer Desktop Bridge remains mandatory product
infrastructure, but user item presentation stays native until the recovery
journal and platform adapter are implemented and tested.

### Explorer desktop-host identity probe

The modern Win32 adapter resolves the Shell desktop window with
`GetShellWindow` and uses `GetWindowThreadProcessId` to identify its
owner. It checks the executable basename, verifies the host and BGW are in the
same interactive session, and records PID plus process creation time. The
creation time distinguishes a restarted Explorer instance from a PID-reuse
case. BGW must re-probe after Explorer restart and before reconnecting.

This Rust probe uses `ProcessIdToSessionId` and
`QueryFullProcessImageNameW`, so it is a Vista-and-newer adapter, not a
universal Windows 95–XP implementation. Legacy targets require their own
classic Shell/process lookup and must not import these APIs unconditionally.
The probe does not perform injection or attach a hook; it establishes the
specific desktop process to which the later version-specific integration is
allowed to attach.

### WLDE acceptance gates

- [ ] Static wallpaper and display-change handling are specified per OS adapter.
- [ ] Resource policy defines sampling, thresholds, hysteresis and fallback.
- [ ] Game Mode transitions are deterministic and user-overridable.
- [ ] Interactive content is isolated with declared capabilities and resource limits.
- [ ] Crash/device loss returns to a usable desktop.
- [ ] Groups remain disabled unless original icon presentation is restored after crash/disable/uninstall.
- [ ] Explorer Desktop Bridge is hosted by BGW/WLDE for every verified Explorer-owned desktop session across supported Windows generations.
- [ ] Per-version adapters detect the actual desktop owner, monitor Explorer restart and preserve native fallback on unknown builds.
- [ ] Adaptive app-icon extraction, safe-zone rendering and icon-theme fallback work without rewriting source assets.
- [ ] WLDE right-click additions preserve native Shell verbs, accessibility, keyboard navigation and OS-version-specific menu behaviour.
- [ ] Rainmeter compatibility is an explicit subset and licensing is reviewed.



## Portable scene-core implementation — 2026-11-14

The first Rust/C implementation layer now exists as paired libraries:

- Rust: rust/library/wl_wlde (package wl_wlde).
- C: C/library/wl_Lwlde (C89, caller-owned fixed-capacity storage).
- Both cores validate a maximum 16-monitor topology, keep monitor bounds and work areas separate, and calculate Fill/Fit/Stretch/Center placement.
- Both use one scene-owned wl_animate/wl_Lanimate clock for all active tracks. Normal schedules at 60 FPS, Optimize and Power Saver at 30 FPS, and Suspend/known full occlusion/static-only states request no animation frames. Unknown visibility remains eligible.
- Rust owns CPUv/GPUv governor instances. C accepts non-owning references to initialized wl_Lcpuv/wl_Lgpuv governors. Estimates and completion signals remain caller-supplied admission/accounting values, not system telemetry.
- Presentation capabilities are explicit. Interactive content remains unsupported unless an isolated adapter reports it; Groups require both virtual presentation and crash-safe restore capabilities.

This is a portable scene/policy core, not a functioning desktop renderer. The branch does not yet implement OS monitor enumeration, static/animated desktop surfaces, image/video decoding, package parsing, widget execution, Store/catalogue, Rainmeter, Groups, or device-loss recovery. Do not mark the WLDE acceptance gates below complete from these portable tests.

For Windows 8 and later, Microsoft's IDesktopWallpaper API documents setting/querying static wallpaper paths per monitor. It is not an animated frame-surface API. Windows.UI.Composition provides application visual composition, but that capability alone does not define a supported desktop-wallpaper layer behind Explorer icons. A future adapter must establish that surface/ownership contract with documented behaviour and restoration; do not assume WorkerW/Progman parenting. References: [IDesktopWallpaper::SetWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setwallpaper), [IDesktopWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper), [Windows.UI.Composition](https://learn.microsoft.com/en-us/windows/apps/develop/ui/visual-layer/).


## Wallpaper Store — integrated provider hub (2026-11-14)

The Manager owns the Store UI; the portable WLDE cores own stable provider
metadata, not provider-specific networking or OS process launch.

Built-in provider contracts currently include:

| Provider | Access | Price class | Manager action |
|---|---|---|---|
| Wallpaper Engine | Existing installed application + browser | Mixed | Browse to its executable, open Steam Workshop, choose a downloaded/local wallpaper and send the documented control command |
| Openverse | Browser | Free/open media | Open the official image catalogue; verify the original source and exact license |
| Wikimedia Commons | Browser | Free/open media | Open media search; follow the file-page license and attribution |
| NASA Image and Video Library | Browser | Free access subject to media rights | Open NASA search; check credits and third-party copyright notices |

The Rust provider list lives in rust/library/wl_wlde/src/store.rs; the
matching C89 table is returned by wl_Lwlde_store_provider_get(). The Rust-only
Wallpaper Engine bridge lives in rust/library/wl_wlde/src/wallpaper_engine.rs,
with OS file-picking and Manager UI in the Win32/WinUI adapters.

For Wallpaper Engine, Steam's official documentation identifies app ID
431960 for Workshop files. The bridge checks common Steam roots and
libraryfolders.vdf, starts the file picker at
steamapps/workshop/content/431960 when present and sends the documented
-control openWallpaper -file command to the existing Wallpaper Engine
executable. The application must already be running. It does not edit the
Workshop directory, initiate subscriptions, unpack packages or redistribute
content.

The free catalogues are linked as providers, not scraped or mirrored. Openverse
explicitly warns that license information can be inaccurate and must be
verified; Wikimedia terms vary per file; NASA media can contain credited
third-party works. The UI's Free label therefore never replaces the item's
rights record.

Still open: in-app catalogue API integration, categories/search/filtering,
previews/details, downloads/resume/hash verification, attribution sidecars,
package import, local library management, independent community catalogues,
moderation and publisher identity.

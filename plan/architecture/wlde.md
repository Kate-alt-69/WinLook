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

Quality ladder: normal configured quality; lower FPS; lower resolution/render scale; disable expensive effects; pause expensive interactive work; freeze/static frame; suspend animation/rendering when pressure is severe or profile says Suspend. Foreground apps/games take priority.

## Game Mode
Per-app profiles: Normal, Optimize, Suspend. Optimize lowers FPS/resolution/effects and background work. Suspend stops animation and unnecessary renderer/decoder work where feasible. Detect through several signals, not executable name alone. Examples include MW4, Roblox and Minecraft but are not hardcoded special cases. Resume smoothly after game exit/focus loss.

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
2. Reduce target frame rate/background updates.
3. Lower render scale or effect quality.
4. Stop nonessential audio-reactive, particle or expensive widget work.
5. Pause background/occluded animation and interactive work.
6. Freeze a last-known-good static frame where safe.
7. Suspend render/decode pipeline and use lightweight/native fallback.

Distinguish a brief spike from sustained pressure. Recovery needs hysteresis to avoid oscillation. The user may choose a stricter profile, but not a setting that prevents preserving system responsiveness.

## Game Mode contract

- **Normal:** respect wallpaper/widget settings and normal resource policy.
- **Optimize:** reduce render rate/scale/effects, suspend background refresh and lower nonessential work.
- **Suspend:** stop animation, decoding and interactive processing where feasible; keep the minimum safe static/native presentation.

Detection combines foreground ownership, fullscreen state, user-selected application rules and system signals. Executable names alone are insufficient. User override must be possible. Return to Normal predictably when the trigger ends.

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

### WLDE acceptance gates

- [ ] Static wallpaper and display-change handling are specified per OS adapter.
- [ ] Resource policy defines sampling, thresholds, hysteresis and fallback.
- [ ] Game Mode transitions are deterministic and user-overridable.
- [ ] Interactive content is isolated with declared capabilities and resource limits.
- [ ] Crash/device loss returns to a usable desktop.
- [ ] Groups remain disabled unless original icon presentation is restored after crash/disable/uninstall.
- [ ] Rainmeter compatibility is an explicit subset and licensing is reviewed.


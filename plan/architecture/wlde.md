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

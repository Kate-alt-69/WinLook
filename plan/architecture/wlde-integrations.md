# WLDE External Engine Integrations

**Status:** first Rust command adapters implemented; Manager/BGW integration and per-monitor presentation arbitration remain future work.

This document describes how WLDE hands selected work to software the user has already installed. These integrations are separate from WLDE's native wallpaper renderer and separate from Store/catalogue providers.

## Design rule

Use the other product's documented public automation interface and let that product own its runtime. Do not embed another full wallpaper/desktop manager inside BGW, copy its engine source, call its private IPC protocol, or load its third-party plugin DLLs into WinLook.

All process invocations are constructed with Rust Command and separate arguments. No cmd.exe, PowerShell, concatenated shell command, arbitrary bang string, executable wallpaper, or automatic package install is exposed by these first adapters.

The selected executable path is user-provided through a future Manager Browse workflow. The Rust API checks that the file exists and has an expected basename; it does not validate the digital publisher signature. A later Manager install-discovery service may add signature and install-record verification.

## Integration capability matrix

| Integration | First supported hand-off | External runtime requirement | Explicitly not supported |
|---|---|---|---|
| Wallpaper Engine | Existing adapter: open a local project/media target via its documented control arguments | Wallpaper Engine must already be running; Steam owns Workshop subscriptions and downloads | Workshop package extraction, redistribution, subscription automation |
| Lively Wallpaper | Set/close wallpaper, playback state, volume and a named property through Lively's documented command utility or main executable | Lively must already be running. Prefer livelycu.exe; direct main-executable dispatch may have an extra delay | Executable/app wallpapers, private named-pipe gRPC, embedding its player, launching or supervising its player processes |
| Rainmeter | Activate/deactivate/refresh a config or refresh all through Rainmeter command-line bangs | Selected Rainmeter installation; Rainmeter owns skin execution and update scheduling | Parsing and executing all Rainmeter skins, loading plugin DLLs in BGW, silently installing .rmskin files |

The first API is exposed by the Rust wl_wlde crate. It does not yet provide matching C89 process-launch functions. The existing C89 provider metadata API remains metadata only.

## Lively Wallpaper

Lively documents a command-line interface for third-party automation. Commands can be sent to the running main executable or through the separate livelycu command utility, which includes its own help. The adapter treats both as user-selected executable paths and makes no timing guarantee between them. The documented controls include setwp, closewp, app --play, app --volume, seekwp and setprop.

The Rust adapter exposes:

- LivelyInstall::from_executable for a user-selected lively.exe, livelywpf.exe or livelycu.exe.
- LivelyTarget::new for a directory containing LivelyInfo.json, supported ordinary media, or an HTML file.
- set_wallpaper(target, monitor), close_wallpaper(Monitor(index)|All), set_playback(bool), set_volume(0..=100), seek_wallpaper(monitor, Absolute(percent)|Relative(delta)), and set_property(name, value, optional monitor).
- Command builders for inspection/testing; arguments are passed separately, never through a shell.

The initial target whitelist follows Lively's documented media set but deliberately excludes executable wallpapers. This avoids turning an external wallpaper integration into an arbitrary application launcher. Project directories must contain LivelyInfo.json. Web/HTML files are still active content and should be selected only when the user trusts the source.

The command utility is a client of a running Lively installation. A successful process spawn only means that the CLI process started; it is not proof that the target wallpaper was accepted or presented. A future integration status channel must observe the command exit and, where possible, query supported state through a documented interface. Do not couple WinLook to Lively's private named-pipe gRPC service.

Lively's current source separates its Core from UI and selects player implementations through a wallpaper-player factory (for example WebView2/Cef, MPV, VLC and WMF). That is a useful design reference for WLDE's future backend selection: define a narrow player capability contract and keep each decoder/runtime behind its own adapter. It is not a reason to copy or embed Lively's GPL-3 implementation.

## Rainmeter

Rainmeter's official command-line interface accepts documented bangs as arguments to Rainmeter.exe. The first adapter exposes only typed operations:

- Activate a config, optionally selecting a simple .ini file.
- Deactivate a config.
- Refresh a config.
- Refresh all Rainmeter configs.

RainmeterConfig normalizes relative folder paths and rejects empty components, traversal, drive-qualified paths, quotes and control/invalid filename characters. RainmeterIni accepts only a simple .ini filename. These types prevent the Manager from forwarding an arbitrary bang or filesystem path as a config.

For a selected .rmskin package, RainmeterSkinPackage checks that the chosen file exists and has the .rmskin extension. It does not parse, unpack or install the archive. The official installer is responsible for showing the components and asking for confirmation. The portable-install workflow is to start SkinInstaller.exe beside Rainmeter.exe and browse to the package; the WinLook adapter therefore does not invent a package-path command-line argument.

Rainmeter packages may contain custom 32-bit/64-bit plugin DLLs. The Rainmeter plugin API has lifecycle callbacks and executes within the Rainmeter host. WinLook must not load those DLLs into BGW or treat a skin as a passive image. The initial integration is an out-of-process control bridge, not full Rainmeter runtime compatibility.

Version awareness matters. The current release API lists Rainmeter 4.5.28 (October 12, 2026) as the latest stable 4.5 release and 5.0.0 RC2 (November 16, 2026) as a prerelease. The 5.0 change notes require 64-bit Windows 10 version 1903+ or Windows 11. There is a documentation mismatch: the version-history page calls 4.5.27 the last version for older Windows/32-bit systems, while the current home page labels the 4.5.28 download for Windows 7 or later. Do not hard-code a 4.5.28 OS/architecture support assumption from those summaries; verify the actual installer/build matrix before enforcing compatibility. The current adapter does not read an installed Rainmeter version or advertise per-skin compatibility.

## Per-monitor wallpaper presenter ownership

The portable MonitorPresentationMap validates a complete current-topology
mapping with exactly one exclusive wallpaper presenter per monitor. Supported
presenter values are native WLDE, Wallpaper Engine and Lively. It supports
mixed multi-monitor layouts, such as one engine on the primary monitor and a
different engine on a secondary monitor, while preserving the last valid map
when a candidate becomes stale or incomplete.

Rainmeter is intentionally not in this exclusive owner enum: a Rainmeter skin
is a widget overlay, not a full wallpaper presenter. Its own config activation
can coexist with a selected wallpaper engine, subject to future overlap,
z-order and input-routing policy.

This portable map is configuration only. It does not start or stop any process,
reserve a desktop surface, or confirm that a command completed. The future
Manager/BGW IPC layer must sequence hand-off operations and collect command
results before committing a runtime-active presenter.

## Presentation ownership and resource policy

These adapters are explicit hand-off APIs, not an automatic multi-engine compositor. Before Manager automation enables an external wallpaper per monitor, WLDE needs a per-monitor presentation-owner policy so its native presenter and an external player cannot unknowingly paint the same display. That arbitration is not implemented yet.

BGW may call only documented external controls that the chosen integration advertises. It must not terminate or suspend unrelated applications to enforce Game Mode. Unsupported pause/volume/property capabilities stay unsupported; never infer them from process names.

All calls must remain outside Explorer callbacks and wallpaper frame loops. A command invocation is control-plane work. The returned Child is the command process handle, not a lifetime handle for the external engine's wallpaper/player.

## Licensing and reuse boundary

- Lively Wallpaper source is GPL-3. WinLook uses its public CLI as an external application and does not copy or bundle its source in this integration.
- Rainmeter core source is GPL-2. WinLook uses its documented public bangs and official installer workflow; it does not copy the Rainmeter core/plugin SDK implementation or load Rainmeter plugin binaries.
- The M0 CorralWindows sample is MIT-licensed and was inspected as a read-only Desktop-view reference. The WinLook Desktop probe is its own implementation with additional BGW ownership/lifetime checks; no source file was copied.

Any future decision to bundle, modify or distribute third-party binaries/source needs its own licence and attribution review. An external-process bridge does not grant rights to repackage wallpapers, skins, plugins or creator assets.

## Dependency-ordered follow-up

1. Wire the typed Rust APIs through authenticated Manager/BGW IPC; keep executable selection and external-process output visible to the user.
2. Add install/version discovery and executable publisher verification. Do not silently scan arbitrary disks.
3. Add command completion/result reporting and supported state checks. A spawned child is not an applied wallpaper/skin acknowledgement.
4. Add per-monitor presentation ownership and conflict prompts between native WLDE, Wallpaper Engine and Lively.
5. Research a read-only Rainmeter skin compatibility subset (measures/meters/variables) separately from arbitrary plugin execution.
6. Define an explicit trust/permission model before any interactive wallpaper or third-party plugin can run in a WinLook-owned host.

The portable command adapters are implemented in wl_wlde. GitHub Actions must verify formatting, Clippy, tests, Windows builds, and security analysis for the exact integration commit.

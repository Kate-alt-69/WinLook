# WinLook — Living Project Plan

> **Document type:** Living planning checklist and architecture decisions  
> **Project:** WinLook  
> **Repository:** [Kate-alt-69/WinLook](https://github.com/Kate-alt-69/WinLook)  
> **Status:** Planning plus early portable animation prototypes; see implementation checkpoints for tested and unverified work.  
> **Primary document:** `Plan.md`  
> **Future consolidated specification:** `Final-Plan.md`

This file records the full scope discussed for WinLook. It is intentionally broader than the Desktop Engine alone: it covers the Windows appearance system, icons, shell integrations, desktop rendering, wallpapers and their Store, widgets, extensions, Standby, security, compatibility, restoration, testing, and releases.

## How to use this checklist

- `[x]` means the initial planning/design point has been discussed and captured in this conversation. **It does not mean code has been implemented or tested.**
- `[ ]` means the item still needs research, a decision, a specification, implementation, or verification.
- When an item is completed, add evidence where useful (document path, decision, test, or PR).
- Keep this file as the evolving checklist. Do not turn it into the final specification prematurely.
- Once the major architecture and unresolved decisions are settled, consolidate the approved decisions into `Final-Plan.md`.
- Detailed designs live under `architecture/`; per-Windows research lives under `compatibility/`; technical evidence lives in `references/SOURCES.md`.

---

# 1. Product vision and boundaries

WinLook is intended to be a comprehensive Windows customization platform. It should change as much of Windows' user-facing appearance and desktop experience as is practical, while prioritizing system stability, security, reversibility, and compatibility.

The ambition is to cover Windows 95 and later generations, but feature availability must be determined by the actual capabilities of each Windows generation. The product must not pretend that every feature can work identically on every release.

## Product identity

- **WinLook:** The main Windows customization platform and manager.
- **WLDE — WinLook Desktop Engine** (pronounced **WALDE**): WinLook's integrated desktop, wallpaper, widget, rendering, and desktop-content engine.
- **WLWM — WinLook Window Manager (WELMO):** augments Windows window management while leaving the Windows windowing system and DWM intact.
- **WLDM — WinLook Display Manager:** monitor/display discovery, profiles and settings.
- **WLOSDE — WinLook On-Screen Display Engine:** customizable WinLook notifications, popups and OSD.
- **wl_animate:** reusable Rust animation crate/API for modern WinLook modules and third-party integrations.
- **wl_Lanimate:** corresponding legacy C animation library.
- **WLT — WinLook Template:** portable settings/assets package using the proposed .wl.t archive.
- **Wallpaper Store:** integrated WLDE feature and community catalogue client, not a separate Wallpaper Engine product.
- **WinCheck is permanently out of scope and will never be a WinLook product.**

## Core principles

- [ ] Define the complete product mission and non-goals.
- [ ] Define supported Windows versions and release tiers.
- [ ] Define what “full customization” means for each OS generation.
- [ ] Define a feature-capability matrix so unavailable features are reported honestly.
- [ ] Define the boundary between supported/native customization and optional experimental integrations.
- [ ] Finalize product branding and release names for the agreed subsystems.
- [ ] Define supported hardware/performance tiers, including low-end systems.
- [ ] Define accessibility requirements and high-contrast/reduced-motion behaviour.

## Non-negotiable safety and stability rules

- [x] Protect the Windows shell from direct executable modification as a core design requirement.
- [ ] **Never patch, replace, binary-modify, or make WinLook dependent on a modified `explorer.exe`.**
- [ ] Do not inject arbitrary code into Explorer as a normal customization strategy.
- [ ] Prefer documented Windows APIs, supported personalization settings, separate processes, and version-specific adapters.
- [ ] Keep shell integrations optional and isolated so a failure can be disabled without taking down the desktop.
- [ ] Never capture, store, transmit, simulate, or process the user's Windows password or PIN.
- [ ] Never assume a normal WinLook window can appear above or replace the secure Windows sign-in desktop.
- [ ] Every system change must have a known restoration path wherever technically possible.
- [ ] Do not silently apply unsupported registry hacks or modify protected system files.
- [ ] Require clear warnings and explicit opt-in for experimental or system-wide changes.

---

# 2. Core architecture — hybrid Windows implementations

The intended architecture uses Rust for modern Windows and a separate C implementation for legacy Windows. They share conceptual APIs and data contracts where feasible, but not one executable or identical runtime capabilities. The main WinLook background worker (BGW) coordinates WLWM process detection, compatibility evaluation/cache, injection and module health.

## Modern implementation — Rust

- [x] Define Rust as the modern core implementation.
- [x] Identify the Microsoft `windows` / `windows-sys` bindings as candidates for native Windows APIs.
- [ ] Choose the supported Rust toolchain, editions, minimum OS versions, and target triples.
- [ ] Define the process model and boundaries between Manager, per-user Agent, rendering workers, and optional Service.
- [ ] Define native UI and rendering technology; evaluate low-overhead options rather than assuming a browser-heavy shell.
- [ ] Define crash handling, structured logs, diagnostics, and safe restart behaviour.
- [ ] Define IPC between WinLook processes and validate all IPC messages.
- [ ] Define privilege boundaries and the smallest required privilege for each operation.

## Legacy implementation — ANSI C / lightweight C++

- [x] Define a separate legacy companion implementation.
- [x] Identify older toolchains such as Open Watcom or Visual Studio 6-era tooling for investigation.
- [ ] Verify compiler, linker, runtime, CPU, and API compatibility for each intended legacy OS.
- [ ] Define the exact first legacy targets (Windows 95 / 98 / ME) and investigate NT-family versions separately.
- [ ] Define the legacy UI, GDI rendering, configuration parser, and installer constraints.
- [ ] Define how legacy code consumes shared configuration without requiring modern runtimes.
- [ ] Document features that cannot be supported on legacy systems.

## Shared foundation

- [x] Define Windows version detection as a core responsibility.
- [x] Design a capability-detection system.
- [x] Design a per-user WinLook Agent and optional privileged Service.
- [ ] Define the shared configuration format and schema.
- [ ] Define schema versioning, validation, defaults, and migration rules.
- [ ] Decide which settings use JSON, TOML, or INI and where legacy constraints require a simpler format.
- [ ] Define shared identifiers for themes, wallpapers, widgets, extensions, and profiles.
- [ ] Define a stable internal API between core modules.
- [ ] Define how unsupported configuration fields are handled on older Windows versions.
- [ ] Define update compatibility between Manager, Agent, Service, legacy companion, and packages.

## Proposed process responsibilities

- [ ] Specify the main Background Worker (BGW) as owner of WLWM target detection, compatibility evaluation/cache, injection orchestration, module health and exclusions.
- [ ] Define the local binary-encoded WLWM compatibility cache, including binary identity, architecture, OS/build, relevant mitigations, module versions, invalidation and confidence/reason codes.
- [ ] Ensure cache hits never bypass per-launch identity and safety checks.
- [ ] Keep injected modules small, versioned and recovery-aware.

- [ ] Specify **WinLook Manager**: user-facing settings, theme management, library, and Store UI.
- [ ] Specify **WinLook Agent**: per-user desktop, appearance, and session integration.
- [ ] Specify **WLDE Runtime**: wallpaper/widget rendering and desktop composition.
- [ ] Specify **WinLook Service**: only operations that genuinely require elevated privileges.
- [ ] Specify isolated workers for untrusted or complex extensions.
- [ ] Specify how all processes recover if another process crashes or is restarted.

---

# 3. Windows Appearance Engine

The Appearance Engine manages cohesive themes and the user-facing Windows appearance. Themes should be component-based: users can apply a complete theme or select individual parts.

## Theme system

- [x] Design a universal theme format.
- [x] Design Windows-specific appearance adapters.
- [ ] Define the theme package manifest and versioning.
- [ ] Define theme dependencies, optional components, and compatibility declarations.
- [ ] Define theme preview and per-component preview.
- [ ] Define partial theme application.
- [ ] Define theme import, export, sharing, and update behaviour.
- [ ] Define theme conflict detection and safe application order.
- [ ] Define theme validation before application.
- [ ] Define a clear “restore Windows defaults” action.

## Appearance areas to support

- [x] Design taskbar appearance integrations.
- [x] Design Start Menu customization.
- [x] Design window and title-bar customization.
- [x] Design fonts, cursors, and sound themes.
- [ ] Design desktop wallpaper and desktop surface integration through WLDE.
- [ ] Design window borders, shadows, transparency, blur, and supported DWM effects.
- [ ] Design supported notification and system-surface customization.
- [ ] Research which Settings and Control Panel surfaces can be themed without unsupported patching.
- [ ] Design per-user versus machine-wide appearance settings.
- [ ] Design light/dark and accent-colour integration.
- [ ] Design accessibility-aware theme variants.
- [ ] Document visual features that cannot be changed through supported interfaces.

## Taskbar and Start Menu

- [x] Design taskbar appearance integrations.
- [x] Design Start Menu customization.
- [ ] Research stable, OS-specific ways to change taskbar colors, transparency, sizing, and supported layout properties.
- [ ] Design pinned shortcut icon customization.
- [ ] Research the limits of changing icons for already-running applications.
- [ ] Design Start Menu layout and app-icon customization per Windows generation.
- [ ] Define optional replacement UI behaviour where native customization is not available.
- [ ] Ensure any replacement is independently hosted and does not replace or patch Explorer.
- [ ] Define how the original taskbar and Start Menu can be restored immediately.
- [ ] Test shell restart, display changes, multiple monitors, and Windows updates.

---

# 4. Universal Icon Engine

The Icon Engine is a first-class part of the Appearance Engine. It should allow a theme to replace matching icon families without modifying application executables.

## Icon coverage

- [x] Define system icon customization as a design area.
- [x] Define application icon mapping as a design area.
- [ ] Design the universal icon package and mapping manifest.
- [ ] Support Windows system icons where the target OS exposes a safe customization route (for example, This PC, Recycle Bin, Network, and other shell items).
- [ ] Support folder icons, including per-folder customization where supported.
- [ ] Support file-type icons and association-aware mappings.
- [ ] Support drive and removable-media icons where supported.
- [ ] Support application shortcut icons.
- [ ] Support pinned taskbar and Start Menu shortcut icons.
- [ ] Research safe options and limitations for icons shown by already-running applications.
- [ ] Support per-theme icon sets and fallback icons.
- [ ] Define icon formats, scaling, DPI handling, and high-resolution variants.
- [ ] Design icon cache refresh behaviour that does not destabilize the shell.
- [ ] Design preview, validation, and conflict reporting.
- [ ] Design complete icon backup and restoration.
- [ ] Define how icon mappings behave across Windows versions.

## Icon integrity rules

- [ ] Never rewrite application executable resources as the default icon-replacement method.
- [ ] Never patch Explorer or other protected system executables to change an icon.
- [ ] Prefer shortcuts, documented association/configuration mechanisms, per-folder metadata, or supported shell interfaces.
- [ ] Clearly report when an icon can only be changed in some locations or on some Windows versions.

---

# 5. WLDE — WinLook Desktop Engine (pronounced “WALDE”)

**WLDE is one integrated WinLook subsystem.** Wallpaper playback, desktop widgets, wallpaper library, and the Wallpaper Store all belong inside the Desktop Engine. Do not split the wallpaper functionality into a separate standalone product.

WLDE manages the desktop experience: desktop composition, wallpaper rendering/playback, visible desktop regions, widgets, Rainmeter-compatible content, performance adaptation, game-aware behaviour, and wallpaper discovery.

## Desktop composition and rendering

- [ ] Design Local desktop Groups as virtual visual objects, never filesystem folders.
- [ ] Keep original desktop shortcuts/items in place; visually substitute a custom Group icon and open a borderless popup containing the original items.
- [ ] Keep Grouping disabled by default.
- [ ] Keep Local Groups machine-specific and excluded from WLT exports; define Fancy Groups separately before supporting portable groups.

- [x] Design the Desktop Engine as a core subsystem.
- [ ] Define the desktop composition architecture and rendering backend per Windows generation.
- [ ] Define how WLDE integrates with the desktop without modifying `explorer.exe`.
- [ ] Design visible-region / occlusion detection.
- [ ] Design partial-region rendering so WLDE draws only wallpaper regions that are actually exposed where the platform and renderer permit it.
- [ ] Design clipping and dirty-region updates for partially covered desktop areas.
- [ ] Design transparency-aware composition for surfaces such as translucent taskbars.
- [ ] Define correct z-order and interaction with desktop icons, windows, widgets, and shell surfaces.
- [ ] Define multi-monitor desktop surfaces, per-monitor DPI, scaling, resolution changes, and orientation.
- [ ] Design fallback behaviour when an OS does not expose enough information for exact occlusion detection.
- [ ] Separate video decoding cost from GPU drawing/compositing cost; skipping an obscured draw region does not necessarily mean a video decoder can skip decoding the corresponding part of a frame.
- [ ] Define GPU device-loss recovery and renderer restart behaviour.
- [ ] Define low-memory and software-rendering fallbacks.

## Wallpaper formats and rendering capabilities

- [ ] Specify .wl.wp as the normal wallpaper asset/package format with source-quality preservation.
- [ ] Specify .wl.i.wp as an interactive wallpaper archive that may contain the wallpaper media, scripts, assets, manifest, preview and declared permissions.

- [ ] Define a renderer abstraction so formats can be added without redesigning WLDE.
- [ ] Support static image wallpapers (PNG, JPEG, WebP, BMP, and other formats selected after codec review).
- [ ] Design animated image playback (including GIF where appropriate).
- [ ] Design video wallpaper playback (container/codec support to be selected and documented).
- [ ] Design web wallpapers (HTML/CSS/JavaScript) with a restricted runtime and explicit permissions.
- [ ] Design layered 2D animated wallpapers.
- [ ] Research 3D wallpaper rendering and supported graphics APIs.
- [ ] Research shader/procedural wallpaper support.
- [ ] Design interactive wallpapers (mouse interaction and other narrowly scoped input).
- [ ] Design audio-reactive wallpapers with explicit audio-capture permission and clear privacy controls.
- [ ] Define wallpaper metadata, previews, thumbnails, tags, dimensions, frame rates, and resource hints.
- [ ] Define package validation and safe failure behaviour for unsupported or damaged wallpapers.

## Wallpaper library and integrated Wallpaper Store

- [ ] Design the local wallpaper library inside WLDE.
- [ ] Support importing local wallpapers without an internet connection.
- [ ] Design favorites, collections, playlists, history, and per-monitor assignments.
- [ ] Design wallpaper installation, update, removal, and backup.
- [ ] Design the **Wallpaper Store as an integrated WLDE feature**, not a separate executable/product.
- [ ] Design Store search, categories, tags, featured content, and discovery.
- [ ] Design wallpaper detail pages, previews, screenshots, metadata, and compatibility information.
- [ ] Design creator profiles and creator publishing/submission flow.
- [ ] Design free and paid content support, subject to later business and payment decisions.
- [ ] Design ratings, reviews, favorites, and reporting.
- [ ] Design download management, integrity checks, resumable downloads, and updates.
- [ ] Design moderation, content review, copyright/licensing declarations, and takedown workflow.
- [ ] Design package signing, publisher identity, and verification.
- [ ] Design Store backend/API requirements and determine hosting/cost implications.
- [ ] Keep locally installed wallpapers usable when the Store or network is unavailable.
- [ ] Define Store privacy rules and telemetry opt-in/opt-out.

## Resource-aware rendering

WLDE should be a good desktop citizen. Foreground applications and games take priority. WLDE should reduce its own CPU, GPU, memory, VRAM, decoding, and rendering work when the system is busy.

- [ ] Design a resource monitor for CPU, GPU, RAM, VRAM, power state, and WLDE's own consumption.
- [ ] Distinguish total system use from use attributable to WLDE.
- [ ] Design adaptive wallpaper resolution/render scale.
- [ ] Design adaptive FPS and frame pacing.
- [ ] Design configurable effect/detail reduction.
- [ ] Design pause, freeze-frame, and static-wallpaper fallback.
- [ ] Design reduced work for fully or mostly occluded wallpaper regions.
- [ ] Design resource recovery and gradual quality restoration.
- [ ] Define sampling windows, sustained-pressure rules, and hysteresis to prevent rapid pause/resume oscillation.
- [ ] Define behaviour when other system resource usage exceeds the user's requested 90% threshold: WLDE should stop animated playback/rendering and minimize its resource use, while preserving a lightweight static fallback only where safe and useful.
- [ ] Define separate policies for CPU, GPU, RAM, and VRAM pressure rather than relying only on one blended percentage.
- [ ] Design battery-aware and power-plan-aware behaviour.
- [ ] Design user-configurable quality caps, FPS limits, and resource budgets.
- [ ] Ensure adaptive controls cannot make the desktop unresponsive.

## Full-screen applications and Game Mode

- [ ] Design detection of foreground applications and fullscreen/borderless fullscreen states.
- [ ] Design game detection using multiple signals, not only executable-name matching.
- [ ] Design automatic Game Mode.
- [ ] Design **Optimize** behaviour: lower FPS/resolution/effects and minimize wallpaper work to leave resources for the game.
- [ ] Design **Suspend** behaviour: stop animation, video decoding where possible, and unnecessary rendering/workers.
- [ ] Support per-application profiles and user choice between Optimize, Suspend, and Normal.
- [ ] Include example profile scenarios for MW4, Roblox, and Minecraft without hardcoding those names as the only supported games.
- [ ] Define default policy for demanding games: prioritize the game, optimize WLDE first, and suspend entirely when the profile or resource pressure calls for it.
- [ ] Design reliable resume after the game exits or loses foreground status.
- [ ] Avoid disruptive wallpaper restarts or sudden quality changes during gameplay.
- [ ] Test games using exclusive fullscreen, borderless fullscreen, overlays, multiple monitors, and anti-cheat environments.

## Widgets and desktop components

- [x] Design widget rendering and isolation.
- [ ] Define WinLook's native widget API and package format.
- [ ] Design floating widgets and desktop-anchored widgets.
- [ ] Define input handling, focus, click-through, scaling, and positioning.
- [ ] Define widget update cadence and per-widget resource budgets.
- [ ] Define crash isolation so one widget cannot terminate WLDE.
- [ ] Design widget permissions and data access.
- [ ] Design widget layout save/restore and per-monitor layouts.

---

# 6. Third-party ecosystem and extensions

## Rainmeter compatibility

- [x] Identify `.rmskin` as a target compatibility format.
- [x] Establish that package extraction/parsing alone does not equal full Rainmeter runtime compatibility.
- [ ] Research the `.rmskin` package structure and manifest variations.
- [ ] Design safe package extraction and path traversal protection.
- [ ] Parse Rainmeter skin INI files and referenced assets.
- [ ] Research and implement compatibility for measures, meters, variables, bangs/actions, plugins, and other required runtime behaviour.
- [ ] Define a compatibility report for each imported skin.
- [ ] Design graceful handling of unsupported Rainmeter features and plugins.
- [ ] Decide whether support is native emulation, conversion, optional integration with an installed Rainmeter runtime, or a combination.
- [ ] Validate licensing and redistribution constraints; do not bundle third-party code/assets without permission.
- [ ] Test representative real-world skins and document the compatibility level.

## Other extension formats

- [ ] Research legacy Windows Gadget (`.gadget`) format and security implications.
- [ ] Decide which formats are imported, converted, rendered natively, or simply linked to external runtimes.
- [x] Design the WinLook extension package concept.
- [x] Design extension permission boundaries.
- [ ] Define package manifest, signing, versioning, dependencies, and update rules.
- [ ] Define extension APIs and compatibility guarantees.
- [ ] Design extension sandboxing and capability-based permissions.
- [ ] Design Store integration for themes, icon packs, widgets, and extensions.
- [ ] Design local extension installation, disabling, updating, and removal.
- [ ] Define moderation and publisher trust for Store-distributed extensions.

---

# 7. Standby Engine and session security

Standby is a custom idle/standby experience, **not a replacement Windows authentication system**. WinLook must leave secure sign-in and credential handling to Windows.

- [x] Define the Standby state machine.
- [x] Design native session event handling.
- [x] Define secure authentication handoff at the architecture level.
- [x] Design process isolation.
- [x] Define crash recovery.
- [ ] Specify exact states and transitions (active desktop, idle standby, user interaction, Windows lock, unlock/resume, sleep, wake, display off).
- [ ] Design the custom Standby experience while the user session is still unlocked.
- [ ] On Windows lock, hide/stop the WinLook Standby surface and allow native Windows secure sign-in to own the screen.
- [ ] On unlock, safely resume the user's desktop/Standby state without handling credentials.
- [ ] Use supported session notifications and verify behaviour on each target OS.
- [ ] Research the `NoLockScreen` policy only as an optional OS policy/compatibility topic; never treat it as a way to put an ordinary WinLook window on the secure desktop.
- [ ] Do not rely on a normal topmost kiosk window being underneath or appearing above `LogonUI`.
- [ ] Do not install a custom Credential Provider as part of the initial design.
- [ ] Define safe keyboard/mouse wake behaviour without recording typed content or global keystrokes.
- [ ] Test lock/unlock, fast user switching, remote sessions, sleep/wake, multiple displays, and crashes.
- [ ] Define what happens if WinLook is unavailable while Windows is locked or unlocking.

---

# 8. Compatibility strategy — Windows 95 and later

Detailed platform compatibility is maintained under [compatibility/](compatibility/README.md). Read the per-OS page before choosing APIs, compilers, imports, event mechanisms, rendering techniques or process integration for that OS. See the [compatibility matrix](compatibility/matrix.md) and [technical source ledger](references/SOURCES.md).


Feature availability varies significantly across the Windows 9x and NT families. Each OS adapter must expose capabilities, not claim universal feature parity.

## Target matrix to research

- [ ] Windows 95.
- [ ] Windows 98 / 98 SE.
- [ ] Windows ME.
- [ ] Windows NT 4.0 (investigate separately; not a 9x system).
- [ ] Windows 2000.
- [ ] Windows XP.
- [ ] Windows Vista.
- [ ] Windows 7.
- [ ] Windows 8.
- [ ] Windows 8.1.
- [ ] Windows 10.
- [ ] Windows 11.

## Compatibility deliverables

- [ ] Create a compatibility test matrix by OS version, edition, architecture, and feature.
- [ ] Define 9x versus NT-family implementation boundaries.
- [ ] Verify 16-bit/32-bit/64-bit and CPU/runtime constraints relevant to each target.
- [ ] Define supported minimum hardware for each release family.
- [ ] Define feature capability detection and clear unsupported-feature messaging.
- [ ] Define per-version taskbar, Start Menu, icon, theme, wallpaper, and widget adapters.
- [ ] Define graphics API fallbacks (including GDI for legacy support).
- [ ] Define installation, uninstallation, and recovery for each supported generation.
- [ ] Decide which historical Windows versions are officially supported, experimental, or research-only.
- [ ] Run tests on real hardware or reliable virtual machines where practical.
- [ ] Document all known limitations and unsupported combinations.

---

# 9. Snapshots, rollback, and Windows restoration

The clean uninstaller must reverse all WinLook-owned changes where safely possible, including Local Group icon presentation, WLWM integrations, WLDE state, WLOSDE surfaces, WLDM display settings, Appearance/Icon Engine changes and installed assets. See [architecture/restoration.md](architecture/restoration.md).


Restoration is a core requirement. Users must be able to undo WinLook changes and return to their prior appearance without reinstalling Windows.

- [x] Define Windows default restoration as a product capability.
- [ ] Design a pre-change snapshot model for all settings WinLook modifies.
- [ ] Snapshot current values before applying a theme or changing system appearance.
- [ ] Track changes by component so users can restore only icons, taskbar settings, cursors, sounds, etc.
- [ ] Design full-theme rollback.
- [ ] Design interrupted-application recovery after a crash or power loss.
- [ ] Design atomic/staged application where possible.
- [ ] Verify that restoration does not overwrite later user changes made outside WinLook.
- [ ] Design snapshot retention, storage limits, and cleanup.
- [ ] Design safe backups for icon mappings, shortcuts, associations, theme settings, and wallpaper profiles.
- [ ] Design a recovery mode that works when the normal Manager UI fails.
- [ ] Design a safe way to disable experimental integrations.
- [ ] Test rollback after partial failure, upgrade, uninstall, and Windows update.
- [ ] Define which changes cannot be automatically restored and warn before applying them.

---

# 10. Security, privacy, and trust

Process injection is an accepted WLWM technique for compatible targets. The main BGW coordinates compatibility classification, cache lookup and injection. Protected/authentication processes, DWM and security-critical targets remain excluded. Explorer.exe must never be modified on disk or replaced; Explorer-targeted injection is excluded by default and requires a separately reviewed compatibility/recovery design. See [architecture/wlwm.md](architecture/wlwm.md).


- [ ] Threat-model the Manager, Agent, Service, WLDE, Store, and extension runtime.
- [ ] Define least-privilege requirements and elevation prompts.
- [ ] Define package integrity verification and signing.
- [ ] Design secure archive extraction and protection against path traversal.
- [ ] Design sandboxing for web wallpapers, scripts, widgets, and third-party extensions.
- [ ] Define permissions for filesystem, network, audio, input, and system information.
- [ ] Ensure wallpaper/extension code cannot read credentials or protected process memory.
- [ ] Define safe update verification and rollback.
- [ ] Define Store moderation, publisher verification, abuse reporting, and security response.
- [ ] Define privacy disclosures and telemetry controls.
- [ ] Define logging that avoids collecting passwords, typed content, or unnecessary personal data.
- [ ] Define vulnerability reporting and security update policy.

---

# 11. Manager UI and user experience

WinLook Manager is where users configure and load subsystem settings, import/export Templates and manage installed content. A taskbar icon/background process may keep WinLook available; there is no separate WLCC Control Center. A custom Launcher is not in scope for now. Creation tools belong inside WinLook itself rather than a separate Creator Studio.


- [ ] Design the WinLook Manager information architecture.
- [ ] Design the dashboard and active-theme status.
- [ ] Design theme browsing, preview, apply, and rollback flows.
- [ ] Design icon management and per-category previews.
- [ ] Design taskbar and Start Menu customization pages.
- [ ] Design WLDE wallpaper library and integrated Wallpaper Store screens.
- [ ] Design monitor layout and per-monitor wallpaper controls.
- [ ] Design resource/performance controls and live WLDE status.
- [ ] Design per-game profiles and Game Mode settings.
- [ ] Design widget and extension management.
- [ ] Design Standby settings and preview.
- [ ] Design compatibility warnings and unsupported-feature explanations.
- [ ] Design recovery and restore-defaults UI.
- [ ] Design accessible controls, keyboard navigation, and localization.
- [ ] Keep UI and rendering responsive on low-end hardware.

---

# 12. Installer, updater, and release

- [x] Define installer and uninstaller as required product components.
- [x] Define a release and update strategy at the planning level.
- [ ] Design installation modes for per-user and machine-wide deployment.
- [ ] Design optional Service installation and removal.
- [ ] Design upgrade and downgrade behaviour for all binaries and packages.
- [ ] Ensure uninstallation stops processes and removes only WinLook-owned files/settings.
- [ ] Ensure uninstallation restores recorded Windows appearance settings where requested.
- [ ] Design update signature verification and staged installation.
- [ ] Design rollback after failed updates.
- [ ] Define separate modern and legacy release artifacts.
- [ ] Define versioning and compatibility policy for themes, widgets, wallpapers, and extensions.
- [ ] Define release channels and support policy.
- [ ] Define portable/zip distribution feasibility.
- [ ] Define offline installation/update behaviour where practical.

---

# 13. Verification and quality

- [ ] Create unit tests for configuration, manifests, mapping, and state machines.
- [ ] Create tests for theme apply/restore transactions.
- [ ] Create tests for icon mapping and restoration.
- [ ] Create tests for WLDE visibility, clipping, and compositing calculations.
- [ ] Create tests for resource thresholds, hysteresis, pause, and resume.
- [ ] Create tests for Game Mode detection and per-app profiles.
- [ ] Create tests for package validation and sandbox boundaries.
- [ ] Create compatibility tests for every officially supported Windows generation.
- [ ] Test multi-monitor layouts, DPI, mixed refresh rates, and display hotplug.
- [ ] Test desktop responsiveness and resource usage on low-end hardware.
- [ ] Test full-screen games and common foreground applications.
- [ ] Test Windows lock/unlock, sleep/wake, crash recovery, and restoration.
- [ ] Test install, update, downgrade, and uninstall.
- [ ] Define measurable startup, idle CPU/GPU, memory, VRAM, and frame-time budgets.
- [ ] Define release acceptance criteria and regression process.

---

# 14. Documentation and planning workflow

Substantial technical claims must include direct links to official API/OS documentation or upstream third-party projects. Keep the claim, scope, confidence and verification date in [references/SOURCES.md](references/SOURCES.md). Maintain [index.md](index.md) and [workspace.md](workspace.md) for agent navigation and durable project context.


- [ ] Keep this `Plan.md` updated as decisions and research change, after user approval for plan edits.
- [ ] Add `Final-Plan.md` only when the core architecture and major decisions are approved.
- [x] Maintain [architecture/overview.md](architecture/overview.md) for the approved high-level architecture.
- [x] Maintain [architecture/appearance-engine.md](architecture/appearance-engine.md) for themes, shell and icon customization.
- [x] Maintain [architecture/wlde.md](architecture/wlde.md) for desktop composition, wallpapers, Store and resource management.
- [x] Maintain [architecture/standby-security.md](architecture/standby-security.md) for session lifecycle and authentication boundaries.
- [x] Maintain [architecture/extensions.md](architecture/extensions.md) for widgets, Rainmeter compatibility, package formats and permissions.
- [x] Maintain [architecture/restoration.md](architecture/restoration.md) for snapshots, rollback and uninstall restoration.
- [x] Maintain [compatibility/README.md](compatibility/README.md), [compatibility/matrix.md](compatibility/matrix.md) and one detailed page per Windows target.
- [x] Maintain version-specific API, toolchain, rendering and compatibility evidence in [compatibility/](compatibility/README.md) and [references/SOURCES.md](references/SOURCES.md).
- [x] Record the source and confidence level for version-specific technical claims in [references/SOURCES.md](references/SOURCES.md).
- [ ] Separate confirmed facts, design decisions, hypotheses, and unresolved questions.
- [ ] Review this plan after each major research phase and revise scope where evidence requires it.

---

# 15. Original planning checklist — status carried forward

The following preserves the checklist and completion statuses supplied during the planning conversation. The checkmarks mean the initial design discussion has covered these items; they are not implementation/test claims.

## Core architecture

- [x] Define Rust/C hybrid architecture
- [x] Define Windows version detection
- [x] Design capability detection system
- [x] Design WinLook Agent and optional Service
- [ ] Define shared configuration format

## Appearance Engine

- [x] Design universal theme format
- [x] Design Windows-specific appearance adapters
- [x] Design system icon customization design
- [x] Design application icon mapping
- [x] Design taskbar appearance integrations
- [x] Design Start Menu customization
- [x] Design window and title-bar customization
- [x] Design fonts, cursors and sound themes

## Desktop and extensions

- [x] Design Desktop Engine
- [x] Research `.rmskin` compatibility at the package/format level
- [x] Design widget rendering and isolation
- [x] Design extension package format
- [x] Design extension permissions

## Standby and security

- [x] Define Standby state machine
- [x] Design native session event handling
- [x] Define secure authentication handoff
- [x] Design process isolation
- [x] Define crash recovery

## Reliability and release

- [ ] Design theme snapshots and rollback
- [x] Design Windows default restoration
- [ ] Create compatibility test matrix
- [ ] Define experimental integration policy
- [x] Define installer and uninstaller
- [x] Define release and update strategy

---

# 16. Decisions still requiring explicit approval

These are not silently treated as finalized:

- [ ] Official release support policy: Windows 95 through 11 all at launch, or phased modern-first releases with legacy support developed/tested separately.
- [x] WinCheck is permanently out of scope.
- [ ] Exact modern UI toolkit and rendering backend.
- [ ] Exact shared configuration format and migration strategy.
- [ ] Exact list of supported wallpaper formats/codecs.
- [ ] Exact Store backend, hosting, creator, payment, and moderation model.
- [ ] Exact meaning and measurement of “other system resource usage above 90%” (CPU, GPU, memory, VRAM, or any constrained resource); the intended behaviour is to stop WLDE animation/rendering under sustained severe pressure.
- [ ] Default game detection policy and whether each game defaults to Optimize or Suspend.
- [ ] Which deep customization integrations are acceptable when a supported Windows API does not expose the desired visual change.
- [ ] Official Rainmeter compatibility target and whether an external Rainmeter installation may be used as an optional backend.
- [ ] Exact legacy OS/toolchain support after prototype testing.

---

## Planning rule

Do not begin high-risk shell modifications, secure-desktop work, or broad system-wide customization before the relevant compatibility research, threat model, backup/rollback design, and test strategy are documented. Prefer a small proof of concept behind an isolated adapter before committing to a platform-wide implementation.


---

# Current approved architecture clarifications (2026-11-08)

This section records the latest user-approved decisions and supersedes any earlier wording that conflicts with them.

## Subsystem identity

- **WLDE (WALDE):** WinLook Desktop Engine. Wallpaper runtime, wallpaper library and community Store belong inside it.
- **WLWM (WELMO):** WinLook Window Manager. Adds Hyprland-inspired layouts, window rules, workspaces and visual behaviour over the existing Windows windowing stack and DWM.
- **WLDM:** WinLook Display Manager. Monitor discovery, display profiles and settings.
- **WLOSDE:** WinLook On-Screen Display Engine. Notifications, temporary popups and highly customizable OSD.
- **wl_animate:** public/reusable Rust crate and modern animation API for WinLook modules and third-party developers.
- **wl_Lanimate:** corresponding C animation library for legacy builds.
- **WLT:** WinLook Template. Portable profile and bundled asset format.
- **Main BGW:** coordinates WLWM detection, compatibility classification/cache, injection and health/recovery.

## DWM, Windows and injection

DWM and the Windows windowing system remain intact. WLWM augments Windows rather than replacing DWM. Process injection and hooks are acceptable techniques for compatible targets and are coordinated by the main BGW. A compatibility layer classifies targets before injection and maintains a local binary cache; the cache must be invalidated when target/environment/module identity changes and cannot replace per-launch safety checks. Explorer.exe must not be modified on disk or replaced; Explorer-targeted injection is excluded by default until a distinct, reversible design is reviewed. DWM and authentication/security-critical processes are excluded.

## Animation suite

The animation suite is reusable across WLWM, WLDE, WLOSDE, Manager UI and third-party integrations. Rust and C implementations provide equivalent concepts where the platform allows, with compiler- and OS-specific optimization.

## Desktop Groups

WLDE Groups are virtual presentation objects, never physical folders. Original desktop Shell items remain in place and retain their targets. WLDE displays a custom Group icon and opens a borderless popup showing original items. Grouping is disabled by default. Local Groups are machine-specific and are not exported in WLT. Fancy Groups are a separate portable concept and must not be conflated with Local Groups.

## WLT and wallpaper packages

One .wl.t file contains one Template. A ZIP containing several .wl.t files is a Template Group and imports them independently. WLT may contain rmskin/, wl/taskbar/, wl/&FONT_FILE&/, wl/wallpaper/, wl/icons/, wl/cursors/, wl/sounds/, wl/desktop/, wl/wlwm/ and wl/display/ content. The special &FONT_FILE& directory is part of the proposed layout. .wl.wp is the normal wallpaper asset/package; .wl.i.wp is an interactive wallpaper archive that can include media and supporting scripts/assets. Preserve source quality and validate archive paths, permissions and licensing. Local desktop Groups are excluded from WLT.

## Excluded product ideas

WinCheck, WLCC, a separate Launcher, a separate Scene Engine, a separate Creator Studio and a separate general Input Customization Engine are out of scope. Creation tools belong in WinLook Manager. Community Store/catalogues should support independent sources rather than requiring one central WinLook-operated service.


## Initial implementation slice — portable animation primitives (alpha.1)

The first code slice establishes the renderer-independent timing/value layer only. It is deliberately **not** the complete OS animation suite: no Win32 window hooks, process injection, BGW integration, DWM/D3D/DirectComposition, GDI drawing, overlay presentation, timer loop, or Windows backend is included yet.

### Rust: `wl_animate`

- Crate: [`crates/wl_animate`](`../../crates/wl_animate/README.md`)
- Public concepts: `Interpolate`, `Easing`, `Tween<T>`, `PlaybackState`, `Keyframe<T>`, `KeyframeTrack<T>`, `Point2` and `Rgba`.
- Host-driven timing: consumers provide elapsed `std::time::Duration`; the crate creates no scheduler, worker thread, timer, window, graphics context or hook.
- Sampling does not allocate internally for the built-in fixed-size value types. A third-party `Interpolate` implementation may allocate inside its own `Clone` implementation; the crate cannot guarantee otherwise.
- Initial version: `0.1.0-alpha.1`. Public API is provisional.

### Legacy C: `wl_Lanimate`

- Library sources: [`legacy/wl_Lanimate`](`../../legacy/wl_Lanimate/README.md`)
- C89-compatible header and implementation; caller-owned tween/keyframe storage; no heap allocation in the per-frame path.
- Includes scalar tween state, linear/quadratic/cubic easing, Point2/RGBA interpolation and timestamped keyframe sampling.
- The library core has no internal worker thread, timer, message pump, DWM, DirectComposition, D3D or modern Win32 imports. OS-specific scheduling and drawing belong in separate adapters.

### Verification evidence

- **C host compile/test: passed** with GCC 14.2.0 using `make clean && make test` and `-std=c89 -Wall -Wextra -Werror -pedantic -O2`. This validates the portable C source against a C89 compiler mode; it does not prove Open Watcom, Windows 95, or any other Windows runtime compatibility.
- **Rust compile/test: not run.** This execution environment does not have `rustc` or `cargo` installed. Install/use a Rust toolchain and run `cargo fmt --check`, `cargo test` and `cargo clippy --all-targets --all-features` before treating the crate as build-verified.
- **Windows runtime/backend tests: not run.** No Win32/DWM/GDI backend or injection integration exists in this slice.
- No compiled test executable or other build output is committed.

### Next implementation gates

1. Build and test `wl_animate` with the selected minimum Rust toolchain.
2. Verify `wl_Lanimate` with the selected Open Watcom compiler/linker and actual legacy target imports.
3. Define the stable shared conceptual API and the C ABI before promising third-party compatibility.
4. Add platform schedulers/renderers behind capability-detected adapters, retaining native fallbacks.
5. Add frame-time, allocation, CPU/GPU and GDI-resource benchmarks before optimizing compiler-specific paths.

## Detailed references

- [Compatibility methodology](compatibility/README.md)
- [Windows compatibility matrix](compatibility/matrix.md)
- [Architecture overview](architecture/overview.md)
- [Appearance and Icon Engine](architecture/appearance-engine.md)
- [WLWM](architecture/wlwm.md)
- [Animation suite](architecture/animation-suite.md)
- [WLDE](architecture/wlde.md)
- [WLT](architecture/wlt.md)
- [WLOSDE and WLDM](architecture/wlosde-wldm.md)
- [Standby and session security](architecture/standby-security.md)
- [Extensions and community ecosystem](architecture/extensions.md)
- [Restoration and clean uninstallation](architecture/restoration.md)
- [Technical source ledger](references/SOURCES.md)
- [Workspace index](index.md)

# WinLook Detailed Planning Roadmap

> Status: proposed sequencing and acceptance gates. This is not a release schedule, implementation claim or final specification. No dates are committed.

## 1. Status vocabulary

- **Decision required:** materially different designs remain open.
- **Research required:** platform/API/runtime evidence is incomplete.
- **Design ready:** contracts are sufficiently specified for a bounded prototype.
- **Prototype verified:** limited prototype passed its stated tests; not a support claim.
- **OS verified:** feature passed the documented record on the exact OS/build/runtime.
- **Release eligible:** dependencies, security, restoration, accessibility and support gates passed.

## 2. Critical path

1. Product scope, OS support tiers and non-goals.
2. Repository layout, source ownership and public API strategy.
3. Per-OS toolchain/runtime baselines and capability contract.
4. Process/session model, IPC and health/restart rules.
5. Change journal, transaction and restoration design.
6. Platform adapters and read-only capability discovery.
7. Reversible appearance and desktop foundations.
8. WLDE static wallpaper, widgets and resource policy.
9. WLDM and WLOSDE display/notification surfaces.
10. Animation presentation and lifecycle.
11. WLWM passive observation, rules and non-injected layouts.
12. Optional target-specific in-process integration after security review.
13. WLT package/import/export and independent catalogue support.
14. Legacy vertical slices, installer/update and release hardening.

Legacy toolchain research proceeds in parallel but remains a separate build/test stream.

## 3. Phase 0 — Product, support and developer experience contract

### Deliverables
- Product mission, non-goals and support tiers.
- Feature-capability matrix for Windows 95 through 11.
- Hardware/performance tiers and accessibility baseline.
- Approved repository layout for C, rust, lib, product metadata and documentation.
- Engine naming convention and public API ownership.
- WinToys-inspired Manager navigation/page model.
- Application guide, public API reference and per-engine documentation standard.

### Exit gate
- [ ] Each Windows release has an intended tier or is marked research-only.
- [ ] No feature is described as universal without evidence.
- [ ] C/Rust source trees and library ownership are approved.
- [ ] Public crate/header responsibilities are clear.
- [ ] UI toolkit choices are not prematurely assumed for legacy targets.
- [ ] Documentation structure and API stability labels are approved.

## 4. Phase 1 — Platform and toolchain baselines

### Deliverables
- Modern Rust toolchain, minimum supported version and target triples.
- Legacy C compiler/linker/runtime/SDK matrix split by OS family.
- Imported DLL/export audit for each binary.
- Process architecture/bitness and cross-bitness policy.
- Build reproducibility and artifact provenance plan.
- Per-OS VM and representative physical hardware test plan.

### Exit gate
- [ ] Each intended build has a reproducible toolchain record.
- [ ] No unsupported import prevents startup.
- [ ] Exact OS/version/architecture combinations are named.
- [ ] Clean-machine install/uninstall path is documented.

## 5. Phase 2 — Runtime, IPC and restoration foundation

### Deliverables
- Process ownership and runtime contracts.
- Versioned local IPC envelope and transport decision per OS family.
- Capability and operation result catalogue.
- Component health, restart, degraded and quarantine rules.
- Change journal, before/after snapshots and external-change handling.
- Service necessity assessment; no privileged Service by default.
- Diagnostic/logging privacy rules.

### Exit gate
- [ ] Manager/Agent/BGW can restart independently in a harness.
- [ ] Protocol mismatch and malformed IPC fail closed.
- [ ] Interrupted mock transaction resumes or rolls back deterministically.
- [ ] Component crash does not make the Windows desktop unusable.
- [ ] No generic elevated command interface exists.

## 6. Phase 3 — Capability discovery and native adapters

### Deliverables
- Exact OS/build/service-pack detection.
- API/export/interface and runtime probing.
- Architecture, bitness, mitigation and session reporting.
- Display, Shell, USER/GDI and DWM capability inventory.
- Per-adapter compatibility report with reason codes.
- Read-only diagnostics mode in Manager.

### Exit gate
- [ ] Probe results match documented platform facts on each test OS.
- [ ] Missing optional APIs do not prevent startup.
- [ ] Unknown capability states do not enable high-risk operations.
- [ ] UI explains partial, unknown and unsupported outcomes.

## 7. Phase 4 — Appearance and restoration vertical slice

Choose a small reversible setting, then prove preview, snapshot, apply, verification, restore and uninstall. The exact first feature remains a decision.

### Exit gate
- [ ] Apply, verify, revert and uninstall pass on every claimed OS.
- [ ] Crash at every transaction boundary has deterministic recovery.
- [ ] External user changes are not overwritten blindly.
- [ ] Unsupported components are skipped with a reason.

## 8. Phase 5 — WLDE desktop and wallpaper foundation

### Deliverables
- Per-monitor desktop surface model.
- Static-image pipeline preserving source quality.
- Display-change and sleep/resume handling.
- Visibility/occlusion policy with conservative fallback.
- Render scheduler and frame budget.
- Resource governor across CPU/GPU/RAM/VRAM/power.
- Game Mode Normal/Optimize/Suspend.
- Crash/disable fallback to native/static desktop.

### Exit gate
- [ ] Static wallpaper works across selected monitor layouts.
- [ ] Renderer stop/restart does not corrupt/hide the native desktop.
- [ ] Resource pressure degrades quality before severe contention.
- [ ] Suspend stops expensive work where feasible.
- [ ] Reduced-motion and accessibility preferences are respected.

## 9. Phase 6 — WLDM, WLOSDE and animation presentation

### Deliverables
- Display topology and monitor identity strategy.
- Display-profile transaction and recovery design.
- Notification queue, priorities, quiet hours and typed actions.
- Multi-monitor OSD placement and collision handling.
- Reduced-motion, high-contrast and keyboard-accessible surfaces.
- Stable conceptual API between wl_animate and wl_Lanimate.

### Exit gate
- [ ] Display disconnect/reconnect does not strand popups or unsafe display settings.
- [ ] OSD never imitates native authentication/UAC/secure-desktop UI.
- [ ] Cancellation, interruption and target destruction are deterministic.
- [ ] Resource policy can pause nonessential transitions.

## 10. Phase 7 — WLWM without injection

Implement lower-risk window observation and management through documented facilities where available. Injection is not the first WLWM milestone.

### Deliverables
- Window identity and observation model.
- Rule matching and deterministic precedence.
- Floating/tiling/layout solver.
- Monitor/work-area/DPI-aware geometry.
- User override and manual-layout preservation.
- Logical workspaces where native public APIs are incomplete.
- Per-OS fallback and safe disable.

### Exit gate
- [ ] Windows retains native focus, z-order and application activation.
- [ ] Owned dialogs, child windows and shell windows have explicit policy.
- [ ] User move/resize/focus can override conflicting rules.
- [ ] Stopping WLWM returns to usable native behaviour.

## 11. Phase 8 — Optional WLWM in-process integration pilot

Only after target-policy and threat-model review, test an opt-in integration against a small non-critical allowlist.

Preconditions: BGW classification/cache reviewed; per-launch checks independent from cache; protected/system/security/Explorer exclusions; mitigation respect; launch unaffected by failure; unload/recovery model; user disable/quarantine action.

### Exit gate
- [ ] Each target has exact binary identity and test record.
- [ ] Unknown/incompatible targets are denied by default.
- [ ] Failed integration leaves the app running natively.
- [ ] Module failure/unload does not corrupt the target or desktop.
- [ ] Security review approves the specific method and targets.

This phase is optional. WLWM must retain meaningful non-injected functionality.

## 12. Phase 9 — WLT, extension and catalogue ecosystem

### Deliverables
- Versioned .wl.t manifest/archive rules.
- Asset hashes, publisher/license/attribution and permissions.
- Preview/dry-run and conflict report.
- Staged extraction and atomic apply with rollback.
- Template Group independent-member import.
- Local, self-hosted and GitHub catalogue sources.
- Restricted runtime for widgets/interactive wallpapers.
- Rainmeter compatibility levels and licensing review.

### Exit gate
- [ ] Malformed archives cannot escape staging or overwrite arbitrary files.
- [ ] One invalid Template Group member cannot corrupt another.
- [ ] Native code is not executed during ordinary import.
- [ ] Every applied setting is journaled/recoverable.
- [ ] Catalogue trust and integrity are visible.

## 13. Phase 10 — Legacy vertical slices

Win9x and older NT are separate compatibility targets. Each has its own compiler/runtime, API adapter, installer and test record.

### Deliverables per OS
- Toolchain/import audit.
- Startup and clean-uninstall smoke test.
- Capability probe record.
- One small reversible appearance/desktop slice.
- WLWM observation and fallback test.
- Animation backend test.
- Restoration/fault test.
- Known exclusions and performance profile.

### Exit gate
- [ ] Exact edition/build/SP, CPU, RAM, GPU/driver, compiler/runtime and test date recorded.
- [ ] No feature is promoted without reproducible evidence.
- [ ] Unavailable modern features are reported, not silently emulated.

## 14. Phase 11 — Installer, updater and release hardening

Deliver signed/provenanced package strategy where supported, install scope and consent, component version compatibility, interrupted-update rollback, startup recovery/safe mode, user-data backup policy and safe uninstaller.

### Exit gate
- [ ] Install, upgrade, repair, rollback and uninstall are tested.
- [ ] Failed update leaves recoverable previous version.
- [ ] Uninstaller removes WinLook-owned changes and preserves unrelated data.
- [ ] Release notes list exact OS and feature support limits.

## 15. Cross-phase Definition of Done

Every feature has a named owner and data contract; OS/API/runtime requirements; safe defaults and consent; partial/error states; crash/cancellation/restart behaviour; restoration; accessibility/reduced-motion behaviour; resource budget; tests; exact OS records; documentation; and a user-visible disable/recovery route.

## 16. Planning queue

1. Confirm root wllook directory purpose and approve source-tree layout.
2. Approve engine list and public Rust crate/C header responsibilities.
3. Select support tiers and hardware baselines.
4. Ratify process/IPC/privilege contracts.
5. Define change journal and conflict policy.
6. Finalize resource threshold sampling and hysteresis.
7. Approve UI navigation and modern/legacy toolkit research.
8. Specify WLWM rule precedence and injection exclusions.
9. Finalize WLT archive, manifest, trust and migration.
10. Select the first reversible vertical slice.

Do not begin broad implementation while these decisions materially affect safety, compatibility or public API shape.

## Accepted Audio & Media and File Explorer workstreams

These product workstreams enter the roadmap after the shared runtime, capability and restoration foundations. This adds dependencies; it does not commit release dates or claim support.

### Audio & Media
1. Verify Vista Core Audio base session interfaces and the Windows 7 `IAudioSessionManager2` enumeration boundary.
2. Verify GSMTC on Windows 10 version 1809+ and document provider-controlled media type, metadata, thumbnails, transport and timeline.
3. Define audio-session/media-session identity correlation and multi-session UI behaviour.
4. Define a legacy player-adapter contract; test named player/version pairs on systems without GSMTC.
5. Prototype endpoint/session controls and Now Playing UI without recording, virtual drivers or broad application injection.

### File Explorer
1. Map documented Shell extension APIs and registration requirements per Windows version.
2. Prototype Shell commands and item handlers without blocking Explorer.
3. Document the strict scope of namespace extensions and custom `IShellView` views.
4. Select one exact modern Windows build for an opt-in Explorer presentation-adapter proof of concept.
5. Implement out-of-band disable, crash quarantine, safe mode and recovery before enabling deeper styling.
6. Expand surface coverage only after build-specific tests for context menus, navigation, address/search, tabs, command bar and views.

Each workstream needs its own bounded prototype plan, compatibility evidence and risk review before application implementation.

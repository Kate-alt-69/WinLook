# WinLook Workspace Index

Read this index first when resuming work, then open only the relevant sections/files.

## Authoritative documents
- Repository landing page: [README.md](../README.md)
- Living project plan: [Plan.md](Plan.md)
- Detailed dependency-ordered roadmap: [roadmap.md](roadmap.md)
- Compatibility methodology: [compatibility/README.md](compatibility/README.md)
- Compatibility matrix: [compatibility/matrix.md](compatibility/matrix.md)
- Per-version pages: [win95](compatibility/win95.md), [win98](compatibility/win98.md), [winme](compatibility/winme.md), [nt4](compatibility/nt4.md), [win2000](compatibility/win2000.md), [winxp](compatibility/winxp.md), [vista](compatibility/vista.md), [win7](compatibility/win7.md), [win8](compatibility/win8.md), [win81](compatibility/win81.md), [win10](compatibility/win10.md), [win11](compatibility/win11.md)
- Architecture overview: [architecture/overview.md](architecture/overview.md)
- Two-executable BGW runtime: [architecture/background-runtime.md](architecture/background-runtime.md)
- User-defined repository layout: [architecture/repository-layout.md](architecture/repository-layout.md)
- Manager UI architecture: [architecture/ui-architecture.md](architecture/ui-architecture.md)
- Public Rust UI foundation and styling API: [../docs/wl_ui/README.md](../docs/wl_ui/README.md)
- Rust design tokens, selectors and CSS output: [../docs/wl_ui/styling.md](../docs/wl_ui/styling.md)
- UI renderer capability map: [../docs/wl_ui/backends.md](../docs/wl_ui/backends.md)
- Legacy C UI API: [../docs/wl_ui/C/README.md](../docs/wl_ui/C/README.md) and [../docs/wl_ui/C/styling.md](../docs/wl_ui/C/styling.md)
- Build system, shared output/cache layout and GitHub Actions: [architecture/build-and-ci.md](architecture/build-and-ci.md)
- Runtime/process/IPC contracts: [architecture/runtime-contracts.md](architecture/runtime-contracts.md)
- Documentation standards: [architecture/documentation-standard.md](architecture/documentation-standard.md)
- Appearance/Icon Engine: [architecture/appearance-engine.md](architecture/appearance-engine.md)
- File Explorer Customization: [architecture/explorer-customization.md](architecture/explorer-customization.md)
- Audio & Media Experience: [architecture/audio-experience.md](architecture/audio-experience.md)
- Clipboard Experience: [architecture/clipboard-experience.md](architecture/clipboard-experience.md)
- Profiles and Optimization: [architecture/profiles-and-optimization.md](architecture/profiles-and-optimization.md)
- WLWM: [architecture/wlwm.md](architecture/wlwm.md)
- Animation libraries: [architecture/animation-suite.md](architecture/animation-suite.md)
- Vector animation and SVG contract: [architecture/vector-animation.md](architecture/vector-animation.md)
- CPUv/GPUv resource-governor architecture: [architecture/resource-governors.md](architecture/resource-governors.md)
- WLDE: [architecture/wlde.md](architecture/wlde.md)
- WLT: [architecture/wlt.md](architecture/wlt.md)
- WLOSDE / WLDM: [architecture/wlosde-wldm.md](architecture/wlosde-wldm.md)
- Standby/session security: [architecture/standby-security.md](architecture/standby-security.md)
- Extensions/community ecosystem: [architecture/extensions.md](architecture/extensions.md)
- Restoration/uninstaller: [architecture/restoration.md](architecture/restoration.md)
- Compatibility test strategy: [compatibility/test-strategy.md](compatibility/test-strategy.md)
- Source ledger: [references/SOURCES.md](references/SOURCES.md)
- Durable workspace notes: [workspace.md](workspace.md)


## Current target source layout
- Complete Windows 10 application: `rust/win10/`.
- Complete Windows 11 application: `rust/win11/`.
- Complete Windows 95 application: `C/win95/`.
- Shared libraries: `rust/library/` and `C/library/`; optional Tauri host adapter: `rust/adapters/wl_ui_tauri2/`.
- Root planning, build runner and workflows: `plan/`, `build.py` and `.github/workflows/`.
- Shared libraries now live under `rust/library/` and `C/library/`; historical `crates/` and `legacy/` prototype copies are scheduled for removal as part of this migration.

## Shared library implementation
- Rust: [`wl_animate`](../rust/library/wl_animate/README.md), [`wl_cpuv`](../rust/library/wl_cpuv/README.md), [`wl_gpuv`](../rust/library/wl_gpuv/README.md), and public [`wl_ui`](../rust/library/wl_ui/README.md).
- C: [`wl_Lanimate`](../C/library/wl_Lanimate/README.md), [`wl_Lcpuv`](../C/library/wl_Lcpuv/README.md), [`wl_Lgpuv`](../C/library/wl_Lgpuv/README.md), and legacy Win32 [`wl_Lui`](../C/library/wl_Lui/README.md).
- UI backend paths: `wl_ui::windows10`, `wl_ui::windows11`, `wl_ui::tauri2`, and experimental `wl_ui::windows7` / `windows8` / `windows81`.
- Vector/SVG contract: [Vector Animation](architecture/vector-animation.md).
- Resource-governor contract: [CPUv/GPUv](architecture/resource-governors.md).
- UI structure and selection rules: [Manager UI Architecture](architecture/ui-architecture.md).

## Resume rules
1. Re-read the current user request and current repository files. This index is navigation, not authority over new instructions.
2. Distinguish accepted product decisions, proposals, verified platform facts and unresolved research.
3. Every API claim needs a primary source link. Third-party behaviour claims need upstream docs/source.
4. Never assume an API is available on Windows 95 because it appears in a modern Win32 header or example.
5. Read the target OS page before proposing APIs, compiler flags, imports or process techniques.
6. Update the source ledger when a decision depends on new research.
7. Do not create Final-Plan.md unless explicitly requested.

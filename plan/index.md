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
- Shared libraries: `rust/library/` and `C/library/`.
- Root planning and workflows: `plan/` and `.github/workflows/`.
- The historical `crates/` and `legacy/` prototype paths may remain until a dedicated migration is completed; do not treat them as the target layout.

## Initial code prototypes
- Modern Rust animation crate: [`crates/wl_animate`](../crates/wl_animate/README.md)
- Legacy C animation library: [`wl_Lanimate`](../legacy/wl_Lanimate/README.md)
- Vector/SVG API design: [Vector Animation](architecture/vector-animation.md)
- Shared CPU/GPU governors are planned as `wl_cpuv`/`wl_gpuv` and `wl_Lcpuv`/`wl_Lgpuv`; the current repository branch does not yet contain releasable implementations.
- Implementation status and verification limits: [Animation Suite](architecture/animation-suite.md)

## Resume rules
1. Re-read the current user request and current repository files. This index is navigation, not authority over new instructions.
2. Distinguish accepted product decisions, proposals, verified platform facts and unresolved research.
3. Every API claim needs a primary source link. Third-party behaviour claims need upstream docs/source.
4. Never assume an API is available on Windows 95 because it appears in a modern Win32 header or example.
5. Read the target OS page before proposing APIs, compiler flags, imports or process techniques.
6. Update the source ledger when a decision depends on new research.
7. Do not create Final-Plan.md unless explicitly requested.

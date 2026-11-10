# WinLook Repository Layout

> Status: user-defined target structure, 2026-11-10. This document records the authoritative directory organization requested for WinLook. It does not claim that every directory or implementation has already been created.

## 1. Root structure

```text
.github/
  workflows/

rust/
  win10/
    [complete Windows 10-specific application implementation]
  win11/
    [complete Windows 11-specific application implementation]
  library/
    [shared Rust libraries]

C/
  win95/
    [complete Windows 95-specific application implementation]
  library/
    [shared C libraries]

plan/
README.md
```

## 2. Platform-specific applications

- `rust/win10/` contains the complete Windows 10-specific WinLook application implementation.
- `rust/win11/` contains the complete Windows 11-specific WinLook application implementation.
- `C/win95/` contains the complete Windows 95-specific WinLook application implementation.

These are full application implementations, not thin compatibility shims around one shared modern executable. Each target owns the OS-specific entry points, adapters, UI/runtime integration, build configuration and tests it needs. Reusable code belongs in the corresponding language's `library/` tree and is consumed through explicit public contracts.

Do not infer that Windows 10 and Windows 11 have identical capabilities just because both use Rust. Likewise, C source compatibility does not by itself prove that a binary runs on Windows 95; compiler, linker, runtime, imports, APIs and hardware assumptions must be verified against the target.

## 3. Shared libraries

- `rust/library/` contains reusable Rust libraries shared by WinLook's modern applications, including planned libraries such as `wl_animate`, `wl_cpuv`, `wl_gpuv` and `wl_ffmpeg`.
- `C/library/` contains reusable C libraries shared by the legacy application, including corresponding libraries such as `wl_Lanimate`, `wl_Lcpuv`, `wl_Lgpuv` and `wl_Lffmpeg`.

The exact library set, public APIs and build manifests are specified by each library's own design and implementation. A library is not a duplicate engine tree: engines consume its public API rather than copying its implementation.

## 4. Background runtime

The two principal application executables are `winlook.exe` (Manager UI) and `winlook.bg.exe` (BGW background runtime). Their architecture and lifecycle are defined in [background-runtime.md](background-runtime.md). BGW hosts/co-ordinates the background subsystems; it is not a separate source-language tree or a requirement to create one executable per subsystem.

Each platform implementation supplies the appropriate executable/build outputs and OS adapters. Names in the architecture describe product roles; exact legacy binary naming and startup mechanics must be validated for the target OS/toolchain.

## 5. Planning and repository metadata

- `.github/workflows/`: CI, validation and release workflows.
- `plan/`: architecture, compatibility research, implementation plans, source ledger, roadmap and durable workspace context.
- Root `README.md`: project overview, current status, architecture entry points and development guidance.

Do not recreate the superseded `crates/`, `legacy/`, root `lib/`, `docs/`, `wllook/` or parallel acronym-engine source trees as the target organization. Historical paths may still exist in Git until a separately planned migration is performed; update references as source moves are actually completed rather than pretending a migration has already happened.

## 6. Migration and implementation rules

1. Keep this directory contract authoritative when adding new source.
2. Before moving historical source, map every file, manifest, import, workflow, test and documentation link.
3. Move source in a dedicated, reversible change; do not leave two active copies of the same library.
4. Preserve per-OS implementation boundaries and explicit capability reports.
5. Keep reusable library APIs independent from platform-specific application internals.
6. Verify each target with its actual compiler/runtime and the OS test matrix. A successful host build is not proof of runtime support.

# Build system and GitHub Actions

> Status: implemented build orchestration and CI workflows; generation selectors, source gates and compiler discovery are centralized. Last updated: 2026-11-12.

## 1. Design goals

- One Python entry point (build.py) for inspecting tools, planning setup, formatting, testing, compiling, packaging and cleaning.
- Python 3.9+ standard library only; no pip install is required by the build runner itself.
- The C and Rust sides share one ignored build root, while outputs and compiler caches are isolated by language.
- Commands use argument arrays rather than shell concatenation. Missing tools and unsupported targets fail with an actionable message.
- Tool downloads are SHA-256 checked before execution; the rustup-init checksum is fetched from the official Rust distribution alongside the installer.
- No build or CI stage claims a platform is supported merely because a source manifest or compiler exists.

## 2. Build and cache directory contract

All generated build contents live under the repository-root build/ directory:

    build/
      rust/
        shared/          Cargo workspace libraries and shared dependencies
        win10/           Windows 10 Manager executable output
        win11/           Windows 11 Manager executable output
        bgw/              BGW executable output, partitioned by Cargo package
      c/
        library-release/  CMake output for regular library builds
        library-sanitized/ CMake output for GCC ASan/UBSan tests
        win95/             Open Watcom Makefile output (when the Win95 app exists)
      artifacts/           preview packages and machine-generated reports
      .rcache/
        cargo-home/        Cargo registry and Git source cache
        sccache/            optional Rust compiler cache
        tools/              verified actionlint and Gitleaks executables
        toolchains/         pinned toolchain installer downloads and managed toolchains
      .Ccache/               ccache compiler object cache

The primary build outputs are in build/rust/ and build/c/. The hidden cache directories are intentionally separate:

- `build/rust/` and `build/c/` are sibling output roots under one common build footprint; they are not separate top-level workspaces.
- Compiled output must stay under the matching language output root. Do not put Cargo's target tree in `.rcache/` or CMake's build tree in `.Ccache/`; those hidden directories are caches only.

- CARGO_HOME=build/.rcache/cargo-home keeps Cargo package and Git caches inside the project build footprint.
- CARGO_TARGET_DIR=build/rust/shared is the default Rust output directory. Platform binaries override it to build/rust/win10, build/rust/win11, or build/rust/bgw/<package> so same-named executables cannot overwrite one another.
- When sccache exists, the runner uses build/.rcache/sccache through RUSTC_WRAPPER. It remains optional.
- CMake binary directories are under build/c/. On non-Windows hosts, if ccache exists, CMake uses it as the C compiler launcher and places its cache in build/.Ccache.
- MSVC uses its normal compiler/linker path; the GCC/Clang ccache integration is not forced onto MSVC.

The root .gitignore excludes /build/, so compiler outputs, caches, tool downloads and generated packages are not committed.

### Cleaning

python build.py clean only displays which output directories would be removed.

python build.py clean --yes removes build/rust/, build/c/ and build/artifacts/, preserving both compiler caches.

python build.py clean --yes --all also removes build/.rcache/ and build/.Ccache/. This removes cached tool downloads, so the next workflow-lint or secret scan must download its pinned executable again.

## 3. Python build runner

From the repository root (Python 3.9+):

The runner requires Rust 1.95.0 because the WinUI/Windows Reactor dependency graph declares that minimum. Before downloading anything, it probes the existing global rustup installation with `rustup run 1.95.0 rustc --version`. When that toolchain already exists, WinLook selects it for build subprocesses using `RUSTUP_TOOLCHAIN=1.95.0`; the user's configured default (for example, `stable` on Rust 1.94) is not changed. If 1.95.0 is missing but global rustup exists, the runner installs it side-by-side with `rustup toolchain install 1.95.0`, which also leaves the default unchanged. Only when no usable rustup installation is available does it bootstrap a project-local rustup under `build/.rcache/rustup-home/`, with Cargo data under `build/.rcache/cargo-home/`. In the global-toolchain path the runner deliberately does not override `CARGO_HOME` or `RUSTUP_HOME`, so rustup can see the toolchains already installed in the user's profile.

Windows application builds now select a toolchain by generation, and source availability is checked before any compiler installer is started:
When modifying an existing Visual Studio installation, the runner invokes the installed `setup.exe modify` operation through the Windows UAC `runas` flow, waits for the elevated process handle and uses `--quiet --norestart` so the Installer UI does not flash open. The user must approve the normal UAC prompt; declining it cancels without changing Visual Studio. The runner does not pass the bootstrapper-only `--wait` switch to `setup.exe`. A non-zero exit is translated to a known Visual Studio setup reason and includes excerpts/paths from newly-created `dd_setup_*.log`, `dd_client_*.log` and `dd_bootstrapper_*.log` files in `%TEMP%`. The installer documentation distinguishes bootstrapper `--wait` from the installed `setup.exe` parameters and documents the log names: https://learn.microsoft.com/en-us/visualstudio/install/use-command-line-parameters-to-install-visual-studio?view=vs-2022. If an existing Enterprise/Professional/Community instance cannot be modified or still does not expose the MSVC compiler/linker/SDK, WinLook may try the standalone Build Tools SKU once. It uses WinGet when available; if WinGet is absent, it downloads the official VS 2022 bootstrapper from `https://aka.ms/vs/17/release/vs_buildtools.exe`, caches it under `build/.rcache/installers/`, and verifies the Microsoft Authenticode signature with PowerShell before execution. The bootstrapper uses `--quiet --wait --norestart`; `--wait` is only used here, not with the installed `setup.exe`. An already-present Build Tools instance is never installed over, a failed first installation is never retried in the same run, and a user-declined UAC request does not trigger a second installer. Later runs rediscover the installed instance and reuse it.

- Windows 95/98/Me use the separate Open Watcom C path. The target Makefile and configured `WATCOM`/`wmake` must exist before compilation. The currently pinned Open Watcom installer artifact is Linux x64 and is downloaded only through the explicit verified setup action; its upstream documentation does not establish safe unattended installation flags.
- Windows NT 4 through Windows 8.1 use separate C targets under `C/<generation>/`. When those projects are implemented, the preferred Windows C path is native MinGW-w64 GCC plus Ninja, MinGW Make or MSYS Make. The runner checks `gcc -dumpmachine`; a generic GCC for another host is not accepted as a Windows compiler. Each target's CMake configuration must still set and test its OS-specific API/CRT/architecture baseline.
- Windows 10/11 use the existing Rust/WinUI 3 projects and the MSVC ABI. The runner checks the active environment, `vswhere`, Visual Studio Setup registry entries and standard install roots, then loads `VsDevCmd`/ `vcvars64`. It verifies `cl`, `link`, `rc`, Windows SDK headers and libraries. If a Visual Studio instance exists but lacks the C++ workload, it modifies that instance; it uses WinGet to install Build Tools only when no instance exists. MinGW GCC is not a substitute for this WinUI/MSVC linker and SDK requirement.

Visual Studio Build Tools are machine-wide and may require administrator approval. CMake is installed through WinGet only when its executable is missing. `setup --with-c --apply` reuses an existing MSVC or verified MinGW installation before considering any Visual Studio setup.

From the repository root (Python 3.9+):

| Command | Purpose |
|---|---|
| python build.py | Run the read-only doctor preflight; this is the default when no action is supplied |
| python build.py --help | Show the colored command index when running in a color-capable terminal |
| python build.py doctor | Report installed tools, compiler targets, Visual Studio instances and source readiness for every Windows generation |
| python build.py setup | Show available setup targets; choose an explicit --with-* option |
| python build.py setup --with-rust | Check existing global and project-local Rust 1.95.0 installations; show the fallback plan if absent |
| python build.py setup --with-rust --apply | Reuse Rust 1.95.0 if installed; otherwise add it side-by-side with existing rustup, or bootstrap a project-local rustup as the last resort |
| python build.py setup --with-c --apply | Reuse installed MSVC or MinGW GCC, then ensure CMake is available; modify an existing Visual Studio instance only if its C++ workload is missing |
| python build.py setup --with-cmake --apply | Install CMake through WinGet on Windows only when CMake is missing |
| python build.py setup --open-watcom --apply | Download and SHA-256 verify the pinned Open Watcom installer; the installer itself must still be run manually |
| python build.py fmt --check | Check Rust formatting |
| python build.py test libraries | Test Rust and C shared libraries |
| python build.py verify libraries --sanitizers | Rust format/tests/Clippy/docs plus C89 sanitizer tests where the host compiler supports them |
| python build.py verify ui | Windows-only Manager UI formatting/tests/Clippy gate |
| python build.py windows.95 | Build Windows 95 when C/win95/Makefile and Open Watcom are present |
| python build.py windows.98 | Build Windows 98 when C/win98/Makefile and Open Watcom are present |
| python build.py windows.me | Build Windows Me when C/winme/Makefile and Open Watcom are present |
| python build.py windows.nt4 | Build Windows NT 4 when its CMake target is present |
| python build.py windows.2000 | Build Windows 2000 when its CMake target is present |
| python build.py windows.xp | Build Windows XP when its CMake target is present |
| python build.py windows.vista | Build Windows Vista when its CMake target is present |
| python build.py windows.7 | Build Windows 7 when its CMake target is present |
| python build.py windows.8 | Build Windows 8 when its CMake target is present |
| python build.py windows.8.1 | Build Windows 8.1 when its CMake target is present |
| python build.py windows.10 | Build the Windows 10 Rust/WinUI Manager |
| python build.py windows.11 | Build the Windows 11 Rust/WinUI Manager |
| python build.py build libraries | Build shared Rust and C libraries |
| python build.py build bgw | Build explicitly declared BGW Cargo binary targets |
| python build.py package win10 | Package a previously built Windows 10 Manager executable |
| python build.py workflow-lint --apply | Download and verify actionlint, then lint all workflow YAML |
| python build.py secret-scan --apply | Download and verify Gitleaks, then scan reachable Git history with redacted output |

Legacy `build win10`, `build win11` and `build win95` forms remain compatibility aliases; new scripts should use the generation selectors. Every `windows.*` command checks its expected source manifest before selecting or installing compilers. Missing source means a clear source-gate error and no toolchain download or setup attempt.

The runner has no implicit setup for the standalone `setup` command: installer actions need `--apply`, clean operations need `--yes`, and dry-run options do not execute commands. A build may provision a required tool after it has confirmed that the requested target source exists and the tool is actually missing. Downloads are kept under `build/.rcache/` where the toolchain supports project-local caching. Color output respects `NO_COLOR` and `WINLOOK_NO_COLOR=1`.

Open Watcom has a documented Windows 95/98/Me target, but the official installer guide does not document the unattended flags used by the current Linux installer asset. The runner therefore verifies and downloads that installer but deliberately does not guess its install flags. A Win95 CI runner must have Open Watcom installed and expose WATCOM, wcc386 and wmake.

## 4. Workflow inventory

All third-party Actions used by these workflows are pinned to a verified commit SHA rather than a moving tag. As checked on 2026-11-11: checkout v7.0.1, setup-python v7.1.0, cache v6.1.0, upload-artifact v7.0.2, download-artifact v8.0.2, CodeQL Action v4.39.3 and dependency-review v5.0.0.

| Workflow | Responsibility | Gate |
|---|---|---|
| workflow-lint.yml | Python syntax/unit tests, default CLI smoke test, workflow-command parser coverage and actionlint validation for every workflow | Must pass before workflow changes merge |
| libraries.yml | Rust shared-library formatting/tests/Clippy/docs (wl_animate, wl_cpuv and wl_gpuv only); C89 MSVC; C89 GCC ASan/UBSan | Portable library gate |
| manager-ui.yml | WinUI 3 Manager unit tests and Clippy | Runs only when all UI manifests exist |
| win10.yml | Windows 10 Manager build and preview package | Runs only when the Win10 app manifest exists |
| win11.yml | Windows 11 Manager build and preview package | Runs only when the Win11 app manifest exists |
| bgw.yml | Compile declared winlook.bg/BGW target(s) | Runs only when a BGW binary target is declared |
| win95.yml | Legacy C application build using Open Watcom | Requires C/win95/Makefile and the configured self-hosted open-watcom runner |
| docs.yml | Required-document checks and Python test compilation | Documentation/build-runner gate |
| security.yml | CodeQL for C/C++, Rust, Actions and Python; dependency review; Gitleaks full-history scan | Security gate |
| build-all.yml | Manual/main-branch strict toolchain preflight, full supported verification, and actual Win10/Win11 application compilation when source manifests exist; reruns when its own workflow or build-runner tests change | Integration build |
| release.yml | Version-tag release after library validation and both modern app builds | Publish gate |

The BGW and Win95 workflows intentionally use source/toolchain detection. If a target is not implemented, it stays skipped; it is not replaced by a dummy binary or a fake passing build.

### Win95 runner setup

When the Win95 source and Makefile are implemented, register a self-hosted Linux x64 runner with labels self-hosted, linux, x64 and open-watcom.

Install the pinned Open Watcom 2.0 compiler on that runner, configure WATCOM, INCLUDE, EDPATH and PATH, and set repository variable WINLOOK_ENABLE_WIN95_RUNNER=true. The workflow will otherwise emit an explicit runner-configuration note rather than trying to use a missing toolchain.

### Release boundary

A v* tag starts the release workflow. It requires the shared-library verification job and successful Windows 10 and Windows 11 application packages. The current preview package documents that it is framework-dependent and does not contain winlook.bg.exe. Win95 and BGW outputs are not implied to exist until their own build gates have been implemented and passed.

## 5. Completed branch/merge order

The staged implementation sequence has landed on `main` in the intended order:

1. Shared libraries — `feature/library-foundation`, merged through PR #1.
2. Manager UI — `feature/wintoys-manager-ui`, merged through PR #2.
3. Build orchestration and CI — `github/workflow`, merged through PR #3 after PR #2.

The focused workflow suite is now the canonical CI configuration. The original staged-branch instructions above are historical and no longer describe pending merge work.

## 6. Known boundaries and follow-up work

- The current WinUI 3 prototype requires Windows App Runtime. The Windows build workflows compile and package the Manager; they do not prove clean-machine runtime bootstrap or low-end UI performance.
- The build-runner unit suite validates helper logic and the supported workflow CLI command shapes. Actual compiler/build evidence comes from the Rust/C library verification workflows and the Windows integration build; a passing Python test suite or actionlint run alone is not a successful application build.
- The current C/Win95 application directory and Makefile do not exist yet. Open Watcom compiler installation still needs an unattended, source-verified approach or the documented self-hosted runner.
- BGW source/binary targets do not exist yet; no BGW executable is produced by this branch.
- Windows application smoke tests, Windows 95 VM tests, installer/uninstaller rollback, hardware graphics, and runtime memory/CPU benchmarks remain distinct acceptance gates.
- CodeQL is source analysis, not a substitute for compiler tests, fuzzing, sanitizer runs or VM tests.
- Rust/C libraries currently build as separate language implementations. No cross-language ABI equivalence is claimed solely from successful compilation.

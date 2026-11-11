# Build system and GitHub Actions

> Status: implemented build orchestration and CI workflows; platform execution gates are kept separate from source-availability gates. Last updated: 2026-11-11.

## 1. Design goals

- One Python entry point (build.py) for inspecting tools, planning setup, formatting, testing, compiling, packaging and cleaning.
- Python 3.9+ standard library only; no pip install is required by the build runner itself.
- The C and Rust sides share one ignored build root, while outputs and compiler caches are isolated by language.
- Commands use argument arrays rather than shell concatenation. Missing tools and unsupported targets fail with an actionable message.
- Tool downloads are pinned and SHA-256 checked before extraction or execution.
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

From the repository root:

| Command | Purpose |
|---|---|
| python build.py doctor | Report Python, Rust, CMake, MSVC/GCC/Clang, cache tools and available source targets |
| python build.py setup | Print toolchain notes; use an explicit --with-* option to select what to install |
| python build.py setup --with-rust | Plan stable Rust installation/update without changing the machine |\n| python build.py setup --with-rust --apply | Install/update stable Rust and rustfmt/Clippy components |\n| python build.py setup --with-rust --install-rustup --apply | Use WinGet to install Rustup when missing; open a new terminal and rerun with --with-rust |
| python build.py setup --with-cmake --apply | Install CMake through WinGet on Windows when missing |
| python build.py setup --open-watcom --apply | Download and SHA-256 verify the pinned Open Watcom installer; the installer itself must still be run manually |
| python build.py fmt --check | Check Rust formatting |
| python build.py test libraries | Test Rust and C shared libraries |
| python build.py verify libraries --sanitizers | Rust format/tests/Clippy/docs plus C89 GCC ASan/UBSan tests |
| python build.py verify ui | Windows-only Manager UI formatting/tests/Clippy gate |
| python build.py build win10 | Build the Windows 10 Manager |
| python build.py build win11 | Build the Windows 11 Manager |
| python build.py build bgw | Build explicitly declared BGW Cargo binary targets |
| python build.py build win95 | Build the C/Win95 Makefile with an installed Open Watcom toolchain |
| python build.py package win10 | Package a previously built Windows 10 Manager executable |
| python build.py workflow-lint --apply | Download and verify actionlint, then lint all workflow YAML |
| python build.py secret-scan --apply | Download and verify Gitleaks, then scan reachable Git history with redacted output |

The runner has no hidden setup side effects. Install operations need explicit --apply; clean operations need --yes; dry-run options print commands without executing them. Downloads are kept under build/.rcache/.

Open Watcom has a documented Windows 95/98/Me target, but the official installer guide does not document the unattended flags used by the current Linux installer asset. The runner therefore verifies and downloads that installer but deliberately does not guess its install flags. A Win95 CI runner must have Open Watcom installed and expose WATCOM, wcc386 and wmake.

## 4. Workflow inventory

All third-party Actions used by these workflows are pinned to a verified commit SHA rather than a moving tag. As checked on 2026-11-11: checkout v7.0.1, setup-python v7.1.0, cache v6.1.0, upload-artifact v7.0.2, download-artifact v8.0.2, CodeQL Action v4.39.3 and dependency-review v5.0.0.

| Workflow | Responsibility | Gate |
|---|---|---|
| workflow-lint.yml | Python syntax/unit tests and actionlint validation for every workflow | Must pass before workflow changes merge |
| libraries.yml | Rust shared-library formatting/tests/Clippy/docs (wl_animate, wl_cpuv and wl_gpuv only); C89 MSVC; C89 GCC ASan/UBSan | Portable library gate |
| manager-ui.yml | WinUI 3 Manager unit tests and Clippy | Runs only when all UI manifests exist |
| win10.yml | Windows 10 Manager build and preview package | Runs only when the Win10 app manifest exists |
| win11.yml | Windows 11 Manager build and preview package | Runs only when the Win11 app manifest exists |
| bgw.yml | Compile declared winlook.bg/BGW target(s) | Runs only when a BGW binary target is declared |
| win95.yml | Legacy C application build using Open Watcom | Requires C/win95/Makefile and the configured self-hosted open-watcom runner |
| docs.yml | Required-document checks and Python test compilation | Documentation/build-runner gate |
| security.yml | CodeQL for C/C++, Rust, Actions and Python; dependency review; Gitleaks full-history scan | Security gate |
| build-all.yml | Manual/main-branch complete build-order validation of all available modern targets | Integration build |
| release.yml | Version-tag release after library validation and both modern app builds | Publish gate |

The BGW and Win95 workflows intentionally use source/toolchain detection. If a target is not implemented, it stays skipped; it is not replaced by a dummy binary or a fake passing build.

### Win95 runner setup

When the Win95 source and Makefile are implemented, register a self-hosted Linux x64 runner with labels self-hosted, linux, x64 and open-watcom.

Install the pinned Open Watcom 2.0 compiler on that runner, configure WATCOM, INCLUDE, EDPATH and PATH, and set repository variable WINLOOK_ENABLE_WIN95_RUNNER=true. The workflow will otherwise emit an explicit runner-configuration note rather than trying to use a missing toolchain.

### Release boundary

A v* tag starts the release workflow. It requires the shared-library verification job and successful Windows 10 and Windows 11 application packages. The current preview package documents that it is framework-dependent and does not contain winlook.bg.exe. Win95 and BGW outputs are not implied to exist until their own build gates have been implemented and passed.

## 5. Required branch/merge order

The workflow branch is intentionally last in the implementation order:

1. Shared libraries — feature/library-foundation; PR #1 has been merged into main.
2. Manager UI — feature/wintoys-manager-ui; PR #2 must be reviewed and have its UI verification failures corrected before merge.
3. Build orchestration and CI — github/workflow; this branch is intended to target feature/wintoys-manager-ui and merge after the UI branch. Once that stack is merged, the workflow suite becomes the repository's canonical CI.

Do not merge github/workflow directly into main ahead of the Manager UI branch. Its pull request should remain stacked until the preceding implementation stages have landed.

## 6. Known boundaries and follow-up work

- The current WinUI 3 prototype requires Windows App Runtime. CI compiles the Manager but does not prove clean-machine runtime bootstrap or low-end UI performance.
- The current C/Win95 application directory and Makefile do not exist yet. Open Watcom compiler installation still needs an unattended, source-verified approach or the documented self-hosted runner.
- BGW source/binary targets do not exist yet; no BGW executable is produced by this branch.
- Windows application smoke tests, Windows 95 VM tests, installer/uninstaller rollback, hardware graphics, and runtime memory/CPU benchmarks remain distinct acceptance gates.
- CodeQL is source analysis, not a substitute for compiler tests, fuzzing, sanitizer runs or VM tests.
- Rust/C libraries currently build as separate language implementations. No cross-language ABI equivalence is claimed solely from successful compilation.

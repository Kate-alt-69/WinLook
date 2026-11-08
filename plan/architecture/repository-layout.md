# Proposed WinLook Repository Layout

> Status: proposed target structure. This is a planning specification only. Existing source files are not moved by this document.

## 1. Goals

Keep the repository easy to navigate for both contributors and tooling:

- Application and engine source lives in the language-specific C and Rust trees.
- The C and Rust trees have nearly parallel subsystem names and responsibilities.
- Every subsystem named by an acronym uses its uppercase directory name so it is easy to locate.
- WinLook-owned reusable libraries live under the top-level lib directory.
- Public APIs are separated from implementation details.
- UI code is separated from engine logic and split into maintainable pages/components.
- Documentation explains the application, each subsystem and every public API surface.
- The current Rust/MSVC Windows target baseline is Windows 10+; the C implementation is the planned route for pre-Windows-10 releases. Both language trees may remain structurally parallel, but similar folders do not mean one binary, identical APIs or equal platform capabilities.

## 2. Proposed repository tree

The following is the intended direction, not a frozen file-by-file manifest:

    ROOT/
      wllook/
        product-level manifests, package metadata and shared application assets
      C/
        UI/
        WLDE/
        WLDM/
        WLWM/
        WLOSDE/
        BGW/
        WLT/
        APPEARANCEENGINE/
        ICONENGINE/
        include/
        tests/
        index.c
      rust/
        UI/
        crate/
          WLDE/
          WLDM/
          WLWM/
          WLOSDE/
          BGW/
          WLT/
          APPEARANCEENGINE/
          ICONENGINE/
        WLDE/
        WLDM/
        WLWM/
        WLOSDE/
        BGW/
        WLT/
        APPEARANCEENGINE/
        ICONENGINE/
        tests/
      lib/
        wl_animate/
        wl_Lanimate/
        wl_cpuv/
        wl_gpuv/
        wl_Lcpuv/
        wl_Lgpuv/
      docs/
        APP/
        API/
        ARCHITECTURE/
        COMPATIBILITY/
        MODULES/
      plan/
        architecture/
        compatibility/
        references/
        roadmap.md
      README.md
      LICENSE
      CONTRIBUTING.md

The exact contents and spelling of the product-level wllook directory still need to be confirmed. The proposal reserves it for product/packaging metadata and shared assets, not for a second copy of engine source. Root-level planning, build and repository metadata remain at the root where appropriate.

## 3. Source-tree symmetry

C and Rust should have the same high-level feature map:

| Feature area | C source | Rust source | Public surface |
|---|---|---|---|
| Desktop engine | C/WLDE | rust/WLDE | C public header and Rust crate/WLDE |
| Display manager | C/WLDM | rust/WLDM | C public header and Rust crate/WLDM |
| Window manager | C/WLWM | rust/WLWM | C public header and Rust crate/WLWM |
| On-screen display | C/WLOSDE | rust/WLOSDE | C public header and Rust crate/WLOSDE |
| Audio & Media | Directory/acronym pending | Directory/acronym pending | Separate public contract; keep audio and media session models distinct |
| File Explorer customization | Appearance Engine submodule | Appearance Engine submodule | Shell handlers plus separate presentation adapter; not a second file manager |
| Background worker | C/BGW | rust/BGW | Language-specific BGW API |
| Templates | C/WLT | rust/WLT | Language-specific parser/manifest API |
| Appearance engine | C/APPEARANCEENGINE | rust/APPEARANCEENGINE | Language-specific API |
| Icon engine | C/ICONENGINE | rust/ICONENGINE | Language-specific API |
| Application UI | C/UI | rust/UI | Platform-appropriate UI entry point |

The symmetry is organizational and conceptual. The Audio & Media acronym/source-directory name remains undecided; do not create a path before approval. Explorer customization is initially an Appearance Engine submodule, not a second Shell or file-manager engine. A module may have fewer files or features on an older OS. A feature unsupported by that OS should be absent or report unsupported, not be represented by a fake implementation.

Use uppercase names for acronym feature directories consistently. Keep source file names and language-specific symbols idiomatic where useful; the uppercase rule is primarily a directory/navigation convention.

## 4. Rust layout and crate exports

The rust directory contains the modern Windows implementation source. Under the currently documented official Rust/MSVC target baseline, WinLook's Rust Windows binaries target Windows 10 and later. The Rust GNU target page is currently a stub and does not state a supported old-client baseline.

For Vista, Windows 7, 8 and 8.1, the C implementation is the planned default unless a separate pinned Rust toolchain/runtime has been proven on the exact OS. This does not prohibit portable Rust libraries or non-Windows host tests; it limits the supported Windows binary/runtime claim.

- rust/WLDE, rust/WLDM, rust/WLWM and the other engine directories contain the actual engine logic, adapters, internal state and tests.
- rust/crate contains the public crate/facade surface for each engine. Each planned public engine crate has its own directory and API documentation.
- A public crate must expose only intentional public types, functions, traits and errors. Internal implementation modules remain private unless the API contract requires them.
- A public crate should re-export stable types from its engine implementation rather than duplicate implementation code.
- Cargo workspace membership, package naming, dependency direction and how the public facade references the engine implementation must be designed before moving or generating manifests. Directory names can remain uppercase even if Cargo package identifiers follow Rust naming conventions.
- rust/UI owns the Rust application's UI entry point, navigation shell, pages, controls and UI state adapters. It consumes public engine APIs; it must not directly reach into private engine internals.

Potential module shape:

    rust/
      UI/
        main.rs
        shell/
        pages/
        components/
        dialogs/
        accessibility/
      crate/
        WLDE/
          Cargo.toml
          src/lib.rs
          README.md
        WLDM/
          Cargo.toml
          src/lib.rs
          README.md
      WLDE/
        README.md
        src-or-module-layout-to-be-decided
        tests/
      WLDM/
        README.md
        implementation-layout-to-be-decided
        tests/

This is an architectural sketch. The final Rust file/module convention should follow Cargo and Rust module best practices rather than forcing every engine into an identical internal file layout.

## 5. C layout and public exports

The C directory contains the legacy application and engine implementation.

- C/WLDE, C/WLDM and the other engine directories contain the actual C implementation, platform adapters and focused tests.
- C/include contains public headers. Each engine owns a public header with the declarations and types consumers are permitted to use.
- C/index.c is a proposed umbrella registration/aggregation entry point for the application build. It is not a substitute for public declarations.
- C/include/index.h may provide a single umbrella include for consumers, while engine-specific headers remain authoritative.
- Exported symbols, calling conventions, structure packing, integer widths and ownership rules must be documented explicitly for every supported compiler/ABI.
- C/UI contains the legacy UI shell and split page/control implementations. It must use controls and APIs that exist on the selected OS/compiler target.
- Do not require a modern CRT, WinUI, modern COM interface or modern system DLL merely to preserve folder symmetry.

Potential module shape:

    C/
      UI/
        main.c
        shell/
        pages/
        components/
        dialogs/
        resources/
      include/
        index.h
        WLDE.h
        WLDM.h
        WLWM.h
        WLOSDE.h
      WLDE/
        WLDE.c
        internal.h
        tests/
      WLDM/
      WLWM/
      WLOSDE/
      BGW/
      WLT/
      index.c

The public C ABI still needs a separate design decision. In particular, decide whether index.c registers available engines, provides startup/shutdown aggregation, or is only a link-time aggregation unit. Keep it small and avoid turning it into a monolithic engine.

## 6. Shared library directory

All WinLook-owned reusable custom libraries live under ROOT/lib. They have their own source, tests, manifests/build files and README/API documentation in their library directory.

Initial mapping:

| Existing prototype path / planned library | Proposed target | Language | Status |
|---|---|---|---|
| crates/wl_animate | lib/wl_animate | Rust | Existing implementation; separate migration required |
| legacy/wl_Lanimate | lib/wl_Lanimate | C | Existing implementation; separate migration required |
| planned wl_cpuv | lib/wl_cpuv | Rust | Architecture only; not implemented |
| planned wl_gpuv | lib/wl_gpuv | Rust | Architecture only; not implemented |
| planned wl_Lcpuv | lib/wl_Lcpuv | C | Architecture only; not implemented |
| planned wl_Lgpuv | lib/wl_Lgpuv | C | Architecture only; not implemented |

Do not move the existing prototypes or create the planned resource-governor libraries as part of a documentation-only layout change. Once the full library set and public contracts are accepted, migrate/create paths in a dedicated repository-structure change and update manifests, imports, tests, documentation links and build workflows together.

A library is not engine source. Engine directories may depend on a library through its public contract, but must not copy its implementation into WLDE/WLWM or fork it per subsystem without a documented reason.

## 7. UI module layout

Both language trees should contain a dedicated UI directory. The UI should itself be split into:

- application entry point and shell;
- navigation model;
- page-level modules;
- reusable controls/components;
- dialogs and confirmation surfaces;
- UI state and API adapters;
- accessibility and localization resources;
- styling/theme tokens supported by the target UI framework.

Page modules should not own duplicated copies of system state. They call the relevant public engine/API surface and display the reported capability and result state.

A configuration tab should be independently editable without changing the navigation shell, unrelated pages or engine implementation. Avoid one giant UI source file and avoid embedding system operations directly in view code.

## 8. Naming rules

- Top-level language directories: C and rust.
- Reusable library root: lib.
- Rust API crate root: rust/crate.
- Engine/acronym directories: uppercase, e.g. WLDE, WLDM, WLWM, WLOSDE and BGW.
- Library identities retain their established names: wl_animate, wl_Lanimate, wl_cpuv, wl_gpuv, wl_Lcpuv and wl_Lgpuv.
- Public C headers use clear engine names and stable include paths.
- Source filenames remain language-idiomatic; do not force uppercase filenames unless a compiler or established convention requires it.

## 9. Structure migration gates

Before changing the actual repository layout:

- [ ] Confirm the purpose and exact spelling of the root wllook directory.
- [ ] Ratify the full list of engine/module directories for both languages.
- [ ] Confirm that lib is the home for both modern and legacy reusable libraries.
- [ ] Decide the Rust workspace/public-facade arrangement.
- [ ] Decide the C public-header and index.c responsibilities.
- [ ] Define mirrored UI page names and platform-specific control implementations.
- [ ] Map current files to their destinations and identify every import/build/doc reference.
- [ ] Plan one reversible migration with no duplicate active source trees.
- [ ] Run both language builds/tests and validate documentation links after the move.

No source files should be moved until these gates are accepted.

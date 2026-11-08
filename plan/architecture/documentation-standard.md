# WinLook Documentation Architecture and Standards

> Status: documentation plan. The public application guide and API references will be produced alongside implementation; this file defines their expected scope.

## 1. Documentation goals

WinLook must be understandable as both an application and a developer platform. Documentation should explain what the user can do, what each subsystem owns, how to integrate through public APIs, what each OS supports, and how to recover from failure.

Documentation is a first-class deliverable. A feature is not ready merely because code and a settings page exist.

## 2. Proposed documentation tree

    docs/
      APP/
        index.md
        getting-started.md
        first-run-and-permissions.md
        manager-overview.md
        appearance-and-icons.md
        desktop-and-wallpapers.md
        window-management.md
        displays-and-notifications.md
        templates-and-store.md
        accessibility.md
        troubleshooting.md
        recovery-and-uninstall.md
      API/
        index.md
        RUST/
          index.md
          WLDE.md
          WLDM.md
          WLWM.md
          WLOSDE.md
          WLT.md
        C/
          index.md
          ABI-and-calling-conventions.md
          WLDE.md
          WLDM.md
          WLWM.md
          WLOSDE.md
          WLT.md
      ARCHITECTURE/
        index.md
        repository-layout.md
        process-and-IPC.md
        configuration-and-capabilities.md
        security-model.md
        restoration-model.md
      COMPATIBILITY/
        index.md
        matrix.md
        per-OS/
      MODULES/
        WLDE.md
        WLDM.md
        WLWM.md
        WLOSDE.md
        BGW.md
        WLT.md
        APPEARANCEENGINE.md
        ICONENGINE.md
      CONTRIBUTING/
        build-from-source.md
        testing.md
        documentation.md
        coding-style.md

This is a proposed public documentation structure. Detailed planning and unresolved research remain under plan/. Do not duplicate the same content into several files; use focused pages and links.

## 3. Application documentation

The user-facing manual should cover:

- What WinLook is, which Windows versions are supported, and what is intentionally unsupported.
- Installation, first-run, permissions, updates and safe-mode/recovery startup.
- Manager navigation and every configuration area.
- Applying, previewing, exporting and restoring appearance settings.
- WLDE wallpapers, widgets, Groups, Store/catalogues and Game Mode.
- WLWM layouts, window rules, workspace behaviour and target compatibility.
- WLDM display information and safe display-profile changes.
- WLOSDE notifications, quiet hours and accessibility options.
- WLT import/export, Template Groups, conflicts, licences and permissions.
- Extension trust, capability prompts and removal.
- Diagnostics, logs, backups, recovery and clean uninstall.
- Differences and unavailable features for each Windows release.

Every guide should include prerequisites, steps, expected outcome, possible partial/unsupported result, undo/recovery route and relevant OS caveats.

## 4. Public API documentation

Each public Rust crate and C API should document:

- Purpose, ownership and supported use cases.
- Public types, functions, constants, traits or structures.
- Input/output semantics and validation rules.
- Error/result codes and failure conditions.
- Threading, reentrancy, callback and cancellation rules.
- Resource ownership and lifetime.
- ABI, calling convention, structure layout and architecture constraints for C.
- Feature/API stability status and version compatibility.
- Minimum OS/runtime/driver requirements.
- Examples that compile or have a clearly stated verification status.
- Security restrictions and actions that the API deliberately refuses.

Public API docs must distinguish stable, experimental, deprecated and internal interfaces. A public crate or header must not accidentally expose private implementation details.

### Rust-specific expectations

- Use module-level documentation for crate purpose and guarantees.
- Document public items at their declaration.
- Provide runnable examples where they add value.
- Generate and check rustdoc output as part of the documented build process.
- Explain feature flags and platform cfg behaviour.

### C-specific expectations

- Document every public header and exported symbol.
- Specify calling convention, integer widths, structure packing/alignment and ownership.
- Define which side allocates and frees each resource.
- Avoid passing compiler-specific CRT objects across ABI boundaries.
- Provide minimal examples for each supported compiler/target.
- Record API availability per OS and runtime.

Exact documentation generators are not selected yet. Prefer toolchains already required by the project and formats that can be built offline.

## 5. Per-engine documentation template

Every subsystem page should contain:

1. Purpose and non-goals.
2. Responsibilities and ownership boundaries.
3. Public API and internal module map.
4. Dependencies and process interactions.
5. Configuration schema and default values.
6. Capability detection and per-OS behaviour.
7. Lifecycle/state machines.
8. Error codes and diagnostics.
9. Security/privacy requirements.
10. Performance/resource budgets.
11. Accessibility behaviour.
12. Failure, fallback, restoration and uninstall behaviour.
13. Tests and exact verification evidence.
14. Examples and common integration patterns.
15. Known limitations and unresolved decisions.

## 6. Documentation quality rules

- Keep planning, design proposal, implementation and tested behaviour visibly separate.
- Never document an unimplemented feature as available.
- Put the exact OS/build/runtime scope next to compatibility claims.
- Update public API docs in the same change as public contract changes.
- Include migration notes for changed APIs, configuration schemas and WLT manifests.
- Give every example a version and verification state.
- Avoid examples that require secrets, elevated access or untrusted native code.
- Use one canonical page per subject and link to it from indexes.
- Check internal links, code examples, generated API output and capitalization.
- Provide an offline documentation build path where practical.

## 7. Documentation release gate

Before a module release:

- [ ] Application guide describes its actual UI and supported options.
- [ ] Every public API declaration is documented.
- [ ] C ABI details are complete for every supported compiler target.
- [ ] Rust crate documentation builds without missing public-item documentation warnings where practical.
- [ ] Examples are compiled/tested or explicitly marked unverified.
- [ ] Per-OS pages state capability and test evidence.
- [ ] Security, permission, recovery and uninstall documentation matches behaviour.
- [ ] Navigation indexes and links are valid.

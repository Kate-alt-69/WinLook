# WinLook Planning Workspace

This directory is the source of truth for WinLook's living plan, compatibility research, subsystem design notes and agent-readable workspace context.

## Start here
1. [Repository README](../README.md) — public project overview and current status.
2. [Plan.md](Plan.md) — living scope, decisions, checklist and links.
3. [Compatibility guide](compatibility/README.md) — how Windows compatibility is evaluated.
4. [Compatibility matrix](compatibility/matrix.md) — cross-generation summary.
5. [Architecture overview](architecture/overview.md) — subsystem relationships and process model.
6. [Appearance Engine](architecture/appearance-engine.md), [File Explorer Customization](architecture/explorer-customization.md) and [Audio & Media Experience](architecture/audio-experience.md) — appearance, Shell and media architecture.
7. [Clipboard Experience](architecture/clipboard-experience.md) and [Profiles and Optimization](architecture/profiles-and-optimization.md) — coordinated settings and portable profile design.
8. [Standby/security](architecture/standby-security.md) and [extensions](architecture/extensions.md) — lifecycle, ecosystem and package trust.
9. [Source ledger](references/SOURCES.md) — primary API/OS references and third-party project sources.
10. [Agent index](index.md) and [workspace memory](workspace.md) — compact navigation and durable context.

## Structure
- Plan.md: current living plan; do not replace it with an implementation specification.
- compatibility/: per-Windows-generation platform records.
- architecture/: detailed subsystem boundaries and design decisions.
- references/SOURCES.md: citation and evidence ledger.
- index.md / workspace.md: Research Companion-compatible agent navigation and durable notes.
- Final-Plan.md: deliberately not created yet; it requires later user approval.

## Evidence and citation policy
Version-sensitive technical statements must carry direct links near the claim and be listed in the source ledger. Each source entry identifies what it supports and what it does not prove. API documentation does not prove that a WinLook implementation has passed runtime tests. Per-OS capability claims remain provisional until verified on the stated build/service pack/runtime.

## Scope guard
Planning documents may describe candidate approaches. They must not turn unresolved ideas into implementation requirements without an explicit decision. Compatibility is capability-based, not one yes/no label for an entire Windows release.

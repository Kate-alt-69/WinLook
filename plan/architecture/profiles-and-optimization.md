# WinLook Profiles and Optimization

> Status: planning proposal. Profiles and Templates are the central way users compose WinLook behaviour. This is not a separate Scene Engine, generic automation platform or startup-app manager.

## 1. Purpose

A Profile is an active set of coordinated preferences for WinLook's existing engines and supported system adapters. It describes how the desktop, windows, displays, audio, notifications and performance policy should behave in a particular context.

A Template is a portable package that may contain selected Profiles, themes, engine settings and assets. The local active Profile and a portable Template are related but not the same object.

## 2. Profile contents

A profile may declare:
- Appearance and icon theme selections, including an optional File Explorer presentation preset and its approved customization layers.
- WLDE wallpaper, widget layout, Group visibility and quality policy.
- WLWM window rules, layout preferences and logical workspace settings.
- WLDM monitor-role preferences and supported per-display settings, including color profile selection where the OS/driver exposes it.
- WLOSDE notification category, quiet-hours and animation policy.
- Audio endpoint/per-app session preferences and supported player preferences through Audio & Media; media playback position, active media and thumbnails are transient and are not profile state.
- Clipboard privacy, retention, exclusions and palette preferences.
- Animation and reduced-motion settings.
- Resource targets: FPS/quality preference, background-work policy, Game Mode and suspend/optimize behaviour.
- Supported power preferences only where a documented adapter and safe restoration path exist.

Profiles must not contain credential data, clipboard payloads, cache entries, runtime process handles or private session state.

## 3. Activation and conditions

Possible activation sources:
- explicit manual selection;
- user-created schedule, if later approved;
- foreground application/window category detected by WLWM;
- monitor topology or display-role change observed by WLDM;
- power-source/battery state where reported by Windows;
- user-selected external event from an approved integration.

The initial release can remain manual-first. Automatic conditions are opt-in, visible and reversible. No profile should silently switch due only to an ambiguous executable name or unstable device identifier.

## 4. Profile apply transaction

1. Resolve profile identity/schema and migration.
2. Detect current OS, capabilities, device topology and active session.
3. Preview supported, partial, unavailable and conflicting settings.
4. Determine the active profile and the source of each setting.
5. Snapshot settings that will change using the Restoration journal.
6. Apply engine settings through public APIs/adapters in dependency order.
7. Verify each state change and record result.
8. If required components fail, roll back affected transaction; optional components may be skipped with a report.
9. Keep a profile activation record and provide a one-step return to the previous profile.

A profile never bypasses capability checks, consent or per-operation security policy.

## 5. Optimization model

Optimization belongs to the profile and existing resource-aware engines:
- WLDE reduces frame rate, render scale, effects, decoder work and background updates.
- WLWM may reduce optional monitoring/animation work but must not compromise focus, process safety or application startup.
- WLOSDE can reduce nonessential notification effects.
- wl_animate / wl_Lanimate honour reduced-motion and frame budgets.
- Audio and clipboard modules apply only their declared preferences.
- WLDM applies only supported display settings and keeps a tested revert path.

There is no generic "kill background processes" or "disable services" optimization action. No blind service disabling, task termination or registry tweak bundle.

A user can choose Normal, Optimize or Suspend profile policies. Suspend means the listed WinLook components pause expensive work; it does not suspend Windows or arbitrarily terminate applications. WLDE's existing resource governor remains responsible for its own 90% sustained-pressure policy after the measurement/hysteresis rules are finalized.

## 6. Profile precedence

Proposed precedence:
1. hard safety/security and OS capability constraints;
2. temporary explicit user override;
3. per-application or per-device override;
4. active profile;
5. selected theme/template defaults;
6. global WinLook preference;
7. platform default.

Changing a profile should not overwrite an active temporary user action. Manager should show which source currently controls a setting.

## 7. Profile versus Template

| Object | Meaning | Storage/lifecycle |
|---|---|---|
| Global preferences | Default WinLook user settings | Local user config |
| Profile | Coordinated active configuration and optional conditions | Local profile store; user can switch/duplicate/export |
| Theme | Appearance/assets subset | Reusable visual component |
| Template (.wl.t) | Portable selected settings, profiles and assets | Export/import package |
| Template Group | ZIP containing multiple independent .wl.t packages | Collection import |

Machine-local Desktop Groups, physical monitor instance IDs, session/process state and clipboard contents are excluded from portable Templates. Device mappings use portable role/name hints and require confirmation where ambiguous.

## 8. UI

Profiles belong in the Manager's main navigation and are presented as a simple list/cards with:
- current active profile;
- manual activation;
- create/duplicate/rename/delete;
- profile composition summary;
- conditions and enabled/disabled state;
- preview/apply and revert;
- export to WLT and import from WLT;
- conflict/unsupported report.

Do not create an entirely separate profile editor application. Reuse the same modular settings pages and component controls defined in [UI architecture](ui-architecture.md).

## 9. Acceptance gates

- [ ] Profile schema and versioning are defined.
- [ ] Profiles only call public engine APIs/adapters.
- [ ] All changes pass capability, permission and restoration checks.
- [ ] Partial profile application is explained and recoverable.
- [ ] User overrides survive automatic profile conditions.
- [ ] Optimization does not kill apps or blindly disable services.
- [ ] Templates export only selected portable profile data.
- [ ] Per-display audio/device mappings are not applied to ambiguous hardware.
- [ ] Performance changes are measured on low-end systems.

## 10. Open decisions

- Profile schema, IDs and storage format.
- Manual-only first release versus automatic conditions.
- Exact condition language and conflict resolution.
- Whether profile schedules ship in the first release.
- Atomicity model for cross-engine activation.
- Whether audio preferences and clipboard options are included in the first Template schema.

# WinLook Manager UI Architecture

> Status: visual direction and information-architecture proposal. WinToys is a design reference for clarity and organization, not a source to copy or a required technology stack.

## 1. Product direction

The Manager should feel like a modern, clean Windows utility: practical, calm, readable and organized. The chosen reference is the WinToys style of a clear left navigation, a focused page per area, compact system information and grouped controls.

The goal is not a highly customized shell or a decorative dashboard. Prefer familiar native controls, predictable Windows interactions and a small visual vocabulary. The visual shell should make a large feature set understandable without showing every option at once.

Reference observations:
- WinToys screenshots show a persistent left-side navigation with distinct pages such as Home, Apps, Services, Performance, Health and Tweaks. The content area changes with the selected category. See the [WinToys screenshot gallery](https://freesoft-100.com/screen/ss-wintoys.html) and [overview/screenshots](https://www.computerbild.de/artikel/Tipps-Windows-Wintoys-Kostenloses-Tuning-Tool-fuer-Windows-11-und-10-045b07f-35673881.html).
- Microsoft Windows app guidance describes navigation, command and content patterns, spacing, type hierarchy and control grouping: [Windows app design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/).

## 2. Main window structure

Proposed layout:

    +---------------------------------------------------------------+
    | WinLook                         Search             Profile    |
    +-------------------+-------------------------------------------+
    | Home              | Page title                 Page actions   |
    | Appearance        | Short page summary                         |
    | Audio & Media     |                                           |
    | File Explorer     |                                           |
    | Icons             |                                           |
    | WLDE              | Page-specific groups/cards/settings       |
    | WLWM              |                                           |
    | Displays          |                                           |
    | Notifications     |                                           |
    | Templates         |                                           |
    | Extensions        |                                           |
    | Compatibility     |                                           |
    | Diagnostics       |                                           |
    | Settings          |                                           |
    +-------------------+-------------------------------------------+

The left navigation stays consistent. The content pane is responsible for one task area. Search should find settings/features without forcing every setting to appear on the home page.

Exact navigation labels and ordering remain open. WLDE, WLWM, WLDM and WLOSDE should be understandable to ordinary users; show the acronym with a plain-language label and explanation rather than assuming the user knows internal names.

## 3. Page patterns

### Home

A small overview of WinLook health, detected Windows version/capabilities, active profile/theme, current desktop mode and recent actions. Avoid a wall of live graphs. Link users to the subsystem page for deeper details.

### Engine pages

Each engine has a landing page with a short purpose statement, current status/capability, key settings and links to advanced configuration. Related controls are grouped into sections with a short explanation.

### Configuration pages

Use consistent setting rows or compact cards:
- setting name and one-sentence explanation;
- current value/control;
- supported/partial/experimental state when relevant;
- reset/default action where safe;
- contextual help and warning only when necessary.

Advanced or rarely used options should be behind progressive disclosure. Do not hide dangerous consequences or permission prompts behind vague labels.

### Lists and catalogues

Templates, wallpapers, widgets, extensions and detected applications should use searchable/filterable lists with clear empty, loading, unsupported and error states. Use context menus for secondary actions and avoid crowding every row with multiple buttons.

### Diagnostics and compatibility

Show exact OS/build/runtime and capability results, plus reason codes and test/support status. Separate documented capability from WinLook runtime verification. Make copy/export diagnostics user-controlled and redact sensitive data.

## 4. Reusable UI components

Design shared components independently from pages:

- navigation item and page header;
- setting row, switch, dropdown, slider and segmented choice;
- info/status card and capability badge;
- list row, filter, search box and context menu;
- confirmation dialog and reversible-change prompt;
- error, warning, unsupported and empty states;
- preview panel and before/after comparison;
- progress/status presentation;
- help text, tooltip and advanced-options disclosure.

Components must expose keyboard and accessibility semantics. A status color must never be the only indication of success, warning, unsupported or error.

## 5. Split UI source structure

Both C and Rust should follow the same information architecture while using controls appropriate to their OS and UI toolkit.

Proposed structure:

    UI/
      main application entry
      shell/
        main window and navigation
        route/page selection
        shared page layout
      pages/
        HOME/
        APPEARANCE/
        ICONS/
        AUDIO_MEDIA/
        FILE_EXPLORER/
        WLDE/
        WLWM/
        WLDM/
        WLOSDE/
        TEMPLATES/
        EXTENSIONS/
        COMPATIBILITY/
        DIAGNOSTICS/
        SETTINGS/
      components/
        settings/
        lists/
        dialogs/
        status/
        previews/
      accessibility/
      resources/

Keep UI components in the UI tree; keep engine logic in engine directories. UI calls public engine APIs through a narrow adapter/view-model layer. Engine code must not depend on concrete Manager pages or UI widgets.

A page such as CONFIGURATION should be editable as an isolated page module, with reusable rows/dialogs shared through components. Do not place all navigation, engine operations, page logic and controls into one source file.

## 6. C and Rust UI parity

Parity means the same concepts, labels, categories and expected user outcomes. It does not mean both binaries must draw identical pixels or use the same toolkit.

- Modern Rust UI can use a native UI framework selected after performance, accessibility, toolchain and deployment research.
- Legacy C UI must use a toolkit/API combination actually available on the target Windows release and compiler.
- Do not require WinUI 3 or Windows App SDK for Win95-era targets.
- Keep windowing, navigation and page state lightweight. Avoid requiring an embedded browser engine merely to render settings.
- Platform-specific controls may differ, but settings identifiers and user-facing meanings should remain consistent.

## 7. Interaction and visual rules

- Keep the primary task visible; use clear titles and short summaries.
- Group related settings by user outcome, not by internal source-file ownership.
- Use consistent spacing, type scale, alignment and control sizing.
- Prefer native title bars/window behaviour unless a custom frame has a compelling tested benefit.
- Use icons as supporting cues, not as the only labels.
- Provide explicit apply/save behaviour where changes are not immediate.
- Distinguish reversible, pending-restart and unsupported changes.
- Explain any operation that changes system-wide state before confirmation.
- Support keyboard navigation, high contrast, text scaling, screen readers and reduced motion.
- Respect small screens and legacy display resolutions; no page may require a modern high-resolution monitor to be operable.

## 8. UI architecture decisions still open

- Exact UI framework/toolkit for modern Rust.
- Exact C UI toolkit per supported legacy OS family.
- Whether UI is a separate process in every edition or shares a constrained legacy process.
- Navigation labels and grouping for the first release.
- Theme support and whether WinLook follows system light/dark mode.
- Localization framework and initial languages.
- How UI pages consume capabilities and asynchronous operation status.
- Layout adaptation for 640x480-era displays and different DPI modes.

## 9. Acceptance criteria

- [ ] Every page belongs to one clear navigation destination.
- [ ] Audio & Media separates audio-session volume from media-session transport.
- [ ] File Explorer settings distinguish documented Shell integration from experimental per-build styling and expose independent recovery.
- [ ] Settings are split into independent page modules and reusable components.
- [ ] Pages use public APIs, not engine-private implementation modules.
- [ ] A broken page cannot crash or corrupt unrelated engine logic.
- [ ] Unsupported and partial capabilities are explained in the UI.
- [ ] Keyboard, contrast, scaling and reduced-motion requirements are addressed.
- [ ] The interface remains responsive on low-resource systems and legacy resolutions.
- [ ] UI framework choices are supported by measured prototypes, not preference alone.

# WinLook Research Catalogue

This directory stores substantial Windows-integration research so future agents can reuse existing discovery instead of repeating broad searches.

## Primary catalogue

- [research-v9.md](research-v9.md) — original WinLook Architectural Blueprint & Deep Systems Research (Edition V).
- [research-v9-review.md](research-v9-review.md) — first-pass architecture/example review and official API references.
- [Root agent instructions](../../AGENTS.md) — required research preflight and example-evaluation guidance.

The original v9 document is preserved as supplied. It contains **250 tactics across 25 categories (A–Y)**, plus architecture diagrams, compatibility notes, Rust/C examples, animation mathematics, XML presets, IPC, watchdog/recovery sketches, and a technical source ledger. Topics span native windows and DWM, UxTheme, Shell/Explorer, desktop and wallpaper surfaces, animation, session/power, DPI/accessibility, taskbars, tiling, AppBars, global input, OSDs, theming, virtual desktops, launchers, hardware monitoring, and compositor effects.

## How agents should use it

1. **Look here first.** For a Windows integration task, search the catalogue for the subsystem, API, object, or tactic before starting broad external research.
2. **Reuse existing work.** Treat ideas, code sketches, candidate APIs, and upstream project references as leads. Avoid repeating searches that merely rediscover information already collected here.
3. **Check the current project contract.** Read the corresponding architecture and per-Windows compatibility documents. Those current project documents—not this catalogue—define accepted direction and constraints.
4. **Classify each tactic:** documented; experimental/undocumented; build-specific; needs correction; or out of scope.
5. **Verify narrowly.** Use existing source links first. Browse only to resolve a concrete gap, version change, conflict, or high-risk assumption. Add important verified sources to `plan/references/SOURCES.md`.
6. **Do not silently promote research to requirements.** A tactic is a candidate, not an accepted feature or proof of implementation.

## How to review examples

Examples in v9 often teach a mechanism or illustrate an architecture; they do not all claim to be complete applications.

- A conceptual example can be valid without being compilable.
- An illustrative snippet can be valid without containing all lifecycle/error-handling code.
- A compile-ready sample needs coherent types, imports, signatures, and setup.
- A production implementation additionally needs complete safety, compatibility, recovery, tests, and verification.

Flag a genuine flaw when the central API cannot do what the text says, the process boundary is wrong, a protocol does not match, a value is incorrect, or the logic contradicts the stated behaviour. Call placeholders and no-op stubs what they are. Do not call an example invalid solely because it is not production-ready.

## First-pass caveats already identified in v9

These notes are not an exhaustive audit of all 250 tactics:

- **Rust dispatcher:** [Section 8](research-v9.md#L2583) is a useful architectural sketch. `GetForegroundWindow()` demonstrates a foreground target, not generic namespace resolution. A fallback build value of `22000` must not be treated as detected OS information. UxTheme/GDI handlers that return success without work are placeholders.
- **Bézier example:** [Section 10.2](research-v9.md#L3016) computes parametric Y for a supplied parameter. It is not by itself a time-to-eased-value solver because it does not invert X; the later root-finding example addresses that concept.
- **Journal/recovery:** [Section 10.5](research-v9.md#L3174) sketches shared-memory journaling but does not establish a lock-free algorithm or complete recovery. Synchronization, clean-run marker lifecycle, crash durability, property coverage, and compare-before-restore need a real design.
- **Watchdog lifetime:** [Section 20](research-v9.md#L4485) opens the named mapping only after the parent exits. If no process still holds a mapping handle/view, it can be destroyed when the parent exits. The watchdog should hold or receive the handle/view before waiting, then test the lifecycle.
- **IPC mismatch:** [Section 17](research-v9.md#L3713) defines a 16-byte header, while the [C89 client](research-v9.md#L4389) uses a 12-byte three-field header with a different magic/layout.
- **DirectComposition target:** [Tactic 241](research-v9.md#L2464) must not imply that WinLook can bind a DirectComposition target directly to another process's HWND. Microsoft documents that the HWND must belong to the process creating the composition device; a WinLook-owned overlay is the suitable pattern for another application's window.
- **Synthetic input:** [Tactic 174](research-v9.md#L1746) should describe `SendInput` as synthetic input, not physical hardware input. Scan-code flags do not turn it into physical input or bypass UIPI; the shown key-down also needs a matching key-up.

## Review scope

This is a targeted first pass over architecture and selected examples. It does **not** certify all 250 tactics, every version claim, third-party reference, or code sample. Check the specific tactic being relied on and preserve evidence/status in the relevant architecture or source-ledger document.

# Windows Shell Integration and WLHome

> **Status:** Proposed product direction; implementation and runtime compatibility are not verified.
> **Scope:** Windows Start experience, taskbar-adjacent integration, selected Explorer extensions, and future WinLook Windows Integration settings.
> **Safety invariant:** WinLook may replace selected user-facing shell experiences, but it must not redesign or interfere with security-critical or recovery-critical Windows interfaces.

## 1. Product goal

**WLHome is intended to replace the user-facing Windows Start experience on compatible Windows builds.** It is not intended to replace `explorer.exe` as the Windows shell process, replace the desktop/window manager, or reimplement Windows system services.

The desired normal interaction is:

- Clicking the taskbar's Start button opens WLHome.
- Pressing the Windows key opens WLHome.
- WLHome Search can also open from a configurable launcher shortcut (initial candidate: `Alt+Space`).
- The native Start experience remains recoverable when an experimental adapter is disabled, unavailable, or quarantined.

Do not assume that the taskbar click and Windows-key paths are identical. They may converge on the same Start experience host on a particular Windows build. **The preferred design is to integrate at the Start-experience activation/hosting layer so Windows' own triggers naturally open WLHome where that is technically possible.** Do not begin by globally suppressing the Windows key. Prove the click and keyboard paths separately on every supported build.

## 2. What is and is not a public Windows contract

### Documented mechanisms

Use these where their documented contracts fit the requested behavior:

- **Shell Launcher:** replaces the configured user's shell at sign-in and monitors the configured shell application. Shell Launcher v2 supports Win32/UWP shell applications and is documented for supported Enterprise, Education and IoT Enterprise editions. It replaces the shell, not just Start; it is therefore a separate optional full-shell product mode, not a partial Start-menu redirection API.
- **Start layout and policy configuration:** supports documented pin/layout and policy configuration. It does not provide a general public callback or API to route the existing Start button to an arbitrary Win32 application.
- **RegisterHotKey:** use for ordinary configurable shortcuts such as `Alt+Space`. The API defines `MOD_WIN`, but Windows-key combinations are reserved for the operating system; do not treat this as a supported Windows-key takeover mechanism.
- **Raw Input:** may be used for passive keyboard observation where appropriate. Observation is not event suppression or Start replacement.
- **ITaskbarList / ITaskbarList3:** provides documented taskbar integration for an application's own button, progress, overlay and thumbnail behavior. It is not a taskbar replacement or Start-button routing interface.
- **Shell extension interfaces:** use documented interfaces for supported Explorer commands, namespace extensions, thumbnails, icons, property pages and related functionality. Their registration and visible behavior vary by Windows release.
- **Notifications and app activation APIs:** use documented OS mechanisms to integrate WinLook-owned notifications and launch applications; do not claim they replace Notification Center or other OS-owned surfaces.

### Undocumented or reverse-engineered mechanisms

There is no documented, general-purpose public API that says: "when the user clicks this Windows taskbar Start button, launch this arbitrary application instead."

Open-source projects show that Start behavior can be influenced through private Shell interfaces, Explorer message paths, hooks, and version-specific in-process integrations. These are **implementation observations**, not supported Windows contracts. They can break after cumulative updates, feature updates, package changes, or architecture changes.

Do not turn an observed private interface, undocumented COM IID, vtable slot, internal message, XAML class name, process-memory layout, or byte pattern into a universal WinLook API.

## 3. Start button and Windows key are separate investigation paths

### Taskbar Start-button click

Public source code gives useful leads:

- Open-Shell locates the Explorer `Progman` window and taskbar, installs a thread-specific `WH_GETMESSAGE` hook in an Explorer thread, relays a private registered message, and re-establishes its hook after the registered `TaskbarCreated` broadcast.
- ExplorerPatcher contains code that observes `WM_SYSCOMMAND` / `SC_TASKLIST` in a message path and calls private Immersive Shell launcher interfaces to show/dismiss Start. Other code paths inspect and integrate with `StartMenuExperienceHost.exe`.

These show that Explorer/taskbar message routing and the Start experience host are productive areas for source study. They do **not** prove that all Windows 10/11 builds use the same click path or that a third party can safely own it through a stable contract.

### Windows-key activation

The Windows key is an OS-reserved input path. A replacement of the Start experience host/activation target may cause the built-in Windows-key gesture to open WLHome without WinLook stealing the key globally, if that build routes the key to the same replaceable Start experience.

That convergence is a **per-build hypothesis** until observed. Trace key-down, key-up, shell activation, host creation, and visible Start presentation separately from the taskbar click. Also test Win+E, Win+R, Win+D, Win+X and Win+Tab so a Start integration does not break other Windows shortcuts.

If host-level replacement cannot cover the Windows key safely, keep the native Windows-key behavior and provide a supported configurable shortcut for WLHome. A low-level keyboard hook is not an equivalent stable substitute.

### Keyboard API limits

- `RegisterHotKey` is the preferred supported mechanism for ordinary shortcuts.
- Windows-key combinations are reserved for the OS even though `MOD_WIN` exists in the API.
- `WH_KEYBOARD_LL` can observe and suppress keyboard events, but its callback must remain extremely fast; Windows can silently remove a hook that exceeds the timeout.
- Right Alt may act as AltGr on many keyboard layouts. Do not reserve it globally by default.
- Never block unrelated key combinations or silently swallow the Windows key when WLHome is not healthy.

## 4. Windows-generation and build matrix

Use build/capability detection, not only a `Windows 10` / `Windows 11` boolean.

| Target | Research baseline | Key considerations |
|---|---|---|
| Windows 10, older generations | Include pre-StartMenuExperienceHost transition builds only when legacy support is genuinely required | Start UI hosting changed during Windows 10. Verify the exact process/package and trigger path on the specific build; do not assume current Windows 10 internals exist. |
| Windows 10 1903–22H2 | Representative modern Windows 10 baseline; include 1809 as a transition boundary in research | Explorer remains taskbar host; inspect Start UI host/package identity and build/UBR. A result from 22H2 does not prove earlier releases. |
| Windows 11 21H2–23H2 | Separate initial Windows 11 taskbar/Start implementation family | Do not reuse Windows 10 hook assumptions or infer private interface stability. |
| Windows 11 24H2 and newer | Treat each feature/build family and cumulative update as a separate compatibility candidate | Taskbar and Start internals have changed; revalidate package, module, class and message assumptions. |
| Future Windows builds / Insider | Unknown until inspected | Experimental adapters must disable themselves on unknown or mismatched builds. |

At runtime, record the exact edition, display version, build, UBR, architecture, Start host package/version, relevant process identity and the adapter's own compatibility signature. A marketing version alone is insufficient.

Shell Launcher availability is edition-dependent. Do not infer that it is available on Home/Pro merely because the application is a Win32 executable. Check the current Microsoft requirements before enabling Full Shell mode.

## 5. Open-source implementation references — mandatory agent research path

When Microsoft does not document the behavior needed for a Windows shell integration, future agents **must inspect maintained, reputable open-source implementations before inventing a technique**. Treat these repositories as technical case studies and reverse-engineering evidence, not as Microsoft documentation or drop-in dependencies.

### ExplorerPatcher

- Repository: https://github.com/valinet/ExplorerPatcher
- Inspected commit: `cc555ca80678c93f185e555fa4603e16610e3280` (2026-10-29).
- Relevant source:
  - [StartMenu.c](https://github.com/valinet/ExplorerPatcher/blob/cc555ca80678c93f185e555fa4603e16610e3280/ExplorerPatcher/StartMenu.c)
  - [StartExperience.cpp](https://github.com/valinet/ExplorerPatcher/blob/cc555ca80678c93f185e555fa4603e16610e3280/ExplorerPatcher/StartExperience.cpp)
  - [StartExperience.h](https://github.com/valinet/ExplorerPatcher/blob/cc555ca80678c93f185e555fa4603e16610e3280/ExplorerPatcher/StartExperience.h)
  - [Taskbar10.cpp](https://github.com/valinet/ExplorerPatcher/blob/cc555ca80678c93f185e555fa4603e16610e3280/ExplorerPatcher/Taskbar10.cpp)
- Observed research value: private Immersive Shell launcher interfaces, `SC_TASKLIST` message handling, Start host integration, taskbar construction changes, build/UBR-specific branches.
- Risk lesson: source includes undocumented interfaces and process-integration techniques. It is evidence of feasibility, not a stable public contract.

### Open-Shell

- Repository: https://github.com/Open-Shell/Open-Shell-Menu
- Inspected commit: `729c3686d093da88ac70095aff18d09f4f1352cf` (2026-11-07).
- Relevant source: [Src/StartMenu/StartMenu.cpp](https://github.com/Open-Shell/Open-Shell-Menu/blob/729c3686d093da88ac70095aff18d09f4f1352cf/Src/StartMenu/StartMenu.cpp)
- Observed research value: Explorer/Progman/taskbar discovery, thread-specific `WH_GETMESSAGE` integration, registered messages, taskbar recreation handling, and a separate Start menu process/UI.
- Risk lesson: hooks into Explorer's message path are not a general public Start replacement API. Study lifecycle and re-hook handling as well as the initial hook.

### Windhawk and its public mod sources

- Mod repository: https://github.com/ramensoftware/windhawk-mods
- Inspected commit: `f6092423955c905d782c17b7c95fd7dca6342fa9` (2026-11-16).
- Relevant source: [Windows 11 Start Menu Styler](https://github.com/ramensoftware/windhawk-mods/blob/f6092423955c905d782c17b7c95fd7dca6342fa9/mods/windows-11-start-menu-styler.wh.cpp)
- Observed research value: practical examples of version-specific Start XAML/resource styling and how a community mod expresses selectors and compatibility settings.
- Risk lesson: styling internals is not the same as replacing the Start activation target. A mod's selector coverage does not establish compatibility with a future Windows build.

### Source-study rules

1. Prefer Microsoft Learn/API documentation and official OS support statements first.
2. When the public contract is incomplete, inspect at least one relevant upstream open-source implementation; for high-impact behavior such as Start activation, compare independent projects where possible.
3. Pin the exact repository commit, source file and relevant code region. Record the repository's license before reusing code; do not copy implementation code without license and security review.
4. Extract the observed technique and its preconditions. Label it `observed in source`, not `supported by Windows`.
5. Compare multiple projects and Windows build targets. Do not assume two projects use the same technique or target the same host.
6. Prefer passive tracing and public symbols over invasive modification. Use disposable VMs for dynamic experiments.
7. If no Windows VM, debugger, REA-capable tool or runtime access is available, say so explicitly. Continue with source-level research and mark all runtime conclusions unverified; never claim to have reverse-engineered or tested the user's machine.
8. Re-check source history/issues/release notes for compatibility regressions before reusing an old implementation.
9. Separate feasibility, correctness, security, licensing and maintainability assessments.

## 6. WinLook architecture

### Windows Integration manager area

The Manager should eventually expose a **Windows Integration** area listing capabilities, compatibility, risk, permission requirements, enablement and rollback state. It is not a universal "replace Windows" switch.

Each provider should declare at least:

- Stable provider/capability ID and owner subsystem.
- Target surface (for example Start experience, taskbar app affordances, Explorer context command).
- Integration class: documented API, documented policy/configuration, third-party extension, or reverse-engineered adapter.
- Supported OS editions, versions, builds/UBRs, architectures and relevant package versions.
- Process/session target and whether any code runs in another process.
- Required privilege and explicit consent requirements.
- Health/heartbeat mechanism, timeout behavior and independent disable switch.
- Restoration/rollback method and stock Windows fallback.
- Evidence links and confidence state: verified, partially verified, hypothesis, unsupported.

### WLHome and related responsibilities

- **WLHome:** own the custom Start/Home presentation, layout, navigation and pinned items.
- **WLHome Search:** separate search experience and provider aggregation; initially expose a supported configurable shortcut.
- **WLHome Application Indexer:** maintain an independent application catalog using documented discovery/activation APIs where possible.
- **Shell Integration Adapter:** detect the Windows build and choose only a matching Start integration backend. Start-button and Windows-key paths are separately probed and separately accepted.
- **BGW:** coordinate capability checks, process lifecycle, health, policy, IPC and rollback state. Keep the Home UI on demand; do not add a permanently running backend process without a concrete need.
- **WLWM:** remains responsible for window management; WLHome does not become a window manager.
- **wl_ui:** owns WLHome's in-app controls/rendering behavior.
- **wl_animate:** supplies the broader advanced animation capabilities intended for WinLook subsystems; do not conflate it with `wl_ui`.

Prefer replacing/intercepting the Start experience endpoint that Windows itself activates over stealing the Windows key globally. The exact UI host/activation strategy is unresolved until build-specific experiments establish a reliable route.

## 7. Safety, permissions and hard exclusions

The following surfaces are permanently outside WinLook's customization boundary:

- Ctrl+Alt+Delete / Secure Attention Sequence.
- Winlogon, sign-in credential entry, authentication and account recovery UI.
- UAC secure prompts and the secure desktop.
- Boot Manager, Windows Recovery Environment and BitLocker recovery.
- Bugcheck/crash handling and other recovery-critical interfaces.
- Kernel, hypervisor, security provider and protected-process interfaces whose modification could compromise system integrity.

Additional rules:

- Never patch, replace, binary-modify or depend on a modified `explorer.exe` on disk.
- Do not patch Windows system binaries, replace Winlogon, install a kernel hook/driver for Start integration, or change the configured Windows shell as a default consumer feature.
- Do not kill or block `StartMenuExperienceHost.exe` as the normal replacement strategy.
- Do not make an experimental Start adapter a dependency for BGW startup, Explorer desktop functionality, or ordinary keyboard input.
- Do not silently swallow the Windows key or leave the user without the stock Start experience if WLHome fails.
- In-process Explorer/Shell extensions can crash or hang their host. Keep callbacks short, bounded, deterministic and free of network or expensive indexing work; broker long-running tasks out of process.
- Require explicit user opt-in for experimental per-build adapters. Check health before activation and automatically disable/quarantine after a failed probe, incompatible signature, host crash or OS update.
- Keep a separate recovery/disable path that does not require WLHome or the Manager UI to be running. Test recovery from another account or Safe Mode before any non-VM rollout.
- Keep the stock Start experience available as the fallback wherever the chosen integration makes that technically possible.

## 8. Safe research and acceptance sequence

1. **Read-only baseline:** in disposable Windows 10/11 VMs, record edition, build, UBR, architecture, package versions, process identities, loaded modules, window ownership and shell host lifecycle.
2. **Observe taskbar click:** capture the message/activation path and process/host transitions without injecting, patching or changing registry settings.
3. **Observe Windows key:** capture key-down/up, OS-reserved shortcut handling, Start activation and host transitions. Separately test Win+E/R/D/X/Tab.
4. **Compare paths:** determine whether taskbar click and Windows key converge on one replaceable Start host for each exact build.
5. **Prototype ordinary shortcut:** verify `RegisterHotKey` with `Alt+Space` independently of shell integration.
6. **Study upstream source:** map the relevant ExplorerPatcher/Open-Shell/Windhawk source techniques to the observed build; identify undocumented dependencies and failure modes.
7. **Only then prototype a per-build Start adapter:** in a disposable VM snapshot, use the least invasive technique supported by evidence. Keep the stock path and a recovery account.
8. **Fault/restart testing:** restart Explorer and Start host, sign out/in, change displays, test multi-monitor, test adapter crash/config corruption, update the OS build, and verify safe disable/rollback.
9. **Acceptance:** both click and Windows-key triggers open WLHome when the adapter claims support; unrelated Win-key shortcuts remain intact; stock Start returns immediately when disabled; a failed/unknown adapter fails open; protected surfaces remain untouched.

No live Windows behavior is considered verified solely because a project source file contains a hook or private interface. Runtime validation must name the exact Windows edition/build/UBR and test environment.

## 9. Primary references

### Microsoft

- [Shell Launcher overview](https://learn.microsoft.com/en-us/windows/configuration/shell-launcher/) — full-shell replacement and supported editions.
- [WESL_UserSetting.SetCustomShell](https://learn.microsoft.com/en-us/windows/configuration/shell-launcher/wesl-usersettingsetcustomshell) — configuring the shell application and exit behavior.
- [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey) — global shortcut registration and Windows-key reservation.
- [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc) — hook timeout and removal constraints.
- [Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/raw-input) — lower-level input observation.
- [ITaskbarList3](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-itaskbarlist3) — documented taskbar application integration.
- [File Explorer context-menu integration](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer) — supported Explorer command extension routes.
- [Customize Start menu layout on Windows 11](https://learn.microsoft.com/en-us/windows/configuration/configure-start-layout) — layout/pinning configuration, not arbitrary Start activation redirection.

### Research metadata

- Research/access date for this design note: 2026-11-16.
- ExplorerPatcher source snapshot: commit `cc555ca80678c93f185e555fa4603e16610e3280`, committed 2026-10-29.
- Open-Shell source snapshot: commit `729c3686d093da88ac70095aff18d09f4f1352cf`, committed 2026-11-07.
- Windhawk mods source snapshot: commit `f6092423955c905d782c17b7c95fd7dca6342fa9`, committed 2026-11-16.
- Claims about source code are verified against the cited source snapshots. Claims about runtime Start-button/Windows-key convergence on WinLook-supported builds remain unverified hypotheses pending VM experiments.

## 10. Open decisions

- Which exact Windows 10/11 builds will receive WLHome Start replacement support?
- Which host/activation boundary can be integrated without global Windows-key interception?
- Can one adapter cover both taskbar click and Windows-key triggers for each build?
- What is the least invasive approach for each build family?
- How should Windows Integration profiles and per-capability consent appear in the Manager?
- What minimum recovery path is required for each experimental adapter?
- Whether Full Shell mode through Shell Launcher is worth supporting separately from the consumer WLHome replacement.

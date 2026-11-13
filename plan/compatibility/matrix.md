# Windows Compatibility Matrix

Planning matrix only; not a release-support promise.

| OS | Family | Graphics/composition | Event/accessibility baseline | Workspace strategy | Animation backend | Classification |
|---|---|---|---|---|---|---|
| Windows 95 | Win9x | USER/GDI; no DWM | No built-in MSAA; 1.3 RDK route | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| Windows 98/SE | Win9x | USER/GDI; no DWM | MSAA 1.1 initially; 2.0 RDK route | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| Windows Me | Win9x | USER/GDI; no DWM | MSAA 1.3 built in | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| NT 4.0 | NT | USER/GDI; no DWM | MSAA 1.3 from SP6; RDK earlier | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| Windows 2000 | NT 5.0 | USER/GDI; no DWM | SetWinEventHook and AnimateWindow documented | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| Windows XP | NT 5.1/5.2 | USER/GDI; no DWM | MSAA 2.0 built in | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Research only |
| Windows Vista | NT 6.0 | DWM introduced; composition conditional | Session 0 isolation begins | WLWM logical | Not in initial library baseline; future C/OS adapter validation | Future compatibility work |
| Windows 7 | NT 6.1 | DWM/Aero; composition conditional | MSAA 3.0 built in | WLWM logical | Experimental `wl_ui::windows7` classic Win32 renderer; Rust 1.77 or custom Tier 3 target | Research/VM testing pending |
| Windows 8 | NT 6.2 | DWM always on | Modern Store app/window separation | WLWM logical | `wl_ui::windows8` aliases the same baseline Win32 renderer | Research/VM testing pending |
| Windows 8.1 | NT 6.3 | DWM always on; DirectComposition DesktopDevice documented | Dynamic code policy documented | WLWM logical/native query where useful | `wl_ui::windows81` aliases the same baseline Win32 renderer | Research/VM testing pending |
| Windows 10 | NT 10.0 | DWM always on | IVirtualDesktopManager documented | Native window/desktop queries+moves; remaining ops researched | `wl_ui::windows10`: Win32/User32; no Windows App SDK dependency | Build profile implemented; runtime testing pending |
| Windows 11 | NT 10.0 build family | DWM always on; modern Shell | Mitigation/build variability | Public IVirtualDesktopManager scope; per-build testing | `wl_ui::windows11`: WinUI 3 / Windows Reactor | Build profile implemented; runtime testing pending |

## Important interpretation
- APIs with Windows 2000 minimum are not assumed on 95/98/ME.
- RDK availability does not mean every modern accessibility event API is built into the base OS.
- DWM presence is separate from whether composition is enabled on Vista/7.
- IVirtualDesktopManager is not a complete public interface for all WLWM workspace operations.
- Injection compatibility depends on target process and runtime state; OS version alone is insufficient.

## Sources
- [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms)
- [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
- [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow)
- [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled)
- [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/desktop-window-manager-is-always-on)
- [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice)
- [IVirtualDesktopManager](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager)
- [Dynamic code policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy)
- [Session 0 isolation](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista)
- [Open Watcom linker guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html)

## Capability interpretation by subsystem

The existing matrix gives the broad platform baseline. This table clarifies the intended design route. These are architecture expectations, not proof that a WinLook adapter works.

| OS group | Core implementation route | Desktop/graphics baseline | Window/workspace model | High-risk areas |
|---|---|---|---|---|
| Windows 95 / 98 / Me | Separate legacy C builds plus the C89 `wl_Lui` window-shell foundation; test each release independently | USER/GDI; no DWM | Logical WLWM workspaces; OS-specific observation adapter | Process observation/injection, desktop icon presentation, display, optional accessibility RDK |
| NT 4.0 | Separate legacy C target; do not reuse Win9x assumptions | USER/GDI; no DWM | Logical workspaces; NT-specific process/session adapter | Service/runtime compatibility, event source, injection and Shell |
| Windows 2000 / XP | Legacy C target unless another runtime is separately verified | USER/GDI; no DWM | Logical workspaces; documented APIs only where minimums match | Runtime/CRT, process bitness, Shell and display variation |
| Vista | C implementation and `wl_Lui`; Rust modern toolchain route is not the baseline | DWM exists; composition/driver/session state matters | Logical workspaces unless native operation is proven | Session 0 split, DWM availability, driver and power |
| Windows 8 / 8.1 | C implementation is the planned default; Rust/MSVC is not the current supported target baseline | DWM always on; DirectComposition interface is version-sensitive | Logical workspace baseline; native interface use is feature-scoped | Composition minimums, app model, mitigations |
| Windows 10 / 11 | Rust/MSVC target candidate (documented Windows 10+ client floor); WinLook still requires exact-build tests | DWM always on; modern Shell/display capabilities | Native public operations only for documented scope; research remaining gaps | Build changes, process mitigations, protected targets, graphics and display |

## Feature-level planning matrix

| Feature | Win9x / NT4 | 2000 / XP | Vista / 7 | 8 / 8.1 | 10 / 11 |
|---|---|---|---|---|---|
| Core Manager/Agent | C/toolchain research | C/toolchain research | C/toolchain baseline; current Rust/MSVC target does not cover this OS | C/toolchain baseline; current Rust/MSVC target does not cover this OS | Rust/MSVC primary target candidate; verify WinLook runtime |
| Native appearance | USER/GDI/Shell adapter per OS | USER/GDI/Shell adapter per OS | Shell/theme/DWM as available | Shell/DWM per release | Shell/DWM per build |
| Wallpaper | Static GDI-first candidate | Static GDI-first candidate | DWM coexistence; driver fallback | DWM coexistence | Modern renderer candidate |
| Widgets | Restricted low-resource subset only after isolation | Restricted legacy subset | Isolated runtime candidate | Isolated runtime candidate | Isolated runtime candidate |
| Desktop Groups | Disabled until native icon recovery proven | Disabled until recovery proven | Disabled until recovery proven | Disabled until recovery proven | Disabled until recovery proven |
| Animation | Outside initial Windows 10 baseline; future C compatibility work | Outside initial Windows 10 baseline; future C compatibility work | Outside initial Windows 10 baseline; future C compatibility work | Outside initial Windows 10 baseline; future C compatibility work | wl_animate (Rust) + wl_Lanimate (C) core; renderer adapters not included |
| WLWM layouts | Legacy observation and USER window APIs | Legacy observation and USER window APIs | Modern observation candidate | Modern observation candidate | Modern observation candidate |
| Native workspaces | Logical unless public API proves otherwise | Logical unless public API proves otherwise | Logical baseline | Logical baseline | Public API only for documented operations; remaining WLWM logic researched |
| WLWM injection | Research only; no default injection | Research only; no default injection | Target-specific review | Target-specific review | Target-specific review and mitigations |
| WLOSDE | Ordinary WinLook-owned windows | Ordinary WinLook-owned windows | Ordinary WinLook-owned surfaces | Ordinary WinLook-owned surfaces | Ordinary WinLook-owned surfaces |
| WLDM | Basic read-only geometry first | Per-OS display adapter | Display API/driver dependent | Display API/driver dependent | Display API/driver dependent |
| WLT | Bounded parser and supported fields only | Legacy-safe parser/subset | Full manifest candidate | Full manifest candidate | Full manifest candidate |
| Store/catalogue | Offline/local import first | Offline/local import first | Optional catalogue client | Optional catalogue client | Optional catalogue client |

## Audio & Media and File Explorer feature baselines

This table separates documented API baselines from WinLook implementation/support status. It is not a release-support promise.

| Windows generation | Audio mixing | Now Playing / media controls | Explorer customization |
|---|---|---|---|
| 95 / 98 / Me / NT4 / 2000 / XP | No Vista Core Audio session model; legacy device/mixer controls only where the exact OS/driver provides them; no generic per-process guarantee | No GSMTC; WinLook UI plus explicit supported-player adapters | Classic Shell handlers researched/tested per OS; no modern command-menu assumptions |
| Vista | Core Audio endpoint and known-session controls; no generic `IAudioSessionManager2` live per-app enumeration | No GSMTC; explicit player adapters | Legacy Shell handlers; custom namespace `IShellView` from XP; `IExplorerCommand` API minimum is Vista |
| 7 | `IAudioSessionManager2` enumeration and notifications | No GSMTC; explicit player adapters | `IExplorerCommand` and classic Shell registration routes |
| 8 / 8.1 | Core Audio enumeration/notifications | No GSMTC; explicit player adapters | Shell handlers and namespace views; deep chrome styling per-build research |
| 10 before 1809 | Core Audio | No GSMTC; player adapters | Shell handlers plus build-specific presentation research |
| 10 version 1809+ | Core Audio | GSMTC media sessions, playback type, metadata, provider thumbnail, transport and timeline where exposed | Shell handlers plus build-specific presentation research |
| 11 | Core Audio | GSMTC as above | Modern context-menu registration has package-identity/manifest requirements; optional build-specific presentation adapter |

### Audio & Media interpretation

- Vista is not a no-session platform: `IAudioSessionManager` and `ISimpleAudioVolume` are documented from Vista. The later `IAudioSessionManager2` enumeration surface is documented from Windows 7.
- GSMTC starts with Windows 10 version 1809 (build 17763). Earlier systems use WinLook's own UI plus explicit supported-player adapters.
- A legacy interface does not imply a universal legacy per-process volume API. Global/device mixer controls and player-owned volume must be labelled separately.
- GSMTC media sessions and Core Audio sessions are distinct; correlation is best-effort.

### Explorer interpretation

- `IExplorerCommand` has a documented Windows Vista minimum. Windows 11 modern File Explorer context-menu integration additionally requires app identity and manifest registration; the API minimum and modern registration model are separate facts.
- `IShellView` is available from Windows XP for custom namespace folder views. It controls the extension's own namespace view, not every ordinary folder or all Explorer chrome.
- Deep navigation/address/search/tab/command-bar styling requires a separate tested build-specific adapter or remains partial/unsupported. Do not infer stable support from one build or one third-party mod.

Sources: [Rust MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html), [Rust GNU target support](https://doc.rust-lang.org/rustc/platform-support/windows-gnu.html), [IExplorerCommand](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iexplorercommand), [IAudioSessionManager](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager), [IAudioSessionManager2](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager2), [GSMTC session manager](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager), [Windows 11 File Explorer context menu](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer), [Shell namespace extensions](https://learn.microsoft.com/en-us/windows/win32/shell/nse-works), [IShellView](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellview).

## Reading the matrix

- Candidate is a research route, not a feature that currently works.
- Current official Rust/MSVC Windows client target requires Windows 10+. A Rust GNU target page that is a stub is not evidence of older-Windows support.
- The broader application architecture may use C for pre-Windows-10 work, but these first animation-library implementations target Windows 10. Any later C port/runtime for older Windows must be separately designed and verified.
- Legacy subset means a deliberately reduced feature set, not an imitation of unsupported modern APIs.
- Groups stay disabled until recovery of native icons is verified on the exact OS.
- WLWM injection is not a baseline requirement for layouts or rules.
- High-risk operations need exact target, OS, driver and recovery test records.

Each per-version page remains authoritative. Update it, this matrix and the [test strategy](test-strategy.md) together when evidence changes.


# Windows Compatibility Matrix

Planning matrix only; not a release-support promise.

| OS | Family | Graphics/composition | Event/accessibility baseline | Workspace strategy | Animation backend | Classification |
|---|---|---|---|---|---|---|
| Windows 95 | Win9x | USER/GDI; no DWM | No built-in MSAA; 1.3 RDK route | WLWM logical | wl_Lanimate/GDI | Research + legacy prototype |
| Windows 98/SE | Win9x | USER/GDI; no DWM | MSAA 1.1 initially; 2.0 RDK route | WLWM logical | wl_Lanimate/GDI | Research + legacy prototype |
| Windows Me | Win9x | USER/GDI; no DWM | MSAA 1.3 built in | WLWM logical | wl_Lanimate/GDI | Research + legacy prototype |
| NT 4.0 | NT | USER/GDI; no DWM | MSAA 1.3 from SP6; RDK earlier | WLWM logical | wl_Lanimate/GDI | Research + legacy prototype |
| Windows 2000 | NT 5.0 | USER/GDI; no DWM | SetWinEventHook and AnimateWindow documented | WLWM logical | wl_Lanimate | Candidate |
| Windows XP | NT 5.1/5.2 | USER/GDI; no DWM | MSAA 2.0 built in | WLWM logical | wl_Lanimate | Candidate |
| Windows Vista | NT 6.0 | DWM introduced; composition conditional | Session 0 isolation begins | WLWM logical | wl_animate + fallback | Candidate |
| Windows 7 | NT 6.1 | DWM/Aero; composition conditional | MSAA 3.0 built in | WLWM logical | wl_animate + fallback | Candidate |
| Windows 8 | NT 6.2 | DWM always on | Modern Store app/window separation | WLWM logical | wl_animate; DComp interface differs | Candidate |
| Windows 8.1 | NT 6.3 | DWM always on; DirectComposition DesktopDevice documented | Dynamic code policy documented | WLWM logical/native query where useful | wl_animate; DComp optional | Candidate |
| Windows 10 | NT 10.0 | DWM always on | IVirtualDesktopManager documented | Native window/desktop queries+moves; remaining ops researched | wl_animate | Candidate |
| Windows 11 | NT 10.0 build family | DWM always on; modern Shell | Mitigation/build variability | Public IVirtualDesktopManager scope; per-build testing | wl_animate | Candidate |

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
| Windows 95 / 98 / Me | Separate legacy C builds; test each release independently | USER/GDI; no DWM | Logical WLWM workspaces; OS-specific observation adapter | Process observation/injection, desktop icon presentation, display, optional accessibility RDK |
| NT 4.0 | Separate legacy C target; do not reuse Win9x assumptions | USER/GDI; no DWM | Logical workspaces; NT-specific process/session adapter | Service/runtime compatibility, event source, injection and Shell |
| Windows 2000 / XP | Legacy C target unless another runtime is separately verified | USER/GDI; no DWM | Logical workspaces; documented APIs only where minimums match | Runtime/CRT, process bitness, Shell and display variation |
| Vista / 7 | Modern Rust candidate only after toolchain proof | DWM exists; composition/driver/session state matters | Logical workspaces unless native operation is proven | Session 0 split, DWM availability, driver and power |
| Windows 8 / 8.1 | Modern Rust candidate | DWM always on; DirectComposition interface is version-sensitive | Logical workspace baseline; native interface use is feature-scoped | Composition minimums, app model, mitigations |
| Windows 10 / 11 | Modern Rust target candidate; record exact build/architecture | DWM always on; modern Shell/display capabilities | Native public operations only for documented scope; research remaining gaps | Build changes, process mitigations, protected targets, graphics and display |

## Feature-level planning matrix

| Feature | Win9x / NT4 | 2000 / XP | Vista / 7 | 8 / 8.1 | 10 / 11 |
|---|---|---|---|---|---|
| Core Manager/Agent | Legacy UI/runtime research | Legacy runtime research | Modern runtime candidate; verify | Modern runtime candidate; verify | Primary modern target candidate |
| Native appearance | USER/GDI/Shell adapter per OS | USER/GDI/Shell adapter per OS | Shell/theme/DWM as available | Shell/DWM per release | Shell/DWM per build |
| Wallpaper | Static GDI-first candidate | Static GDI-first candidate | DWM coexistence; driver fallback | DWM coexistence | Modern renderer candidate |
| Widgets | Restricted low-resource subset only after isolation | Restricted legacy subset | Isolated runtime candidate | Isolated runtime candidate | Isolated runtime candidate |
| Desktop Groups | Disabled until native icon recovery proven | Disabled until recovery proven | Disabled until recovery proven | Disabled until recovery proven | Disabled until recovery proven |
| Animation | wl_Lanimate + GDI/USER adapter | wl_Lanimate + OS adapter | wl_animate + capability-selected adapter | wl_animate; DComp version-specific | wl_animate + verified adapters |
| WLWM layouts | Legacy observation and USER window APIs | Legacy observation and USER window APIs | Modern observation candidate | Modern observation candidate | Modern observation candidate |
| Native workspaces | Logical unless public API proves otherwise | Logical unless public API proves otherwise | Logical baseline | Logical baseline | Public API only for documented operations; remaining WLWM logic researched |
| WLWM injection | Research only; no default injection | Research only; no default injection | Target-specific review | Target-specific review | Target-specific review and mitigations |
| WLOSDE | Ordinary WinLook-owned windows | Ordinary WinLook-owned windows | Ordinary WinLook-owned surfaces | Ordinary WinLook-owned surfaces | Ordinary WinLook-owned surfaces |
| WLDM | Basic read-only geometry first | Per-OS display adapter | Display API/driver dependent | Display API/driver dependent | Display API/driver dependent |
| WLT | Bounded parser and supported fields only | Legacy-safe parser/subset | Full manifest candidate | Full manifest candidate | Full manifest candidate |
| Store/catalogue | Offline/local import first | Offline/local import first | Optional catalogue client | Optional catalogue client | Optional catalogue client |

## Reading the matrix

- Candidate is a research route, not a feature that currently works.
- Modern runtime candidate requires exact Rust standard library/dependency support for the OS; it is not inferred from the language alone.
- Legacy subset means a deliberately reduced feature set, not an imitation of unsupported modern APIs.
- Groups stay disabled until recovery of native icons is verified on the exact OS.
- WLWM injection is not a baseline requirement for layouts or rules.
- High-risk operations need exact target, OS, driver and recovery test records.

Each per-version page remains authoritative. Update it, this matrix and the [test strategy](test-strategy.md) together when evidence changes.


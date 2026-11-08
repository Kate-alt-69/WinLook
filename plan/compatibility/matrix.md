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

# WinLook Animation Suite

## Two implementations

| Name | Language | Contract |
|---|---|---|
| wl_animate | Rust crate | Modern reusable animation engine and public developer API |
| wl_Lanimate | C library | Legacy animation engine/library |

wl_animate is meant for WinLook modules and third-party developers creating apps or integrations for WinLook. wl_Lanimate provides corresponding conceptual functionality for legacy Windows. They need not share source code or implementation details.

## Whole-OS suite scope
Window open/minimize/restore/close, move/resize/snap/focus, workspace changes, desktop icon/Group interactions, wallpaper/widget transitions, WLOSDE notification/OSD transitions, WinLook Manager controls and third-party UI.

## Shared conceptual features
Duration/delay, easing, interpolation, keyframes, timelines, sequences, parallel groups, pause/resume/cancel, interruption handling, progress/state callbacks, frame pacing, target FPS, reduced-motion, resource budgets and deterministic disposal. Exact API signatures remain open.

## Modern wl_animate
Candidate backends: Win32 window/message observation, DWM APIs (Vista+), D3D11, DirectComposition (selected interfaces have Windows 8.1+ minimums) and independent layered/overlay surfaces where appropriate. Capability detection must select backend at runtime.

Sources: [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled), [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice).

## Legacy wl_Lanimate
Optimize specifically for selected compiler/linker/runtime, CPU baseline, GDI/USER APIs and old hardware. Research:
- calling convention and C ABI;
- minimal imports/CRT assumptions;
- bounded allocation and no per-frame heap churn;
- GDI off-screen bitmaps/dirty rectangles;
- timer/message-loop scheduling;
- resource cleanup for DCs, bitmaps, regions and windows;
- fixed-point arithmetic only when measured beneficial;
- static/no-animation fallback.

Open Watcom documents Win95 and NT target variants: [Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html). That is toolchain evidence, not a tested support statement.

## API minimums
Microsoft documents AnimateWindow with Windows 2000 minimum client; it must not be assumed on Windows 95/98/ME. SetWinEventHook has a Windows 2000 minimum, with a separate Active Accessibility 1.3 RDK route documented for Windows 95 and NT4 SP6. Legacy adapters must detect actual installed APIs/runtime.

Sources: [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow), [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook).

## Third-party developer contract
Publish Rust API docs/examples and semver once stable. Define a C ABI for legacy consumers. Provide capability queries, bounded sequences, cancellation and errors. Third-party code must not be able to deadlock the BGW or Manager. WLT may include animation configuration/assets, not arbitrary native code disguised as an animation profile.

## Verification
Profile frame times, CPU/GPU usage, allocations, GDI handle leaks, cancellation, target destruction, display changes, reduced-motion, low power, concurrency, legacy VM/hardware and third-party ABI compatibility.

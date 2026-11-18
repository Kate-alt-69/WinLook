# research-v9 — First-Pass Review

## Overall assessment

**Research and architectural usefulness: 8/10.**

| Dimension | Review score |
|---|---:|
| Feature coverage and idea inventory | 9.5/10 |
| Architectural direction | 9/10 |
| Illustrative examples | 7/10 |
| Technical accuracy | 5.5/10 |
| Ready-to-implement specification | 4/10 |

These are qualitative scores, not measurements. The overall rating reflects breadth and reuse value—not a claim that every tactic is correct or supported.

## What makes the document valuable

The 250 tactics across categories A–Y expand WinLook's candidate integration map beyond taskbar, Shell, DWM, and animation. The original v9 source calls the proposed layer `wl_translation`; its current accepted name is Traton (`wl_traton`). The catalogue includes native window styling, UxTheme, taskbar and Shell notifications, desktop/wallpaper hosting, session/power state, DPI/accessibility, tiling, AppBars, global input, OSDs, theming, virtual desktops, launchers, hardware telemetry, and compositor effects. It also relates `wl_traton`, `wl_animate`, WLDE, C interoperability, XML presets, IPC, and recovery.

## Selected example review

This is not a production-readiness checklist; each example is judged against its stated purpose.

| Example | Assessment | Why |
|---|---|---|
| Historical `wl_translation` dispatcher ([v9 §8](research-v9.md#L2583); informs Traton) | **Valid architectural example; not production-ready** | Path parsing and subsystem dispatch communicate the architecture. Foreground-window lookup is a simplified target demonstration, not generic namespace resolution; build 22000 as a detection fallback is incorrect; UxTheme/GDI success stubs should be labelled. |
| Bézier helper ([v9 §10.2](research-v9.md#L3016)) | **Valid parametric-curve illustration; incomplete easing solver** | It computes Y for parameter `t`; a timing function must solve X for the input time. The later root-finding example addresses that missing concept. |
| Journal ([v9 §10.5](research-v9.md#L3174)) | **Useful architecture sketch; claims exceed implementation** | The sample does not establish a lock-free algorithm. It needs cross-process synchronization, clean-run marker lifecycle, durability policy, property coverage, and conflict-safe restoration. |
| Watchdog ([v9 §20](research-v9.md#L4485)) | **Useful architecture; lifecycle bug/risk** | It opens the named mapping only after the parent exits. The watchdog should open/receive and hold it before waiting. |
| IPC ([v9 §17](research-v9.md#L3713), [C client](research-v9.md#L4389)) | **Protocol mismatch** | The specification depicts a 16-byte header; the C example uses a 12-byte three-field struct and a different magic value. |
| DirectComposition glow ([v9 §Tactic 241](research-v9.md#L2464)) | **Valid goal; target ownership must be corrected** | `CreateTargetForHwnd` requires the HWND to belong to the process that created the composition device. Use a WinLook-owned overlay for a foreign window's visual frame. |
| Scan-code input ([v9 §Tactic 174](research-v9.md#L1746)) | **Capability is valid; description is incorrect** | `SendInput` inserts synthetic events and is subject to UIPI. Scan-code flags do not make the event physical or bypass integrity restrictions. The sample also omits key-up. |

## Official references for the verified API caveats

- [About window procedures — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-window-procedures) — subclassing must occur in the same process as the target window.
- [IDCompositionDevice::CreateTargetForHwnd — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd) — the target HWND must belong to the calling process.
- [SendInput — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) — inserts events into the input stream and is subject to UIPI.
- [Sharing Files and Memory — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/memory/sharing-files-and-memory) — cross-process shared-memory access must be coordinated; a mapping object remains until users close their handles.
- [Closing a File Mapping Object — Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/memory/closing-a-file-mapping-object) — view unmapping, handle closure, and explicit flushing considerations.

## Architectural guidance

Do not move every tactic into Traton (`wl_traton`). Keep platform discovery, native event normalization, capability resolution, and Windows-specific provider selection there. Feature ownership remains with the relevant subsystem: WLDE owns desktop presentation, WLWM owns layout/window policy, WLOSDE owns notification/OSD presentation, and `wl_animate` owns platform-neutral motion/timeline logic. Hardware telemetry and launchers should remain in their own feature libraries.

Use the catalogue as a **candidate inventory**. A tactic can become a documented capability, experimental adapter, exact-build quirk, redesigned approach, or rejected idea. The original file is preserved separately from this review.

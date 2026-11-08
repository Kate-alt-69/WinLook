# WLOSDE and WLDM

## WLOSDE — WinLook On-Screen Display Engine
WLOSDE renders WinLook-generated notifications, temporary popups and configurable OSD. It is not a replacement for native Windows security, UAC, authentication or secure-desktop surfaces.

Responsibilities:
- WinLook notifications and status;
- volume/brightness/media OSD where trusted state is available;
- WLDE wallpaper/Game Mode events;
- WLT import/apply/export messages;
- WLWM workspace/window messages;
- position/monitor selection, collision and stack management;
- entrance/exit animations using wl_animate or wl_Lanimate;
- active theme adaptation and accessibility-aware behaviour.

Settings: position, monitor, size, opacity, background, border, shadow, font, icon, color, animation, duration, priority, sound, quiet hours, per-category controls and safe actions.

WLOSDE uses independent WinLook-owned surfaces and leaves native security UI untouched.

## WLDM — WinLook Display Manager
WLDM handles monitor/display discovery and supported display settings:
- enumerate monitors and topology;
- detect connect/disconnect;
- query geometry/work area/orientation/DPI where supported;
- inspect resolution/refresh choices;
- store display profiles where APIs permit;
- support docking/undocking;
- expose display information to WLDE, WLWM, WLOSDE and Manager;
- snapshot original display state before WinLook changes it.

Boundaries:
- WLDM discovers/configures displays.
- WLDE renders wallpaper/widgets.
- WLWM manages windows across monitors.
- WLOSDE selects a display for popup placement.

Compatibility varies by OS, driver and hardware. Do not assume Windows 95 supports modern monitor enumeration, DPI, HDR or DisplayConfig. Begin with actual legacy API capabilities and add features after testing.

Open research: monitor identity persistence, mixed DPI/refresh, HDR API minimums, display switching safety, remote desktop and virtual displays.

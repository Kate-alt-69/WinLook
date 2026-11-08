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

## Detailed lifecycle and safety contracts

### WLOSDE notification lifecycle

A notification declares category, priority, content, expiry, target-display policy, actions and accessibility metadata. The engine validates it, queues it, resolves a display, applies quiet-hours/focus policy, renders it, handles permitted interaction and disposes it.

Proposed states: Received, Validated, Queued, Suppressed, Visible, ActionPending, Expired, Dismissed and Failed. Expiry/quiet-hours policy must not block critical WinLook recovery warnings. WLOSDE must never impersonate native Windows security prompts.

Priority and collision rules must be deterministic. A popup avoids taskbars, reserved work areas, full-screen content and other WinLook surfaces where reliable geometry exists. Unknown geometry uses a conservative position and must not steal focus for ordinary information.

Actions are typed and allowlisted. A notification cannot send arbitrary shell commands or elevate a process. Dangerous/system-wide operations open Manager for explicit consent.

### Accessibility and user control

- Keyboard users can reach, inspect, dismiss and activate permitted actions.
- Routine notifications do not steal focus.
- Reduced-motion disables or simplifies transitions.
- High contrast and text scaling are respected.
- Quiet hours, category controls, sound and persistent-notification preferences are user-owned.
- Notification content does not expose credentials, secure desktop content or private data from another process.

### WLDM display-profile transaction

Discovery does not imply permission to change a display.

1. Resolve requested target and verify API/driver capability.
2. Snapshot current display configuration and identity confidence.
3. Show proposed change and recovery explanation.
4. Apply through the OS-specific adapter.
5. Start a user-visible confirmation/revert window.
6. Commit only after user confirmation or an approved safe validation policy.
7. Restore snapshot after timeout, no confirmation, adapter error or invalid topology.

Confirmation timeout and rollback mechanism remain open. WLDM must not strand the user on an unusable resolution/monitor configuration. If reliable restoration is unavailable on an OS/driver, configuration remains read-only.

### Display identity and topology

Keep logical monitor role, current topology and physical identity confidence separate. Device names and enumeration order are not permanent identifiers. Profiles need fallback matching by user-selected role, connector/path and display characteristics without silently applying to another monitor.

Remote desktop, virtual displays, docking, mixed DPI, refresh changes, HDR and rotation are separate capabilities. Add each only after API/driver verification. WLDE uses topology for rendering; WLWM uses work areas/geometry; WLOSDE uses display selection.

### Acceptance gates

- [ ] Notifications have bounded queues, expiry and typed actions.
- [ ] Routine notifications do not steal focus.
- [ ] Secure desktop, logon and UAC are never replaced or covered by design.
- [ ] Display changes have verified snapshot/revert path before offer.
- [ ] Monitor disconnect/reconnect and docking are tested.
- [ ] Read-only discovery works where configuration is unsupported.


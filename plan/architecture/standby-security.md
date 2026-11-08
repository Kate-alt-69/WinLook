# Standby Engine and Session Security

> Status: security architecture proposal. WinLook does not authenticate users and does not replace Windows sign-in.

## Purpose

The Standby Engine provides a customizable idle experience while the user's interactive desktop session is still available. It can display clocks, animations, WLDE content and other permitted WinLook components. When Windows locks the session or switches to a protected desktop, WinLook must get out of the way and leave authentication to Windows.

## Security boundary

Windows uses window stations and desktops to separate interactive UI environments. Microsoft documents the Default, ScreenSaver and Winlogon desktops; the Winlogon desktop is active during sign-in and secure attention sequences, and its security descriptor restricts access to a small set of accounts. Ordinary applications generally cannot access it or switch to it while it is active.

Source: [Microsoft — Desktops](https://learn.microsoft.com/en-us/windows/desktop/winstation/desktops).

Consequences for WinLook:

- Never draw a fake password/PIN prompt.
- Never capture, store, forward or inspect credentials or credential-entry keystrokes.
- Never attempt to overlay, replace, or inject into Winlogon, LogonUI, LSA, Credential UI, UAC secure prompts or other authentication/security-critical processes.
- Do not treat \`NoLockScreen\` or another personalization policy as permission to run code on the secure desktop.
- A topmost window on the normal user desktop is not a secure sign-in surface and must not be described as one.
- On a real lock, hide or destroy WinLook-owned Standby surfaces and allow native Windows security UI to own the display.

## State model

| State | Meaning | WinLook behaviour |
|---|---|---|
| Active desktop | User is interacting with the normal session | Normal WLDE and WinLook behaviour |
| Idle candidate | Configured idle interval or policy is reached | Evaluate power, fullscreen, user activity and settings |
| Standby visible | WinLook idle surface is displayed on the normal interactive desktop | Render permitted content; no credential UI |
| Lock transition | Windows reports or otherwise establishes that the session is locking | Immediately stop input handling and hide/destroy Standby surface |
| Locked / secure desktop | Windows owns authentication UI | No WinLook overlay or authentication interaction |
| Unlock transition | Windows returns the user to the interactive session | Wait for session/window/display readiness |
| Resume | Normal interactive desktop is available | Restore the prior WLDE/Standby state without handling credentials |
| Sleep / display off | Power/display lifecycle changes | Suspend rendering and release unnecessary resources |

The exact event sources and ordering must be verified for each Windows generation. Do not assume that every event arrives in the same order or that a lock notification is always available.

## Session notifications and version boundaries

On Vista and later, a window can register with \`WTSRegisterSessionNotification\` and receive \`WM_WTSSESSION_CHANGE\` messages. It must unregister before destruction. Microsoft documents Vista as the minimum supported client for this API.

References:
- [WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification)
- [WM_WTSSESSION_CHANGE](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtsession_change)
- [WTSUnRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsunregistersessionnotification)

This API is **not** the universal Win95-to-Windows-11 solution. For Windows 95/98/ME, NT4, 2000, XP and Windows 7, research the relevant logon/session, screen-saver, power and desktop-switch mechanisms separately. A per-version adapter must declare which transitions it can reliably detect and what fallback it uses. Do not import Vista-only WTS functions into a legacy binary.

## Agent, BGW and Service responsibilities

- The per-user Agent owns the Standby window and interactive desktop rendering.
- The main BGW coordinates session state with WLDE, WLWM, WLOSDE and other WinLook modules.
- An optional Service may observe or perform privileged system operations only when required; it must not host interactive UI.
- Since Vista, Session 0 is reserved for services and other non-interactive processes. A Service cannot directly display UI in the user's session. See [Microsoft — Service changes for Windows Vista](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista).
- The Agent must be associated with the correct user session, including fast user switching and Remote Desktop scenarios.

## Failure and recovery

- If the Agent or WLDE crashes, Windows sign-in must remain unaffected.
- If the lock transition is ambiguous, fail closed: stop/hide Standby rather than risking a window over a security surface.
- If WinLook restarts during unlock, wait for a confirmed interactive desktop state before restoring UI.
- If notification registration fails, log the failure and use only a separately verified fallback for that OS.
- Never rely on the Standby surface being underneath LogonUI.
- The uninstaller must remove startup/session registrations and restore any settings WinLook changed.

## Verification matrix

- [ ] Lock with Win+L, lock from Start/power UI, idle lock and policy-enforced lock.
- [ ] Unlock with password, PIN, biometric and other Windows-native methods without WinLook observing credential input.
- [ ] Fast user switching and multiple simultaneous sessions.
- [ ] Remote Desktop connect/disconnect and session lock/unlock.
- [ ] Sleep, hibernate, display-off and wake.
- [ ] Multiple monitors, monitor removal during Standby and mixed DPI.
- [ ] Agent/BGW crash at every state transition.
- [ ] Exact Vista+ WTS registration/unregistration behaviour.
- [ ] Exact legacy event mechanisms on each officially targeted OS.

## References

- [Microsoft — Desktops](https://learn.microsoft.com/en-us/windows/desktop/winstation/desktops)
- [Microsoft — WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification)
- [Microsoft — WM_WTSSESSION_CHANGE](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtsession_change)
- [Microsoft — Session 0 and service changes in Windows Vista](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista)

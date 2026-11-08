# WinLook Audio Experience

> Status: planning proposal. EarTrumpet is a user-experience reference; WinLook should use documented Windows audio interfaces where available, not copy another application's implementation. The module name/acronym is undecided.

## 1. Purpose

Provide a clean, fast per-application and global audio control surface inside WinLook Manager, with an optional compact popup. The experience should feel familiar to users of EarTrumpet and Windows Volume Mixer.

This is an audio-control feature, not a replacement audio driver, virtual cable, DSP host or full hardware effects suite.

## 2. Scope

### Global output controls
- Current default output device and its master volume/mute.
- Output-device selection where the OS/API permits.
- Device availability, connection and default-role changes.
- Optional per-device saved volume preference, subject to safe restoration and Windows policy.
- Clear indication of which endpoint is being controlled.

### Per-application controls
- Enumerate active audio sessions and associate them with application/process identity where possible.
- Per-session volume and mute.
- Show session display name, process/application identity and current output endpoint when available.
- Handle an application that owns multiple audio sessions without silently merging unrelated sessions.
- Follow new session creation, session end, process restart and endpoint changes.
- Optional per-application output-device choice only when the OS exposes a supported route and the selected app/session supports it.

Global endpoint volume and per-session volume are separate controls and must not be conflated. Changing one must not unexpectedly rewrite the other's saved value.

## 3. Windows API baseline

For Vista and later, investigate Windows Core Audio:
- IAudioSessionManager2 and IAudioSessionEnumerator for session discovery.
- IAudioSessionNotification for session creation.
- IAudioSessionEvents for session state/display/volume notifications.
- ISimpleAudioVolume for per-session master volume/mute.
- IAudioEndpointVolume for endpoint master volume/mute.
- IMMDeviceEnumerator / endpoint notification interfaces for output devices and default-device changes.

Primary references:
- [Core Audio APIs](https://learn.microsoft.com/en-us/windows/win32/api/_coreaudio/)
- [ISimpleAudioVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-isimpleaudiovolume)
- [ISimpleAudioVolume::SetMasterVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-isimpleaudiovolume-setmastervolume)

Windows 95 through XP do not have the Vista Core Audio session model. Do not promise equivalent per-application volume on those systems. Research each legacy mixer API and determine whether it controls only hardware mixer lines/global levels. Per-app control must remain unavailable unless a safe, tested architecture exists; do not inject into every application or install a virtual audio driver just to imitate modern session mixing.

## 4. Session identity and edge cases

A process can create more than one audio session; sessions may be short-lived, shared, renamed, or detached from an obvious foreground window. Process ID alone is not a durable application identity.

The design must handle:
- multiple sessions for one process;
- multiple processes contributing to one logical app;
- packaged apps and brokers;
- system sounds and communication streams;
- apps switching output devices;
- endpoint disconnect while a user is changing volume;
- process restart and PID reuse;
- session creation/termination while the popup is open;
- session volume changes made by another controller.

Prefer the actual audio-session identity and current process/file metadata. Do not infer identity from a display name alone. Use event-context values to distinguish WinLook-originated changes where the API supports it.

## 5. Manager UI

Proposed page:
- a master output row with endpoint picker, volume and mute;
- a searchable list of active app sessions;
- compact per-app volume/mute controls;
- group repeated sessions by app only when the relationship is verified;
- show the endpoint for each session where available;
- empty/loading/disconnected/unsupported states;
- a small popup surface suitable for fast access without opening the full Manager.

Do not turn this into a sound-effects editor. Advanced device controls can remain in Windows settings or the device vendor's own app.

## 6. Profiles and Templates

Audio preferences may participate in Profiles:
- global output volume/mute preference;
- preferred output role/device matching rule;
- per-application session level/mute;
- per-app endpoint preference where supported.

A profile should match devices by a portable, user-understandable identity rule where possible. Do not export a machine-specific endpoint instance ID as a portable promise. On profile activation, show unavailable mappings and preserve the user's current Windows state when a match is uncertain.

WLT may export selected audio profile settings and rules, but must not export device serials, private device identifiers, credentials or current session contents.

## 7. Permissions, safety and privacy

- Ordinary audio-session control should not require administrator elevation.
- Do not capture audio or record microphone/system output as part of the mixer.
- No telemetry of application audio levels.
- Device changes are explicit user actions; profile automation needs a visible rule and opt-out.
- If an endpoint disappears, stop applying changes to stale handles and re-resolve device identity.
- If the module exits, Windows' existing audio services and endpoint routing continue normally.

## 8. Compatibility states

Expose Available, Partial, OptionalDependency, Experimental, Unsupported and Unknown states consistent with the shared capability model. Vista+ Core Audio support is a documented API baseline, not proof of WinLook implementation. Legacy per-app volume remains research-only until tested.

## 9. Acceptance gates

- [ ] Global endpoint control and per-app session control are separate.
- [ ] Session add/remove and volume events update the view without polling loops.
- [ ] Multiple sessions and PID reuse are handled.
- [ ] Device disconnect/reconnect cannot cause writes to a stale endpoint.
- [ ] External volume changes are reflected correctly.
- [ ] Profile apply and restore respect user changes made afterwards.
- [ ] No audio capture, driver replacement or blanket app injection is required.
- [ ] Each OS page records its actual global/per-app capability and evidence.

## 10. Open decisions

- Final module name and acronym.
- Whether the first release includes only output or also capture/microphone session controls.
- Grouping rules for multi-session apps.
- Whether per-app endpoint routing is included in the first version.
- Exact legacy volume capabilities for 95/98/Me/NT4/2000/XP.
- Whether profiles store relative volume, absolute volume or both.

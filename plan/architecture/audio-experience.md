# WinLook Audio & Media Experience

> Status: accepted product direction; implementation design and final module name remain open. EarTrumpet is a UX reference, not an implementation dependency.

## 1. Purpose

Provide one WinLook-owned experience combining global output controls, per-application audio controls where the OS exposes them, and a rich Now Playing/media-control surface.

It includes endpoint/device selection and master volume/mute, per-app audio-session volume/mute where supported, source application and current media item, Audio/Video classification, player/browser-provided artwork or video thumbnails, available metadata, transport controls, timeline/seek where supported, a full Manager page and a compact popup.

This is not an audio driver, virtual cable, recorder, DSP/effects host or replacement for Windows audio services.

## 2. Audio sessions and media sessions are separate

An **audio session** answers which application/session is producing sound and what volume control Windows exposes for it. A **media session** answers what item is playing, what metadata/artwork is available, and which transport operations the player supports.

They may not map one-to-one: an application can play audio without publishing a media session; a media session may exist while paused or silent; browsers choose how tabs are grouped into sessions; and one process may create multiple audio sessions.

Never bind a media card to a volume slider based only on display-name similarity. Correlate using reliable session/application identity; if ambiguous, show separate entries.

## 3. Audio controls

### Global endpoint controls
- Default output endpoint, master volume and mute.
- Output-device selection where supported.
- Device connection/default-role changes and clear endpoint identity.
- Endpoint volume and per-session volume remain independent.
- Per-app output routing only where a documented, tested route exists.

### Per-application controls
- Enumerate sessions where the OS exposes enumeration.
- Show app identity, display name, state and output endpoint where available.
- Per-session volume/mute and reflect changes made elsewhere.
- Handle multiple sessions per process, session creation/end, process restart, PID reuse, endpoint removal and stale handles.
- Do not claim control of exclusive-mode streams or paths outside the shared session mixer.

### Windows API generations

**Vista:** Core Audio provides `IAudioSessionManager`, `IAudioSessionControl`, `ISimpleAudioVolume` and `IAudioEndpointVolume`. The base session manager is documented from Vista. It does not provide the later `IAudioSessionManager2` enumerator/notification surface, so a complete live per-app list must not be assumed from Vista's base interface alone.

**Windows 7 and later:** `IAudioSessionManager2` adds session enumeration and notifications. Correctly maintain the enumerator and event lifecycle.

**Windows 95 through XP:** these systems predate Core Audio. WinLook still supplies its own UI, but generic per-process volume is not assumed. Legacy mixer/device APIs may expose global or hardware/driver controls; exact capability must be tested per OS, driver and device. A player-owned volume control is not an OS-level per-process mixer and must be labelled separately.

Primary references:
- [IAudioSessionManager](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager) — Vista baseline.
- [IAudioSessionManager2](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager2) — Windows 7+ enumeration/notification.
- [ISimpleAudioVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-isimpleaudiovolume) — shared-mode session volume/mute.
- [IAudioEndpointVolume](https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume) — endpoint volume/mute.

## 4. Now Playing and media controls

### Modern Windows: GSMTC

Windows 10 version 1809 (build 17763) introduces `GlobalSystemMediaTransportControlsSessionManager`. For applications publishing a Windows media session, WinLook can enumerate sessions, observe changes, identify the source App User Model ID, read playback state and available controls, retrieve playback type (including Audio or Video), title/subtitle and available metadata, retrieve the provider thumbnail, read the timeline, request a seek, and request play/pause/previous/next/stop/fast-forward/rewind when supported.

A YouTube video can appear as a Video item with the thumbnail supplied by the browser/player. A music application can appear as Audio with album artwork. This does not require scraping the website's UI.

Primary references:
- [GSMTC session manager](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager) — Windows 10 version 1809+.
- [GSMTC session](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssession) — app identity, transport, timeline and events.
- [Media properties](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmediaproperties) — playback type, metadata and thumbnail.
- [Timeline properties](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessiontimelineproperties) — playback timeline.

### Provider and browser limitations

- The browser/player chooses which media session(s) it publishes. Do not assume one Windows media session per tab.
- Metadata, artwork, playback type, seeking and individual commands are optional; display only what the provider supplies/supports.
- Missing thumbnail uses a neutral fallback. Do not scrape the website.
- If seeking is unsupported, do not show a functional-looking seek slider.
- Multiple published sessions remain individually selectable.
- GSMTC sessions and Core Audio sessions are distinct; correlation is best-effort.

### Legacy Windows and players without GSMTC

The WinLook-owned UI exists on Vista and older systems too. Without GSMTC, metadata and playback controls require explicit integrations:
- documented player SDK, COM automation interface or plugin API;
- WinLook adapter for a named supported player/version;
- narrowly scoped documented message/control interface after semantics are verified.

A Legacy Player Adapter must declare player identity/version, title/subtitle, media type, artwork source, playback state, duration/position, supported commands, update mechanism, installation/removal and failure isolation. Do not infer or fabricate metadata.

On pre-Vista systems, expose only global/device mixer controls actually provided by the OS/driver. A supported player's own volume may be shown separately. Do not install a virtual audio driver or inject into every application to manufacture a universal per-process mixer.

## 5. Manager UI

The **Audio & Media** page includes output-device/master controls, an active audio-session list with per-app volume/mute where supported, and a distinct Now Playing area with source application, Audio/Video type, provider thumbnail/artwork, available metadata, supported transport and a timeline that is seekable only when allowed.

The compact popup prioritizes master output controls and the current media item. The full Manager page exposes all available sessions. Keep mixing controls and media controls visually distinct. Expose Available, Partial, OptionalDependency, Experimental, Unsupported and Unknown states.

## 6. Profiles and Templates

Profiles may coordinate endpoint role/device preference, endpoint volume/mute, per-app session volume/mute, supported per-app endpoint preference and player settings supplied by an adapter.

Do not store transient title, thumbnail cache, current playback position, process handles or private session state in a Profile. WLT exports selected portable preferences, not hardware serials, private endpoint instance IDs or active media payloads.

## 7. Safety and privacy

- Ordinary endpoint/session controls should not require elevation.
- No audio recording, microphone capture or telemetry of audio levels is part of the mixer.
- Consume only metadata/thumbnail references supplied by a supported system session or explicit adapter.
- Isolate long-running player adapters from the Manager.
- Stop writes through stale handles when a session/endpoint disappears.
- If WinLook exits, native audio and player functionality continue.
- The standard modern path requires no arbitrary app injection.

## 8. Compatibility interpretation

| Windows generation | Audio controls | Now Playing/media controls |
|---|---|---|
| 95 / 98 / Me / NT4 / 2000 / XP | Legacy global/device mixer only where verified; no generic per-process guarantee | WinLook UI plus explicit supported-player adapters |
| Vista | Core Audio endpoint and base session interfaces; no `IAudioSessionManager2` enumerator guarantee | WinLook UI plus player adapters; no GSMTC |
| 7 / 8 / 8.1 | Core Audio session enumeration/notifications through `IAudioSessionManager2` | WinLook UI plus player adapters; no GSMTC |
| 10 before 1809 | Core Audio | Player adapters; GSMTC unavailable |
| 10 version 1809+ / 11 | Core Audio | GSMTC sessions, metadata, provider thumbnails, controls/timeline where published |

This is an API baseline, not a WinLook support claim. Per-version evidence must record OS build, player, endpoint/driver and runtime tests.

## 9. Acceptance gates

- [ ] Endpoint volume and per-session volume remain separate.
- [ ] Vista/Windows 7 Core Audio interface boundaries are detected correctly.
- [ ] Windows 7+ session events handle creation/end and PID reuse.
- [ ] Exclusive-mode streams are not advertised as session-controllable.
- [ ] GSMTC discovery is gated to Windows 10 version 1809+.
- [ ] Audio/video type, title, metadata, artwork/thumbnail and timeline are shown only when provided.
- [ ] Transport/seek controls match the session's supported commands.
- [ ] Browser grouping ambiguity cannot bind the wrong volume control to a media item.
- [ ] Legacy UI works without pretending unavailable system media APIs exist.
- [ ] Legacy player adapters have versioned contracts and isolation.
- [ ] Profile apply/restore respects external changes.
- [ ] No recording, virtual audio driver or blanket app injection is required.

## 10. Open decisions

- Final module name/acronym and source-directory placement.
- Whether microphone/capture controls ship initially.
- Exact legacy mixer/device capability by OS, driver and hardware.
- Which legacy players get official adapters first.
- Adapter IPC/ABI and third-party SDK stability.
- Correlation rules when media/audio identities are ambiguous.
- Whether per-app endpoint routing ships initially.

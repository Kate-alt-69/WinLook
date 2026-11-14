# wl_wlde_win32

The small Win32 platform adapter for the portable `wl_wlde` scene core.

## Monitor snapshots

`enumerate_monitors()` discovers desktop-attached GDI display devices with
`EnumDisplayDevicesW`, excludes mirroring-driver pseudo-devices, then obtains
monitor rectangles and device names through `EnumDisplayMonitors` and
`GetMonitorInfoW(MONITORINFOEXW)`.

The result is a validated `MonitorTopology` with at most 16 records. Each
record contains the GDI device name for the current snapshot, the monitor
rectangle, the work-area rectangle and the primary flag. GDI rectangles use
virtual-screen coordinates; monitors left of or above the primary display can
have negative coordinates. Device names such as `\\.\DISPLAY1` are snapshot
identifiers, not durable hardware identities and must not be used as permanent
profile keys.

`refresh_topology(&mut current)` gathers and validates a complete replacement
before swapping it in. If enumeration or validation fails, the caller's last
known-good topology is unchanged.

## Event-loop contract

This crate does not create a window, timer, polling thread, hook or message
pump. The host owns its event loop and should call `refresh_topology` after
`should_refresh_topology(message, w_param)` returns true:

- `WM_DISPLAYCHANGE` indicates a display mode/resolution change.
- `WM_DEVICECHANGE` with `DBT_DEVNODES_CHANGED` is a device-tree notification
  that can accompany monitor arrival or removal.

Hosts may coalesce multiple notifications into one refresh. DPI changes,
monitor color profiles, orientation and persistent monitor identity are not
represented by this first snapshot adapter and remain separate work.

## Public entry points

The same functions are re-exported by the shared Win32 backend and the public
`wl_ui::windows10` / `wl_ui::windows11` modules on their respective builds.

The adapter uses a narrow, private Win32 FFI boundary and has no additional
runtime crate dependencies. The portable Rust core remains
`#![forbid(unsafe_code)]`; this crate contains the platform-specific unsafe
calls and exposes only owned, validated Rust data.

## Explorer desktop-host discovery

`discover_desktop_host()` identifies the process that owns the current
Shell desktop window. It verifies the image basename is `explorer.exe`,
checks that the process belongs to the same interactive session as BGW, and
records PID, session ID, executable path and process creation time. The
creation timestamp prevents a restarted Explorer process from being mistaken
for the previous process if Windows recycles its PID. The probe opens the
process with query-information access only; it does not request memory-read,
thread-control or process-modification rights.

Call `is_same_desktop_host(&identity)` after a Shell restart notification
or before reattaching a native adapter. A mismatch means BGW must reconcile
the old bridge and discover the new desktop host. The probe retains no process
handle and changes no Explorer state.

This is host discovery, not the in-process integration itself. It does not
inject a DLL, install a context-menu handler, hide icons or create a Group
presenter. Those operations belong to the next generation-specific Shell
adapter and must be backed by a proven restoration path. The API uses
`QueryFullProcessImageNameW` and is for Vista and newer. Earlier Windows
generations need their own classic process/image lookup adapter.

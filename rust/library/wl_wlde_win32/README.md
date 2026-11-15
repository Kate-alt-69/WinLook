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

The adapter uses a narrow, private Win32 FFI boundary. Its `sha2 0.10.9`
dependency is used only for path-hash fallbacks. Stable inventory IDs prefer
the volume serial number and 128-bit `FILE_ID_128` returned by
`GetFileInformationByHandleEx(FileIdInfo)`, plus the creation FILETIME
returned by `GetFileInformationByHandle` as a reuse discriminator. The
legacy 64-bit file index is deliberately not used as a stable key because
Microsoft does not guarantee that it is unique on ReFS. If the 128-bit file
identifier is unavailable/zero, or the filesystem does not expose a creation
time, the item uses a marked path-hash fallback instead.

Identity handles request only `FILE_READ_ATTRIBUTES`, share
read/write/delete access, and use `FILE_FLAG_OPEN_REPARSE_POINT` so a
shortcut/reparse entry is identified without opening its target.
`FILE_FLAG_BACKUP_SEMANTICS` permits directory handles. No file content is
read or changed.

## Desktop item identity and Group continuity

`DesktopInventoryItem::identity_stability()` distinguishes
`FilesystemFileId` from `PathFallback`. A unique single-link filesystem
identity keeps the same Group key through ordinary same-volume NTFS renames.
The adapter includes creation time as an additional discriminator; this
reduces accidental key reuse but is not a universal anti-reuse guarantee.

Filesystem support varies. File IDs can be reused after deletion, and
network filesystems may return partial identity information. A 128-bit ID
reduces collision risk and avoids the documented ReFS limitation of the
legacy 64-bit index, but it is not a permanent identity across deletion and
re-creation. When identity cannot be read, when Windows reports multiple hard
links, or when duplicate identities occur in the visible inventory, the
adapter uses distinct path-hash IDs and marks them `PathFallback` rather than
merging desktop entries. Path fallbacks change on rename and must be
reconciled before durable Group membership is restored. The portable Rust/C89 Group models remain
host-ID agnostic; legacy C platform adapters need their own per-generation
identity implementation.

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


## Explorer desktop-host process monitor

`discover_desktop_host()` identifies the process that owns the current Shell
desktop window. It verifies that the image basename is `explorer.exe`, checks
that the process belongs to BGW's interactive session, and records the PID,
session ID, image path and process creation FILETIME.

`monitor_desktop_host()` opens a second process handle with only
`PROCESS_QUERY_INFORMATION | SYNCHRONIZE` rights, verifies the creation time
against the discovered identity, and returns a waitable monitor.
`wait_for_exit()` uses `WaitForSingleObject`; it does not spin. Dropping the
monitor closes the handle. The BGW host loop rechecks the Shell owner every
five seconds so it can detect a replacement even when the old Explorer
process remains alive.

The composed Rust probe is Vista+ because it uses
`ProcessIdToSessionId` and `QueryFullProcessImageNameW`. The host monitor is
not a Shell hook: it does not inject code, enumerate desktop PIDLs, suppress
icons, intercept context menus or draw Group proxies. Windows 95–XP and legacy
C targets need their own classic host-discovery/runtime implementation.


The Win32 watcher uses the documented change-notification handle pattern:
wait on up to two non-recursive User/Public Desktop handles, call
`FindNextChangeNotification` after the signaled handle, and then wait again.
It watches only file-name and directory-name changes; ordinary content writes
do not wake the inventory watcher. It refreshes the full bounded inventory
because the notification APIs do not identify the changed entry. UNC roots
retain a five-second fallback because remote filesystems are not guaranteed
to deliver notifications. Paths at or beyond the default MAX_PATH limit are
passed with the extended-length prefix.
This watcher is for the modern Win10/11 Rust adapter; the documented API's
minimum client is Windows XP, not Windows 95/legacy.


## Read-only Shell icon extraction

`extract_desktop_shell_icon(&DesktopInventoryItem, DesktopShellIconSize)`
uses `SHGetFileInfoW(SHGFI_ICON | SHGFI_SHELLICONSIZE)` to obtain the current
Shell icon for a desktop filesystem path. The returned `DesktopShellIcon`
owns the HICON and destroys it when dropped. Its dimensions are read with
`GetIconInfo` and `GetObjectW`; the temporary mask/color HBITMAPs returned
by `GetIconInfo` are deleted when the metadata query finishes.

Run extraction on a BGW worker, not the Manager UI thread: Shell icon handlers
may do disk/COM work. The icon API does not replace Explorer's system image
list, alter a Shell registration, hide a native tile, write source icon
resources or intercept context menus. It provides an owned native handle and
dimensions only; BGW's next stage still needs bounded HICON-to-RGBA
rasterization and an identity/DPI/theme-aware cache before calling the
portable adaptive-icon compositor.

The adapter handles both color icons and legacy monochrome mask-only icons
when reporting dimensions. Unsupported, inaccessible or missing icons return
a typed error and leave Explorer's native desktop unchanged. This path-based
first step does not cover virtual Shell namespace items or exact IFolderView
membership; those need the dedicated PIDL/Shell-view bridge.

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
may do disk/COM work. The caller must initialize COM on that worker before
calling SHGetFileInfoW and balance the successful initialization with
CoUninitialize; BGW's dedicated icon worker owns this apartment lifetime. The icon API does not replace Explorer's system image
list, alter a Shell registration, hide a native tile, write source icon
resources or intercept context menus. It provides an owned native handle and dimensions. The new
`rasterize_desktop_shell_icon_rgba()` converts that handle into a bounded
512×512 straight-alpha RGBA buffer by drawing onto opaque black and white
probe backgrounds and reconstructing alpha from their output difference.
This works for alpha-blended icons and classic AND/XOR mask icons where their
pixel operation is representable as static RGBA. Screen-dependent inversion
pixels return `UnsupportedRasterOperation` rather than being misrepresented.
The RGBA wrapper exposes `as_adaptive_image_view()` so consumers can
borrow its pixels directly as WLDE compositor input without a copy. The DIB is top-down, GDI commands are flushed before the CPU reads
its bits, the selected bitmap is restored before its handle is deleted, and
all dimensions/allocations are bounded. The BGW runtime now owns a bounded
identity/DPI/theme-aware memory cache; live presentation remains separate work.

The adapter handles both color icons and legacy monochrome mask-only icons
when reporting dimensions and rasterizing their representable pixels. `SHGetFileInfoW` documents a `MAX_PATH`-bounded
path argument, so paths over 259 UTF-16 code units (excluding the terminator)
return `PathTooLong` rather than being passed through undocumented extended
path behavior. Unsupported, inaccessible or missing icons return a typed
error and leave Explorer's native desktop unchanged. This path-based
first step does not cover virtual Shell namespace items or exact IFolderView
membership; those need the dedicated PIDL/Shell-view bridge.


## Reuse as a Windows platform adapter

Package: `wl_wlde_win32` `0.1.0-alpha.1` (Rust 2021; declared MSRV 1.77).
Its portable dependency is `wl_wlde`; `sha2 0.10.9` is compiled only for
Windows and is used for explicitly marked path-based fallback identities.

```toml
[dependencies]
wl_wlde = { path = "../WinLook/rust/library/wl_wlde" }
wl_wlde_win32 = { path = "../WinLook/rust/library/wl_wlde_win32" }
```

Public API families:

- `enumerate_monitors`, `refresh_topology`,
  `should_refresh_topology` and the `WM_DISPLAYCHANGE` /
  `WM_DEVICECHANGE` constants.
- `enumerate_desktop_items`, `watch_desktop_folders`,
  `DesktopInventoryItem`, `DesktopFolder`,
  `DesktopItemIdentityStability` and the fixed inventory limit.
- `discover_desktop_host`, `monitor_desktop_host`,
  `is_same_desktop_host`, `DesktopHostIdentity` and the process-monitor
  contract.
- `DesktopBrowserWindow`, `OwnedDesktopBrowser`, `DesktopBrowserSnapshot`,
  `DesktopBrowserItemPosition`, `DesktopBrowserBounds` and the bounded owned
  ExplorerBrowser Shell-view host.
- `extract_desktop_shell_icon`, `DesktopShellIcon`,
  `rasterize_desktop_shell_icon_rgba` and the RGBA/icon-size/error types.

Refresh monitor snapshots from the host's event loop; this crate does not
install listeners or own a loop. Keep inventory paths ephemeral and use the
opaque item reference plus its identity-stability classification for model
and persistence work. A path fallback is not rename-stable.

The Shell icon entry points must run on a worker that initialized COM as an
STA and dispatches its message queue. The returned HICON is owned by
`DesktopShellIcon`; its drop releases the handle. Rasterized pixels are
bounded to 512×512 RGBA and can be borrowed as an
`AdaptiveIconImageView`. The Shell path entry point enforces the documented
MAX_PATH limit. This adapter only acquires data: it does not suppress native
icons, replace Shell registrations, intercept menus or draw Group tiles.

Non-Windows builds retain pure helper tests and return explicit
`UnsupportedPlatform` errors for the Windows collection operations. Run
`cargo test -p wl_wlde_win32`; the actual Win32 monitor, inventory, host
monitor and HICON smoke checks require Windows CI. See the [shared library
catalog](../README.md) for dependency selection and the repository-wide gate.

## Plan B: WinLook-owned Shell desktop view

The Plan B adapter hosts the virtual Desktop namespace through the documented ExplorerBrowser control instead of acquiring Explorer's live desktop view with IShellWindows. The caller must create a native parent window in the WinLook process and on the current STA thread, then wrap that HWND with the unsafe DesktopBrowserWindow::from_raw constructor. The adapter verifies the HWND owner process and thread before creating ExplorerBrowser.

OwnedDesktopBrowser initializes IExplorerBrowser, browses to the Desktop namespace root, and retrieves its own IFolderView through GetCurrentView. Each snapshot reacquires that view, enumerates at most 4,096 entries, records the per-view enumeration ordinal and GetItemPosition coordinates, then checks the count and view identity again. COM pointers and PIDLs remain private to the adapter and are released on the creating STA thread.

The returned ordinals are transient snapshot positions, not Shell item IDs. The coordinates are relative to the ExplorerBrowser view client area, not absolute screen or monitor coordinates. GetItemPosition failures are retained per item instead of fabricating a coordinate. A monotonically increasing view generation changes when the IFolderView identity or view HWND changes.

This is the owned-view acquisition layer only. It does not create the parent window, position it behind the native desktop, hide the existing Explorer desktop, suppress native icons, modify Shell state or enable Groups. The parent window must outlive OwnedDesktopBrowser and the browser must be created, used, resized and dropped on the same STA thread. Native icon replacement remains blocked by the separate durable restoration, adapter ownership and fault-injection gates.

Microsoft documents IExplorerBrowser as an application-hosted Shell browser whose view can be retrieved with GetCurrentView. Initialize requires the caller to invoke Destroy to release its windowed resources. See [the Plan B architecture](../../../plan/architecture/desktop-browser-hosting.md) and the [source ledger](../../../plan/references/SOURCES.md) for the API scope and remaining gates.
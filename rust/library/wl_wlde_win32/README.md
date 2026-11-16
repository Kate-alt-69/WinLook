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

## Read-only Shell Desktop namespace snapshot

enumerate_desktop_namespace_items() calls SHGetDesktopFolder to obtain the root Desktop IShellFolder, then calls EnumObjects for folder and non-folder children. Each child is inspected with GetAttributesOf and GetDisplayNameOf; filesystem-backed children may also expose an ephemeral parsing path. Virtual namespace records are represented separately as VirtualItem or VirtualFolder.

The snapshot is deliberately bounded: at most 4,096 enumerated children, one single-level child PIDL per item (64 KiB maximum), 32,768 UTF-16 units per Shell name and 16 MiB of retained label/path text. Entries whose PIDL, attributes or display name cannot be validated are skipped and included in skipped_items(). The call uses a null owner window so a provider that needs user input must fail silently rather than display a prompt.

Each id() is a SHA-256 digest of the current PIDL bytes. Treat it as session-only: Shell PIDL encodings are not durable file/profile identities and must never be persisted into Groups, templates or remote messages. Any filesystem_path() is also ephemeral and must not be logged or persisted.

This enumerates direct children exposed by the root Shell namespace; it is not an IFolderView snapshot. It does not report the actual desktop view's current visible/filtered items, icon positions, ordering, selection, hidden-item policy or view state. It does not enumerate descendants. The existing filesystem inventory remains a separate path-free identity source for local Group state.

The function initializes an STA COM apartment on its calling thread and balances CoUninitialize. Run it on a dedicated worker: Shell namespace providers can be slow or unresponsive. BGW uses one bounded STA worker, one request at a time, and does not block its Explorer-host monitor loop. A 20-second timeout is diagnostic only; the worker is not force-killed or replaced while a Shell call may still be active.

The current adapter path is the modern Windows implementation. This snapshot is read-only: it does not hide or replace native icons, register Shell objects, intercept menus, display Groups, or change Explorer state.

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
icon extraction still does not operate on virtual namespace items or the
actual IFolderView membership. The separate read-only namespace inventory
below classifies root namespace items, but it does not provide live view state
or render icons into the desktop.


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
- `enumerate_desktop_namespace_items`, `DesktopNamespaceSnapshot`,
  `DesktopNamespaceItem`, `DesktopNamespaceItemKind` and the namespace limits.
- `discover_desktop_host`, `monitor_desktop_host`,
  `is_same_desktop_host`, `DesktopHostIdentity` and the process-monitor
  contract.
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
`cargo test -p wl_wlde_win32`; the actual Win32 monitor, filesystem inventory, Shell namespace, host
monitor and HICON smoke checks require Windows CI. See the [shared library
catalog](../README.md) for dependency selection and the repository-wide gate.


## Live Explorer Desktop view probe

\`probe_desktop_folder_view(&DesktopHostIdentity)\` attempts to acquire the
actual Desktop \`IFolderView\` from the registered \`SWC_DESKTOP\` Shell window.
It resolves the active Shell view, checks that its HWND owner is the exact
Explorer process instance supplied by BGW, and verifies that the view folder's
\`IPersistFolder2::GetCurFolder\` PIDL matches the virtual \`CSIDL_DESKTOP\`
PIDL. It tries the \`CSIDL_DESKTOP\` VARIANT and \`VT_EMPTY\` locator forms
independently and reports which one succeeded.

The function returns only a \`DesktopViewProbe\` containing the owner PID and
locator form. COM pointers, HWNDs and PIDLs stay on the calling STA and are
released there. It performs no selection, position, view-mode, icon or Shell
registration changes. If the process identity, view HWND or Desktop-root
check fails, it returns an error instead of accepting an arbitrary Explorer
folder.

Call it from a dedicated STA with a message pump, such as BGW's existing
\`DesktopNamespaceWorker\`. Runtime acquisition and per-build compatibility
still require an interactive Windows test; compiling the binding is not proof
that a target build returns the expected live Desktop view.

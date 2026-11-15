# wl_bgw_runtime

Initial shared runtime library for WinLook's BGW process.

## Implemented in this slice

- Discovers the process that owns the current Shell desktop window through
  `wl_wlde_win32::monitor_desktop_host()`.
- Confirms the process is `explorer.exe` and belongs to the same interactive
  session as BGW.
- Holds a query/synchronization-only process handle and waits for Explorer
  process exit without a busy loop.
- Revalidates the Shell desktop owner every five seconds, so BGW can detect a
  desktop-host replacement even when the original Explorer process stays alive.
- Retries discovery every two seconds only while the Shell desktop is absent,
  belongs to a non-Explorer shell, or cannot be queried.
- Sends status output through `OutputDebugStringW` on Windows. This is a
  diagnostic channel, not a persistent application log; without a debugger or
  active system debugger, Windows may discard the messages.

## Deliberately not implemented yet

This is a host-lifecycle foundation, not the finished BGW runtime or the full
Explorer Desktop Bridge. It does not register itself in Windows startup, expose
Manager IPC or a stop command, load committed configuration, install a Shell
COM extension or in-process hook, enumerate desktop PIDLs, suppress native
icons, persist restoration journals, render wallpaper, display Groups or
intercept context menus. Those operations remain gated on their dedicated
capability and restoration implementations.

The shared runtime library is hosted by the independent `winlook-bgw` Cargo
package under `rust/bgw/`. It is intentionally not a dependency of either
Manager package, so BGW builds do not pull in WinUI/Windows Reactor.

Run the target through the repository build runner:

- `python build.py build bgw` builds the shared BGW executable.
- `cargo build --release -p winlook-bgw` builds the executable directly.
- The build runner normalizes the Windows executable filename to
  `winlook.bg.exe` under `build/rust/bgw/winlook-bgw/release/`.

The initial runtime is intended to run in the user's interactive session, not
as a Session 0 service. To stop this prototype, end the BGW process from Task
Manager; authenticated Manager-controlled shutdown is part of the later IPC work.


## BGW inventory handoff

After the desktop-host monitor verifies the actual Explorer-owned process,
BGW obtains one read-only filesystem snapshot from the user's Desktop and
Public Desktop folders. It retains the path-to-opaque-ID map only in memory
for that host lifetime and reports aggregate counts through OutputDebugStringW
(no item names or paths are logged). The snapshot is dropped before BGW
rediscovers a replacement host.

This handoff is a foundation for local Group reconciliation, not the live
Explorer Desktop Bridge. It does not persist Groups, hide icons, replace
native icon registrations, install a Shell extension/hook or create Group
popups. Inventory failure keeps the native desktop unchanged.


The folder watcher uses the documented Windows change-notification handle
sequence and retains a five-second bounded snapshot fallback for UNC roots
because remote filesystem notifications are not guaranteed. It watches only
top-level file/directory-name changes, not ordinary content writes, and
re-enumerates the full bounded snapshot after each signal; it does not rely
on the notification to identify the changed item.


## Local Desktop Groups and layout state

BGW now has a bounded local state store in `desktop_store.rs`:

- The default Windows path is `%LOCALAPPDATA%/WinLook/WLDE/desktop-state.v1.bin`.
- The versioned binary format stores Group IDs/labels/member IDs, identity kinds and normalized top-level layout placements. It does not store item paths, file content, icon artwork, executable paths or portable-template data.
- The format is capped at 4 MiB and has a CRC32 integrity check. The checksum detects accidental corruption; it is not a digital signature or tamper-proof authentication mechanism.
- Writes use a unique temporary file in the same directory, flush the file, then rename it into place. BGW is the single-writer owner; a second independent writer is not supported.
- Loads validate the complete Group set, current desktop inventory, identity stability, all required layout entries and the current monitor topology before returning a snapshot. A stale path-fallback ID, missing item, incomplete inventory, changed monitor ID or corrupt file is rejected. The saved file is left untouched so recovery/reconciliation can be handled separately.
- Filesystem-backed item IDs are persisted with their identity kind. Path-fallback IDs are never silently treated as rename-stable.

BGW loads the committed snapshot after it has verified the Explorer desktop host and collected the current inventory/topology. It also revalidates after inventory notifications. This currently restores only the validated in-memory configuration model: Manager IPC and a Manager-driven save/edit flow are not implemented yet, and Groups remain non-presenting until native icon restoration is proven.

Monitor IDs in the current layout are adapter snapshot IDs (for example, GDI display-device names), not stable physical monitor identities. Until WLDM supplies a durable monitor mapping/reconciliation layer, a changed monitor identifier rejects the stored layout rather than guessing where a Group should move.

This file is configuration persistence, not the restoration journal. It does not record or restore native icon suppression, Explorer hooks, Shell registrations or other OS mutations. Crash-safe native restoration remains a separate prerequisite before Groups can be shown.

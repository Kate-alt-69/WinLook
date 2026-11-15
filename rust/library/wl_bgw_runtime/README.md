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

# wl_bgw_runtime

Shared runtime library for WinLook's BGW process and the current WLDE desktop-inventory/state-store foundation.

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
- Starts one bounded STA worker for a read-only root Shell namespace snapshot
  after each verified Explorer-host attachment. It refreshes every 30 seconds
  and after filesystem Desktop change notifications, and discards results from
  an older host generation.
- Reports aggregate filesystem/virtual-item counts only; no Shell item names or
  filesystem paths are written to the BGW diagnostic channel. PIDL-derived IDs
  are session-only and are not passed into the persistent Group/layout store.
- Reports a 20-second namespace-worker timeout without blocking BGW or spawning
  an unsafe replacement thread. The in-flight Shell call may still complete.
- Sends status output through `OutputDebugStringW` on Windows. This is a
  diagnostic channel, not a persistent application log; without a debugger or
  active system debugger, Windows may discard the messages.

## Deliberately not implemented yet

This is not the finished BGW runtime or the full Explorer Desktop Bridge. It
does not register itself in Windows startup, expose Manager IPC or a stop
command, provide a Manager-driven Group editing/save route, install a Shell
COM extension or in-process hook, enumerate the live IFolderView desktop
contents/order/filter/visibility state, suppress native icons, persist
restoration journals, render wallpaper, display Groups or intercept context
menus. Those operations remain gated on their dedicated
capability and restoration implementations. The current local store can load
and validate an existing Groups/layout snapshot, but it cannot receive edits
from Manager until the authenticated IPC/configuration path is implemented.

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
Public Desktop folders. It retains the current inventory and ephemeral filesystem paths only in
memory for that host lifetime and reports aggregate counts through
OutputDebugStringW (no item names or paths are logged). The snapshot is dropped
before BGW rediscovers a replacement host.

The separate versioned local store persists only opaque item IDs, identity
kinds, Group metadata and normalized layout placements. It never persists
filesystem paths. This is a validated configuration model, not the live
Explorer Desktop Bridge: BGW does not yet accept Manager edits, hide icons,
replace native icon registrations, install a Shell extension/hook or create
Group popups. Inventory or state-load failure keeps the native desktop unchanged.


The folder watcher uses the documented Windows change-notification handle
sequence and retains a five-second bounded snapshot fallback for UNC roots
because remote filesystem notifications are not guaranteed. It watches only
top-level file/directory-name changes, not ordinary content writes, and
re-enumerates the full bounded snapshot after each signal; it does not rely
on the notification to identify the changed item.


## Read-only Shell namespace worker

The BGW host loop owns a separate DesktopNamespaceWorker in addition to the icon-extraction worker. It starts one STA thread, initializes COM before accepting work, and pumps its message queue between requests and while waiting for the result slot. The request and result channels each have capacity one; at most one namespace enumeration is active. It uses the same message-aware STA helper as the Shell icon worker.

BGW requests a snapshot when a verified Explorer host attaches, every 30 seconds, and after a User/Public Desktop filesystem-change refresh. Each request carries the current host generation. If Explorer is replaced while a Shell call is running, BGW discards the old generation's result instead of treating it as the new desktop snapshot.

The runtime reports only total entries, filesystem-backed entries, virtual folders, virtual items and skipped records. It does not log individual Shell names or paths. The snapshot is read-only and is dropped after aggregate reporting; the PIDL hashes are not persisted or used as Group member IDs.

If a namespace provider takes more than 20 seconds, BGW reports the delay and continues its host-monitor loop. The worker is not terminated or replaced: there is no safe way to kill a thread blocked inside an arbitrary Shell provider, and repeatedly spawning workers could leak unbounded threads. If the call eventually returns, that result is handled normally; otherwise the namespace capability remains unavailable for this BGW instance.

This is a root Shell namespace inventory, not an IFolderView snapshot. It does not yet observe the actual desktop view's filtered/visible entries, positions, ordering, selection or view state. It does not suppress icons, install hooks, render a presenter or enable Groups.

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


## Memory-only Shell icon acquisition pipeline

BGW now creates one dedicated Shell icon worker after startup. The worker
initializes an STA COM apartment before calling SHGetFileInfoW and balances
it with CoUninitialize when the worker exits. It pumps the STA message queue
while waiting for requests and while the bounded result queue is full, so
cross-apartment COM calls can be dispatched without blocking indefinitely. The host-monitor loop only
queues bounded requests and drains completed results; Shell extraction and
HICON rasterization never run on the host-monitor thread. BGW polls at 250 ms
while icon work is pending, then returns to the normal five-second host check.

- Work and result channels each hold at most 16 messages; there is one worker.
- The in-memory LRU cache is capped at 8 MiB and 256 icons. Each icon is limited
  to 512×512 straight-alpha RGBA.
- Cache keys include the opaque desktop item ID, identity-stability kind,
  requested Shell icon size, DPI bucket and theme generation. The initial
  host context is 96 DPI / theme generation 0 until BGW receives real DPI and
  appearance notifications.
- A verified Explorer-host change advances the worker generation and clears
  the cache. Results from older generations or removed inventory identities
  are discarded. Path-fallback icons are reacquired after every inventory
  refresh because their IDs cannot prove that the same file remains at a path;
  filesystem-ID entries are retained when still present.
- Missing, inaccessible, overlong or non-representable icons are recorded as
  failed for the current context so the monitor loop does not retry the same
  item on every five-second wake. A new host or icon context retries those
  failures; path-fallback items are also retried after inventory refresh.

The cache is process memory only. It does not write paths, pixels or icon
artwork into the local Group/layout file. The cache exposes a borrowed WLDE AdaptiveIconImageView over each RGBA
buffer without copying. This supplies compositor-ready pixels for a future
presenter, but it does not display them, hide native Explorer icons,
change Shell registrations or make Groups available. Crash-safe restoration
remains a separate prerequisite.


## Public API and reuse

Package: `wl_bgw_runtime` `0.1.0-alpha.1` (Rust 2021; declared MSRV 1.77).
It depends on `wl_wlde` and `wl_wlde_win32`. Add it as a path dependency
only when the consuming program is the BGW host or deliberately reuses its
local-state persistence contract:

```toml
[dependencies]
wl_bgw_runtime = { path = "../WinLook/rust/library/wl_bgw_runtime" }
```

The public entry point `run()` starts the current Windows BGW host loop.
The reusable configuration API is exported from the crate root:

- `DesktopItemIdentityRecord::from_inventory()` turns the current inventory
  into path-free identity records for state validation.
- `DesktopStateStore::for_current_user()` selects the normal per-user store;
  `at_path()` is for controlled tests/embedding and places path-security
  responsibility on the caller.
- `DesktopStateStore::load()` returns `Option<DesktopStateSnapshot>` after
  validating inventory identities, Groups, layout completeness and monitor
  topology. Missing state is `Ok(None)`; stale/corrupt/incompatible state is
  a typed error and is not silently overwritten.
- `DesktopStateStore::save()` validates against the current inventory and
  monitor snapshot before committing.
- `DesktopStateSnapshot::groups()` and `layout()` provide read-only views.

Example:

```rust
use wl_bgw_runtime::{DesktopItemIdentityRecord, DesktopStateStore};
use wl_wlde_win32::{enumerate_desktop_items, enumerate_monitors};

fn load_state() -> Result<(), Box<dyn std::error::Error>> {
    let inventory = enumerate_desktop_items()?;
    let topology = enumerate_monitors()?;
    let identities = DesktopItemIdentityRecord::from_inventory(&inventory);
    let store = DesktopStateStore::for_current_user()?;
    let _snapshot = store.load(&identities, &topology)?;
    Ok(())
}
```

The store is single-writer and contains configuration, not a recovery journal.
It stores opaque identities, identity kinds, Group metadata and normalized
placements only. Its CRC32 detects accidental damage; it is not encryption,
authentication or tamper protection. Protect the parent directory using the
interactive user's normal profile ACLs, and do not point `at_path()` at a
shared or untrusted location. Save only on the BGW owner thread/process.

The Shell icon worker, bounded channels and RGBA LRU cache are private
implementation details, not stable public types. The BGW runtime remains
Windows-only and is not intended to be embedded in the Manager UI. It does
not register startup, expose Manager IPC, suppress native icons or display
Groups. `cargo test -p wl_bgw_runtime` checks portable store/runtime tests;
the Explorer process and STA Shell path requires Windows CI/device validation.
See the [shared library catalog](../README.md) before choosing a dependency.

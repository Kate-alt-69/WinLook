# WLDE Plan B — WinLook-owned ExplorerBrowser view

## Decision

WLDE's first Shell-view host uses the documented IExplorerBrowser control inside a native parent HWND created and owned by WinLook. It does not acquire the live Explorer desktop view through IShellWindows::FindWindowSW(SWC_DESKTOP).

The Shell remains responsible for namespace enumeration, item activation, filesystem operations and security. WinLook owns only the parent window and the ExplorerBrowser control instance embedded in that window. This provides an independent Shell view that WLDE can size, destroy and reacquire without retaining COM pointers from Explorer's live desktop view.

This is an intermediate hosting layer, not a claim that WLDE already replaces the native desktop. The ordinary Explorer desktop remains visible and unchanged.

## Ownership and apartment contract

- The host creates a valid native parent HWND in the WinLook process.
- The HWND is created and used on one dedicated STA thread; the same thread initializes COM, creates IExplorerBrowser, navigates, obtains IFolderView, takes snapshots, resizes and destroys the control.
- The parent HWND must outlive OwnedDesktopBrowser. The control is destroyed before the parent HWND is destroyed and before the STA calls CoUninitialize.
- The adapter verifies that the parent HWND belongs to the current process and current thread. It also verifies that the child view HWND belongs to the same process/thread and is a descendant of the supplied parent.
- The API is deliberately non-Send/non-Sync. COM pointers, HWNDs and PIDLs do not cross to BGW coordination threads or the Manager process.
- The current OwnedDesktopBrowser accepts an existing parent HWND. Creating and hosting that HWND in BGW, including its message-pump integration, is a separate host-integration task and is not represented as complete here.

IExplorerBrowser::Initialize receives the parent HWND, child rectangle and folder settings. The adapter navigates to the virtual Desktop root using an absolute Desktop PIDL and SBSP_ABSOLUTE, then requests the control's own IFolderView using GetCurrentView.

## Snapshot contract

Each snapshot performs a fresh GetCurrentView(IID_IFolderView) and records the canonical IUnknown identity plus the current child view HWND. A change to either advances the adapter's monotonically increasing view_generation. The adapter then:

1. Calls IFolderView::ItemCount(SVGIO_ALLVIEW).
2. Rejects negative counts and counts above MAX_DESKTOP_BROWSER_ITEMS (4,096).
3. Calls IFolderView::Items(SVGIO_ALLVIEW, IID_IEnumIDList).
4. Enumerates exactly the announced number of child PIDLs, releasing each PIDL immediately after its position query.
5. Calls IFolderView::GetItemPosition for each item.
6. Performs an extra enumerator read and a second item count to reject a changing membership snapshot.
7. Reacquires the current view and rejects the snapshot if the view identity or child HWND changed during enumeration.

The result contains only the view generation, a zero-based enumeration ordinal valid for that snapshot only, an optional POINT from GetItemPosition, and the HRESULT when a position could not be obtained.

The ordinal is not a stable item identity and must never be persisted as a Group member key or layout target. No PIDL, COM pointer, HWND or source path is returned. Stable per-item identity mapping is a separate follow-up stage.

## Coordinate semantics

GetItemPosition returns the upper-left point in the folder's view. The point is scoped to the ExplorerBrowser view's own coordinate space; it is not an absolute virtual-screen position and must not be treated as monitor-relative coordinates. A future presenter must obtain the current child view rectangle, DPI and monitor topology, then explicitly transform coordinates. This adapter does not perform that transform.

## Lifecycle and invalidation

The adapter reacquires IFolderView for every snapshot and detects replacement by comparing canonical IUnknown identity and the view HWND. view_generation is updated when either changes. This is bounded polling-on-request, not a subscription to Shell navigation or view events. A future IExplorerBrowserEvents sink may provide prompt notifications, but it must remain on the owning STA, use bounded delivery, and still retain periodic reconciliation.

No worker is force-terminated when a COM call stalls. The host must own one bounded STA and a separately specified timeout/quarantine/recovery strategy; it must not spawn an unbounded series of replacement COM threads.

## Explicit non-goals for this stage

- No Progman message 0x052C, WorkerW parenting or undocumented desktop z-order assumptions.
- No Explorer process injection, DLL loading, cross-process subclassing or modification of explorer.exe on disk.
- No hiding the native desktop, changing native icon visibility, replacing the stock desktop or taskbar, or modifying Shell registrations.
- No Group tile presenter, icon suppression, native layout application or context-menu interception.
- No persistence of PIDLs, view ordinals, HWND values or COM pointers.
- No assertion that ExplorerBrowser has the same filtering, ordering, virtual-item policy or selection state as the live Explorer desktop.

## Milestone mapping and acceptance gates

- Plan B host foundation: implemented in rust/library/wl_wlde_win32/src/desktop_browser.rs; the parent HWND host integration and interactive Windows smoke test remain outstanding.
- M1: partially advanced. Item count, enumeration and view-relative position observations exist for the WinLook-owned view. Stable item identity, ordering guarantees, selection state, view mode and interactive behavior still need validation.
- M2: partially advanced. Each snapshot reacquires the current view and compares canonical COM identity plus child HWND. Event-driven invalidation, navigation notification and bounded recovery from a permanently blocked COM call remain outstanding.
- M3: unchanged. The BGW durable restoration journal writer, ACL/owner validation, locking, append/flush protocol and startup recovery remain required before native desktop mutations.
- M4/M6: unchanged. Adapter-specific capture/apply/inspect/ownership/verify/restore proof and full crash/fault-injection evidence remain required.
- M5: blocked. Keep native icon suppression, Group presentation and Shell mutation disabled until M3, M4 and M6 pass on supported builds.

## Primary API references

- IExplorerBrowser hosted Shell view: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iexplorerbrowser
- Initialize receives an owner HWND and requires a later Destroy: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iexplorerbrowser-initialize
- BrowseToIDList supports an absolute PIDL relative to the Desktop: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iexplorerbrowser-browsetoidlist
- GetCurrentView returns the requested current view interface: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-iexplorerbrowser-getcurrentview
- IFolderView::ItemCount, Items and GetItemPosition:
  https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifolderview-itemcount
  https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifolderview-items
  https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifolderview-getitemposition

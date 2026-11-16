# WLDE Explorer Desktop view acquisition — M0

**Status:** Read-only BGW probe implemented; interactive runtime ownership and compatibility validation pending  
**Date:** 2026-11-16  
**Scope:** Research/design for acquiring the actual Explorer-owned Desktop IFolderView from BGW. This is not an implementation or a permission to mutate Explorer.

## Decision summary

The best public-API starting point is the Shell's registered Desktop window, not a generic search for an Explorer folder window:

1. Create or obtain IShellWindows through CLSID_ShellWindows.
2. Call IShellWindows::FindWindowSW with the Desktop window class SWC_DESKTOP and SWFO_NEEDDISPATCH.
3. Query the returned Desktop IDispatch for IServiceProvider.
4. Request IShellBrowser using QueryService(SID_STopLevelBrowser, IID_IShellBrowser).
5. Call IShellBrowser::QueryActiveShellView.
6. Query that IShellView for IFolderView.

Microsoft documents SWC_DESKTOP as the Windows Desktop selector on Windows Vista and later. Microsoft documents the browser/site-service relationship and QueryActiveShellView; Raymond Chen's Microsoft engineering blog demonstrates the same Desktop acquisition chain. These contracts are sufficient to select this route for the M0 prototype. They do not prove that every WinLook-supported build exposes identical runtime behaviour.

## Acquisition details

### Shell window selection

- Use CLSID_ShellWindows and IShellWindows::FindWindowSW, not enumeration of every SWC_EXPLORER window followed by title/path heuristics.
- Set swClass = SWC_DESKTOP (0x8; Vista+) and request SWFO_NEEDDISPATCH so the call must return the window's IDispatch.
- Require the exact success result and a non-null dispatch object. Treat S_FALSE, E_NOINTERFACE, E_PENDING, access errors and other failures as "Desktop view unavailable"; do not silently fall back to an arbitrary Explorer folder.
- pvarLocRoot is documented as null or VT_EMPTY. For pvarLoc, the current Microsoft API page describes an absolute PIDL. Raymond Chen's sample uses a CSIDL_DESKTOP variant with SWC_DESKTOP; the independent CorralWindows experiment uses VT_EMPTY for both VARIANTs. The M0 probe must compare these calling forms on target builds and record the exact result. Do not confuse the virtual CSIDL_DESKTOP namespace root with CSIDL_DESKTOPDIRECTORY, the filesystem directory.
- The legacy FindWindowSW phwnd output is not the HWND used for process ownership validation. Obtain a native HWND from the returned IShellView via IOleWindow::GetWindow.

### Browser and view interfaces

- Query the returned IDispatch for IServiceProvider.
- Use IServiceProvider::QueryService(SID_STopLevelBrowser, IID_IShellBrowser) to obtain the browser that hosts the selected Desktop view.
- Use QueryActiveShellView and require S_OK plus a non-null IShellView.
- Query IID_IFolderView from that view. If the view does not expose IFolderView, report the capability as unavailable. Do not substitute SHGetDesktopFolder()->EnumObjects: that inventories the namespace and does not represent the live view's current collection or presentation state.
- Ask IFolderView::GetFolder(IID_IShellFolder) for the parent folder used by the view. Where the returned folder supports IPersistFolder2, inspect GetCurFolder as a secondary root-identity check. A folder path such as %USERPROFILE%\Desktop is not by itself proof of the virtual Desktop namespace root. If the selected view cannot be shown to belong to the Desktop selection, fail closed.

## Proving the actual Explorer owner

WinLook already has a modern Explorer desktop-host identity probe based on GetShellWindow, GetWindowThreadProcessId, process image path, session ID and process creation time. Reuse that record rather than inventing a second process-identity system.

1. Query IOleWindow::GetWindow on the acquired IShellView.
2. Require a valid returned HWND and use GetWindowThreadProcessId to obtain its owner PID.
3. Require that PID to equal the currently monitored Explorer desktop-host PID and that the host generation/process creation-time record is still current.
4. Require the view and BGW to belong to the same interactive user session. Do not elevate BGW to work around an access failure.
5. Recheck the host generation after the COM work, before publishing any result. If Explorer changed during the request, discard the result.

The PID match is a necessary ownership check, not proof that the view object will survive an Explorer restart. The COM interfaces are generation-scoped and must be reacquired after host restart or view recreation. Do not key ownership on window class names such as Progman, WorkerW or SHELLDLL_DefView; those are not part of this acquisition contract.

## COM apartment and lifetime

- Perform the sequence on a dedicated BGW STA worker, using the existing CoInitializeEx(COINIT_APARTMENTTHREADED) and balanced CoUninitialize pattern.
- Reuse the existing message-aware STA wait helper. Do not perform COM work in the BGW Explorer-monitor loop.
- Keep IShellWindows, IDispatch, IServiceProvider, IShellBrowser, IShellView, IFolderView, IShellFolder and enumerator references on that STA. Release them there before returning.
- Return only bounded copied data and the host/view generation to BGW. Never send COM interface pointers, raw HWNDs or PIDLs to other threads as durable identities.
- A Shell/provider call may hang or be rejected while Explorer is busy. Preserve the current one-in-flight worker rule and diagnostic timeout. A timeout does not make the COM call cancellable: do not kill the thread, create replacement workers, or block the BGW monitor loop.
- If COM activation or a Shell call returns E_ACCESSDENIED, preserve the native desktop and report unavailable. The CorralWindows experiment recorded an access-denied result in a restricted execution context and successful observation in its interactive user session; this is a warning about test context, not a reason to elevate or bypass access controls.

## M1 read-only snapshot contract

Once M0 ownership is proven, the first consumer of the view remains read-only:

- Use IFolderView::ItemCount(SVGIO_ALLVIEW) for the displayed-view count and IFolderView::Items(SVGIO_ALLVIEW, IID_IEnumIDList) for the corresponding item enumeration.
- Request the parent IShellFolder through GetFolder(IID_IShellFolder) and interpret enumerated child PIDLs only in that parent-folder scope.
- Use GetItemPosition only as a read operation. Microsoft defines the returned POINT as the item's upper-left corner in the folder view; do not assume it is an absolute virtual-screen point or persist it as a monitor coordinate until per-build/DPI/monitor tests establish the mapping.
- IFolderView enumeration is not an atomic transaction. Bound the item count, capture host and view generations, detect count/view changes during the read, and discard inconsistent snapshots rather than publishing partial state as authoritative.
- Enumeration order is not item identity. Do not persist a view index or view PIDL hash as a durable Group/layout key. Continue to resolve filesystem identities through the existing inventory identity provider; treat virtual namespace identities as ephemeral until a separate stable mapping is designed.
- Do not call SelectItem, SelectAndPositionItems, SetCurrentViewMode or any other mutator during M0/M1. Do not hide or replace icons, register Shell objects, install hooks, inject code, alter the registry or enable Groups.

## External implementation evidence

The MIT-licensed [CorralWindows](https://github.com/zoot661/CorralWindows/tree/d548e36f8aa1031098166d2076513cc649d8c51f) experiment (source commit d548e36f8aa1031098166d2076513cc649d8c51f, inspected 2026-11-16) uses IShellWindows with SWC_DESKTOP, IShellBrowser, IFolderView, GetFolder, Items(SVGIO_ALLVIEW) and GetItemPosition. Its README reports live interactive-session observations on Windows 10/11, including Recycle Bin, but explicitly leaves multi-monitor/DPI and some empty-view cases unverified. It is a useful implementation reference, not a Microsoft support guarantee or a substitute for WinLook's own testing. Do not copy its code without reviewing its MIT licence and adapting the stronger BGW ownership/lifetime boundaries above.

## M0 validation matrix and completion gate

Record, for every run: OS edition, display version, build, UBR, architecture, servicing channel, BGW session ID, Explorer PID and creation time, HRESULT from every acquisition step, view HWND owner PID, parent-folder identity result, item count, and whether Explorer/view restart occurred during the request.

Minimum modern matrix:

- Windows 10 22H2 (19045); include Windows 10 1809 (17763) only if it remains an intended supported target.
- Windows 11 21H2 (22000), 22H2 (22621), 23H2 (22631), 24H2 (26100), and the current supported 25H2 servicing build.
- x64 first; add ARM64 only if it is a supported WinLook target.

For each build, test a normal unelevated interactive user session, Explorer restart, view recreation, hidden desktop icons, an empty Desktop, user and Public Desktop items, virtual items such as Recycle Bin, multiple monitors, mixed DPI, sign-out/sign-in and Remote Desktop/session transitions. Test both observed pvarLoc forms rather than assuming they are interchangeable.

**M0 is complete only when** the read-only probe acquires the same Desktop view repeatedly, its view HWND owner matches the exact Explorer process generation BGW already trusts, the parent folder is confirmed as the Desktop namespace, and re-acquisition behaves correctly after Explorer/view recreation on every supported build. No user-machine or VM runtime validation has been performed by this research pass.

Until that gate passes, M0 is **partial** and M1 must not be implemented against an unverified interface. Native presentation remains blocked by the separate M3/M4/M6 restoration and ownership gates in [restoration.md](restoration.md).


## Runtime probe implementation checkpoint — 2026-11-17

The research design now has a first implementation on the WLDE inventory PR:

- \`rust/library/wl_wlde_win32/src/desktop_view.rs\` exports \`probe_desktop_folder_view()\`. It uses the Shell COM chain above and keeps all interfaces and temporary PIDLs local to the calling STA.
- \`FindWindowSW\` is attempted with the \`CSIDL_DESKTOP\` VARIANT form from Raymond Chen's Microsoft engineering sample, then with \`VT_EMPTY\` as used by the pinned CorralWindows experiment. The chosen form is returned as a small enum so the BGW diagnostic can show which one worked on that system.
- The returned \`IShellView\` HWND is checked with \`GetWindowThreadProcessId\`; its owner PID must equal the exact \`DesktopHostIdentity\` passed from BGW. The probe then re-discovers the host after the COM calls and rejects a host change.
- The view's \`IFolderView::GetFolder(IID_IShellFolder)\` result must expose \`IPersistFolder2\`; its current-folder PIDL is compared with the PIDL returned for \`CSIDL_DESKTOP\`. \`ILIsEqual\` is a binary comparison, so a non-identical encoding is deliberately treated as unavailable rather than guessed equivalent.
- The bounded DesktopViewWorker carries the exact host identity with each request and runs the IFolderView probe on its own message-aware STA. The independent DesktopNamespaceWorker continues to enumerate the root Shell namespace on a different STA. Each worker is single-flight, generation-tagged and has its own diagnostic timeout; a stuck view probe cannot block namespace enumeration. The view worker returns only the owner PID and locator enum. COM pointers, HWNDs and PIDLs remain on the view worker's STA.
- A failed view probe does not suppress the independent namespace inventory. The BGW only reports the view as unavailable and keeps native Explorer presentation unchanged.

**M0 remains PARTIAL.** GitHub Windows CI can verify compilation, but this session has not executed the probe in an interactive Windows VM/device. Before M0 can be marked complete, validate both locator forms, actual view-window ownership, Desktop-root PIDL matching, Explorer/view recreation, empty/hidden icon states, multiple monitors, mixed DPI and every supported OS build/UBR. M1 remains gated on that runtime evidence.


### Architecture-sensitive VARIANT ABI correction — 2026-11-17

The manual COM binding represents the VARIANT value union with two pointer-sized
words, matching the native BRECORD arm's two-pointer capacity. Therefore the
complete VARIANT is 24 bytes on x64 and 16 bytes on x86. The Windows-specific
test asserts the appropriate size for the compilation target. Although this
probe only passes VT_EMPTY and VT_I4 values, it must still allocate and align the
full native VARIANT object because FindWindowSW receives pointers to it.

Source evidence is recorded as WLDE-064 in SOURCES.md.


### Raw COM vtable slot verification — 2026-11-17

The probe uses explicit absolute COM vtable slots. They were checked against
the Windows SDK IDL method order and the Microsoft Node.js Tools interop sample:

- \`IShellWindows::FindWindowSW\`: slot 15 (seven inherited \`IDispatch\`
  entries, then \`Count\`, \`Item\`, \`_NewEnum\`, \`Register\`,
  \`RegisterPending\`, \`Revoke\`, \`OnNavigate\`, \`OnActivated\`,
  \`FindWindowSW\`).
- \`IServiceProvider::QueryService\`: slot 3 after \`IUnknown\`.
- \`IShellBrowser::QueryActiveShellView\`: slot 15 after \`IUnknown\` and
  \`IOleWindow\`; the preceding twelve \`IShellBrowser\` methods are in the
  same order as Microsoft's Node.js Tools VTableGap01 through
  VTableGap12 declarations.
- \`IShellView::GetWindow\`: slot 3 after \`IUnknown\`.
- \`IFolderView::GetFolder\`: slot 5 after \`IUnknown\`
  (\`GetCurrentViewMode\`, \`SetCurrentViewMode\` precede it).
- \`IPersistFolder2::GetCurFolder\`: slot 5 after \`IUnknown\`
  (\`IPersist::GetClassID\` and \`IPersistFolder::Initialize\` precede it).

The code keeps named constants for both slot-15 calls and a regression test.
Source evidence is WLDE-065 in [SOURCES.md](../references/SOURCES.md).

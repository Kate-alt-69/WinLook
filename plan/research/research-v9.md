# WinLook Architectural Blueprint & Deep Systems Research (Win95 to Win11) - Edition V
## For the `/plan/` Architecture Series: `wl_translation`, `wl_wlde`, `wl_animate`, and System Interfaces

---

## 1. Executive Summary & Design Philosophy

The WinLook project is designed to eliminate bloated customization software (utilities requiring 200MB–500MB Chromium/Electron runtimes or invasive kernel detours) by re-implementing Windows UI orchestration directly through native operating system interfaces. 

WinLook’s foundational philosophy is **Reversible Non-Invasive Orchestration**:
1. **Never Patch System Binaries on Disk**: No patching of `explorer.exe`, `uxtheme.dll`, or `win32k.sys`.
2. **Never Touch Critical Debugging or Reset Interfaces**: Avoid `WerFault.exe`, crash dumps, reset/recovery hooks, or Windows Update orchestration.
3. **Never Interfere with Settings App or Critical Shell System Packages**: Keep core security, accounts, and system configuration intact.
4. **Transform Basic OS Elements into Rich Visual Experiences**: Utilize native handle properties (`HWND`), Desktop Window Manager (DWM) composition surfaces, window subclassing via `comctl32.dll`, and shell hook subscriptions to animate, reposition, and restyle components safely.
5. **Absolute Reversibility**: Any modification applied by WinLook must be immediately restorable upon process shutdown or crash via state journals.

---

## 2. Windows Subsystem Architecture & Execution Boundaries (Win95 to Win11)

### 2.1 Ring 3 to Ring 0 Transitions and Graphics Migration
Across the evolution of Windows, the boundary between user-mode application logic and graphics rendering shifted fundamentally:
* **Windows 95/98/ME (Hybrid 16/32-bit)**: User and GDI graphics routines executed in user mode (`USER32.DLL`, `GDI32.DLL`, `USER.EXE`, `GDI.EXE`) with direct interaction with Ring 0 virtual device drivers (`VMM32.VXD`).
* **Windows NT 3.51 (Early User-Mode NT)**: Window management and GDI were isolated in Ring 3 inside `csrss.exe`. IPC context switches between client applications and `csrss.exe` caused severe performance bottlenecks.
* **Windows NT 4.0 through Windows 2000/XP (Kernel Graphics Consolidation)**: Microsoft moved USER and GDI into Ring 0 driver space as `win32k.sys`. System calls shifted to trap dispatches (`int 0x2e` on x86, `sysenter`/`syscall` on modern architectures).
* **Windows Vista through Windows 11 (DWM Hardware Redirection)**: Hardware acceleration and composition moved to the Desktop Window Manager (`dwm.exe`). Rather than writing to a shared screen framebuffer, windows draw to off-screen DirectX redirection surfaces (`DWMWA_REDIRECTIONBITMAP_ALPHA`). DWM composites all windows using Direct3D shaders, drop shadows, rounded corners, and materials (Mica, Acrylic).

### 2.2 Desktop Isolation & Window Station Boundaries
Windows security partitions UI sessions via Window Stations and Desktops:
* **Window Station (`\WinSta0`)**: The only interactive window station containing the user's interactive display, keyboard, and mouse.
* **Default Desktop (`\WinSta0\Default`)**: The standard user desktop where `explorer.exe`, application windows, and taskbars reside.
* **Winlogon Desktop (`\WinSta0\Winlogon`)**: The isolated secure desktop where credential providers, `LogonUI.exe`, and the Secure Attention Sequence (`Ctrl+Alt+Delete`) execute. Standard user-mode hooks cannot cross this boundary.
* **Screen-Saver Desktop (`\WinSta0\Screen-Saver`)**: The legacy isolated desktop used for screen-savers prior to modern lock screen integration.

---



### 2.3 The In-Process vs. Out-of-Process Customization Boundary: Why External Utilities Cannot Subclass Third-Party Windows
A foundational rule of the Win32 subsystem is **process memory isolation**:
* **Window Subclassing (`SetWindowSubclass`, `SetWindowLongPtr(GWLP_WNDPROC)`)**: The Window Manager in `win32k.sys` dispatches window messages directly to user-mode function pointers stored in thread-specific structures. Because code pointers cannot be executed across separate process address spaces without crashing the target process, `win32k.sys` **strictly forbids setting a window subclass procedure on a window owned by another process**.
* **Implications for Shell Customization**: An external utility (`winlook.exe` or `winlook.bg.exe`) cannot subclass windows owned by `explorer.exe` (such as `Shell_TrayWnd`, the desktop `SysListView32`, or Explorer breadcrumb bars) without injecting dynamic-link libraries (DLLs) into Explorer's address space.
* **WinLook's Non-Invasive Approach**: To maintain absolute operating system stability and avoid anti-cheat or antivirus interference, WinLook **never injects DLLs into external system processes**. Instead, WinLook uses:
  1. **Out-of-Process Event Monitoring**: `RegisterShellHookWindow` and out-of-context `SetWinEventHook`.
  2. **Window Management & DWM APIs**: `SetWindowPos`, `ShowWindow`, and `DwmSetWindowAttribute`.
  3. **Shell Protocols & AppBars**: `SHAppBarMessage` to negotiate custom screen bounds.
  4. **Native Decoupled Hosting (Path B)**: Hiding native shell surfaces and creating WinLook's own native windows, which CAN be cleanly subclassed with `SetWindowSubclass` because WinLook is the creator and owner.

---

## 3. The Addressable Object Namespace (`windows.<version>.<subsystem>.<item>.<property>`)

WinLook organizes all controllable elements into a hierarchical, strongly-typed configuration tree:

```
windows
├── <version>               (all | 95 | 98 | 2000 | xp | vista | 7 | 8 | 10 | 11)
│   ├── <subsystem>         (shell | dwm | uxtheme | gdi | user32 | comctl32 | lockscreen)
│   │   ├── <item>          (taskbar | window | start_menu | desktop | caption | icon)
│   │   │   └── <property>  (visibility | bounds | corner_radius | backdrop | color | opacity)
```

Each namespace path maps directly to a native handler in the `wl_translation` registry, supporting queries (`get()`), assignments (`set()`), and event callbacks (`on_ready()`, `on_change()`).

---

## 4. Shell & Taskbar Decoupling: Pinned vs. Running vs. Start

A major limitation of monolithic customization utilities is treating the Windows Taskbar (`Shell_TrayWnd`) as an indivisible monolithic bar. WinLook divides taskbar management into two distinct architectural paths [cite: 98]:

```
                                  +---------------------------------------+
                                  | WinLook Taskbar Orchestrator (WLWM)   |
                                  +-------------------+-------------------+
                                                      |
                         +----------------------------+----------------------------+
                         |                                                         |
                         v                                                         v
         +-------------------------------+                         +-------------------------------+
         | Path A: Explorer Sub-HWND     |                         | Path B: Standalone WLDE Bar   |
         | Translation Adapter           |                         | Canvas Bar                    |
         +---------------+---------------+                         +---------------+---------------+
                         |                                                         |
         +---------------+---------------+                         +---------------+---------------+
         | - Shell_TrayWnd               |                         | - Custom DirectComposition    |
         | - ReBarWindow32               |                         | - Custom XAML Islands / WinUI |
         | - MSTaskSwWClass (Running)    |                         | - Virtual Pinned Launchers    |
         | - TaskListThumbnailWnd        |                         | - RegisterShellHookWindow sync|
         | - TrayNotifyWnd (SysTray)     |                         | - Arbitrary screen floating   |
         +-------------------------------+                         +-------------------------------+
```

### 4.1 Path A: Native Explorer Taskbar Decomposition
Using Win32 hierarchy traversal without modifying `explorer.exe` on disk [cite: 23]:
1. Find top-level taskbar: `FindWindowW(L"Shell_TrayWnd", NULL)`.
2. Locate child rebar control: `FindWindowExW(hTray, NULL, L"ReBarWindow32", NULL)`.
3. Locate running window task switch band: `FindWindowExW(hReBar, NULL, L"MSTaskSwWClass", NULL)`.
4. In Windows 10, pinned shortcuts and running apps live within `MSTaskListWClass`. WinLook inspects item states via `TB_GETBUTTON` messages sent to the internal toolbar control, enabling/disabling visibility for individual buttons or shifting button rects during `WM_WINDOWPOSCHANGING`.
5. Start Button isolation: `FindWindowExW(hTray, NULL, L"Start", NULL)` or secondary start buttons on multi-monitor arrays (`Shell_SecondaryTrayWnd`). WinLook preserves the native Windows logo glyph while reparenting or translating its render rectangle [cite: 98].

### 4.2 Path B: Full WLDE Taskbar Canvas
When arbitrary floating offsets, custom docking curves, or complex non-linear layouts are desired, WinLook initiates an independent `wlde.taskbar` window:
1. Shell Hook Synchronization: BGW registers `RegisterShellHookWindow` to receive `HSHELL_WINDOWCREATED`, `HSHELL_WINDOWDESTROYED`, and `HSHELL_WINDOWACTIVATED` in real time [cite: 65].
2. The native taskbar is hidden cleanly via `ShowWindow(hShellTrayWnd, SW_HIDE)`.
3. WLDE renders custom running application pills, badges, progress bars (reading taskbar progress via `ITaskbarList3`), and tray notification mirrors.
4. If WinLook terminates, the BGW watchdog immediately invokes `ShowWindow(hShellTrayWnd, SW_SHOW)` to restore the native shell instantly.

---


## 5. Live Desktop Presentation & Wallpaper Mechanics (`wl_wlde`)

### 5.1 The `Progman` -> `WorkerW` Split Injection
To render animated scenes, 3D shaders, or dynamic widgets directly on the desktop without occluding desktop icons, WinLook uses the undocumented Explorer split-window protocol:

```
[Progman ("Program Manager")]
     |
     +--> SendMessageTimeoutW(hProgman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &result)
     |
     V
[WorkerW (Upper)] -> Contains SHELLDLL_DefView -> SysListView32 ("FolderView") [Icons]
     |
[WorkerW (Lower - Spawned Behind)] <--- WinLook reparents WLDE Render Canvas here!
```

**Execution Steps:**
1. Retrieve handle: `HWND hProgman = FindWindowW(L"Progman", NULL)`.
2. Dispatch message `0x052C` with `SendMessageTimeoutW(hProgman, 0x052C, 0x0000000D, 0, SMTO_NORMAL, 1000, &dwResult)`.
3. Iterate top-level windows using `EnumWindows`. Locate the `WorkerW` instance that sits directly behind `SHELLDLL_DefView`.
4. Set the WLDE renderer window parent to this `WorkerW` via `SetParent(hWldeCanvas, hWorkerW)`.
5. Desktop icons retain hit-testing, marquee selection, drag-and-drop, and shell context menus, while the dynamic background animates underneath at full display refresh rate.

### 5.2 Resource Governance (`wl_cpuv` & `wl_gpuv`)
Animated wallpapers must never degrade gaming or application performance:
* **Occlusion Detection:** If a foreground application window is maximized (`GetWindowPlacement` or `GetWindowRect` covers `GetMonitorInfo`), `wl_wlde` immediately suspends rendering cycles and pauses shader clocks [cite: 27].
* **Battery / Fullscreen Detection:** WinLook monitors `SHQueryUserNotificationState`. If `QUNS_RUNNING_D3D_FULL_SCREEN` or `QUNS_PRESENTATION_MODE` is returned, rendering freezes on the last displayed frame to ensure zero CPU/GPU cycles are consumed during intense tasks [cite: 27].

---


## 6. Windows Lock Screen, LogonUI & Wallpaper Ingestion

### 6.1 Authentication Boundaries and LogonUI
* The Windows Lock Screen is managed by `LogonUI.exe`, which executes within the isolated `\WinSta0\Winlogon` desktop [cite: 70].
* User-mode hooks and DWM injection cannot interact with `LogonUI.exe` controls due to Mandatory Integrity Control (System Integrity / `S-1-5-18`) and SAS hardware protection [cite: 44, 93].

### 6.2 The Four Lock Screen Implementation Tactics
1. **Administrative Lock Screen Policy (Win8 - Win11):** Writing a target image path to `HKLM\SOFTWARE\Policies\Microsoft\Windows\Personalization\LockScreenImage` forces `LogonUI.exe` to render the selected asset.
2. **WinRT User Profile API (Standard User, Win10/Win11):** WinLook invokes `Windows.System.UserProfile.LockScreen.SetImageFileAsync(file)` via the Windows Runtime C++/WinRT or Rust `windows` crate projection, updating the lock screen wallpaper without requiring UAC elevation.
3. **Spotlight Asset Extraction:** WinLook monitors `%LocalAppData%\Packages\Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy\LocalState\Assets`. Assets exceeding 400 KB with landscape dimensions (1920x1080 or 3840x2160) are automatically identified via image header parsing and cataloged into WinLook's Wallpaper Store.
4. **Session Notification & Pre-Lock Animation:** BGW calls `WTSRegisterSessionNotification(hBgWnd, NOTIFY_FOR_THIS_SESSION)`. When `WM_WTSSESSION_CHANGE` fires with `WTS_SESSION_LOCK`, WinLook executes an ambient pre-lock fade/blur transition before the kernel switches active desktops to `Winlogon` [cite: 93].

---


## 7. The 250 Architectural Implementation Tactics for WinLook

### Category A: Window Architecture, Non-Client Styling & Hit-Testing (Tactics 1–10)

#### Tactic 1: Dynamic Window Region Clipping via `SetWindowRgn`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowRgn`, `CreateRoundRectRgn`, `CreatePolygonRgn`), `GDI32.DLL`.
* **How-To & Safe Implementation**:
  Window regions enforce custom non-rectangular clipping directly inside `win32k.sys` at the kernel level.
  1. Construct a GDI region handle using `CreateRoundRectRgn(x1, y1, x2, y2, round_w, round_h)` or `CreatePolygonRgn`.
  2. Apply the region to the target window using `SetWindowRgn(hwnd, hrgn, TRUE)`.
  3. **Safety & Reversibility**: The operating system takes ownership of the `HRGN` handle once passed to `SetWindowRgn`—do not call `DeleteObject(hrgn)` on success. To reverse, pass `SetWindowRgn(hwnd, NULL, TRUE)` to instantly restore rectangular geometry.

#### Tactic 2: Non-Client Geometry Recalculation via `WM_NCCALCSIZE`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`DefWindowProcW`, `SetWindowPos`), `win32k.sys`.
* **Process Boundary Notice**:
  `WM_NCCALCSIZE` message interception via `SetWindowSubclass` applies to windows created and owned by the calling application (such as WinLook's configuration manager, custom AppBars, or companion windows). For third-party windows, non-client customization without DLL injection is performed using DWM attributes (`DwmExtendFrameIntoClientArea`, `DWMWA_CAPTION_COLOR`, `DWMWA_WINDOW_CORNER_PREFERENCE`) or external transparent overlay frames.
* **How-To & Safe Implementation**:
  When creating borderless or fully customized title bars on windows owned by the application:
  1. Subclass the target window using `SetWindowSubclass`.
  2. Handle `WM_NCCALCSIZE`. When `wParam == TRUE`, cast `lParam` to `LPNCCALCSIZE_PARAMS`.
  3. Returning `0` tells `win32k.sys` that the client rectangle occupies the entire window area, stripping standard borders without losing standard sizing shadows.
  4. **Safety**: Trigger frame metrics updates using `SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED)`.

#### Tactic 3: Custom Non-Client Frame Painting via `WM_NCPAINT`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowDC`, `ReleaseDC`), `GDI32.DLL`.
* **How-To & Safe Implementation**:
  1. In windows where DWM frame rendering is disabled or unsupported (Win95–2000, or classic fallback), intercept `WM_NCPAINT`.
  2. Obtain the full window device context via `GetWindowDC(hwnd)` (which covers both non-client borders and client areas).
  3. Draw custom frame borders, gradients, and custom title bars.
  4. Always release the device context using `ReleaseDC(hwnd, hdc)` and return `0`.
  5. **Safety**: On Windows Vista+, enable `DWMWA_ALLOW_NCPAINT` if rendering alongside DWM composition to prevent frame clipping artifacts.

#### Tactic 4: Hit-Test Redirection via `WM_NCHITTEST`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`DefWindowProcW`).
* **How-To & Safe Implementation**:
  When title bars or sizing borders are drawn in the client area, `WM_NCHITTEST` maps screen coordinates back to window actions:
  1. Extract cursor coordinates from `lParam` (`GET_X_LPARAM(lParam)`, `GET_Y_LPARAM(lParam)`).
  2. Map point to window client space via `ScreenToClient(hwnd, &pt)`.
  3. Return `HTCAPTION` (2) over custom draggable headers, `HTLEFT` (10), `HTRIGHT` (11), `HTTOP` (12), or `HTBOTTOM` (15) along margins.
  4. **Safety**: Delegate non-customized points to `DefSubclassProc` to preserve default button hit-testing.

#### Tactic 5: Layered Window Alpha Blending via `WS_EX_LAYERED`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowLongPtrW`, `SetWindowLongPtrW`, `SetLayeredWindowAttributes`).
* **How-To & Safe Implementation**:
  1. Retrieve current window extended styles using `GetWindowLongPtrW(hwnd, GWL_EXSTYLE)`.
  2. Apply `WS_EX_LAYERED` (0x00080000): `SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED)`.
  3. Call `SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA)` where `alpha` ranges from `0` (transparent) to `255` (opaque).
  4. **Safety**: Check that the window is not a child window (`WS_CHILD`) on versions prior to Windows 8, as layered child windows were only introduced in Windows 8.

#### Tactic 6: Per-Pixel Alpha Surface Updates via `UpdateLayeredWindow`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`UpdateLayeredWindow`), `GDI32.DLL` (`CreateDIBSection`, `AlphaBlend`).
* **How-To & Safe Implementation**:
  Allows complex non-rectangular translucent UI overlays with anti-aliased shadows:
  1. Allocate a 32-bit ARGB DIB section using `CreateDIBSection`.
  2. Render the custom UI with pre-multiplied alpha values.
  3. Call `UpdateLayeredWindow(hwnd, hdc_screen, &pt_dst, &sz_window, hdc_mem, &pt_src, 0, &blend_function, ULW_ALPHA)`.
  4. **Safety**: Layered windows updated via `UpdateLayeredWindow` do not receive `WM_PAINT` messages; all updates are pushed through this API.

#### Tactic 7: Native Window Margin Inset Expansion via `DwmExtendFrameIntoClientArea`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmExtendFrameIntoClientArea`), `dwmapi.h`.
* **How-To & Safe Implementation**:
  Seamlessly extends glass or modern background materials into the client canvas:
  1. Initialize a `MARGINS` structure: `MARGINS margins = { -1, -1, -1, -1 };` (or specify positive left, right, top, bottom insets).
  2. Call `DwmExtendFrameIntoClientArea(hwnd, &margins)`. Passing `-1` creates a complete "sheet of glass" canvas.
  3. Ensure the client area background is painted with a black brush (`CreateSolidBrush(RGB(0, 0, 0))`) in `WM_PAINT` so DWM treats the black pixels as transparent glass compositing surfaces.

#### Tactic 8: System Menu Stripping and Custom Command Injection via `GetSystemMenu`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetSystemMenu`, `AppendMenuW`, `RemoveMenu`, `EnableMenuItem`).
* **How-To & Safe Implementation**:
  1. Obtain a handle to the window system menu: `HMENU hmenu = GetSystemMenu(hwnd, FALSE)`.
  2. To strip unwanted items (e.g., disable Maximize/Restore on floating widgets), call `RemoveMenu(hmenu, SC_MAXIMIZE, MF_BYCOMMAND)`.
  3. To inject custom WinLook options, call `AppendMenuW(hmenu, MF_STRING, ID_WINLOOK_CONFIG, L"WinLook Options...")`.
  4. Intercept `WM_SYSCOMMAND` in the subclass procedure to capture `ID_WINLOOK_CONFIG`.
  5. **Safety**: Passing `GetSystemMenu(hwnd, TRUE)` immediately restores the default OS system menu.

#### Tactic 9: Window Boundary Enforcing via `WM_GETMINMAXINFO`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`MINMAXINFO`).
* **How-To & Safe Implementation**:
  Prevents layout deformation when users or the OS resize customized floating bars:
  1. In the window procedure, handle `WM_GETMINMAXINFO`.
  2. Cast `lParam` to `LPMINMAXINFO pMMI = (LPMINMAXINFO)lParam;`.
  3. Set `pMMI->ptMinTrackSize.x = min_w;`, `pMMI->ptMinTrackSize.y = min_h;`.
  4. Set `pMMI->ptMaxTrackSize.x = max_w;`, `pMMI->ptMaxTrackSize.y = max_h;`.
  5. **Safety**: Never restrict bounds smaller than title bar icon minimums unless `WS_THICKFRAME` is removed.

#### Tactic 10: Non-Client Caption Button Suppression via `SetWindowLongPtr`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowLongPtrW`, `SetWindowLongPtrW`, `GWL_STYLE`).
* **How-To & Safe Implementation**:
  1. Read current window style: `LONG_PTR style = GetWindowLongPtrW(hwnd, GWL_STYLE)`.
  2. Strip system buttons: `style &= ~(WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);`.
  3. Write back: `SetWindowLongPtrW(hwnd, GWL_STYLE, style)`.
  4. Update frame: `SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED)`.
  5. **Safety**: Retain `WS_BORDER` or `WS_THICKFRAME` if resizing capability is required.

### Category B: Desktop Window Manager (DWM) Modern Frame Decoration & Hardware Backdrops (Tactics 11–20)

#### Tactic 11: Dynamic Window Corner Rounding Control (`DWMWA_WINDOW_CORNER_PREFERENCE`)
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 33).
* **How-To & Safe Implementation**:
  1. Define preference: `0 = DWMWCP_DEFAULT`, `1 = DWMWCP_DONOTROUND` (Square), `2 = DWMWCP_ROUND`, `3 = DWMWCP_ROUNDSMALL`.
  2. Call: `DwmSetWindowAttribute(hwnd, 33, &pref, sizeof(pref))`.
  3. **Safety**: Resolve function via `GetProcAddress(LoadLibraryW(L"dwmapi.dll"), "DwmSetWindowAttribute")` to avoid crashes on Windows 10 or earlier.

#### Tactic 12: Hardware-Accelerated System Backdrops (`DWMWA_SYSTEMBACKDROP_TYPE`)
* **Target Versions**: Windows 11 (Build 22621+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 38).
* **How-To & Safe Implementation**:
  1. Define backdrop type: `DWMSBT_AUTO = 0`, `DWMSBT_NONE = 1`, `DWMSBT_MAINWINDOW = 2` (Mica), `DWMSBT_TRANSIENTWINDOW = 3` (Acrylic), `DWMSBT_TABBEDWINDOW = 4` (Mica Alt).
  2. Invoke `DwmSetWindowAttribute(hwnd, 38, &backdrop_type, sizeof(backdrop_type))`.
  3. Combine with `DwmExtendFrameIntoClientArea` with margins `{-1, -1, -1, -1}` for full-window material coverage.

#### Tactic 13: Immersive Dark Mode Activation (`DWMWA_USE_IMMERSIVE_DARK_MODE`)
* **Target Versions**: Windows 11 (Build 22000+) officially documented; Windows 10 (Build 17763–19045) via undocumented compatibility attributes.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`).
* **Subsystem Architecture & Build Requirements**:
  * **Windows 11 (Build 22000+)**: `DWMWA_USE_IMMERSIVE_DARK_MODE` (Attribute ID 20) is officially documented and guaranteed by Microsoft Learn.
  * **Windows 10 (Build 19041–19045, 2004 through 22H2)**: Attribute ID 20 functions as an undocumented compatibility flag before formal SDK documentation.
  * **Windows 10 (Build 17763–18363, 1809 through 1909)**: Attribute ID 19 (`DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1 = 19`) must be used as an undocumented fallback.
* **How-To & Safe Implementation**:
  1. Query OS build via `RtlGetVersion`.
  2. Attempt `DwmSetWindowAttribute(hwnd, 20, &dark, sizeof(dark));`.
  3. If status != S_OK and build is between 17763 and 18363, fallback to Attribute ID 19.
  4. Follow with `SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);` to repaint.

#### Tactic 14: Custom Window Border Color Injection (`DWMWA_BORDER_COLOR`)
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 34).
* **How-To & Safe Implementation**:
  1. Define color: `COLORREF border_color = RGB(r, g, b);` or `DWMWA_COLOR_DEFAULT` (0xFFFFFFFF).
  2. Call: `DwmSetWindowAttribute(hwnd, 34, &border_color, sizeof(border_color))`.
  3. **Safety**: Setting `0xFFFFFFFE` (`DWMWA_COLOR_NONE`) completely hides the border line without affecting window shadow geometry.

#### Tactic 15: Custom Title Bar Caption Color Assignment (`DWMWA_CAPTION_COLOR`)
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 35).
* **How-To & Safe Implementation**:
  1. Define color: `COLORREF caption_color = RGB(r, g, b);`.
  2. Call: `DwmSetWindowAttribute(hwnd, 35, &caption_color, sizeof(caption_color))`.
  3. Replaces default system caption background with custom brand/theme tinting while preserving hardware acceleration.

#### Tactic 16: Custom Title Bar Text Color Overrides (`DWMWA_TEXT_COLOR`)
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 36).
* **How-To & Safe Implementation**:
  1. Define text color: `COLORREF text_color = RGB(r, g, b);`.
  2. Call: `DwmSetWindowAttribute(hwnd, 36, &text_color, sizeof(text_color))`.
  3. Ensures contrast against customized caption background colors.

#### Tactic 17: Window Cloaking for Seamless Transformation Proxies (`DWMWA_CLOAK`)
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 13).
* **How-To & Safe Implementation**:
  1. Call: `BOOL cloak = TRUE; DwmSetWindowAttribute(hwnd, 13, &cloak, sizeof(cloak));`.
  2. The window is concealed from the user's view by DWM while retaining its active rendering buffer, allowing `wl_animate` to capture its visual surface and perform smooth custom motion transforms.
  3. Passing `cloak = FALSE` uncloaks the window instantly.

#### Tactic 18: Non-Client Rendering Policy Enforcement (`DWMWA_NCRENDERING_POLICY`)
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 2).
* **How-To & Safe Implementation**:
  1. Options: `DWMNCRP_USEWINDOWSTYLE = 0`, `DWMNCRP_DISABLED = 1`, `DWMNCRP_ENABLED = 2`.
  2. Pass `DWMNCRP_ENABLED` to force hardware non-client rendering even if window style suggests otherwise.
  3. Pass `DWMNCRP_DISABLED` when fully custom non-client GDI frames are rendered.

#### Tactic 19: Forcible DWM Animation Suppression (`DWMWA_TRANSITIONS_FORCEDISABLED`)
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, Attribute 3).
* **How-To & Safe Implementation**:
  1. `BOOL disable_anim = TRUE; DwmSetWindowAttribute(hwnd, 3, &disable_anim, sizeof(disable_anim));`.
  2. Disables the default OS zoom/fade animations during minimize, maximize, and restore, eliminating visual conflicts when `wl_animate` executes custom transforms.

#### Tactic 20: Accurate Geometry Querying via `DWMWA_EXTENDED_FRAME_BOUNDS`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmGetWindowAttribute`, Attribute 9).
* **How-To & Safe Implementation**:
  1. Call: `RECT rect; DwmGetWindowAttribute(hwnd, 9, &rect, sizeof(rect));`.
  2. Returns the exact physical pixel boundary of the visible window, excluding invisible DWM drop shadows that distort standard `GetWindowRect` measurements.

### Category C: UxTheme, Control Redirection & Win32 Dark Mode Subsystem (Tactics 21–30)

#### Tactic 21: Undocumented Win32 Preferred App Mode Ingestion
* **Target Versions**: Windows 10 (1809+) and Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (Ordinal 135).
* **How-To & Safe Implementation**:
  1. Resolve ordinal 135: `SetPreferredAppMode = (fn)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(135));`.
  2. Modes: `Default = 0`, `AllowDark = 1`, `ForceDark = 2`, `ForceLight = 3`, `Max = 4`.
  3. Call `SetPreferredAppMode(ForceDark)` during process initialization before creating any controls.

#### Tactic 22: Window-Level Dark Mode Authorization
* **Target Versions**: Windows 10 (1809+) and Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (Ordinal 133).
* **How-To & Safe Implementation**:
  1. Resolve ordinal 133: `AllowDarkModeForWindow = (fn)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(133));`.
  2. Call `AllowDarkModeForWindow(hwnd, TRUE)` immediately after window creation.
  3. Informs the UxTheme engine to load dark theme assets for common controls child to this window.

#### Tactic 23: Win32 Dark Mode Menu Synchronization (`FlushMenuThemes`)
* **Target Versions**: Windows 10 (1809+) and Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (Ordinal 136).
* **How-To & Safe Implementation**:
  1. Resolve ordinal 136: `FlushMenuThemes = (fn)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(136));`.
  2. Call `FlushMenuThemes()` whenever app theme toggles between light and dark to force cached popup menu themes to reload.

#### Tactic 24: Common Control Subclassing via ComCtl32
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`SetWindowSubclass`, `RemoveWindowSubclass`, `DefSubclassProc`).
* **How-To & Safe Implementation**:
  1. Attach subclass: `SetWindowSubclass(hwnd, SubclassProc, uIdSubclass, dwRefData)`.
  2. Inside `SubclassProc`, pass unhandled messages to `DefSubclassProc(hwnd, uMsg, wParam, lParam)`.
  3. Clean up in `WM_NCDESTROY` using `RemoveWindowSubclass(hwnd, SubclassProc, uIdSubclass)`.
  4. **Safety**: Avoids pointer collisions associated with legacy `SetWindowLongPtr(GWLP_WNDPROC)`.

#### Tactic 25: Custom Control Theme Redirection (`SetWindowTheme`)
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (`SetWindowTheme`).
* **How-To & Safe Implementation**:
  1. Switch control theme: `SetWindowTheme(hListView, L"Explorer", NULL)` to adopt modern Explorer visual styling.
  2. Use `SetWindowTheme(hControl, L"DarkMode_Explorer", NULL)` on Windows 10/11 to force dark mode styles.
  3. Passing `SetWindowTheme(hControl, L"", L"")` strips visual styles entirely for custom GDI drawing.

#### Tactic 26: High-Performance Double-Buffered Rendering via `BeginBufferedPaint`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (`BufferedPaintInit`, `BeginBufferedPaint`, `EndBufferedPaint`).
* **How-To & Safe Implementation**:
  1. Initialize subsystem on startup: `BufferedPaintInit()`.
  2. In `WM_PAINT`, wrap drawing: `HDC hdc_paint; HPAINTBUFFER hpb = BeginBufferedPaint(hdc, &rc, BPBF_COMPOSITED, NULL, &hdc_paint);`.
  3. Perform drawing operations to `hdc_paint`.
  4. Call `EndBufferedPaint(hpb, TRUE)` to atomically blit the composited buffer, eliminating flicker.

#### Tactic 27: Buffered Animation Cross-Fades via `BeginBufferedAnimation`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (`BeginBufferedAnimation`, `EndBufferedAnimation`).
* **How-To & Safe Implementation**:
  1. Set animation parameters: `BP_ANIMATIONPARAMS params = { sizeof(params), 0, BPBF_COMPOSITED, 250 };` (250ms duration).
  2. Capture previous and next states: `BeginBufferedAnimation(hwnd, hdc, &rc, BPBF_COMPOSITED, &params, &hdcFrom, &hdcTo)`.
  3. Paint starting state in `hdcFrom` and target state in `hdcTo`.
  4. Call `EndBufferedAnimation` to execute a hardware cross-fade transition.

#### Tactic 28: Theme Part and State Bitmap Extraction (`OpenThemeData`)
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (`OpenThemeData`, `DrawThemeBackground`, `CloseThemeData`).
* **How-To & Safe Implementation**:
  1. Open theme class: `HTHEME hTheme = OpenThemeData(hwnd, L"BUTTON");`.
  2. Draw button states: `DrawThemeBackground(hTheme, hdc, BP_PUSHBUTTON, PBS_NORMAL, &rc, NULL);`.
  3. Always close handle: `CloseThemeData(hTheme)`.

#### Tactic 29: System Metrics Querying via `GetThemeMetric`
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `UXTHEME.DLL` (`GetThemeMetric`, `GetThemeColor`).
* **How-To & Safe Implementation**:
  1. Obtain exact pixel dimensions for themed items: `GetThemeMetric(hTheme, hdc, part, state, TMT_BORDERSIZE, &val)`.
  2. Query exact theme color palette: `GetThemeColor(hTheme, part, state, TMT_FILLCOLOR, &colorref)`.
  3. Keeps custom UI proportions synchronized with the user's active theme.

#### Tactic 30: System Color Remapping via `SetSysColors`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetSysColors`, `GetSysColor`).
* **How-To & Safe Implementation**:
  1. Set target elements: `int elements[] = { COLOR_WINDOW, COLOR_WINDOWTEXT }; COLORREF colors[] = { RGB(30,30,30), RGB(220,220,220) };`.
  2. Call: `SetSysColors(2, elements, colors)`.
  3. **Safety**: Broadcasts `WM_SYSCOLORCHANGE` globally. Always record previous values using `GetSysColor` to restore upon WinLook shutdown.

### Category D: Shell Hook, Window Lifecycle & Taskbar Non-Invasive Manipulation (Tactics 31–40)

#### Tactic 31: Top-Level Window Lifecycle Tracking via `RegisterShellHookWindow`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterShellHookWindow`, `DeregisterShellHookWindow`, `RegisterWindowMessageW`).
* **How-To & Safe Implementation**:
  1. Register message: `UINT msg_shellhook = RegisterWindowMessageW(L"SHELLHOOK");`.
  2. Register window: `RegisterShellHookWindow(hwnd_listener);`.
  3. In message loop, handle `msg_shellhook`:
     * `wParam == HSHELL_WINDOWCREATED`: `(HWND)lParam` is newly created window.
     * `wParam == HSHELL_WINDOWDESTROYED`: `(HWND)lParam` was closed.
     * `wParam == HSHELL_WINDOWACTIVATED`: `(HWND)lParam` received focus.
  4. **Safety**: Zero injection; runs purely via OS shell notification queue.

#### Tactic 32: Dynamic WinEvent Hook Subscriptions via `SetWinEventHook`
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWinEventHook`, `UnhookWinEvent`).
* **How-To & Safe Implementation**:
  1. Install out-of-context hook: `HWINEVENTHOOK hHook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, NULL, WinEventProc, 0, 0, WINEVENT_OUTOFCONTEXT);`.
  2. In `WinEventProc`, receive event, `hwnd`, `idObject`, and `idChild`.
  3. **Safety**: `WINEVENT_OUTOFCONTEXT` guarantees the hook executes on the calling thread without injecting DLLs into external processes.

#### Tactic 33: Multi-Monitor Taskbar Traversal (`Shell_SecondaryTrayWnd`)
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowW`, `FindWindowExW`).
* **How-To & Safe Implementation**:
  1. Locate primary taskbar: `HWND hPrimary = FindWindowW(L"Shell_TrayWnd", NULL);`.
  2. Iterate secondary monitor taskbars: `HWND hSec = NULL; while ((hSec = FindWindowExW(NULL, hSec, L"Shell_SecondaryTrayWnd", NULL)) != NULL) { /* process hSec */ }`.
  3. Allows applying positioning, visibility, or backdrop rules symmetrically across multi-monitor setups.

#### Tactic 34: Taskbar Offset Positioning & Floating Margins via Non-Invasive Shell Coordination
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowPos`, `SetWinEventHook`), `SHELL32.DLL` (`SHAppBarMessage`, `ABM_SETPOS`).
* **Architectural Execution Boundary Notice**:
  *Important*: Win32 enforces strict process isolation for window subclassing (`SetWindowSubclass` / `SetWindowLongPtr`). An external utility cannot call `SetWindowSubclass` on `Shell_TrayWnd` because subclass procedures must reside within the memory address space of the process that created the window (`explorer.exe`). Attempting cross-process subclassing from an external process without DLL injection is rejected by `win32k.sys`.
* **How-To & Safe Implementation (Out-of-Process Non-Invasive Methods)**:
  * **Method A (Out-of-Context WinEvent Synchronization)**:
    1. Register an out-of-context hook using `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, NULL, ShellLocationHookProc, explorer_pid, 0, WINEVENT_OUTOFCONTEXT)`.
    2. When Explorer recalculates taskbar geometry, evaluate target screen bounds and call `SetWindowPos(hTray, NULL, target_x, target_y - floating_margin, target_w, target_h, SWP_NOZORDER | SWP_NOACTIVATE)`.
  * **Method B (Official Shell AppBar Negotiation)**:
    1. Send `SHAppBarMessage(ABM_SETPOS, &pData)` to negotiate desktop reservation boundaries with the shell.
  * **Method C (Path B Shell Decoupling - Recommended for Floating Docks)**:
    1. Hide native `Shell_TrayWnd` via `ShowWindow(hTray, SW_HIDE)`.
    2. Host WinLook's own native AppBar window (`SHAppBarMessage(ABM_NEW)`), which CAN be cleanly subclassed with `SetWindowSubclass` because WinLook owns and creates that window.
  4. Ensure `pPos->flags &= ~SWP_NOMOVE;` to allow position changes.

#### Tactic 35: Taskbar Progress State Ingestion via `ITaskbarList3`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (COM Interface `ITaskbarList3`).
* **How-To & Safe Implementation**:
  1. Query `ITaskbarList3` from `CLSID_TaskbarList`.
  2. Call `SetProgressState(hwnd, TBPF_NORMAL | TBPF_ERROR | TBPF_INDETERMINATE)`.
  3. Call `SetProgressValue(hwnd, ullCompleted, ullTotal)` to project progress directly into custom dock representations.

#### Tactic 36: Start Menu Hardware Key Interception via Low-Level Keyboard Hooks
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowsHookExW`, `WH_KEYBOARD_LL`).
* **How-To & Safe Implementation**:
  1. Install hook: `HHOOK hHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, hInst, 0);`.
  2. In hook proc, check for `VK_LWIN` or `VK_RWIN` (flags: `p->vkCode == VK_LWIN`).
  3. If custom WinLook Start menu is enabled, return `1` to suppress native OS menu activation and trigger WinLook's animated canvas.
  4. Pass other keystrokes to `CallNextHookEx(hHook, nCode, wParam, lParam)`.

#### Tactic 37: Explorer Process Crash Detection via `RegisterWaitForSingleObject`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`RegisterWaitForSingleObject`, `OpenProcess`), `USER32.DLL` (`GetWindowThreadProcessId`).
* **How-To & Safe Implementation**:
  1. Obtain Explorer process handle: `GetWindowThreadProcessId(hTray, &pid); HANDLE hProc = OpenProcess(SYNCHRONIZE, FALSE, pid);`.
  2. Register wait: `RegisterWaitForSingleObject(&hWait, hProc, OnExplorerCrashCallback, NULL, INFINITE, WT_EXECUTEONLYONCE);`.
  3. When Explorer terminates, callback executes immediately, allowing WinLook to un-dock canvases and re-dock once the new `Shell_TrayWnd` spawns.

#### Tactic 38: Notification Area (SysTray) HWND Structure Enumeration
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`).
* **How-To & Safe Implementation**:
  Traverse the classic notification tray hierarchy:
  `Shell_TrayWnd` -> `TrayNotifyWnd` -> `SysPager` -> `ToolbarWindow32`.
  Query count of active notification icons via `SendMessageW(hToolbar, TB_BUTTONCOUNT, 0, 0)`.

#### Tactic 39: Virtual Desktop Switch Detection via Shell View Hooks
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowRect`, `IsWindowVisible`), `RegisterShellHookWindow`.
* **How-To & Safe Implementation**:
  When a user switches virtual desktops, Windows cloaks windows on inactive desktops:
  1. Catch `HSHELL_WINDOWACTIVATED`.
  2. Query `DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &cloaked, sizeof(cloaked))`.
  3. If `cloaked != 0`, update dock state to reflect the active virtual desktop context.

#### Tactic 40: Application Switcher (Alt+Tab) Visibility Filtering
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowLongPtrW`, `GWL_EXSTYLE`).
* **How-To & Safe Implementation**:
  To ensure customized background widgets and floating docks do not clutter the Alt+Tab switcher:
  1. Add `WS_EX_TOOLWINDOW` (0x00000080) to extended style.
  2. Remove `WS_EX_APPWINDOW` (0x00040000).
  3. Result: Windows completely omits the window handle from the Alt+Tab task switcher list.

### Category E: Desktop Shell Canvas, Live Wallpaper & WorkerW Embedding (Tactics 41–50)

#### Tactic 41: Progman Undocumented Message `0x052C` Timeout Injection
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowW`, `SendMessageTimeoutW`).
* **How-To & Safe Implementation**:
  1. Locate program manager: `HWND hProgman = FindWindowW(L"Progman", NULL);`.
  2. Send message: `SendMessageTimeoutW(hProgman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &result);`.
  3. Forces `explorer.exe` to spawn an unmanaged `WorkerW` window directly behind desktop icons.

#### Tactic 42: WorkerW Z-Order Hierarchy Traversal and Verification
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `GetWindow`).
* **How-To & Safe Implementation**:
  1. Enumerate top-level `WorkerW` windows using `FindWindowExW(NULL, NULL, L"WorkerW", NULL)`.
  2. Check if `FindWindowExW(hWorkerW, NULL, L"SHELLDLL_DefView", NULL)` is not null.
  3. The `WorkerW` directly behind this window in Z-order (`GetWindow(hWorkerW, GW_HWNDNEXT)`) is the target wallpaper canvas parent.

#### Tactic 43: Clean Live Wallpaper Canvas Reparenting
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetParent`, `SetWindowPos`).
* **How-To & Safe Implementation**:
  1. Reparent: `SetParent(hMyCanvas, hTargetWorkerW);`.
  2. Adjust bounds: `SetWindowPos(hMyCanvas, NULL, 0, 0, screen_w, screen_h, SWP_NOZORDER | SWP_SHOWWINDOW);`.
  3. Renders behind desktop icons while leaving `SysListView32` folder view interactive.

#### Tactic 44: Desktop Icon Listview Reparenting and Isolation
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `ShowWindow`).
* **How-To & Safe Implementation**:
  1. Find listview: `Progman` -> `SHELLDLL_DefView` -> `SysListView32`.
  2. To temporarily hide desktop icons without killing Explorer: `ShowWindow(hListView, SW_HIDE);`.
  3. To restore: `ShowWindow(hListView, SW_SHOW);`.

#### Tactic 45: Multi-Monitor Virtual Desktop Dimension Querying
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetSystemMetrics`).
* **How-To & Safe Implementation**:
  1. Retrieve virtual bounds:
     * `x = GetSystemMetrics(SM_XVIRTUALSCREEN);`
     * `y = GetSystemMetrics(SM_YVIRTUALSCREEN);`
     * `w = GetSystemMetrics(SM_CXVIRTUALSCREEN);`
     * `h = GetSystemMetrics(SM_CYVIRTUALSCREEN);`
  2. Ensures live wallpaper canvases span across multi-monitor setups with negative coordinates.

#### Tactic 46: Per-Monitor Static Wallpaper Configuration via `IDesktopWallpaper`
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (COM Interface `IDesktopWallpaper`).
* **How-To & Safe Implementation**:
  1. CoCreateInstance `CLSID_DesktopWallpaper`.
  2. Call `GetMonitorDevicePathAt(i, &monitor_id)`.
  3. Call `SetWallpaper(monitor_id, pwszWallpaperFile)`.
  4. Call `SetPosition(DWPOS_FILL | DWPOS_FIT | DWPOS_STRETCH)`.

#### Tactic 47: Legacy Active Desktop Configuration via `IActiveDesktop`
* **Target Versions**: Windows 98, ME, 2000, XP.
* **APIs / Drivers**: `SHELL32.DLL` (COM Interface `IActiveDesktop`).
* **How-To & Safe Implementation**:
  1. CoCreateInstance `CLSID_ActiveDesktop`.
  2. Call `SetWallpaper(pwszPath, 0)`.
  3. Call `ApplyChanges(AD_APPLY_ALL | AD_APPLY_FORCE)`.

#### Tactic 48: Transcoded Wallpaper Cache Bypass
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: Win32 Shell API / File System.
* **How-To & Safe Implementation**:
  Windows recompresses wallpapers to JPEG at `%APPDATA%\Microsoft\Windows\Themes\TranscodedWallpaper`.
  To ensure uncompressed PNG/BMP fidelity, write directly to the registry key `HKCU\Control Panel\Desktop\WallPaper` and broadcast `SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, (void*)path, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE)`.

#### Tactic 49: Desktop Mouse Event Pass-Through and Hit-Testing
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageW`, `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`).
* **How-To & Safe Implementation**:
  If the live wallpaper canvas needs to respond to mouse motion without blocking icon selection:
  1. Register mouse hook or handle `WM_MOUSEMOVE`.
  2. Forward unhandled clicks to `SendMessageW(hListView, uMsg, wParam, lParam)` so desktop icons maintain selection marquees.

#### Tactic 50: Desktop Window Cloaking and Background Repaint Invalidation
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RedrawWindow`, `InvalidateRect`).
* **How-To & Safe Implementation**:
  When wallpaper shaders update:
  `RedrawWindow(hWorkerW, NULL, NULL, RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN);`.
  Refreshes background textures without causing desktop icon flickering.

### Category F: Animation Mechanics, Physics Curves & Timing (Tactics 51–60)

#### Tactic 51: High-Resolution Multimedia Timer Clocks (`timeBeginPeriod`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `WINMM.DLL` (`timeBeginPeriod`, `timeEndPeriod`, `timeGetTime`).
* **How-To & Safe Implementation**:
  1. On startup: `timeBeginPeriod(1);` (lowers system timer interrupt resolution from 15.6ms to 1ms).
  2. Drives smooth 60fps/120fps/144fps frame evaluations in `wl_animate`.
  3. **Safety**: Always pair with `timeEndPeriod(1)` during process shutdown to avoid unnecessary power drain on laptops.

#### Tactic 52: DWM Vertical Blank Refresh Rate Synchronization
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmGetCompositionTimingInfo`, `DwmFlush`).
* **How-To & Safe Implementation**:
  1. Call `DwmFlush()` before updating animated window coordinates.
  2. Blocks until the DWM compositor finishes the current vertical blank, eliminating frame tearing.
  3. Query `DwmGetCompositionTimingInfo(NULL, &timing)` to adapt physics steps to variable refresh rates (G-Sync / FreeSync).

#### Tactic 53: Atomic Multi-Window Coordinate Updates (`DeferWindowPos`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`BeginDeferWindowPos`, `DeferWindowPos`, `EndDeferWindowPos`).
* **How-To & Safe Implementation**:
  1. Allocate batch handle: `HDWP hdwp = BeginDeferWindowPos(nWindows);`.
  2. For each element: `hdwp = DeferWindowPos(hdwp, hwnd, NULL, x, y, cx, cy, SWP_NOZORDER | SWP_NOACTIVATE);`.
  3. Execute batch: `EndDeferWindowPos(hdwp);`.
  4. **Safety**: Updates positions of multiple custom controls in a single hardware frame pass.

#### Tactic 54: Smooth DirectComposition Visual Layer Transforms
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `DCOMP.DLL` (`DCompositionCreateDevice`, `IDCompositionVisual`).
* **How-To & Safe Implementation**:
  1. Create DirectComposition device bound to DXGI swapchain.
  2. Attach visual child: `pVisual->SetTransform(pTransform);`.
  3. Animate 2D affine matrix (translation, scale, rotation) with hardware acceleration without triggering Win32 window message loops.

#### Tactic 55: Cubic Bezier Motion Curve Evaluation for UI Transforms
* **Target Versions**: Platform independent (Rust / C89 in WinLook).
* **APIs / Drivers**: Pure math in `wl_animate`.
* **How-To & Safe Implementation**:
  Evaluate cubic bezier coordinates:
  $$P(t) = (1-t)^3 P_0 + 3(1-t)^2 t P_1 + 3(1-t) t^2 P_2 + t^3 P_3$$
  Maps normalized time $t \in [0.0, 1.0]$ to position deltas, providing natural ease-in, ease-out, and elastic trajectories for UI components.

#### Tactic 56: Win32 Built-In Slide and Blend Transitions via `AnimateWindow`
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`AnimateWindow`).
* **How-To & Safe Implementation**:
  1. Flags: `AW_HOR_POSITIVE`, `AW_VER_NEGATIVE`, `AW_SLIDE`, `AW_BLEND`, `AW_HIDE`.
  2. Call: `AnimateWindow(hwnd, 200, AW_SLIDE | AW_VER_NEGATIVE);`.
  3. Executes hardware-synchronized window open/close transitions without manual timer loops.

#### Tactic 57: Minimize/Restore Trajectory Animation via `DrawAnimatedRects`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`DrawAnimatedRects`).
* **How-To & Safe Implementation**:
  1. Define source and destination: `RECT rcFrom, rcTo;`.
  2. Call: `DrawAnimatedRects(hwnd, IDANI_CAPTION, &rcFrom, &rcTo);`.
  3. Draws the classic OS wireframe/surface zoom animation between floating dock icons and opened windows.

#### Tactic 58: Non-Linear Spring Physics Simulation for Floating UI Elements
* **Target Versions**: Platform independent (`wl_animate`).
* **APIs / Drivers**: Numerical integration (Velocity Verlet).
* **How-To & Safe Implementation**:
  $$F = -k(x - x_{	ext{target}}) - c \cdot v$$
  Simulates physical spring resistance ($k$) and damping ($c$), producing responsive, bounce-free dock expansions.

#### Tactic 59: Dynamic Frame Rate Throttling under CPU Load (`wl_cpuv`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`GetSystemTimes`).
* **How-To & Safe Implementation**:
  1. Periodically evaluate CPU utilization via `GetSystemTimes`.
  2. If system CPU usage exceeds 85%, throttle animation render rate from 60fps to 30fps or pause non-essential live wallpaper shaders.

#### Tactic 60: Direct3D Swap Chain V-Sync Coordination
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DXGI.DLL` (`IDXGISwapChain::Present`).
* **How-To & Safe Implementation**:
  1. Call `pSwapChain->Present(1, 0);` (interval 1 ensures presentation synchronizes exactly with monitor refresh rate).
  2. Eliminates pipeline contention with the desktop compositor.

### Category G: Lock Screen, Session Lifecycle & Power Awareness (Tactics 61–70)

#### Tactic 61: Terminal Services Session Lock Detection
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `WTSAPI32.DLL` (`WTSRegisterSessionNotification`, `WTSUnRegisterSessionNotification`).
* **How-To & Safe Implementation**:
  1. Register: `WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);`.
  2. Catch `WM_WTSSESSION_CHANGE`:
     * `wParam == WTS_SESSION_LOCK`: Workstation locked. Instantly halt wallpaper render pipelines.
     * `wParam == WTS_SESSION_UNLOCK`: Workstation unlocked. Resume UI loops.

#### Tactic 62: System Power Setting Ingestion via `RegisterPowerSettingNotification`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterPowerSettingNotification`, `GUID_BATTERY_PERCENTAGE_REMAINING`, `GUID_POWER_SAVING_POLICY`).
* **How-To & Safe Implementation**:
  1. Register: `RegisterPowerSettingNotification(hwnd, &GUID_POWER_SAVING_POLICY, DEVICE_NOTIFY_WINDOW_HANDLE);`.
  2. In `WM_POWERBROADCAST`, detect when Windows Battery Saver engages and drop background rendering overhead to 0%.

#### Tactic 63: Fullscreen Direct3D Game Detection via `SHQueryUserNotificationState`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHQueryUserNotificationState`).
* **Subsystem Architecture & Correct Enum Values**:
  The Windows SDK `shellapi.h` defines `QUERY_USER_NOTIFICATION_STATE` as follows:
  `QUNS_NOT_PRESENT = 1`, `QUNS_BUSY = 2`, `QUNS_RUNNING_D3D_FULL_SCREEN = 3`, `QUNS_PRESENTATION_MODE = 4`, `QUNS_ACCEPTS_NOTIFICATIONS = 5`, `QUNS_QUIET_HOURS = 6`, `QUNS_APP = 7`.
  *Note*: Value 3 is specifically `QUNS_RUNNING_D3D_FULL_SCREEN` (an exclusive fullscreen Direct3D application is active), while Value 2 is `QUNS_BUSY` (a fullscreen or non-notifying application is active), and Value 4 is `QUNS_PRESENTATION_MODE`.
* **How-To & Safe Implementation**:
  1. Query: `QUERY_USER_NOTIFICATION_STATE state; SHQueryUserNotificationState(&state);`.
  2. If `state == QUNS_RUNNING_D3D_FULL_SCREEN` (3), `state == QUNS_BUSY` (2), or `state == QUNS_PRESENTATION_MODE` (4):
     Pause all background animations, wallpaper shaders, and dock redraws to drop CPU/GPU consumption to 0%.

#### Tactic 64: Modern Lock Screen Wallpaper Override via WinRT
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: WinRT API (`Windows.System.UserProfile.LockScreen`).
* **How-To & Safe Implementation**:
  1. Access `LockScreen::SetImageFileAsync(file)`.
  2. Updates lock screen image legally for the user profile without administrative elevation.

#### Tactic 65: Administrative Group Policy Lock Screen Image Configuration
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: Windows Registry.
* **How-To & Safe Implementation**:
  1. Key: `HKLM\SOFTWARE\Policies\Microsoft\Windows\Personalization`.
  2. Value: `LockScreenImage` (`REG_SZ`) -> absolute path to wallpaper.
  3. Directs `LogonUI.exe` to render the designated image upon workstation locking.

#### Tactic 66: Windows Spotlight 4K Cache Scraping and Extraction
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: Windows Shell / File System.
* **How-To & Safe Implementation**:
  1. Ingest files from `%LocalAppData%\Packages\Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy\LocalState\Assets`.
  2. Filter files where size > 1MB and aspect ratio is 16:9 or 9:16.
  3. Ingest directly into WinLook's Wallpaper Store without network calls.

#### Tactic 67: Pre-Lock Desktop Ambient Transition and Particle Fade
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `wl_animate` + `WTS_SESSION_LOCK`.
* **How-To & Safe Implementation**:
  When `WTS_SESSION_LOCK` arrives, execute a 150ms ambient opacity fade to black on custom desktop widgets before the secure desktop transition freezes the default desktop rasterizer.

#### Tactic 68: Display Topology Change Detection (`WM_DISPLAYCHANGE`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_DISPLAYCHANGE`, `EnumDisplayMonitors`).
* **How-To & Safe Implementation**:
  1. Catch `WM_DISPLAYCHANGE`.
  2. Re-enumerate monitor geometries via `EnumDisplayMonitors`.
  3. Recompute floating bar anchors and wallpaper canvas bounds to fit the new multi-monitor topology.

#### Tactic 69: User Idle State Detection via `GetLastInputInfo`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetLastInputInfo`, `LASTINPUTINFO`).
* **How-To & Safe Implementation**:
  1. Call `GetLastInputInfo(&lii)`.
  2. Evaluate `idle_time = GetTickCount() - lii.dwTime;`.
  3. If idle time > 5 minutes, engage ambient idle animations or sleep state in `wl_wlde`.

#### Tactic 70: System Suspend / Resume Lifecycle Handling
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_POWERBROADCAST`, `PBT_APMSUSPEND`, `PBT_APMRESUMESUSPEND`).
* **How-To & Safe Implementation**:
  1. On `PBT_APMSUSPEND`: Flush state journals, release hardware surface locks.
  2. On `PBT_APMRESUMESUSPEND`: Reconnect DWM handles and restore UI anchors.

### Category H: Font, DPI Scaling, Accessibility & High-Contrast Subsystems (Tactics 71–80)

#### Tactic 71: Per-Monitor DPI Awareness Registration
* **Target Versions**: Windows 10 (1607+) and Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetProcessDpiAwarenessContext`, `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`).
* **How-To & Safe Implementation**:
  1. Call `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` during startup.
  2. Prevents blurry OS bitmap scaling when moving windows between monitors with different scale factors.

#### Tactic 72: Dynamic DPI Change Interception (`WM_DPICHANGED`)
* **Target Versions**: Windows 8.1 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_DPICHANGED`).
* **How-To & Safe Implementation**:
  1. In window proc, extract new DPI: `UINT new_dpi = HIWORD(wParam);`.
  2. Extract suggested rect from `lParam`: `RECT* prc = (RECT*)lParam;`.
  3. Rescale custom fonts and call `SetWindowPos(hwnd, NULL, prc->left, prc->top, prc->right - prc->left, prc->bottom - prc->top, SWP_NOZORDER | SWP_NOACTIVATE);`.

#### Tactic 73: Custom Private Font Loading without System Installation
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `GDI32.DLL` (`AddFontResourceExW`, `RemoveFontResourceExW`, `FR_PRIVATE`).
* **How-To & Safe Implementation**:
  1. Load: `AddFontResourceExW(font_path, FR_PRIVATE, NULL);`.
  2. Font becomes immediately accessible via `CreateFontW` for the calling process without polluting the global Windows Fonts registry.
  3. Clean up on exit: `RemoveFontResourceExW(font_path, FR_PRIVATE, NULL);`.

#### Tactic 74: High-Contrast Accessibility Mode Detection
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_GETHIGHCONTRAST`, `HIGHCONTRASTW`).
* **How-To & Safe Implementation**:
  1. Query: `HIGHCONTRASTW hc = { sizeof(hc) }; SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(hc), &hc, 0);`.
  2. If `hc.dwFlags & HCF_HIGHCONTRASTON`, disable custom transparency/materials and enforce high-contrast system colors.

#### Tactic 75: Non-Client Metrics Font and Border Configuration
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_SETNONCLIENTMETRICS`, `NONCLIENTMETRICSW`).
* **How-To & Safe Implementation**:
  1. Populate `NONCLIENTMETRICSW` (caption height, menu font, status font, border width).
  2. Call: `SystemParametersInfoW(SPI_SETNONCLIENTMETRICS, sizeof(ncm), &ncm, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);`.
  3. Updates system window frames globally across classic Win32 applications.

#### Tactic 76: ClearType and Font Smoothing State Preservation
* **Target Versions**: Windows 98/2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SPI_GETFONTSMOOTHING`, `SPI_GETFONTSMOOTHINGTYPE`).
* **How-To & Safe Implementation**:
  Query smoothing parameters before rendering custom typography to ensure text anti-aliasing matches user preferences.

#### Tactic 77: GDI Region Path Outlining for Bespoke Window Contours
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `GDI32.DLL` (`BeginPath`, `EndPath`, `PathToRegion`, `PolyBezier`).
* **How-To & Safe Implementation**:
  1. Call `BeginPath(hdc);`, draw Bezier curves or geometric paths, call `EndPath(hdc);`.
  2. Convert to region: `HRGN hrgn = PathToRegion(hdc);`.
  3. Apply to window: `SetWindowRgn(hwnd, hrgn, TRUE);`.

#### Tactic 78: Window Text Translation via Subclassed `WM_SETTEXT`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`DefSubclassProc`).
* **How-To & Safe Implementation**:
  Intercept `WM_SETTEXT` in subclassed controls to dynamically localize, format, or mask display text in flight.

#### Tactic 79: Custom Caret and Selection Drawing for Lightweight Controls
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`CreateCaret`, `SetCaretPos`, `ShowCaret`, `DestroyCaret`).
* **How-To & Safe Implementation**:
  Provides bespoke text entry indicators with custom colors and blinking cadences matching the active theme.

#### Tactic 80: Desktop Contrast and Color Filtering Ingestion
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `MAGNIFICATION.DLL` (`MagSetColorEffect`).
* **How-To & Safe Implementation**:
  Applies a 5x5 color transformation matrix globally to adjust desktop saturation, sepia, or colorblindness correction without external shaders.

### Category I: Native Controls, Dialogs & Common Subclassing Customization (Tactics 81–90)

#### Tactic 81: Button Owner-Drawn Customization via `BS_OWNERDRAW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_DRAWITEM`, `DRAWITEMSTRUCT`).
* **How-To & Safe Implementation**:
  1. Apply `BS_OWNERDRAW` to button control.
  2. Parent receives `WM_DRAWITEM` with `LPDRAWITEMSTRUCT pDIS = (LPDRAWITEMSTRUCT)lParam;`.
  3. Paint custom states (selected, disabled, focused) to `pDIS->hDC`.

#### Tactic 82: ListView Custom Draw via `NM_CUSTOMDRAW`
* **Target Versions**: Windows 95 (IE3+) through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`WM_NOTIFY`, `NMLVCUSTOMDRAW`).
* **How-To & Safe Implementation**:
  1. In `WM_NOTIFY`, inspect `pnmhdr->code == NM_CUSTOMDRAW`.
  2. On `CDDS_PREPAINT`, return `CDRF_NOTIFYITEMDRAW`.
  3. On `CDDS_ITEMPREPAINT`, customize row background and font colors, then return `CDRF_DODEFAULT`.

#### Tactic 83: Custom Scroll Bar Theme Supplantation
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`ShowScrollBar`, `SetScrollInfo`), `COMCTL32.DLL`.
* **How-To & Safe Implementation**:
  1. Hide legacy scrollbar: `ShowScrollBar(hwnd, SB_BOTH, FALSE);`.
  2. Render lightweight, auto-hiding custom scroll bars overlaid via DirectComposition or layered child windows.

#### Tactic 84: Common Dialog Hooking and Template Redirection
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `COMDLG32.DLL` (`GetOpenFileNameW`, `OPENFILENAMEW`, `LPOFNHOOKPROC`).
* **How-To & Safe Implementation**:
  1. Specify `OFN_ENABLEHOOK | OFN_EXPLORER` in flags.
  2. Assign custom hook: `ofn.lpfnHook = OpenFileNameHookProc;`.
  3. Intercept `CDN_INITDONE` to reposition controls or inject custom styling panes into system file dialogs.

#### Tactic 85: Edit Control Background and Text Tinting
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_CTLCOLOREDIT`, `WM_CTLCOLORSTATIC`).
* **How-To & Safe Implementation**:
  1. In parent window, intercept `WM_CTLCOLOREDIT`.
  2. `SetTextColor((HDC)wParam, text_color); SetBkColor((HDC)wParam, bg_color);`.
  3. Return handle to background solid brush: `(LRESULT)hBrushBackground;`.

#### Tactic 86: Context Menu Popup Skinning via Subclassed `WM_INITMENUPOPUP`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_INITMENUPOPUP`, `SetMenuItemInfoW`).
* **How-To & Safe Implementation**:
  Assign `MFT_OWNERDRAW` to menu items during `WM_INITMENUPOPUP` to draw modern rounded menus with custom iconography.

#### Tactic 87: Tooltip Custom Styling and Delay Timing
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`TTM_SETDELAYTIME`, `TTM_SETTIPBKCOLOR`, `TTM_SETTIPTEXTCOLOR`).
* **How-To & Safe Implementation**:
  Send tooltip messages to adjust show delays (`TTDT_INITIAL`) and inject cohesive theme colors without third-party frameworks.

#### Tactic 88: Progress Bar Smooth State and State Color Setting
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`PBM_SETSTATE`, `PBM_SETRANGE32`, `PBM_SETPOS`).
* **How-To & Safe Implementation**:
  1. States: `PBST_NORMAL = 1` (Green), `PBST_ERROR = 2` (Red), `PBST_PAUSED = 3` (Yellow).
  2. Send `PBM_SETSTATE` to adapt progress bar colors to context dynamically.

#### Tactic 89: Status Bar Part Partitioning and Owner-Draw
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`SB_SETPARTS`, `SB_SETTEXT`, `SBT_OWNERDRAW`).
* **How-To & Safe Implementation**:
  Partition status bar into proportional blocks and custom-render indicators (battery, CPU, network status).

#### Tactic 90: TreeView Indentation, Line Style and Node Rendering
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `COMCTL32.DLL` (`TVM_SETINDENT`, `TVM_SETLINECOLOR`).
* **How-To & Safe Implementation**:
  Configure tree lines, custom node glyphs, and row heights via `TVM_SETITEMHEIGHT` for clean navigation sidebars.

### Category J: Process Safety, Memory Governance, Watchdogs & Reversibility (Tactics 91–100)

#### Tactic 91: Lock-Free State Reversal Journaling (`wl_journal`)
* **Target Versions**: Platform independent (Rust / C89 in WinLook).
* **APIs / Drivers**: SPSC Lock-free Ring Buffer / Memory mapped files.
* **How-To & Safe Implementation**:
  1. Every modification pushes an inverse record: `Record { hwnd, attribute, old_value, new_value }`.
  2. On process termination, emergency signal, or unhandled panic, the journal flushes in LIFO order, restoring Windows to its exact original state.

#### Tactic 92: Process Working Set Trimming on Idle
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`SetProcessWorkingSetSize`, `EmptyWorkingSet`).
* **How-To & Safe Implementation**:
  1. Call `EmptyWorkingSet(GetCurrentProcess());` after UI components transition to idle.
  2. Releases unused physical RAM pages back to the operating system, keeping WinLook's memory footprint under 15MB.

#### Tactic 93: Win32 Handle Leak Monitoring and Table Auditing
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetGuiResources`).
* **How-To & Safe Implementation**:
  1. Query: `DWORD user_objs = GetGuiResources(GetCurrentProcess(), GR_USEROBJECTS);`.
  2. Query: `DWORD gdi_objs = GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);`.
  3. Asserts thresholds in debug and test builds to guarantee zero GDI handle leaks.

#### Tactic 94: Dynamic DLL Loading and Ordinal Fallback
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`LoadLibraryW`, `GetProcAddress`, `FreeLibrary`).
* **How-To & Safe Implementation**:
  Never statically link against modern or undocumented APIs. Dynamically load libraries (`dwmapi.dll`, `uxtheme.dll`) and check function pointer nullability before invoking.

#### Tactic 95: External Process Crash Watchdog and Ghost Window Cleaning
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`IsWindow`, `IsHungAppWindow`).
* **How-To & Safe Implementation**:
  Periodically audit monitored HWND lists. If `IsWindow(hwnd)` is false or `IsHungAppWindow(hwnd)` is true, gracefully decouple subclass handlers and drop proxy rendering representations.

#### Tactic 96: UIPI Integrity Level Self-Evaluation
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `ADVAPI32.DLL` (`OpenProcessToken`, `GetTokenInformation`, `TokenIntegrityLevel`).
* **How-To & Safe Implementation**:
  Check process integrity level (Low, Medium, High). If running at Medium Integrity, log a warning that window messages cannot be sent to elevated (High Integrity) applications due to User Interface Privilege Isolation.

#### Tactic 97: Thread Lock List Contention Mitigation in `win32k.sys`
* **Target Versions**: Windows NT 4.0 through Windows 11.
* **APIs / Drivers**: Win32 Kernel Architecture (`gpresUser`).
* **How-To & Safe Implementation**:
  Avoid rapid consecutive cross-process `SetWindowPos` or `SendMessage` loops on UI threads. Dispatch bulk window modifications through a dedicated worker thread to prevent freezing the interactive session critical section.

#### Tactic 98: Structured Exception Handling (SEH) Wrappers for Win32 API Probing
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Compiler intrinsic (`__try` / `__except`).
* **How-To & Safe Implementation**:
  Wrap calls to undocumented ordinals or experimental DWM attributes inside SEH blocks:
  `__try { CallUndocumentedAPI(); } __except(EXCEPTION_EXECUTE_HANDLER) { FallbackToSafePath(); }`.

#### Tactic 99: Named Pipe Fast IPC Protocol for Decoupled Settings Coordination
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`CreateNamedPipeW`, `ConnectNamedPipe`, `ReadFile`, `WriteFile`).
* **How-To & Safe Implementation**:
  Decouple the WinLook configuration GUI from the background engine (`winlook.bg.exe`) via local named pipes (`\\.\pipe\WinLook_IPC`). Ensures zero GUI overhead in background daemon.

#### Tactic 100: Emergency Failsafe Hotkey and Absolute Desktop State Restoration
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterHotKey`, `MOD_CONTROL | MOD_ALT | MOD_SHIFT`).
* **How-To & Safe Implementation**:
  1. Register global emergency shortcut: `RegisterHotKey(hwnd, ID_FAILSAFE, MOD_CONTROL | MOD_ALT | MOD_SHIFT, 'W');`.
  2. On receipt: Instantly unhide native taskbar, detach subclass hooks, un-dock wallpaper canvases, flush `wl_journal`, and exit cleanly.

---

### Category K: Deep Shell Integration, Taskbar Buttons & Jump Lists (Tactics 101–110)

#### Tactic 101: Dynamic AppIcon Extraction via `SHGetFileInfoW` and `IExtractIconW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHGetFileInfoW`, `SHGFI_ICON | SHGFI_LARGEICON | SHGFI_SMALLICON`), `SHCreateItemFromParsingName`, `IExtractIconW`.
* **Subsystem Architecture**: The Windows shell maintains a system image list (`HIMAGELIST`) in `SHELL32.DLL` containing cached icon handles mapped to executable paths, CLSIDs, and document extensions. To customize dock or taskbar icon representations without touching process executables, applications extract clean, unscaled icon handles directly through shell abstractions rather than naive PE header parsing.
* **How-To & Safe Implementation**:
  1. Call `SHGetFileInfoW(pszPath, 0, &sfi, sizeof(SHFILEINFO), SHGFI_ICON | SHGFI_LARGEICON)` to acquire the standard 32x32 icon handle `sfi.hIcon`.
  2. For high-DPI scaling (48x48, 64x64, 256x256), instantiate the shell item via `SHCreateItemFromParsingName` and query `IShellItemImageFactory::GetImage(size, SIIGBF_RESIZETOFIT, &hBitmap)`.
  3. Cache the resulting `HBITMAP` in `WinLook` memory indexed by application path.
  4. Always destroy intermediate icon handles with `DestroyIcon(sfi.hIcon)` to prevent GDI object leaks.

#### Tactic 102: Dynamic Window Icon Overriding via `WM_SETICON`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageW`, `WM_SETICON`, `ICON_SMALL`, `ICON_BIG`, `GetClassLongPtrW`).
* **Subsystem Architecture**: Top-level windows query their visual icon representation from two sources: the window instance icon (`tagWND.hIcon`) set via `WM_SETICON`, and the class fallback icon (`WNDCLASSEX.hIcon`). The taskbar and Alt+Tab switchers display the `ICON_BIG` handle, whereas caption bars and notification icons display `ICON_SMALL`.
* **How-To & Safe Implementation**:
  1. Load the replacement icon via `LoadImageW` or rasterize from SVG using Direct2D.
  2. Send message: `SendMessageTimeoutW(hwndTarget, WM_SETICON, ICON_SMALL, (LPARAM)hNewIconSmall, SMTO_ABORTIFHUNG, 50, &dwResult)`.
  3. Send message: `SendMessageTimeoutW(hwndTarget, WM_SETICON, ICON_BIG, (LPARAM)hNewIconBig, SMTO_ABORTIFHUNG, 50, &dwResult)`.
  4. Before applying, record original icon handles retrieved via `(HICON)GetClassLongPtrW(hwndTarget, GCLP_HICON)` in `wl_journal` to guarantee 100% reversible state restoration.

#### Tactic 103: Taskbar Application ID (AppID) Binding via `SetCurrentProcessExplicitAppUserModelID`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SetCurrentProcessExplicitAppUserModelID`), `PROPSYS.DLL` (`SHSetWindowPropertyStore`, `PKEY_AppUserModel_ID`).
* **Subsystem Architecture**: Windows 7 introduced the Application User Model ID (AUMID), an explicit string token that groups taskbar buttons, routes Jump Lists, and pins items to the taskbar. By default, Explorer groups windows sharing the same executable path. Setting an explicit AUMID allows `WinLook` companion tools or running windows to separate into individual custom taskbar buttons or combine under unified groupings.
* **How-To & Safe Implementation**:
  1. Formulate a canonical AUMID string: `CompanyName.SuiteName.ApplicationName.SubComponent`.
  2. Call `SetCurrentProcessExplicitAppUserModelID(L"WinLook.CustomDock.Instance1")` during initialization before creating any `HWND`.
  3. To alter an external window's taskbar grouping dynamically, initialize a `PROPVARIANT` with `VT_LPWSTR` and assign it to the window's property store: `SHGetPropertyStoreForWindow(hwnd, IID_IPropertyStore, &pStore); pStore->SetValue(PKEY_AppUserModel_ID, propvar); pStore->Commit()`.

#### Tactic 104: Custom Jump List & Task Categorization via `ICustomDestinationList`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ICustomDestinationList`, `IObjectCollection`, `IShellLinkW`, `IPropertyStore`).
* **Subsystem Architecture**: Jump Lists are context-sensitive menus displayed when right-clicking taskbar buttons. The shell aggregates recent destinations and custom user tasks via COM interfaces backed by automatic destination files in `%APPDATA%\Microsoft\Windows\Recent\CustomDestinations`.
* **How-To & Safe Implementation**:
  1. Instantiate `CLSID_DestinationList` via `CoCreateInstance(CLSID_DestinationList, NULL, CLSCTX_INPROC_SERVER, IID_ICustomDestinationList, &pDestList)`.
  2. Call `pDestList->SetAppID(pszAppID)` to bind to the target taskbar button.
  3. Call `pDestList->BeginList(&cMinSlots, IID_IObjectArray, &pRemoved)` to query user-removed items.
  4. Build category links using `IShellLinkW` (configuring path, arguments, title, and icon index) and add them via `pDestList->AddUserTasks(pTasksArray)`.
  5. Finalize with `pDestList->CommitList()`.

#### Tactic 105: Taskbar Badge Overlay Icons via `ITaskbarList3::SetOverlayIcon`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `EXPLORER.EXE`, `SHELL32.DLL` (`ITaskbarList3::SetOverlayIcon`).
* **Subsystem Architecture**: The taskbar button manager supports a dedicated badge overlay slot at the bottom-right corner of the button rectangle (typically 16x16 at 96 DPI). This allows notification badges, unread counters, or status indicators without modifying the underlying application window or taskbar structure.
* **How-To & Safe Implementation**:
  1. Acquire an `ITaskbarList3` instance via `CoCreateInstance(CLSID_TaskbarList, NULL, CLSCTX_INPROC_SERVER, IID_ITaskbarList3, &pTaskbar)`.
  2. Initialize via `pTaskbar->HrInit()`.
  3. Render dynamic badge overlay (e.g., number pill) to an in-memory 16x16 or 32x32 `HICON`.
  4. Call `pTaskbar->SetOverlayIcon(hwndTarget, hBadgeIcon, L"3 Unread Notifications")`.
  5. Clear overlay by calling `pTaskbar->SetOverlayIcon(hwndTarget, NULL, NULL)`.

#### Tactic 106: Taskbar Thumbnail Clip Rectangles via `ITaskbarList3::SetThumbnailClip`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ITaskbarList3::SetThumbnailClip`).
* **Subsystem Architecture**: When hovering over a taskbar button, DWM composites a live preview thumbnail of the client area. `ITaskbarList3::SetThumbnailClip` directs DWM to crop the preview to a specific sub-rectangle of the target window, eliminating redundant toolbars or chrome in the preview.
* **How-To & Safe Implementation**:
  1. Determine the relevant content rectangle within the target client space: `RECT rcClip = { 0, 50, 800, 600 };`.
  2. Call `pTaskbar->SetThumbnailClip(hwndTarget, &rcClip)`.
  3. Pass `NULL` to restore default full-window thumbnail composition.

#### Tactic 107: Custom Taskbar Thumbnail Toolbar Buttons via `ITaskbarList3::ThumbBarAddButtons`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ITaskbarList3::ThumbBarAddButtons`, `ThumbBarUpdateButtons`, `THUMBBUTTON`).
* **Subsystem Architecture**: Windows taskbar hover previews host up to 7 interactive toolbar buttons embedded beneath the live DWM thumbnail. Clicks on these buttons generate `WM_COMMAND` messages sent directly to the target window procedure with `HIWORD(wParam) == THBN_CLICKED`.
* **How-To & Safe Implementation**:
  1. Define an array of `THUMBBUTTON` structs specifying `iId`, `iBitmap` (or `hIcon`), `dwFlags` (`THBF_ENABLED`), and tooltip strings.
  2. Call `pTaskbar->ThumbBarAddButtons(hwnd, nButtons, pButtons)`.
  3. Intercept button activations via window procedure message handling: `if (msg == WM_COMMAND && HIWORD(wParam) == THBN_CLICKED) { u16 button_id = LOWORD(wParam); ... }`.
  4. Dynamically toggle icon or enabled states with `pTaskbar->ThumbBarUpdateButtons(hwnd, nButtons, pButtons)`.

#### Tactic 108: Taskbar Button Grouping & Ungrouping via Process AppUserModelID Partitioning
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `PROPSYS.DLL` (`SHGetPropertyStoreForWindow`), `PKEY_AppUserModel_IsDestListSeparator`, `PKEY_AppUserModel_ID`.
* **Subsystem Architecture**: Explorer uses AUMID equality to collapse multiple windows into a single stacked taskbar button. By injecting unique AUMID identifiers across windows of the same process, `WinLook` can force ungrouped, individual button representations even when system-wide taskbar settings mandate grouping.
* **How-To & Safe Implementation**:
  1. Enumerate child windows of target application: `EnumThreadWindows(dwThreadId, Callback, 0)`.
  2. For each top-level `HWND`, assign a unique per-window AUMID: `StringCchPrintfW(szAumid, 128, L"%s.Instance_%p", pszBaseAumid, hwnd)`.
  3. Set property: `pPropertyStore->SetValue(PKEY_AppUserModel_ID, propvarAumid)`.
  4. Force taskbar refresh: `SetWindowPos(hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED)`.

#### Tactic 109: Taskbar Button Flash & Attention State Interception via `FlashWindowEx`
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FlashWindowEx`, `FLASHWINFO`, `FLASHW_ALL`, `FLASHW_TIMERNOFG`).
* **Subsystem Architecture**: Background applications signal pending events (messages, downloads) by triggering taskbar button flashing. `USER32.DLL` issues internal non-client redraw requests to the taskbar button tray. Custom dock interfaces must track these flashes to render custom glow or bounce animations.
* **How-To & Safe Implementation**:
  1. Intercept shell hook notification `HSHELL_FLASH` registered via `RegisterShellHookWindow`.
  2. When received (`lParam == hwndFlashing`), trigger `wl_animate` particle or pulse effect on the corresponding custom dock item.
  3. To reset or simulate: `FLASHWINFO fwi = { sizeof(fwi), hwnd, FLASHW_STOP, 0, 0 }; FlashWindowEx(&fwi);`.

#### Tactic 110: Taskbar Progress Dual-State Synchronization via `ITaskbarList3::SetProgressValue` & `SetProgressState`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ITaskbarList3::SetProgressState`, `SetProgressValue`, `TBPFLAG`).
* **Subsystem Architecture**: The taskbar button background can display a horizontal progress bar rendered directly behind the title and icon. The state machine supports `TBPF_NOPROGRESS` (0), `TBPF_INDETERMINATE` (1), `TBPF_NORMAL` (green / 2), `TBPF_ERROR` (red / 4), and `TBPF_PAUSED` (yellow / 8).
* **How-To & Safe Implementation**:
  1. Bind to target window using `pTaskbarList3->HrInit()`.
  2. Update state: `pTaskbarList3->SetProgressState(hwnd, TBPF_NORMAL)`.
  3. Update fraction: `pTaskbarList3->SetProgressValue(hwnd, ullCompleted, ullTotal)`.
  4. Custom docks mirror this state by polling or intercepting `WM_TASKBARBUTTONCREATED` and taskbar list calls via COM interceptor stubs.


### Category L: Modern App Enumeration, Package Identity & Asset Harvesting (Tactics 111–120)

#### Tactic 111: UWP / MSIX Package Icon Extraction via AppxManifest XML & MRT Resource Index (`.pri`)
* **Target Versions**: Windows 10 (Build 10240+) and Windows 11.
* **APIs / Drivers**: `KERNELBASE.DLL`, Modern Resource Technology (MRT) (`MrmCreateResourceManager`, `MrmCreateResourceContext`, `MrmLoadPriFile`).
* **Subsystem Architecture**: Modern Windows Store apps (UWP/MSIX) do not store PE `.rsrc` icon tables. Instead, their visual assets reside inside their install root (`C:\Program Files\WindowsApps\<PackageFullName>\`), indexed by an AppxManifest.xml file and compiled Binary Resource Index (`resources.pri`).
* **How-To & Safe Implementation**:
  1. Identify target package family name from process token via `GetPackageFullName(hProcess, &nLen, szPackageName)`.
  2. Parse `AppxManifest.xml` to extract `<Logo>` or `<Square44x44Logo>` relative paths.
  3. Load the package's `.pri` file using the MRT API: `MrmCreateResourceManager(szPriPath, &hResMgr)`.
  4. Resolve the specific resource URI: `MrmResolveResource(hResMgr, hResContext, L"ms-resource://Files/Assets/Square44x44Logo.png", &szResolvedFile)`.
  5. Load PNG surface via Direct2D / WIC for high-DPI rendering inside `WinLook`.

#### Tactic 112: WinRT CoreApplication & ApplicationView Enumeration via `IApplicationViewManager`
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `TWINAPI.APPCORE.DLL` (Undocumented WinRT Interfaces `IApplicationViewManager`, `IApplicationView`).
* **Subsystem Architecture**: Universal Windows apps host their UI inside a host process (`ApplicationFrameHost.exe`) that wraps the unmanaged `CoreWindow`. Multiple modern apps can be hosted inside one frame or detached into separate views.
* **How-To & Safe Implementation**:
  1. Enumerate top-level windows of class `ApplicationFrameWindow`.
  2. Query child window `Windows.UI.Core.CoreWindow` via `FindWindowExW(hFrame, NULL, L"Windows.UI.Core.CoreWindow", NULL)`.
  3. Retrieve the active `IApplicationView` instance to discover TitleBar color, Cloak status, and virtual desktop binding without crashing the sandbox container.

#### Tactic 113: AppUserModelId (AUMID) Resolving via `IPropertyStore` (`PKEY_AppUserModel_ID`)
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `PROPSYS.DLL` (`SHGetPropertyStoreForWindow`, `PropVariantToString`).
* **Subsystem Architecture**: Modern apps rely on AUMIDs for shell execution, protocol handling (`ms-windows-store:`), and notification delivery. Identifying an app's AUMID enables clean command execution without navigating restricted file paths.
* **How-To & Safe Implementation**:
  1. Query property store: `SHGetPropertyStoreForWindow(hwnd, IID_IPropertyStore, (void**)&pStore)`.
  2. Read property: `pStore->GetValue(&PKEY_AppUserModel_ID, &propvar)`.
  3. Convert to string: `PropVariantToStringAlloc(propvar, &pszString)`.
  4. Launch or activate corresponding modern view via `IApplicationActivationManager::ActivateApplication(pszString, NULL, AO_NONE, &dwPid)`.

#### Tactic 114: Shell Item Parsing and Launching via `IShellItem` and `ShellExecuteExW`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHCreateItemFromParsingName`, `ShellExecuteExW`, `SHELLEXECUTEINFOW`).
* **Subsystem Architecture**: Rather than launching raw paths via `CreateProcessW` (which fails on UWP apps, virtual folders, and URL shortcuts), `SHELL32.DLL` uses `IShellItem` parsing. This delegates execution to the shell verb dispatch system, ensuring proper UAC token inheritance and ShellExecute hooks.
* **How-To & Safe Implementation**:
  1. Construct `SHELLEXECUTEINFOW sei = { sizeof(sei) };`.
  2. Configure: `sei.lpFile = pszTarget; sei.nShow = SW_SHOWNORMAL; sei.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_INVOKEIDLIST;`.
  3. Invoke: `ShellExecuteExW(&sei)`.
  4. Handles everything from `.lnk` targets to Control Panel items (`shell:::{...}`) with zero system crash risk.

#### Tactic 115: Installed Application Discovery via `AppsFolder` Known Folder (`SHGetKnownFolderItem`)
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`FOLDERID_AppsFolder`, `IShellFolder`, `IEnumIDList`).
* **Subsystem Architecture**: The virtual shell namespace folder `shell:AppsFolder` aggregates every launchable application across the system—including classic Win32 desktop apps, MSIX packages, UWP Store apps, and system settings applets—in a single structured virtual directory.
* **How-To & Safe Implementation**:
  1. Acquire folder item: `SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, NULL, IID_IShellItem, (void**)&pAppsFolder)`.
  2. Bind to folder enumeration: `pAppsFolder->BindToHandler(NULL, BHID_EnumItems, IID_IEnumShellItems, (void**)&pEnum)`.
  3. Iterate items: Extract display name, parsing path (AUMID or exe path), and icon via `IShellItemImageFactory`.
  4. Supplies `WinLook` with a 100% comprehensive, zero-footprint app catalog.

#### Tactic 116: Running App Architecture Identification (x86, x64, ARM64) via `IsWow64Process2`
* **Target Versions**: Windows 10 (Build 1709+) and Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`IsWow64Process2`, `IMAGE_FILE_MACHINE_AMD64`, `IMAGE_FILE_MACHINE_ARM64`).
* **Subsystem Architecture**: Modern Windows runs diverse processor architectures under transparent emulation (e.g., x86 and x64 on ARM64). Custom dock utilities must determine whether target processes are emulated or native to select compatible injection or IPC hooks.
* **How-To & Safe Implementation**:
  1. Open process handle with `PROCESS_QUERY_LIMITED_INFORMATION`.
  2. Query architecture: `USHORT pProcessMachine, pNativeMachine; IsWow64Process2(hProcess, &pProcessMachine, &pNativeMachine)`.
  3. If `pProcessMachine == IMAGE_FILE_MACHINE_UNKNOWN`, the process runs natively under `pNativeMachine`.

#### Tactic 117: Process Icon Caching and Extraction via `ExtractIconExW` & `PrivateExtractIconsW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ExtractIconExW`), `USER32.DLL` (`PrivateExtractIconsW`).
* **Subsystem Architecture**: High-performance extraction of multiple icon sizes from Win32 executables without launching the host process. `PrivateExtractIconsW` directly parses the PE resource directory table, extracting icons at requested pixel boundaries (e.g., 256x256 uncompressed PNG icons introduced in Windows Vista).
* **How-To & Safe Implementation**:
  1. Call: `PrivateExtractIconsW(pszExePath, 0, 256, 256, &phiconLarge, &piconId, 1, LR_LOADFROMFILE)`.
  2. If 256x256 is unavailable, fall back to `ExtractIconExW(pszExePath, 0, &hLarge, &hSmall, 1)`.
  3. Store extracted handles in `WinLook` texture cache.

#### Tactic 118: High-DPI Vector/SVG Icon Ingestion via Direct2D SVG Parsing
* **Target Versions**: Windows 10 (Build 1703+) and Windows 11.
* **APIs / Drivers**: `D2D1.DLL` (`ID2D1DeviceContext5::CreateSvgDocument`, `DrawSvgDocument`).
* **Subsystem Architecture**: Modern applications increasingly embed SVG vector glyphs rather than bitmap icon tables. Direct2D provides native hardware-accelerated SVG rasterization inside the GPU pipeline.
* **How-To & Safe Implementation**:
  1. Load SVG stream into an `IStream`.
  2. Parse SVG document: `pD2DContext5->CreateSvgDocument(pStream, D2D1::SizeF(width, height), &pSvgDoc)`.
  3. Render to target texture: `pD2DContext5->DrawSvgDocument(pSvgDoc)`.
  4. Guarantees infinite crispness across 100% to 400% DPI scaling displays.

#### Tactic 119: System Icon Cache Invalidation via `SHChangeNotify(SHCNE_ASSOCCHANGED)`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHChangeNotify`, `SHCNE_ASSOCCHANGED`, `SHCNF_IDLIST`).
* **Subsystem Architecture**: Windows caches rendered file and application icons in `%LocalAppData%\IconCache.db` (legacy) or `IconCache_*.db` inside `%LocalAppData%\Microsoft\Windows\Explorer`. When custom icon themes are applied or restored, the shell must be directed to flush its internal memory caches.
* **How-To & Safe Implementation**:
  1. Update target registry icon mapping under `HKCU\Software\Classes`.
  2. Issue global notification: `SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, NULL, NULL)`.
  3. Explorer immediately re-queries file associations and redraws desktop/taskbar icons without killing `explorer.exe`.

#### Tactic 120: Dynamic File Type Association Icon Hooking via Registry Per-User ProgId Overlay
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `ADVAPI32.DLL` (`RegCreateKeyExW`, `RegSetValueExW`), `HKCU\Software\Classes`.
* **Subsystem Architecture**: File extension icons are determined by ProgID lookups. Windows prioritizes user-specific mappings in `HKEY_CURRENT_USER\Software\Classes` over system-wide mappings in `HKEY_LOCAL_MACHINE\SOFTWARE\Classes`.
* **How-To & Safe Implementation**:
  1. Create user ProgID: `HKCU\Software\Classes\WinLook.<Extension>\DefaultIcon`.
  2. Set value to custom icon path: `C:\Path\To\Icons\Custom.ico,0`.
  3. Set user choice mapping under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\<Extension>\UserChoice`.
  4. Requires zero administrator privileges and leaves system files completely intact.


### Category M: Notification Area (SysTray), Flyouts & Shell Overflows (Tactics 121–130)

#### Tactic 121: Notification Area Overflow Window Detection (`NotifyIconOverflowWindow`)
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowW`, `FindWindowExW`).
* **Subsystem Architecture**: The system tray splits active icons between the primary taskbar (`TrayNotifyWnd`) and the hidden overflow flyout (`NotifyIconOverflowWindow`). The overflow window is a top-level popup hosting a `ToolbarWindow32` control containing user-hidden tray icons.
* **How-To & Safe Implementation**:
  1. Locate overflow window: `HWND hOverflow = FindWindowW(L"NotifyIconOverflowWindow", NULL);`.
  2. Locate toolbar child: `HWND hToolbar = FindWindowExW(hOverflow, NULL, L"ToolbarWindow32", NULL);`.
  3. Query button count via `SendMessageW(hToolbar, TB_BUTTONCOUNT, 0, 0)`.
  4. Allows `WinLook` to enumerate and mirror hidden background applets into custom status bars.

#### Tactic 122: Notification Area Icon Metrics & Placement Querying via Shell Desktop Tray Messages
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`Shell_NotifyIconGetRect`, `NOTIFYICONIDENTIFIER`).
* **Subsystem Architecture**: To anchor custom flyouts, context menus, or animations precisely above a tray icon, the shell provides `Shell_NotifyIconGetRect` (introduced in Windows 7), which queries the screen bounding rectangle of any active tray icon by GUID or `HWND` / `uID` pair.
* **How-To & Safe Implementation**:
  1. Initialize `NOTIFYICONIDENTIFIER nid = { sizeof(nid) }; nid.hWnd = hwndApp; nid.uID = uAppTrayId;`.
  2. Call: `RECT rcIcon; Shell_NotifyIconGetRect(&nid, &rcIcon)`.
  3. Use `rcIcon` coordinates to position `WinLook` interactive popups seamlessly adjacent to the taskbar.

#### Tactic 123: Custom System Tray Icon Animation via Cyclic `Shell_NotifyIconW(NIM_MODIFY)`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`Shell_NotifyIconW`, `NIM_MODIFY`, `NOTIFYICONDATAW`).
* **Subsystem Architecture**: Tray icons can display dynamic animations (e.g., spinning sync indicators, frame-by-frame meters) by cycling icon handles on high-resolution timer ticks.
* **How-To & Safe Implementation**:
  1. Initialize `NOTIFYICONDATAW nid = { sizeof(nid) }; nid.hWnd = hwndHost; nid.uID = 100; nid.uFlags = NIF_ICON;`.
  2. Pre-generate or load array of `HICON` animation frames.
  3. On each timer tick (e.g., 60ms): `nid.hIcon = hFrames[nFrameIndex]; Shell_NotifyIconW(NIM_MODIFY, &nid);`.
  4. Destroy frame handles upon teardown via `DestroyIcon`.

#### Tactic 124: Rich Balloon Tooltips & Modern Toast Notification Interception via Windows Action Center
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`NIF_INFO`, `NIIF_USER | NIIF_LARGE_ICON`), Windows Runtime (`Windows.UI.Notifications`).
* **Subsystem Architecture**: The shell supports two tiers of notifications: classic balloon tooltips dispatched via `NIM_MODIFY` with `NIF_INFO`, and modern XML toast notifications routed via the Windows Action Center (`WNS`).
* **How-To & Safe Implementation**:
  1. For classic balloons, populate `szInfo`, `szInfoTitle`, and `dwInfoFlags` in `NOTIFYICONDATAW`.
  2. For modern toasts, register an AUMID shortcut in the Start Menu and dispatch a `ToastNotification` via `Windows.UI.Notifications.ToastNotificationManager`.
  3. Intercept user click responses via `NIN_BALLOONUSERCLICK` window messages.

#### Tactic 125: Volume & Network Flyout Anchor Interception via Subclassed Shell Hook Events
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWinEventHook`, `EVENT_OBJECT_SHOW`, `EVENT_OBJECT_HIDE`).
* **Subsystem Architecture**: Clicking the system volume or network tray icons spawns standalone modern flyout windows owned by `ShellHostExperienceHost.exe` or `explorer.exe`. Monitoring `EVENT_OBJECT_SHOW` for specific flyout class names enables custom animations or layout substitutions.
* **How-To & Safe Implementation**:
  1. Register out-of-context hook: `SetWinEventHook(EVENT_OBJECT_SHOW, EVENT_OBJECT_SHOW, NULL, WinEventProc, 0, 0, WINEVENT_OUTOFCONTEXT)`.
  2. Filter class names: `GetClassNameW(hwnd, szClass, 64);` matching `ControlCenterWindow` or `NativeHWNDHost`.
  3. Reposition or apply custom backdrop materials via `DwmSetWindowAttribute`.

#### Tactic 126: Clock & Calendar Flyout Custom Triggering via Simulated Tray Click Messages
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `SendMessageTimeoutW`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP`).
* **Subsystem Architecture**: The system clock is a child window of `TrayNotifyWnd` with class `TrayClockWClass`. Sending simulated click events opens the native calendar flyout programmatically from custom dock widgets.
* **How-To & Safe Implementation**:
  1. Locate clock handle: `HWND hClock = FindWindowExW(hTrayNotify, NULL, L"TrayClockWClass", NULL);`.
  2. Send activation: `PostMessageW(hClock, WM_LBUTTONDOWN, MK_LBUTTON, MAKELPARAM(5, 5)); PostMessageW(hClock, WM_LBUTTONUP, 0, MAKELPARAM(5, 5));`.
  3. Opens the official calendar without injecting code or modifying Explorer memory.

#### Tactic 127: System Tray Auto-Hide Suppression via AppBar Message (`ABM_SETSTATE`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHAppBarMessage`, `ABM_SETSTATE`, `ABS_AUTOHIDE`, `ABS_ALWAYSONTOP`).
* **Subsystem Architecture**: The taskbar's auto-hide and always-on-top behaviors are controlled by application desktop bar (AppBar) state machines maintained by `explorer.exe`.
* **How-To & Safe Implementation**:
  1. Initialize `APPBARDATA abd = { sizeof(abd) }; abd.hWnd = FindWindowW(L"Shell_TrayWnd", NULL);`.
  2. Set state: `abd.lParam = ABS_ALWAYSONTOP; SHAppBarMessage(ABM_SETSTATE, &abd);`.
  3. Eliminates erratic auto-hide flickering when custom floating docks are active.

#### Tactic 128: Action Center / Notification Center Programmatic Toggling via Shell Shortcut Simulation
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`keybd_event`, `SendInput`, `VK_LWIN`, `'A'`, `'N'`).
* **Subsystem Architecture**: The modern notification center (Win+N in Win11, Win+A in Win10) is bound to shell global hotkeys dispatched to `ShellExperienceHost.exe`.
* **How-To & Safe Implementation**:
  1. Construct `INPUT` array simulating `VK_LWIN` down, `'N'` down, `'N'` up, `VK_LWIN` up.
  2. Call `SendInput(4, pInputs, sizeof(INPUT))` from custom dock buttons.
  3. Toggles the native notifications pane instantly without requiring undocumented COM interfaces.

#### Tactic 129: Multi-Monitor Tray Mirroring via Secondary Taskbar Window Enumeration
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `EnumWindows`, `Shell_SecondaryTrayWnd`).
* **Subsystem Architecture**: By default, Windows displays the system notification area only on the primary monitor's taskbar. Secondary taskbars (`Shell_SecondaryTrayWnd`) lack a `TrayNotifyWnd` child.
* **How-To & Safe Implementation**:
  1. Enumerate secondary taskbars: `EnumWindows(FindSecondaryTraysProc, 0)`.
  2. Discover geometry and monitor bounds for each secondary tray.
  3. Instantiate a lightweight `WinLook` mirrored tray window anchored onto the secondary taskbar rect, forwarding user clicks back to the primary tray handles.

#### Tactic 130: Notification Area Clean Dismissal and Orphaned Icon Purge via Simulated Hover Sweep
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetClientRect`, `PostMessageW`, `WM_MOUSEMOVE`).
* **Subsystem Architecture**: When background processes crash or terminate abnormally, their notification icons persist in `TrayNotifyWnd` until the user manually hovers the cursor over them, causing Explorer to detect the invalid `HWND` and purge the dead slot.
* **How-To & Safe Implementation**:
  1. Locate `TrayNotifyWnd` and its child `ToolbarWindow32`.
  2. Query client rect: `GetClientRect(hToolbar, &rc);`.
  3. Programmatically post a sweep of `WM_MOUSEMOVE` coordinates across the width of the toolbar at 8-pixel increments: `PostMessageW(hToolbar, WM_MOUSEMOVE, 0, MAKELPARAM(x, y));`.
  4. Automatically sweeps away ghost icons instantly upon `WinLook` startup.


### Category N: Advanced Desktop Surfaces, Wallpaper Engines & Multi-Head Topology (Tactics 131–140)

#### Tactic 131: Wallpaper Slideshow Interval and Playlist Control via `IDesktopWallpaper::SetSlideshow`
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IDesktopWallpaper`, `IShellItemArray`, `SHCreateShellItemArrayFromIDLists`).
* **Subsystem Architecture**: Windows 8 introduced the `IDesktopWallpaper` COM interface, providing native multi-monitor wallpaper configuration and automated slideshow controls managed directly by the shell.
* **How-To & Safe Implementation**:
  1. Instantiate `CLSID_DesktopWallpaper` via `CoCreateInstance`.
  2. Assemble an `IShellItemArray` representing the folder of high-resolution images.
  3. Call `pDesktopWallpaper->SetSlideshow(pItemArray)`.
  4. Configure advance timing: `pDesktopWallpaper->SetSlideshowOptions(DSO_SHUFFLEIMAGES, 60000)` (e.g., 60 seconds).
  5. Advances wallpapers natively without running background polling threads.

#### Tactic 132: Per-Monitor Wallpaper Color Background Synchronization via `IDesktopWallpaper::SetBackgroundColor`
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IDesktopWallpaper::SetBackgroundColor`, `COLORREF`).
* **Subsystem Architecture**: When wallpapers use `DWPOS_FIT` or `DWPOS_CENTER`, letterbox borders appear around the image. `IDesktopWallpaper` allows setting an exact matching background solid color to create a seamless borderless effect.
* **How-To & Safe Implementation**:
  1. Sample dominant border color of the active image via `Gdiplus` or Direct2D.
  2. Call: `pDesktopWallpaper->SetBackgroundColor(RGB(r, g, b))`.
  3. Synchronizes solid background color with the image palette instantly.

#### Tactic 133: Live Video Wallpaper Playback via Media Foundation / DirectShow Docked behind `WorkerW`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `MFPLAT.DLL`, `EVR.DLL` (Enhanced Video Renderer), `DXVA2.DLL`.
* **Subsystem Architecture**: Rendering video wallpapers (MP4/WebM) at 60 FPS with near-zero CPU requires hardware-accelerated video decoding via Media Foundation, presenting output surfaces directly to the reparented `WorkerW` child canvas.
* **How-To & Safe Implementation**:
  1. Initialize Media Foundation: `MFStartup(MF_VERSION)`.
  2. Create an Enhanced Video Renderer (EVR) instance bound to the reparented `WorkerW` window handle.
  3. Configure hardware video acceleration (DXVA2 / D3D11VA) to decode directly into GPU video memory.
  4. Connect `wl_cpuv` watchdog to pause playback immediately when any foreground window covers 90%+ of the desktop screen area.

#### Tactic 134: Interactive Desktop Web Content Canvas via Embedded Lightweight WebView2 Controller
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `WebView2Loader.dll`, `ICoreWebView2Controller`, `ICoreWebView2Environment`.
* **Subsystem Architecture**: Interactive live wallpapers (HTML5/Canvas/WebGL) can be hosted inside an isolated Microsoft Edge WebView2 rendering surface docked behind desktop icons, offering web rendering without packaging multi-hundred megabyte Electron runtimes.
* **How-To & Safe Implementation**:
  1. Create child `HWND` docked into `WorkerW`.
  2. Initialize WebView2 environment: `CreateCoreWebView2EnvironmentWithOptions(NULL, szUserDataFolder, NULL, pCallback)`.
  3. In callback, create controller: `pEnv->CreateCoreWebView2Controller(hChildWorker, pControllerCallback)`.
  4. Size controller bounds to match monitor geometry: `pController->put_Bounds(rcMonitor)`.
  5. Configure transparent background color: `pController->put_DefaultBackgroundColor({ 0, 0, 0, 0 })`.

#### Tactic 135: Transparent Window Passthrough for Mouse Input via `WS_EX_TRANSPARENT`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WS_EX_TRANSPARENT`, `WS_EX_LAYERED`, `SetWindowLongPtrW`).
* **Subsystem Architecture**: When overlaying animated particles, falling snow, or ambient light effects across the desktop, the overlay window must not steal clicks intended for desktop icons or application windows. `WS_EX_TRANSPARENT` directs `win32k.sys` hit-testing to ignore the window completely for mouse messages.
* **How-To & Safe Implementation**:
  1. Create overlay window with extended styles: `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW`.
  2. All `WM_LBUTTONDOWN`, `WM_RBUTTONDOWN`, and mouse moves pass through to underlying desktop icons and windows automatically.

#### Tactic 136: Multi-Monitor Virtual Display Coordinate Remapping via `EnumDisplayMonitors`
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`EnumDisplayMonitors`, `GetMonitorInfoW`, `MONITORINFOEXW`).
* **Subsystem Architecture**: Multi-monitor desktop coordinates span virtual screen space where secondary monitors can have negative coordinate origins (e.g., monitor positioned to the left of primary has negative `rcMonitor.left`).
* **How-To & Safe Implementation**:
  1. Call: `EnumDisplayMonitors(NULL, NULL, MonitorEnumProc, 0)`.
  2. In callback, query: `MONITORINFOEXW mi = { sizeof(mi) }; GetMonitorInfoW(hMonitor, &mi);`.
  3. Track `mi.rcMonitor` (total display area) and `mi.rcWork` (work area excluding taskbar).
  4. Map `WinLook` canvas coordinates relative to virtual screen bounds: `SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`.

#### Tactic 137: Display Orientation Change Interception via `WM_DISPLAYCHANGE`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_DISPLAYCHANGE`, `ChangeDisplaySettingsW`).
* **Subsystem Architecture**: When a user rotates a tablet or display (switching from landscape to portrait) or changes display resolution, `win32k.sys` broadcasts `WM_DISPLAYCHANGE` across all top-level windows.
* **How-To & Safe Implementation**:
  1. In `WinLook` main window procedure, listen for `WM_DISPLAYCHANGE`.
  2. Parameters provide new depth and resolution: `bpp = (UINT)wParam; width = LOWORD(lParam); height = HIWORD(lParam);`.
  3. Re-evaluate monitor topology via `EnumDisplayMonitors` and resize `WorkerW` wallpaper surfaces to prevent visual tearing or distortion.

#### Tactic 138: Desktop Grid Icon Alignment & Snap Interception via `LVM_GETITEMPOSITION` & `LVM_SETITEMPOSITION`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageW`, `LVM_GETITEMPOSITION`, `LVM_SETITEMPOSITION32`, `LVM_ARRANGE`).
* **Subsystem Architecture**: The desktop icon surface is an instance of the SysListView32 common control (`SHELLDLL_DefView -> SysListView32`). ListView messages allow querying and positioning individual desktop icons programmatically.
* **How-To & Safe Implementation**:
  1. Obtain desktop ListView handle `hDeskList`.
  2. Query icon count: `int count = (int)SendMessageW(hDeskList, LVM_GETITEMCOUNT, 0, 0);`.
  3. Reposition icon: `SendMessageW(hDeskList, LVM_SETITEMPOSITION32, iIndex, (LPARAM)&ptPosition);`.
  4. Allows custom layout grids, circular arrangements, or auto-sorting without modifying shell binaries.

#### Tactic 139: Dynamic Desktop Icon Size Control via `LVM_SETICONSPACING`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageW`, `LVM_SETICONSPACING`, `WM_COMMAND`).
* **Subsystem Architecture**: The spacing grid between desktop icons can be configured programmatically using `LVM_SETICONSPACING`, specifying horizontal and vertical pixel margins between adjacent icons.
* **How-To & Safe Implementation**:
  1. Send message: `SendMessageW(hDeskList, LVM_SETICONSPACING, 0, MAKELPARAM(cxSpacing, cySpacing));`.
  2. To cycle standard sizes (Small, Medium, Large), post shell command IDs to `SHELLDLL_DefView`: `SendMessageW(hDefView, WM_COMMAND, MAKEWPARAM(0x7051, 0), 0)` (Large), `0x7052` (Medium), `0x7053` (Small).

#### Tactic 140: Desktop Background Shading and Drop Shadow Toggle via `SPI_SETDROPSHADOW`
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_SETDROPSHADOW`, `SPIF_UPDATEINIFILE | SPIF_SENDCHANGE`).
* **Subsystem Architecture**: Desktop icon labels feature translucent drop shadows to remain legible over bright photographic wallpapers. `SystemParametersInfoW` controls whether GDI renders these drop shadows.
* **How-To & Safe Implementation**:
  1. Enable: `SystemParametersInfoW(SPI_SETDROPSHADOW, 0, (void*)TRUE, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);`.
  2. Invalidate desktop: `InvalidateRect(hDeskList, NULL, TRUE);`.
  3. Cleanly configurable via `windows.all.desktop.icon.drop_shadow` namespace path.


### Category O: Shell Namespace, Virtual Folders & Context Menu Extensibility (Tactics 141–150)

#### Tactic 141: Shell Namespace Folder Extension via Shell Virtual Root Registration
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace`).
* **Subsystem Architecture**: Windows Explorer populates the Desktop and This PC roots by enumerating CLSID keys registered under `...\Explorer\Desktop\NameSpace`. Adding a custom CLSID allows `WinLook` to present a virtual folder representing themes, widgets, or tools directly on the desktop.
* **How-To & Safe Implementation**:
  1. Register custom CLSID under `HKCU\Software\Classes\CLSID\{GUID}` with `LocalizedString` and `DefaultIcon`.
  2. Add key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace\{GUID}`.
  3. Implement shell folder interfaces (`IShellFolder2`, `IPersistFolder`) in a lightweight DLL.
  4. Unregistration simply removes the single registry key, leaving no residual system footprint.

#### Tactic 142: Shell Context Menu Non-Invasive Extension via Directory/Background Shell Verbs
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKCU\Software\Classes\Directory\Background\shell`).
* **Subsystem Architecture**: Right-clicking the empty desktop background invokes context menu verbs registered under `Directory\Background\shell`. Registering per-user shell verbs allows adding a "WinLook Settings" or "Switch Theme" action without installing COM shell extensions.
* **How-To & Safe Implementation**:
  1. Create key: `HKCU\Software\Classes\Directory\Background\shell\WinLook`.
  2. Set values: `MUIVerb = "WinLook Customizer"`, `Icon = "C:\Path\To\WinLook.exe,0"`.
  3. Create subkey: `...\WinLook\command` with `Default = ""C:\Path\To\WinLook.exe" --settings"`.
  4. Works across all Windows versions with zero risk of crashing `explorer.exe`.

#### Tactic 143: Start Menu Search Flyout Interception via Window Class Discovery (`Windows.UI.Core.CoreWindow`)
* **Target Versions**: Windows 10 and Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowW`, `SetWinEventHook`, `EVENT_SYSTEM_FOREGROUND`).
* **Subsystem Architecture**: The Windows Search flyout (`SearchHost.exe`) runs inside a modern container window. Tracking foreground window changes allows `WinLook` to detect search activation and display complementary quick-action docks or custom search widgets.
* **How-To & Safe Implementation**:
  1. Listen for foreground transitions to window class `Windows.UI.Core.CoreWindow` titled `Search`.
  2. Query position and bounds via `GetWindowRect`.
  3. Display animated companion HUD docked alongside the search interface.

#### Tactic 144: Task View / Timeline Virtual Desktop Interface Querying via `IVirtualDesktopManager`
* **Target Versions**: Windows 10 (Build 10240+) and Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IVirtualDesktopManager`, `CLSID_VirtualDesktopManager`).
* **Subsystem Architecture**: Modern Windows supports multiple virtual desktops. The shell provides `IVirtualDesktopManager` to determine whether a given window is present on the currently active desktop or belongs to an alternate workspace.
* **How-To & Safe Implementation**:
  1. Instantiate manager: `CoCreateInstance(CLSID_VirtualDesktopManager, NULL, CLSCTX_INPROC_SERVER, IID_IVirtualDesktopManager, (void**)&pVDM);`.
  2. Check window visibility: `BOOL bOnCurrent; pVDM->IsWindowOnCurrentVirtualDesktop(hwnd, &bOnCurrent);`.
  3. Move window across desktops: `pVDM->MoveWindowToDesktop(hwnd, desktopGuid);`.
  4. Ensures custom taskbars and docks filter out windows residing on inactive virtual desktops.

#### Tactic 145: Pin/Unpin Shortcuts Programmatically via Shell Item Verbs (`taskbarpin` / `taskbarunpin`)
* **Target Versions**: Windows 7 through Windows 10 (Build 1809).
* **APIs / Drivers**: `SHELL32.DLL` (`IShellItem`, `IShellFolder`, `IContextMenu`, `InvokeCommand`).
* **Subsystem Architecture**: Direct programmatic taskbar pinning is restricted by Microsoft to prevent adware abuse. In supported builds, invoking the shell context verb `taskbarpin` via `IContextMenu` allows user-initiated pinning of curated toolsets.
* **How-To & Safe Implementation**:
  1. Bind to target shortcut shell item via `SHCreateItemFromParsingName`.
  2. Query `IContextMenu` interface.
  3. Invoke verb: `CMINVOKECOMMANDINFO cmi = { sizeof(cmi) }; cmi.lpVerb = "taskbarpin"; pContextMenu->InvokeCommand(&cmi);`.
  4. On modern Windows 10/11 where verb invocation is blocked, manage pinned items declaratively via TaskbarLayoutModification.xml during system setup.

#### Tactic 146: Recent Files & Frequent Places Ingestion via `SHAddToRecentDocs`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHAddToRecentDocs`, `SHARD_PATHW`, `SHARD_SHELLITEM`).
* **Subsystem Architecture**: The shell maintains tracking lists of recently opened documents displayed in Jump Lists and File Explorer Quick Access. Applications can programmatically push items into this list or clear history.
* **How-To & Safe Implementation**:
  1. Push new document: `SHAddToRecentDocs(SHARD_PATHW, pszFilePath);`.
  2. Clear entire recent documents history: `SHAddToRecentDocs(0, NULL);`.
  3. Keeps shell Jump Lists synchronized with files manipulated inside `WinLook`.

#### Tactic 147: Explorer Address Bar Breadcrumb Inspection & Out-of-Process UI Automation
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`), `UIAutomationCore.dll` (`IUIAutomation`, `IUIAutomationElement`).
* **Subsystem Architecture**: The Windows Explorer address bar is hosted inside a `WorkerW` container within the cabinet window (`CabinetWClass`), featuring a breadcrumb bar control (`Breadcrumb Parent`).
* **Architectural Execution Boundary Notice**:
  *Important*: A separate process cannot call `SetWindowSubclass` on Explorer's internal breadcrumb control due to Win32 address space isolation. Forcing code into Explorer requires DLL injection, violating WinLook's non-invasive design.
* **How-To & Safe Implementation**:
  1. Locate target Explorer cabinet: `HWND hCab = FindWindowW(L"CabinetWClass", NULL);`.
  2. Bind to Explorer's UI Automation tree via `IUIAutomation::ElementFromHandle(hCab)`.
  3. Query navigation path state via `UIA_ValuePatternId` or `UIA_TextPatternId` to read breadcrumbs out-of-process.
  4. For styling, rely on system-wide UxTheme / High-Contrast registry styles rather than attempting cross-process subclassing.

#### Tactic 148: Shell Folder Change Notifications via `SHChangeNotifyRegister`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHChangeNotifyRegister`, `SHChangeNotifyDeregister`, `SHCNF_TYPE`).
* **Subsystem Architecture**: Subscribing to shell notifications allows background utilities to receive asynchronous window messages whenever files, folders, or drive mappings are created, renamed, deleted, or updated.
* **How-To & Safe Implementation**:
  1. Define PIDL list: `SHChangeNotifyEntry entry = { pidlDesktop, TRUE };`.
  2. Register: `ULONG uId = SHChangeNotifyRegister(hwndNotify, SHCNRF_ShellLevel | SHCNRF_InterruptLevel, SHCNE_ALLEVENTS, WM_USER_SHELLCHANGE, 1, &entry);`.
  3. Handle `WM_USER_SHELLCHANGE` messages to refresh custom desktop file widgets in real time.
  4. Deregister upon shutdown: `SHChangeNotifyDeregister(uId)`.

#### Tactic 149: Drag-and-Drop Target Customization via `IDropTarget` Subclassing on Custom Docks
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `OLE32.DLL` (`RegisterDragDrop`, `RevokeDragDrop`, `IDropTarget`).
* **Subsystem Architecture**: OLE Drag and Drop enables custom dock bars to accept files dragged from Explorer, extracting `CF_HDROP` clipboard formats to pin new shortcuts or launch documents dynamically.
* **How-To & Safe Implementation**:
  1. Initialize OLE: `OleInitialize(NULL);`.
  2. Implement `IDropTarget` interface (`DragEnter`, `DragOver`, `DragLeave`, `Drop`).
  3. Register on custom dock handle: `RegisterDragDrop(hwndDock, pDropTarget);`.
  4. On `Drop`, query `HDROP hDrop = (HDROP)medium.hGlobal; DragQueryFileW(hDrop, 0, szPath, MAX_PATH);`.
  5. Dynamically spawns new dock item with extracted icon and title.

#### Tactic 150: Graceful Explorer Shell Restart with Persistent State Handshake via Custom Mutex Watchdog
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`PostMessageW`, `WM_QUIT`), `KERNEL32.DLL` (`CreateMutexW`, `CreateProcessW`).
* **Subsystem Architecture**: When shell configuration updates require an Explorer reload (e.g., refreshing taskbar button policy), forcibly killing `explorer.exe` via `TerminateProcess` risks file corruption and system tray icon loss. Graceful shutdown requests preserve shell state.
* **How-To & Safe Implementation**:
  1. Signal graceful shell exit: `HWND hTray = FindWindowW(L"Shell_TrayWnd", NULL); PostMessageW(hTray, WM_USER + 436, 0, 0);` (legacy clean exit) or `SendMessageTimeoutW(hTray, WM_CLOSE, 0, 0, SMTO_ABORTIFHUNG, 2000, &dwResult)`.
  2. WinLook watchdog detects Explorer termination via `RegisterWaitForSingleObject`.
  3. Watchdog re-launches `explorer.exe` via `CreateProcessW(L"C:\Windows\explorer.exe", ...)`.
  4. Once `Shell_TrayWnd` reappears, WinLook re-establishes shell hooks and WorkerW wallpaper canvases seamlessly.


---


### Category P: Linux-Style Tiling Window Management & Custom Layout Engines (Tactics 151–160)

#### Tactic 151: Window Event Synchronization via Out-of-Context Event Hooks
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWinEventHook`, `UnhookWinEvent`, `EVENT_OBJECT_LOCATIONCHANGE`, `EVENT_OBJECT_SHOW`, `EVENT_OBJECT_HIDE`, `WINEVENT_OUTOFCONTEXT`).
* **Subsystem Architecture**: Tiling window managers (like i3/bspwm on Linux) require instantaneous awareness when windows move, resize, minimize, or appear. In Windows, out-of-context WinEvent hooks receive asynchronous notifications from `win32k.sys` without injecting DLLs into external processes.
* **Open-Source Reference**: `LGUG2Z/komorebi` (Rust tiling window manager) and `glzr-io/glazewm` (C# tiling WM).
* **How-To & Safe Implementation**:
  1. Register hook: `SetWinEventHook(EVENT_OBJECT_SHOW, EVENT_OBJECT_LOCATIONCHANGE, NULL, WinEventProc, 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)`.
  2. In callback, filter out non-window objects: verify `idObject == OBJID_WINDOW` and `idChild == CHILDID_SELF`.
  3. Validate candidate `HWND` with `IsWindowVisible(hwnd)` and ensure it is not minimized (`!IsIconic(hwnd)`).
  4. Post an internal event to WinLook's layout thread to queue a layout recalculation without blocking the hook thread.

#### Tactic 152: Cloaked Window Detection to Prevent Ghost Virtual Desktop Tiles
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmGetWindowAttribute`, `DWMWA_CLOAKED`, `DWM_CLOAKED_SHELL`, `DWM_CLOAKED_APP`, `DWM_CLOAKED_INHERITED`).
* **Subsystem Architecture**: When a user switches virtual desktops on Windows 10/11, Windows does not hide background windows via `ShowWindow(SW_HIDE)`; instead, DWM sets their `DWMWA_CLOAKED` attribute. Unaware tiling managers treat cloaked windows as active, leaving empty gaps or distorted layouts.
* **Open-Source Reference**: `glzr-io/glazewm` and `LGUG2Z/komorebi`.
* **How-To & Safe Implementation**:
  1. Before tiling a window, query: `DWORD cloaked = 0; DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &cloaked, sizeof(cloaked));`.
  2. If `cloaked != 0`, immediately exclude the window from the active tiling tree.
  3. Re-evaluate window state on `HSHELL_WINDOWACTIVATED` or `EVENT_SYSTEM_FOREGROUND` to restore tiles when their workspace becomes active.

#### Tactic 153: Non-Client Frame Stripping for Borderless Window Tiling
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowLongPtrW`, `SetWindowLongPtrW`, `GWL_STYLE`, `WS_CAPTION`, `WS_THICKFRAME`, `WS_MINIMIZEBOX`, `WS_MAXIMIZEBOX`).
* **Subsystem Architecture**: Classic Windows applications render thick resizing borders and caption bars. Linux-like aesthetic tiling requires stripping decorations while retaining interactive window procedures.
* **Open-Source Reference**: `glzr-io/glazewm` and `neovim/neovim` Windows embedding.
* **How-To & Safe Implementation**:
  1. Read current style: `LONG_PTR style = GetWindowLongPtrW(hwnd, GWL_STYLE);`.
  2. Cache original style in `wl_journal` for 100% reversibility.
  3. Strip caption and sizing borders: `SetWindowLongPtrW(hwnd, GWL_STYLE, style & ~(WS_CAPTION | WS_THICKFRAME));`.
  4. Flush frame metrics: `SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);`.

#### Tactic 154: Binary Space Partitioning (BSP) Tree Layout Computation
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Pure computational mathematics engine in Rust (`wl_wlde` / `wl_wm`).
* **Subsystem Architecture**: Linux bspwm organizes screen space as the leaves of a binary tree, splitting nodes horizontally or vertically according to container aspect ratios.
* **Open-Source Reference**: `baskerville/bspwm` (Linux reference) and `LGUG2Z/komorebi` (Rust Windows port).
* **How-To & Safe Implementation**:
  1. Represent monitors as root nodes with physical bounding rectangles `(x, y, w, h)`.
  2. On window insertion, choose split orientation: `if (node.width > node.height) Split::Vertical else Split::Horizontal`.
  3. Recursively calculate leaf bounds applying configurable inner/outer margins: `rect.left += margin_left; rect.right -= margin_right;`.
  4. Pass computed leaf rectangles to atomic positioning subsystem (Tactic 155).

#### Tactic 155: Atomic Multi-Window Geometry Dispatch via `BeginDeferWindowPos`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`BeginDeferWindowPos`, `DeferWindowPos`, `EndDeferWindowPos`, `SWP_NOACTIVATE`, `SWP_NOZORDER`).
* **Subsystem Architecture**: Calling `SetWindowPos` sequentially across 10 open windows causes visible stutter, flickering, and multiple DWM recomposition passes. `BeginDeferWindowPos` issues an atomic batch update to `win32k.sys`.
* **Open-Source Reference**: `LGUG2Z/komorebi` and `AutoHotkey/AutoHotkey`.
* **How-To & Safe Implementation**:
  1. Initialize batch: `HDWP hdwp = BeginDeferWindowPos(window_count);`.
  2. For each tile in BSP tree: `hdwp = DeferWindowPos(hdwp, tile.hwnd, NULL, tile.x, tile.y, tile.w, tile.h, SWP_NOACTIVATE | SWP_NOZORDER);`.
  3. Commit atomically: `EndDeferWindowPos(hdwp);`—`win32k.sys` executes all moves within a single graphics refresh cycle.

#### Tactic 156: Monitor Work Area Inset Modification via `SPI_SETWORKAREA`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_GETWORKAREA`, `SPI_SETWORKAREA`, `SPIF_SENDCHANGE`).
* **Subsystem Architecture**: Maximized windows clamp to the primary monitor's work area. Modifying the work area allows WinLook to reserve desktop margins for custom status bars (Polybar style) or floating docks without window clipping.
* **Open-Source Reference**: `valinet/ExplorerPatcher` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Retrieve current area: `RECT rcWork; SystemParametersInfoW(SPI_GETWORKAREA, 0, &rcWork, 0);`.
  2. Cache original `rcWork` in `wl_journal`.
  3. Shrink work area by top bar height: `rcWork.top += custom_bar_height;`.
  4. Apply globally: `SystemParametersInfoW(SPI_SETWORKAREA, 0, &rcWork, SPIF_SENDCHANGE);`. On process exit, restore original cached work area.

#### Tactic 157: Focus-Follows-Mouse Tracking via `SPI_SETACTIVEWINDOWTRACKING`
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_GETACTIVEWINDOWTRACKING`, `SPI_SETACTIVEWINDOWTRACKING`, `SPI_SETACTIVEWNDTRKZORDER`, `SPI_SETACTIVEWNDTRKTIMEOUT`).
* **Subsystem Architecture**: A staple of Linux X11 window managers (XFCE, i3) is "focus follows mouse," where hovering over an unfocused window brings it to focus without requiring a click. Windows possesses native kernel support for this via SystemParametersInfo.
* **Open-Source Reference**: `neovim/neovim` and native X11 port utilities.
* **How-To & Safe Implementation**:
  1. Enable tracking: `BOOL track = TRUE; SystemParametersInfoW(SPI_SETACTIVEWINDOWTRACKING, 0, (void*)track, SPIF_SENDCHANGE);`.
  2. Configure Z-order raising policy: `BOOL raise_z = FALSE; SystemParametersInfoW(SPI_SETACTIVEWNDTRKZORDER, 0, (void*)raise_z, SPIF_SENDCHANGE);` (keeps window focused without bringing it to top).
  3. Set tracking delay (milliseconds): `DWORD delay = 50; SystemParametersInfoW(SPI_SETACTIVEWNDTRKTIMEOUT, 0, (void*)(uintptr_t)delay, SPIF_SENDCHANGE);`.

#### Tactic 158: Active Window Colored Border Overlay via Layered Tracking Windows
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`CreateWindowExW`, `WS_EX_LAYERED`, `WS_EX_TRANSPARENT`, `WS_EX_TOOLWINDOW`, `WS_EX_NOACTIVATE`, `SetWindowPos`).
* **Subsystem Architecture**: In Linux tiling WMs, the currently active window is emphasized with a sharp, vibrant border (e.g. 2px neon cyan). On Windows, modern DWM borders are thin and often unresponsive to custom RGB colors on older builds. A borderless transparent layered window tracking the active target renders crisp hardware-accelerated accent frames.
* **Open-Source Reference**: `LGUG2Z/komorebi` (active-border implementation) and `fannheyward/hypr-border`.
* **How-To & Safe Implementation**:
  1. Create overlay window with styles: `WS_POPUP`, `WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`.
  2. On active window change (`EVENT_SYSTEM_FOREGROUND`), query target geometry via `GetWindowRect`.
  3. Size overlay window to match target rect + 2px border thickness.
  4. Paint 2px colored border using GDI `FrameRect` or Direct2D into the layered surface, leaving interior transparent (`RGB(0,0,0)` color key or per-pixel alpha).

#### Tactic 159: Native Fullscreen Window Toggle Bypassing `WS_MAXIMIZE`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetWindowLongPtrW`, `SetWindowLongPtrW`, `SetWindowPos`, `GetMonitorInfoW`).
* **Subsystem Architecture**: In Windows, standard window maximization (`ShowWindow(SW_MAXIMIZE)`) preserves taskbar insets and invokes DWM frame expansion. In Linux, monocle/fullscreen mode stretches the client surface over the entire monitor resolution.
* **Open-Source Reference**: `mpv-player/mpv` and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. When entering monocle mode, query monitor bounds via `GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mi)`.
  2. Cache current placement (`GetWindowPlacement`).
  3. Strip borders: `SetWindowLongPtrW(hwnd, GWL_STYLE, style & ~(WS_CAPTION | WS_THICKFRAME));`.
  4. Stretch across exact monitor bounds: `SetWindowPos(hwnd, HWND_TOP, mi.rcMonitor.left, mi.rcMonitor.top, mi.rcMonitor.right - mi.rcMonitor.left, mi.rcMonitor.bottom - mi.rcMonitor.top, SWP_NOOWNERZORDER | SWP_FRAMECHANGED);`.

#### Tactic 160: Declarative Window Rule Engine Matching AUMID, ClassName, and Process
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`GetClassNameW`), `KERNEL32.DLL` (`QueryFullProcessImageNameW`), WinRT (`AppUserModelId`).
* **Subsystem Architecture**: Tiling WMs require granular rules (e.g. float dialogs, tile main windows, assign Discord to workspace 3).
* **Open-Source Reference**: `LGUG2Z/komorebi` (rules schema) and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. On window creation, extract: ClassName via `GetClassNameW`, ProcessName via `QueryFullProcessImageNameW`, and Title via `GetWindowTextW`.
  2. Check against `WinLookConfig` rules regex: `<Rule process=\"spotify.exe\" workspace=\"4\" floating=\"false\"/>`.
  3. If window matches floating criteria (e.g. style has `WS_POPUP` or dialog class `#32770`), route to floating manager instead of tiling BSP tree.

### Category Q: Custom AppBars, Floating Status Bars & Polybar-Style Panels (Tactics 161–170)

#### Tactic 161: Application Desktop Toolbar Registration via `SHAppBarMessage(ABM_NEW)`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHAppBarMessage`, `ABM_NEW`, `ABM_REMOVE`, `APPBARDATA`).
* **Subsystem Architecture**: The official, system-sanctioned way to create custom desktop panels (like Linux Polybar, Waybar, or tint2) that reserve screen space without overlapping other maximized windows is the Shell AppBar interface.
* **Open-Source Reference**: `dremin/RetroBar` (classic taskbar replacement) and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Populate structure: `APPBARDATA abd = { sizeof(abd) }; abd.hWnd = hwndCustomBar; abd.uCallbackMessage = WM_APPBAR_CALLBACK;`.
  2. Register with shell: `SHAppBarMessage(ABM_NEW, &abd);`.
  3. Window procedure now receives notifications whenever display resolution changes or other AppBars update.
  4. Always deregister on shutdown: `SHAppBarMessage(ABM_REMOVE, &abd);` to restore full screen dimensions.

#### Tactic 162: Exclusive Screen Real Estate Reservation via `ABM_SETPOS`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHAppBarMessage`, `ABM_QUERYPOS`, `ABM_SETPOS`, `ABE_TOP`, `ABE_BOTTOM`, `ABE_LEFT`, `ABE_RIGHT`).
* **Subsystem Architecture**: AppBars dynamically negotiate docking dimensions with `explorer.exe` so neither the native taskbar nor the custom bar clip each other.
* **Open-Source Reference**: `dremin/RetroBar` and `AmN/yasb` (Yet Another Status Bar).
* **How-To & Safe Implementation**:
  1. Propose desired rect: `abd.uEdge = ABE_TOP; abd.rc = { 0, 0, screen_width, bar_height };`.
  2. Ask shell to adjust for existing bars: `SHAppBarMessage(ABM_QUERYPOS, &abd);`.
  3. Commit adjusted rect: `SHAppBarMessage(ABM_SETPOS, &abd);`.
  4. Move the custom window to the approved dimensions: `MoveWindow(hwnd, abd.rc.left, abd.rc.top, abd.rc.right - abd.rc.left, abd.rc.bottom - abd.rc.top, TRUE);`.

#### Tactic 163: Fullscreen Application Detection via `ABN_FULLSCREENAPP`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`APPBARDATA`, `ABN_FULLSCREENAPP`, `ABN_POSCHANGED`).
* **Subsystem Architecture**: When a game or media player goes fullscreen, custom bars must automatically yield Z-order to prevent visual tearing or blocking gameplay.
* **Open-Source Reference**: `dremin/RetroBar` and `valinet/ExplorerPatcher`.
* **How-To & Safe Implementation**:
  1. In `WM_APPBAR_CALLBACK`, evaluate `wParam`:
  2. When `wParam == ABN_FULLSCREENAPP`: `BOOL is_fullscreen = (BOOL)lParam;`.
  3. If `is_fullscreen` is TRUE, push bar down in Z-order: `SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);`.
  4. When fullscreen app closes (`is_fullscreen == FALSE`), restore top-level visibility: `SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);`.

#### Tactic 164: Non-Activating Floating Panel Creation (`WS_EX_NOACTIVATE`)
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`CreateWindowExW`, `WS_EX_NOACTIVATE`, `WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`).
* **Subsystem Architecture**: Status bars, widgets, and floating islands should never steal focus or activate away from the user's active coding editor or browser when clicked.
* **Open-Source Reference**: `zebar-io/zebar` (custom HTML/Rust bar) and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Instantiate window with extended styles: `WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW`.
  2. Subclass window to handle mouse clicks: intercept `WM_MOUSEACTIVATE` and return `MA_NOACTIVATE`.
  3. Users can click widgets, volume sliders, or workspace buttons on the bar without unfocusing their active terminal.

#### Tactic 165: Hiding Windows from the Alt+Tab Task Switcher
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowLongPtrW`, `GWL_EXSTYLE`, `WS_EX_TOOLWINDOW`, `WS_EX_APPWINDOW`).
* **Subsystem Architecture**: Custom bars and floating widgets should reside cleanly on the desktop without cluttering the user's Alt+Tab or taskbar pagers.
* **Open-Source Reference**: `microsoft/PowerToys` (PowerToys Run) and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Ensure window has style `WS_EX_TOOLWINDOW` and does NOT have `WS_EX_APPWINDOW`.
  2. Set unowned parent or parent to a hidden message-only window (`HWND_MESSAGE`).
  3. Result: Windows completely omits the window from Alt+Tab, Task View, and the native taskbar icon tray.

#### Tactic 166: Multi-Monitor Independent Status Bar Spanning
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`EnumDisplayMonitors`, `GetMonitorInfoW`, `MONITORINFOEXW`).
* **Subsystem Architecture**: Polybar allows per-monitor configurations. On Windows, each monitor requires its own dedicated AppBar window instance docked to that display's individual work bounds.
* **Open-Source Reference**: `dremin/RetroBar` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Enumerate displays via `EnumDisplayMonitors`.
  2. For each monitor: instantiate a separate `HWND`, setting `abd.rc` to the monitor's coordinate rectangle `(mi.rcMonitor.left, mi.rcMonitor.top, ...)`.
  3. Execute `SHAppBarMessage(ABM_NEW)` per monitor handle.
  4. Enables independent widgets (e.g. monitor 1 displays workspaces 1-5; monitor 2 displays workspaces 6-10).

#### Tactic 167: Rounded "Pill-Style" Island Dock Geometry via DirectComposition
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `DCOMP.DLL` (`DCompositionCreateDevice`, `IDCompositionVisual`, `IDCompositionRectangleClip`), Direct2D.
* **Subsystem Architecture**: Modern Linux rice setups frequently feature floating, rounded island status bars disconnected from screen edges.
* **Open-Source Reference**: `zebar-io/zebar` and `microsoft/WindowsCompositionSamples`.
* **How-To & Safe Implementation**:
  1. Create a borderless layered window with `WS_EX_LAYERED | WS_EX_NOACTIVATE`.
  2. Initialize DirectComposition visual tree target bound to the `HWND`.
  3. Create an `IDCompositionRectangleClip` with rounded corner radius (e.g. `SetRadiusX(16.0f)`, `SetRadiusY(16.0f)`).
  4. Render background with Acrylic/Mica or dark translucent brush, delivering anti-aliased floating pills with zero clipping jaggedness.

#### Tactic 168: Secondary Native Taskbar Suppression (`Shell_SecondaryTrayWnd`)
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `ShowWindow`, `SW_HIDE`, `SW_SHOW`).
* **Subsystem Architecture**: While the primary taskbar is `Shell_TrayWnd`, auxiliary monitors host instances of `Shell_SecondaryTrayWnd`. When deploying multi-monitor custom bars, all secondary bars must be cleanly suppressed.
* **Open-Source Reference**: `valinet/ExplorerPatcher` and `dremin/RetroBar`.
* **How-To & Safe Implementation**:
  1. Enumerate secondary taskbars:
     ```cpp
     HWND hSec = NULL;
     while ((hSec = FindWindowExW(NULL, hSec, L"Shell_SecondaryTrayWnd", NULL)) != NULL) {
         ShowWindow(hSec, SW_HIDE);
     }
     ```
  2. Cache hidden handles in `wl_journal`. On shutdown, iterate and restore with `SW_SHOW`.

#### Tactic 169: Desktop Work Area Restoration on Shell Restart
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterWindowMessageW`, `WM_SETTINGCHANGE`), `SHELL32.DLL` (`SHAppBarMessage`).
* **Subsystem Architecture**: If `explorer.exe` crashes or restarts, all existing AppBar registrations are invalidated by the shell. Custom bars must re-register themselves to maintain screen reservations.
* **Open-Source Reference**: `dremin/RetroBar` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Register shell restart message: `UINT msgTaskbarCreated = RegisterWindowMessageW(L"TaskbarCreated");`.
  2. In window procedure: `if (msg == msgTaskbarCreated) { ReRegisterAppBar(); }`.
  3. Re-execute `SHAppBarMessage(ABM_NEW)` and `ABM_SETPOS` to seamlessly reclaim reserved desktop space.

#### Tactic 170: Mouse Event Passthrough on Transparent Widget Backdrops
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_NCHITTEST`, `HTTRANSPARENT`, `HTCLIENT`).
* **Subsystem Architecture**: Floating widget panels (like Conky on Linux) often render system metrics over the desktop. Transparent empty regions must pass clicks through to desktop icons.
* **Open-Source Reference**: `rainmeter/rainmeter` and `fannheyward/hypr-border`.
* **How-To & Safe Implementation**:
  1. Intercept `WM_NCHITTEST` in widget window procedure.
  2. Map cursor screen coordinates to widget client coordinates (`ScreenToClient`).
  3. Sample alpha value of pixel at coordinate. If `alpha == 0` (empty background), return `HTTRANSPARENT`.
  4. Windows routes the mouse click directly to the underlying desktop icon listview or application window.

### Category R: Global Input, Hotkeys & Daemon Interception (Tactics 171–180)

#### Tactic 171: High-Performance Low-Level Keyboard Interception (`WH_KEYBOARD_LL`)
* **Target Versions**: Windows NT 4.0 SP3 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowsHookExW`, `UnhookWindowsHookEx`, `CallNextHookEx`, `KBDLLHOOKSTRUCT`).
* **Subsystem Architecture**: Linux power users rely on `sxhkd` to bind arbitrary key combinations to scripts. On Windows, `WH_KEYBOARD_LL` installs a global hook that intercepts keystrokes before applications process them.
* **Open-Source Reference**: `AutoHotkey/AutoHotkey` and `evilwm/evilwm` Windows ports.
* **How-To & Safe Implementation**:
  1. Register hook: `HHOOK hHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, GetModuleHandleW(NULL), 0);`.
  2. In callback, inspect `KBDLLHOOKSTRUCT`: check `vkCode`, `flags & LLKHF_ALTDOWN`, and `flags & LLKHF_UP`.
  3. To swallow an intercepted keybind, return `1`; to pass through to the OS, call `CallNextHookEx(hHook, nCode, wParam, lParam)`.
  4. Safety mandate: Never execute long-running tasks inside the hook procedure; dispatch to a worker thread within 5ms to avoid Windows killing the hook via `LowLevelHooksTimeout`.

#### Tactic 172: Low-Level Hooks Timeout Mitigation via Watchdog Health Checks
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Control Panel\Desktop\LowLevelHooksTimeout`), `USER32.DLL` (`UnhookWindowsHookEx`).
* **Subsystem Architecture**: Windows will silently remove a low-level hook without notification if a callback blocks for longer than the registry value `LowLevelHooksTimeout` (default 200ms–1000ms).
* **Open-Source Reference**: `AutoHotkey/AutoHotkey` and `microsoft/PowerToys` (Keyboard Manager).
* **How-To & Safe Implementation**:
  1. The hook procedure only places raw `KBDLLHOOKSTRUCT` into a lock-free ring buffer and sets an event flag.
  2. Background worker thread consumes the buffer and evaluates shortcut combinations.
  3. Heartbeat watchdog periodically verifies hook presence; if silently dropped, it reinstalls automatically.

#### Tactic 173: Low-Level Mouse Gestures and Edge Snapping (`WH_MOUSE_LL`)
* **Target Versions**: Windows NT 4.0 SP3 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowsHookExW`, `MSLLHOOKSTRUCT`, `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`).
* **Subsystem Architecture**: Enables mouse gestures (e.g. flicking to screen corners to switch workspaces or show application overview).
* **Open-Source Reference**: `StrokesPlus/StrokesPlus.net` and `AutoHotkey/AutoHotkey`.
* **How-To & Safe Implementation**:
  1. Register `WH_MOUSE_LL` hook.
  2. In callback, record `MSLLHOOKSTRUCT.pt` trajectory history.
  3. Calculate directional vectors across time $\Delta t$. If gesture matches pattern (e.g. Right -> Down), trigger action.
  4. Swallows triggering mouse click if gesture recognized, preventing accidental clicks in underlying apps.

#### Tactic 174: Hardware Scancode Input Synthesis via `SendInput`
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendInput`, `INPUT`, `KEYBDINPUT`, `KEYEVENTF_SCANCODE`, `KEYEVENTF_KEYUP`).
* **Subsystem Architecture**: Unlike legacy `keybd_event` or `SendMessage(WM_KEYDOWN)`, modern anti-cheat and protected applications ignore synthetic virtual-key events. `SendInput` with `KEYEVENTF_SCANCODE` emits true hardware keyboard scancodes.
* **Open-Source Reference**: `AutoHotkey/AutoHotkey` and `microsoft/PowerToys`.
* **How-To & Safe Implementation**:
  1. Map virtual key to scan code: `UINT sc = MapVirtualKeyW(VK_SPACE, MAPVK_VK_TO_VSC);`.
  2. Populate `INPUT`: `inp.type = INPUT_KEYBOARD; inp.ki.wScan = sc; inp.ki.dwFlags = KEYEVENTF_SCANCODE;`.
  3. Submit batch: `SendInput(1, &inp, sizeof(INPUT));`.

#### Tactic 175: Global Windows Key (Super Key) Binding Override
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowsHookExW`, `WH_KEYBOARD_LL`, `VK_LWIN`, `VK_RWIN`).
* **Subsystem Architecture**: In Linux i3/sway, the Super/Windows key is the primary modifier for all WM actions (Super+Enter for terminal, Super+Q to close). In Windows, pressing Win opens the Start Menu unless intercepted before `explorer.exe`.
* **Open-Source Reference**: `AutoHotkey/AutoHotkey` and `LGUG2Z/komorebi`.
* **How-To & Safe Implementation**:
  1. In `WH_KEYBOARD_LL`, track whether `VK_LWIN` or `VK_RWIN` is pressed down.
  2. If accompanied by another key (e.g. `Win + Return`): consume both, trigger terminal launch, and send an extra dummy keyup (`VK_NONCONVERT` or `VK_CONTROL`) to prevent Explorer from opening the Start menu.
  3. If released alone without combination, allow normal Start menu activation.

#### Tactic 176: High-DPI Sub-Pixel Mouse Navigation via Raw Input API
* **Target Versions**: Windows XP through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterRawInputDevices`, `GetRawInputData`, `RAWINPUTDEVICE`, `RIDEV_INPUTSINK`).
* **Subsystem Architecture**: Standard mouse messages (`WM_MOUSEMOVE`) are subject to OS pointer ballistics and DPI clamping. Raw Input receives raw count deltas direct from the USB HID driver.
* **Open-Source Reference**: `libsdl-org/SDL` and `obsproject/obs-studio`.
* **How-To & Safe Implementation**:
  1. Setup device struct: `RAWINPUTDEVICE rid = { 0x01, 0x02, RIDEV_INPUTSINK, hwndReceiver };` (UsagePage 1, Usage 2 = Mouse).
  2. Register: `RegisterRawInputDevices(&rid, 1, sizeof(rid));`.
  3. In `WM_INPUT`, read `RAWINPUT.data.mouse.lLastX` and `lLastY` for ultra-precise drag calculations.

#### Tactic 177: Direct Alt+Click Window Drag & Resize (Linux-Style Window Mover)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WindowFromPoint`, `SetWindowPos`, `GetAsyncKeyState`, `GetWindowRect`).
* **Subsystem Architecture**: In Linux desktop environments (GNOME, KDE, i3), holding `Alt` (or `Super`) and left-clicking anywhere inside a window allows dragging it without grabbing the title bar. Holding `Alt` + right-clicking resizes it.
* **Open-Source Reference**: `valinet/ExplorerPatcher` (Alt+Drag implementation) and `RaimundSpecht/AltDrag`.
* **How-To & Safe Implementation**:
  1. In `WH_MOUSE_LL`, detect left mouse button down while `GetAsyncKeyState(VK_MENU) < 0`.
  2. Identify target window beneath cursor: `HWND hTarget = WindowFromPoint(pt); hTarget = GetAncestor(hTarget, GA_ROOT);`.
  3. Swallow the mouse click (`return 1`).
  4. Track mouse movement deltas: compute new `(x, y)` and move window continuously via `SetWindowPos(hTarget, NULL, new_x, new_y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);`.

#### Tactic 178: Intercepting Multimedia Hardware Keys via `WM_APPCOMMAND`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_APPCOMMAND`, `APPCOMMAND_VOLUME_MUTE`, `APPCOMMAND_VOLUME_DOWN`, `APPCOMMAND_VOLUME_UP`, `APPCOMMAND_MEDIA_PLAY_PAUSE`).
* **Subsystem Architecture**: Keyboard volume, mute, and media keys generate `WM_APPCOMMAND` messages routed through the active window hierarchy before triggering Windows default handlers.
* **Open-Source Reference**: `ModernFlyouts-Community/ModernFlyouts` and `AutoHotkey/AutoHotkey`.
* **How-To & Safe Implementation**:
  1. Intercept `WM_APPCOMMAND` in top-level message loop:
     ```cpp
     short cmd = GET_APPCOMMAND_LPARAM(lParam);
     if (cmd == APPCOMMAND_VOLUME_UP) {
         wl_volume_increment();
         return TRUE; // Swallows default OS volume OSD
     }
     ```
  2. WinLook renders its own custom minimalist on-screen display (OSD) instead of Windows' bulky default flyout.

#### Tactic 179: Non-Invasive Modal Keybind States (Vim-Style Window Navigation)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Internal state machine in Rust (`wl_animate` / `wl_wm`).
* **Subsystem Architecture**: Linux window managers frequently employ leader keys (e.g. `Super + M` enters "Move Mode", where `H/J/K/L` move windows).
* **Open-Source Reference**: `LGUG2Z/komorebi` and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. Define modal state: `enum Mode { Normal, Resize, Move, Workspace }`.
  2. When leader combination is pressed, change mode and display a tiny HUD indicator pill on the active status bar.
  3. While in `Move` mode, `H/J/K/L` keystrokes are intercepted by `WH_KEYBOARD_LL` and converted to tile swaps.
  4. Pressing `Escape` returns to `Normal` mode.

#### Tactic 180: Global Hotkey Registration via `RegisterHotKey` for Elevated Shortcuts
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterHotKey`, `UnregisterHotKey`, `WM_HOTKEY`, `MOD_NOREPEAT`).
* **Subsystem Architecture**: `RegisterHotKey` registers system-wide combinations handled directly by `win32k.sys`. Unlike hooks, hotkeys execute even if an application running as administrator has focus.
* **Open-Source Reference**: `microsoft/PowerToys` and `AutoHotkey/AutoHotkey`.
* **How-To & Safe Implementation**:
  1. Register with `MOD_NOREPEAT`: `RegisterHotKey(hwnd, ID_TERMINAL, MOD_WIN | MOD_NOREPEAT, VK_RETURN);`.
  2. In message loop, handle `WM_HOTKEY` when `wParam == ID_TERMINAL`.
  3. Guarantees 100% reliable invocation regardless of foreground application integrity level (UIPI).

### Category S: Custom Notification Daemons, OSDs & Heads-Up Displays (Tactics 181–190)

#### Tactic 181: System Audio Endpoint Monitoring via Core Audio APIs
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `MMDevApi.DLL` (`IMMDeviceEnumerator`, `IMMDevice`, `IAudioEndpointVolume`, `IAudioEndpointVolumeCallback`).
* **Subsystem Architecture**: Rather than guessing volume changes from keyboard events, Windows Core Audio provides an asynchronous callback interface invoked whenever volume changes from any source (hardware knob, game slider, or keyboard).
* **Open-Source Reference**: `ModernFlyouts-Community/ModernFlyouts` and `File-New-Project/EarTrumpet`.
* **How-To & Safe Implementation**:
  1. Instantiate device enumerator: `CoCreateInstance(__uuidof(MMDeviceEnumerator), ..., &pEnum)`.
  2. Retrieve default audio endpoint: `pEnum->GetDefaultAudioEndpoint(eRender, eMultimedia, &pDevice)`.
  3. Activate volume interface: `pDevice->Activate(__uuidof(IAudioEndpointVolume), ..., &pVolume)`.
  4. Register callback: `pVolume->RegisterControlChangeNotify(pMyCallback)`. In `OnNotify(AUDIO_VOLUME_NOTIFICATION_DATA *pData)`, trigger WinLook's animated volume HUD.

#### Tactic 182: Per-Application Audio Session Enumeration & Individual Volume Control
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `MMDevApi.DLL` (`IAudioSessionManager2`, `IAudioSessionEnumerator`, `IAudioSessionControl2`, `ISimpleAudioVolume`).
* **Subsystem Architecture**: The Windows Audio Session API (WASAPI) partitions audio streams per process ID. WinLook can display an in-bar mixer showing which app is playing audio with individual volume sliders.
* **Open-Source Reference**: `File-New-Project/EarTrumpet` (volume control replacement).
* **How-To & Safe Implementation**:
  1. Query session manager: `pDevice->Activate(__uuidof(IAudioSessionManager2), ..., &pManager)`.
  2. Get enumerator: `pManager->GetSessionEnumerator(&pSessionList)`.
  3. Iterate sessions: for each session, extract process ID via `pControl2->GetProcessId(&pid)` and current master volume via `pSimpleVol->GetMasterVolume(&level)`.
  4. Set individual app volume: `pSimpleVol->SetMasterVolume(new_level, NULL);`.

#### Tactic 183: Media Playback Metadata & Album Art Ingestion via WinRT SMTC
* **Target Versions**: Windows 10 (Build 19041+) through Windows 11.
* **APIs / Drivers**: `Windows.Media.Control` (`GlobalSystemMediaTransportControlsSessionManager`, `GlobalSystemMediaTransportControlsSessionMediaProperties`).
* **Subsystem Architecture**: Modern media players (Spotify, Chrome, Apple Music, VLC) register with the System Media Transport Controls (SMTC). WinLook can query the active session to display track title, artist, and album art thumbnail directly in the desktop status bar.
* **Open-Source Reference**: `ModernFlyouts-Community/ModernFlyouts` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Request session manager asynchronously: `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()`.
  2. Get current session: `auto session = manager.GetCurrentSession();`.
  3. Query properties: `auto props = session.TryGetMediaPropertiesAsync().get();`.
  4. Extract `props.Title()`, `props.Artist()`, and stream album art thumbnail `props.Thumbnail().OpenReadAsync()`.

#### Tactic 184: Hardware-Accelerated OSD Layer Rendering via Direct2D & DirectWrite
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `D2D1.DLL` (`ID2D1Factory`, `ID2D1HwndRenderTarget`, `ID2D1SolidColorBrush`), `DWRITE.DLL` (`IDWriteFactory`).
* **Subsystem Architecture**: Win32 GDI text rendering (`DrawTextW`) is CPU-bound and lacks anti-aliased font rendering over transparent backgrounds. Direct2D + DirectWrite composites typography with GPU acceleration and sub-pixel ClearType directly into layered surfaces.
* **Open-Source Reference**: `microsoft/WindowsCompositionSamples` and `ModernFlyouts-Community/ModernFlyouts`.
* **How-To & Safe Implementation**:
  1. Create Direct2D factory and Hwnd render target with alpha mode `D2D1_ALPHA_MODE_PREMULTIPLIED`.
  2. Create DirectWrite text format with modern font (e.g. Segoe UI Variable or Cascadia Code).
  3. In render loop: `pRenderTarget->BeginDraw(); pRenderTarget->Clear(D2D1::ColorF(0, 0, 0, 0)); pRenderTarget->DrawTextLayout(...); pRenderTarget->EndDraw();`.

#### Tactic 185: Per-Pixel Alpha Smooth Fade Animations via `UpdateLayeredWindowIndirect`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`UpdateLayeredWindowIndirect`, `UPDATELAYEREDWINDOWINFO`, `BLENDFUNCTION`, `AC_SRC_OVER`, `AC_SRC_ALPHA`).
* **Subsystem Architecture**: For custom OSDs, popups, and floating menus, smooth alpha cross-fades require updating the window's alpha channel frame-by-frame without triggering client paint passes.
* **Open-Source Reference**: `rainmeter/rainmeter` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Setup blend function: `BLENDFUNCTION bf = { AC_SRC_OVER, 0, (BYTE)alpha, AC_SRC_ALPHA };`.
  2. Populate `UPDATELAYEREDWINDOWINFO`: specify source memory DC, destination point, size, and `&bf`.
  3. Call `UpdateLayeredWindowIndirect(hwnd, &ulwi);` on each frame of `wl_animate` bezier curve.
  4. Smoothly modulates global opacity from 0 to 255 with zero window flicker.

#### Tactic 186: Non-Stealing Focus Window Flags for Ambient Notifications
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SetWindowPos`, `SWP_SHOWWINDOW`, `SWP_NOACTIVATE`, `SWP_NOSIZE`, `SWP_NOMOVE`).
* **Subsystem Architecture**: When an OSD or Dunst-style notification appears, calling `ShowWindow(SW_SHOW)` erroneously steals keyboard focus from fullscreen games or terminals.
* **Open-Source Reference**: `microsoft/PowerToys` (FancyZones OSD) and `ModernFlyouts-Community/ModernFlyouts`.
* **How-To & Safe Implementation**:
  1. To display: `SetWindowPos(hwndOsd, HWND_TOPMOST, 0, 0, 0, 0, SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);`.
  2. Window displays instantly on top of all applications, but the user's active caret and typing focus remain uninterrupted.

#### Tactic 187: Asynchronous Timed Dismissal via High-Resolution Threadpool Timers
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`CreateThreadpoolTimer`, `SetThreadpoolTimer`, `CloseThreadpoolTimer`, `FILETIME`).
* **Subsystem Architecture**: Traditional Win32 `SetTimer` generates `WM_TIMER` messages with low priority in the thread message queue, causing dismiss delays if the UI thread is busy. Threadpool timers fire on background worker threads with microsecond accuracy.
* **Open-Source Reference**: `microsoft/PowerToys` and `libuv/libuv`.
* **How-To & Safe Implementation**:
  1. Create timer: `PTP_TIMER timer = CreateThreadpoolTimer(TimerCallback, context, NULL);`.
  2. Schedule 3-second timeout: `ULARGE_INTEGER due; due.QuadPart = (ULONGLONG)-(30000000LL); SetThreadpoolTimer(timer, (PFILETIME)&due, 0, 0);`.
  3. In callback, post message to UI thread to trigger `wl_animate` fade-out exit transition.

#### Tactic 188: Click-Through Hit-Testing for Transparent Heads-Up Displays
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_NCHITTEST`, `HTTRANSPARENT`, `HTCLIENT`, `SetWindowLongPtrW`, `WS_EX_TRANSPARENT`).
* **Subsystem Architecture**: If an OSD or heads-up display is displaying passive telemetry in the corner of the screen, clicks passing through the empty space should interact with whatever window is underneath.
* **Open-Source Reference**: `rainmeter/rainmeter` and `fannheyward/hypr-border`.
* **How-To & Safe Implementation**:
  1. If entire window is passive: add `WS_EX_TRANSPARENT` to window ex-styles.
  2. If window has interactive buttons (e.g. "Dismiss" or "Play"): handle `WM_NCHITTEST`.
  3. Check if mouse cursor is within interactive button bounding boxes. If yes, return `HTCLIENT`; otherwise, return `HTTRANSPARENT`.

#### Tactic 189: Runtime Default Audio Output Device Switching via `IPolicyConfig`
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: Undocumented COM interface `IPolicyConfig` (`SetDefaultEndpoint`, `ERole::eMultimedia`, `ERole::eConsole`).
* **Subsystem Architecture**: Linux bars allow single-click switching between headphones and speakers. Windows lacks a public API for this, but the undocumented `IPolicyConfig` COM interface allows instant audio device redirection without launching the Windows Settings app.
* **Open-Source Reference**: `File-New-Project/EarTrumpet` (audio device switching) and `audiorouterdev/audio-router`.
* **How-To & Safe Implementation**:
  1. Query `IPolicyConfig` from system COM catalog (`CLSID_CPolicyConfigClient`).
  2. Retrieve target device endpoint ID (e.g. `{0.0.0.00000000}.{GUID}`).
  3. Call: `pPolicyConfig->SetDefaultEndpoint(wszDeviceId, eMultimedia); pPolicyConfig->SetDefaultEndpoint(wszDeviceId, eConsole);`.
  4. Audio output switches instantly across the entire system.

#### Tactic 190: Media Session Play/Pause/Skip Control via WinRT SMTC Interfaces
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `Windows.Media.Control` (`GlobalSystemMediaTransportControlsSession::TryPlayAsync`, `TryPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync`).
* **Subsystem Architecture**: Enables media control buttons directly on WinLook custom taskbars or floating islands to control Spotify, YouTube, or VLC without sending global keyboard shortcuts.
* **Open-Source Reference**: `ModernFlyouts-Community/ModernFlyouts` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. From active `GlobalSystemMediaTransportControlsSession`, call: `session.TryTogglePlayPauseAsync();` or `session.TrySkipNextAsync();`.
  2. Provides reliable media control even if the media application is running minimized or in the background.

### Category T: Dynamic Theming, Color Palettes & System-Wide Sync (Tactics 191–200)

#### Tactic 191: Wallpaper Dominant Color Extraction for Linux Pywal-Style Dynamic Palettes
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Image processing in Rust (`wl_wlde` / `image` crate), K-Means Clustering or Median Cut.
* **Subsystem Architecture**: The popular Linux utility `pywal` extracts a 16-color palette from the active desktop wallpaper and applies it across the terminal, window manager, and status bar. WinLook integrates this directly into `wl_wlde`.
* **Open-Source Reference**: `dylanaraps/pywal` (Linux baseline) and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Retrieve active wallpaper image buffer from `wl_wlde` provider hub.
  2. Downscale image to 128x128 for rapid analysis.
  3. Run K-Means clustering ($k=8$ or $k=16$) to extract dominant background, foreground, and vibrant accent colors.
  4. Publish palette to `windows.all.theme.palette.*` namespace so all WinLook components re-theme dynamically.

#### Tactic 192: Dynamic Window Title Bar & Border Accent Color Synchronization
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, `DWMWA_BORDER_COLOR`, `DWMWA_CAPTION_COLOR`, `DWMWA_TEXT_COLOR`).
* **Subsystem Architecture**: Dynamically updates top-level application window borders and caption colors to match the extracted wallpaper palette.
* **Open-Source Reference**: `MicaForEveryone/MicaForEveryone` and `valinet/ExplorerPatcher`.
* **How-To & Safe Implementation**:
  1. Convert extracted accent color to `COLORREF`: `COLORREF crBorder = RGB(r, g, b);`.
  2. Enumerate visible top-level windows (`EnumWindows`).
  3. Apply to each: `DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &crBorder, sizeof(crBorder));`.
  4. Apply contrasting caption color: `DwmSetWindowAttribute(hwnd, DWMWA_CAPTION_COLOR, &crCaption, sizeof(crCaption));`.

#### Tactic 193: System-Wide Accent Color Registry Synchronization
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Software\Microsoft\Windows\DWM\AccentColor`), `USER32.DLL` (`PostMessageW`, `HWND_BROADCAST`, `WM_DWMCOLORIZATIONCOLORCHANGED`).
* **Subsystem Architecture**: Synchronizes the Windows shell accent color (used in taskbar highlights, start menu borders, and system buttons) with the dynamic palette.
* **Open-Source Reference**: `TranslucentTB/TranslucentTB` and `valinet/ExplorerPatcher`.
* **How-To & Safe Implementation**:
  1. Update registry: `RegSetValueExW(hKeyDwm, L"AccentColor", 0, REG_DWORD, (BYTE*)&dwColor, sizeof(DWORD));`.
  2. Update `AccentColorMenu` under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent`.
  3. Notify DWM: `PostMessageW(HWND_BROADCAST, WM_DWMCOLORIZATIONCOLORCHANGED, (WPARAM)dwColor, (LPARAM)TRUE);`.

#### Tactic 194: Undocumented Taskbar Acrylic & BlurBehind via `SetWindowCompositionAttribute`
* **Target Versions**: Windows 10 (Build 1703+) through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (Undocumented `SetWindowCompositionAttribute`, `ACCENT_POLICY`, `ACCENT_STATE`).
* **Subsystem Architecture**: `SetWindowCompositionAttribute` (SWCA) is the internal API used by the Windows shell to apply Fluent Acrylic blur behind specific windows.
* **Open-Source Reference**: `TranslucentTB/TranslucentTB` (core implementation) and `ysc3839/win32-darkmode`.
* **How-To & Safe Implementation**:
  1. Resolve entrypoint: `pfnSetWindowCompositionAttribute pSetWindowCompositionAttribute = (pfnSWCA)GetProcAddress(hUser32, "SetWindowCompositionAttribute");`.
  2. Define `ACCENT_POLICY`: `policy.AccentState = ACCENT_ENABLE_ACRYLICBLURBEHIND; policy.GradientColor = (alpha << 24) | (b << 16) | (g << 8) | r;`.
  3. Wrap in `WINDOWCOMPOSITIONATTRIBDATA`: `wcad.Attrib = WCA_ACCENT_POLICY; wcad.pvData = &policy; wcad.cbData = sizeof(policy);`.
  4. Call on `FindWindowW(L"Shell_TrayWnd", NULL)`: taskbar instantly turns frosted acrylic or crystal clear.

#### Tactic 195: Per-Process System Backdrop Rule Overrides via DWM Interception
* **Target Versions**: Windows 11 (Build 22621+).
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmSetWindowAttribute`, `DWMWA_SYSTEMBACKDROP_TYPE`, `DWMSBT_MAINWINDOW`, `DWMSBT_TRANSIENTWINDOW`, `DWMSBT_TABBEDWINDOW`).
* **Subsystem Architecture**: Force-enables Mica (`DWMSBT_MAINWINDOW`), Acrylic (`DWMSBT_TRANSIENTWINDOW`), or Mica Alt (`DWMSBT_TABBEDWINDOW`) on classic applications that do not natively support it (Notepad, Command Prompt, custom editors).
* **Open-Source Reference**: `MicaForEveryone/MicaForEveryone` (core rule engine).
* **How-To & Safe Implementation**:
  1. Listen for window creation via `RegisterShellHookWindow` (`HSHELL_WINDOWCREATED`).
  2. Match window class/process against rules: `if (process_name == "notepad.exe") backdrop = DWMSBT_TABBEDWINDOW;`.
  3. Apply attribute: `DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &backdrop, sizeof(backdrop));`.
  4. Refresh frame: `SetWindowPos(hwnd, NULL, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);`.

#### Tactic 196: Global Theme Broadcast via `WM_SETTINGCHANGE` and `WM_THEMECHANGED`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageTimeoutW`, `HWND_BROADCAST`, `WM_SETTINGCHANGE`, `WM_THEMECHANGED`, `SMTO_ABORTIFHUNG`).
* **Subsystem Architecture**: When a customization utility modifies system fonts, colors, or metrics, running applications must be notified to re-query system metrics and repaint.
* **Open-Source Reference**: `valinet/ExplorerPatcher` and `dremin/RetroBar`.
* **How-To & Safe Implementation**:
  1. Broadcast with timeout to prevent hanging if an application is frozen:
     ```cpp
     DWORD_PTR dwResult;
     SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"ImmersiveColorSet", SMTO_ABORTIFHUNG, 1000, &dwResult);
     SendMessageTimeoutW(HWND_BROADCAST, WM_THEMECHANGED, 0, 0, SMTO_ABORTIFHUNG, 1000, &dwResult);
     ```
  2. All top-level windows refresh their theme brushes and visual style handles immediately.

#### Tactic 197: Dynamic Custom Cursor Scheme Loading via `SystemParametersInfoW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Control Panel\Cursors`), `USER32.DLL` (`SystemParametersInfoW`, `SPI_SETCURSORS`).
* **Subsystem Architecture**: Allows switching cursor themes (e.g. minimalist Nord or retro Win95 pointers) dynamically without restarting the system.
* **Open-Source Reference**: `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Update registry paths under `HKEY_CURRENT_USER\Control Panel\Cursors` (Arrow, Help, Wait, Crosshair, IBeam, Hand).
  2. Apply scheme globally: `SystemParametersInfoW(SPI_SETCURSORS, 0, NULL, SPIF_SENDCHANGE | SPIF_UPDATEINIFILE);`.
  3. `win32k.sys` re-reads cursor assets and replaces the active hardware mouse pointer.

#### Tactic 198: System Sound Event Scheme Customization via AppEvents Registry
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\AppEvents\Schemes\Apps\.Default`), `WINMM.DLL` (`PlaySoundW`).
* **Subsystem Architecture**: Configures classic or custom retro audio chimes (startup chime, window minimize sound, error sound) directly from `.wlt` template packages.
* **Open-Source Reference**: `dremin/RetroBar`.
* **How-To & Safe Implementation**:
  1. Map sound event keys (e.g. `SystemAsterisk`, `SystemExclamation`, `RestoreUp`, `Minimize`).
  2. Write path to custom `.wav` file into `HKEY_CURRENT_USER\AppEvents\Schemes\Apps\.Default\<Event>\.Current`.
  3. Play preview asynchronously: `PlaySoundW(wszWavPath, NULL, SND_ASYNC | SND_FILENAME);`.

#### Tactic 199: Accessibility High-Contrast Mode Ingestion & Clean Bypass
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SystemParametersInfoW`, `SPI_GETHIGHCONTRAST`, `HIGHCONTRASTW`, `HCF_HIGHCONTRASTON`).
* **Subsystem Architecture**: Respects users who require high-contrast accessibility themes. Custom styling must adapt gracefully rather than overriding high-contrast colors.
* **Open-Source Reference**: `microsoft/PowerToys` (accessibility compliance).
* **How-To & Safe Implementation**:
  1. Query: `HIGHCONTRASTW hc = { sizeof(hc) }; SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(hc), &hc, 0);`.
  2. If `hc.dwFlags & HCF_HIGHCONTRASTON`: disable dynamic transparency and acrylic backdrops, falling back to pure high-contrast system color definitions (`COLOR_WINDOW`, `COLOR_WINDOWTEXT`).

#### Tactic 200: Live In-Memory Terminal Color Palette Injection (Windows Terminal / Console)
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`GetConsoleScreenBufferInfoEx`, `SetConsoleScreenBufferInfoEx`, `CONSOLE_SCREEN_BUFFER_INFOEX`).
* **Subsystem Architecture**: Automatically applies the extracted wallpaper palette to active command prompt and PowerShell console windows.
* **Open-Source Reference**: `microsoft/terminal` and `dylanaraps/pywal`.
* **How-To & Safe Implementation**:
  1. Query console buffer: `CONSOLE_SCREEN_BUFFER_INFOEX csbi = { sizeof(csbi) }; GetConsoleScreenBufferInfoEx(hConsole, &csbi);`.
  2. Populate `csbi.ColorTable[0..15]` with the 16 extracted palette colors.
  3. Commit: `SetConsoleScreenBufferInfoEx(hConsole, &csbi);`. The active terminal repaints with matching theme colors immediately.

### Category U: Virtual Desktops, Pagers & Workspaces (Tactics 201–210)

#### Tactic 201: Programmatic Virtual Desktop Querying via Internal COM Interfaces
* **Target Versions**: Windows 10 (Build 10240+) through Windows 11.
* **APIs / Drivers**: Undocumented Shell COM Interfaces (`IVirtualDesktopManagerInternal`, `IVirtualDesktop`, `IServiceProvider`).
* **Subsystem Architecture**: The official `IVirtualDesktopManager` is limited to querying whether a window is on the current desktop. The internal `IVirtualDesktopManagerInternal` interface (obtained via `ImmersiveShellProvider`) provides full control: enumerating all desktops, creating new ones, switching, and moving windows.
* **Open-Source Reference**: `MScholtes/VirtualDesktop` (canonical C#/C++ implementation) and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. Instantiate shell service provider: `CoCreateInstance(CLSID_ImmersiveShell, NULL, CLSCTX_LOCAL_SERVER, __uuidof(IServiceProvider), (void**)&pServiceProvider)`.
  2. Query internal manager: `pServiceProvider->QueryService(__uuidof(IVirtualDesktopManagerInternal), &pDesktopManagerInternal)`.
  3. Query count: `IObjectArray* pDesktops; pDesktopManagerInternal->GetDesktops(hMonitor, &pDesktops);`.
  4. Build-specific adaptation: WinLook maintains UUID tables across Windows 10 (1507..22H2) and Windows 11 (21H2..24H2) to handle interface GUID shifts transparently.

#### Tactic 202: Instant Keyboard-Driven Workspace Switching
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `IVirtualDesktopManagerInternal::SwitchDesktop`.
* **Subsystem Architecture**: Enables instant Linux-style `Super + 1..9` switching between virtual workspaces without triggering the slow Windows Task View animation.
* **Open-Source Reference**: `MScholtes/VirtualDesktop` and `LGUG2Z/komorebi`.
* **How-To & Safe Implementation**:
  1. In `WH_KEYBOARD_LL`, detect `Win + <Number>`.
  2. Retrieve target virtual desktop object `pDesktop` at index `Number - 1`.
  3. Invoke: `pDesktopManagerInternal->SwitchDesktop(pDesktop);`.
  4. Desktop switches instantly; custom status bar updates its workspace indicator badge.

#### Tactic 203: Moving Windows Between Workspaces via `MoveViewToDesktop`
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `IVirtualDesktopManagerInternal::MoveViewToDesktop`, `IApplicationView`.
* **Subsystem Architecture**: Allows sending the active window to another workspace (e.g. `Win + Shift + 2` moves window to workspace 2).
* **Open-Source Reference**: `MScholtes/VirtualDesktop` and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. Query active window's `IApplicationView` via `IApplicationViewCollection::GetViewForHwnd(hwnd, &pAppView)`.
  2. Invoke: `pDesktopManagerInternal->MoveViewToDesktop(pAppView, targetDesktop);`.
  3. The window smoothly migrates to the target desktop without flickering or taskbar reordering.

#### Tactic 204: Application & Window Pinning Across All Workspaces
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `IVirtualDesktopPinnedApps` (`PinApp`, `UnpinApp`, `PinView`, `IsAppIdPinned`).
* **Subsystem Architecture**: In Linux, "sticky" windows remain visible on all workspaces (e.g. video player or reference document). Windows supports this natively via `IVirtualDesktopPinnedApps`.
* **Open-Source Reference**: `MScholtes/VirtualDesktop`.
* **How-To & Safe Implementation**:
  1. Query `IVirtualDesktopPinnedApps` from `IServiceProvider`.
  2. To pin single window: `pPinnedApps->PinView(pAppView);`.
  3. To pin entire application (all its windows): `pPinnedApps->PinApp(aumid);`.
  4. Window now persists across all workspace transitions.

#### Tactic 205: Custom Workspace Pager UI with Real-Time Window Mini-Previews
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`, `DWM_THUMBNAIL_PROPERTIES`).
* **Subsystem Architecture**: Linux desktop environments (XFCE, GNOME) feature pagers showing live mini-previews of all open windows on each workspace. On Windows, DWM hardware thumbnails allow zero-copy rendering of live window content inside a custom pager bar.
* **Open-Source Reference**: `LorenzCK/OnTopReplica` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. In the pager bar window, create a tile for each workspace.
  2. For windows on that workspace, register thumbnail: `DwmRegisterThumbnail(hwndPager, hwndTarget, &hThumb);`.
  3. Scale to thumbnail bounds: `DWM_THUMBNAIL_PROPERTIES props; props.rcDestination = miniRect; props.dwFlags = DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE; DwmUpdateThumbnailProperties(hThumb, &props);`.
  4. DWM hardware-composites a live, 60 FPS scaled preview of the application without polling pixels on the CPU.

#### Tactic 206: Subscribing to Workspace Changes via `IVirtualDesktopNotification`
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `IVirtualDesktopNotificationService`, `IVirtualDesktopNotification`.
* **Subsystem Architecture**: Asynchronous push notification system notifying WinLook whenever a virtual desktop is created, destroyed, or switched.
* **Open-Source Reference**: `MScholtes/VirtualDesktop` and `glzr-io/glazewm`.
* **How-To & Safe Implementation**:
  1. Implement `IVirtualDesktopNotification` interface in Rust/C++.
  2. Register via `IVirtualDesktopNotificationService::Register(pMyNotification, &cookie)`.
  3. On `CurrentVirtualDesktopChanged(oldDesktop, newDesktop)`: update workspace UI, trigger `wl_animate` slide transitions, and refresh active tile borders.

#### Tactic 207: Independent Multi-Monitor Workspaces (Per-Display Workspaces)
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: WinLook Subsystem Architecture (`wl_wlde` / `wl_wm`).
* **Subsystem Architecture**: By default, Windows switches virtual desktops across ALL monitors simultaneously. Linux tiling managers (XMonad, AwesomeWM, i3) decouple workspaces so Monitor 1 and Monitor 2 switch independently.
* **Open-Source Reference**: `LGUG2Z/komorebi` (independent monitor workspaces).
* **How-To & Safe Implementation**:
  1. Pin all managed windows to all virtual desktops via `IVirtualDesktopPinnedApps::PinView` so the OS never hides them.
  2. WinLook takes full ownership of workspace visibility: maintain internal workspace mappings per monitor (`Monitor1: [WS1, WS2]`, `Monitor2: [WS3, WS4]`).
  3. On workspace switch: hide/unhide windows on that specific monitor using `SetWindowPos(SWP_HIDEWINDOW / SWP_SHOWWINDOW)` or `DWMWA_CLOAK`.

#### Tactic 208: Persistent Window Layout Restoration Across Reboots
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry / SQLite (`wl_journal`), `USER32.DLL` (`GetWindowPlacement`, `SetWindowPlacement`).
* **Subsystem Architecture**: Automatically saves which windows belonged to which workspaces and screen coordinates, restoring them when applications restart.
* **Open-Source Reference**: `glzr-io/glazewm` and `microsoft/PowerToys` (FancyZones).
* **How-To & Safe Implementation**:
  1. On session logoff/shutdown (`WM_QUERYENDSESSION`), iterate managed windows.
  2. Record: ProcessPath, AUMID, WindowClass, WorkspaceIndex, and `WINDOWPLACEMENT`.
  3. On boot: listen for `HSHELL_WINDOWCREATED`, match launched processes against stored layout, and automatically assign them to their designated workspaces.

#### Tactic 209: Taskbar Icon Filtering per Active Workspace
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: Shell Hook Window (`RegisterShellHookWindow`), Custom Status Bar.
* **Subsystem Architecture**: In multi-workspace setups, status bars should only display running task buttons for windows residing on the currently focused workspace.
* **Open-Source Reference**: `dremin/RetroBar` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. When drawing running task buttons in the custom status bar, query: `IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop(hwnd, &isOnCurrent)`.
  2. If `!isOnCurrent`, suppress the task button from the visible bar.
  3. Users enjoy an uncluttered taskbar dedicated strictly to current workspace tasks.

#### Tactic 210: Direct Keyboard Navigation for "Send Window to Workspace N and Follow"
* **Target Versions**: Windows 10 through Windows 11.
* **APIs / Drivers**: `IVirtualDesktopManagerInternal`, `WH_KEYBOARD_LL`.
* **Subsystem Architecture**: Implements the common Linux WM binding where moving a window to another workspace automatically transitions focus to that workspace as well.
* **Open-Source Reference**: `glzr-io/glazewm` and `LGUG2Z/komorebi`.
* **How-To & Safe Implementation**:
  1. In `WH_KEYBOARD_LL`, detect `Win + Ctrl + <Number>`.
  2. Call `MoveViewToDesktop(pAppView, targetDesktop)`.
  3. Immediately follow by calling `SwitchDesktop(targetDesktop)`.
  4. Set foreground: `SetForegroundWindow(hwndTarget)`.

### Category V: Application Launchers, Runners & Command Palettes (Tactics 211–220)

#### Tactic 211: Instant-Launch Command Palette Window Setup
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`CreateWindowExW`, `WS_POPUP`, `WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`).
* **Subsystem Architecture**: Linux power users rely on `rofi`, `dmenu`, or `fzf` for launching applications. WinLook provides a sub-millisecond, native command palette without bundling Chromium.
* **Open-Source Reference**: `microsoft/PowerToys` (PowerToys Run) and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Pre-create the launcher window at startup and keep it hidden (`ShowWindow(SW_HIDE)`).
  2. On hotkey (`Alt + Space` or `Win + Space`), position at screen center: `SetWindowPos(hwnd, HWND_TOPMOST, center_x, center_y, width, height, SWP_SHOWWINDOW);`.
  3. Focus edit control: `SetFocus(hwndEdit)`. Instantaneous appearance with zero process startup latency.

#### Tactic 212: Start Menu Shortcut (`.lnk`) Parsing via `IShellLinkW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IShellLinkW`, `IPersistFile`).
* **Subsystem Architecture**: Indexes installed desktop applications from `CSIDL_COMMON_PROGRAMS` and `CSIDL_PROGRAMS`.
* **Open-Source Reference**: `Flow-Launcher/Flow.Launcher` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Instantiate: `CoCreateInstance(CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, IID_IShellLinkW, (void**)&pShellLink)`.
  2. Load file: `pShellLink->QueryInterface(IID_IPersistFile, &pPersistFile); pPersistFile->Load(wszLnkPath, STGM_READ);`.
  3. Extract target executable: `pShellLink->GetPath(wszTarget, MAX_PATH, &wfd, SLGP_UNCPRIORITY);`.
  4. Cache title and target path in in-memory fuzzy search index.

#### Tactic 213: Direct UWP App Execution via `IApplicationActivationManager`
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IApplicationActivationManager::ActivateApplication`).
* **Subsystem Architecture**: Modern Windows Store apps (Settings, Calculator, Terminal) cannot be launched by calling `CreateProcess` on their executable due to AppContainer sandboxing. They must be activated via AUMID.
* **Open-Source Reference**: `microsoft/PowerToys` (PowerToys Run) and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Create activation manager: `CoCreateInstance(CLSID_ApplicationActivationManager, NULL, CLSCTX_LOCAL_SERVER, IID_IApplicationActivationManager, (void**)&pActManager)`.
  2. Launch via AUMID: `DWORD processId = 0; pActManager->ActivateApplication(wszAumid, NULL, AO_NONE, &processId);`.
  3. Launches target UWP app cleanly without shell dependency.

#### Tactic 214: High-Performance Fuzzy Search Matching (Modified Smith-Waterman)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Pure Rust fuzzy matching algorithm (`wl_translation` / `fuzzy_matcher`).
* **Subsystem Architecture**: Allows typing acronyms (e.g. "gc" matches "Google Chrome"; "term" matches "Windows Terminal") with scoring bonuses for word boundaries and prefix matches.
* **Open-Source Reference**: `junegunn/fzf` (algorithm baseline) and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Tokenize application names.
  2. Score input query against tokens with prefix weighting ($+10$), camelCase transition weighting ($+8$), and penalty for unmatched characters.
  3. Sort results in descending score order and render top 6 suggestions in under 1ms.

#### Tactic 215: Elevated Command Execution Fallback via `ShellExecuteExW`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`ShellExecuteExW`, `SHELLEXECUTEINFOW`, `lpVerb = L"runas"`).
* **Subsystem Architecture**: When user types a command with `Ctrl + Shift + Enter`, execute with administrator privileges triggering standard UAC elevation prompt.
* **Open-Source Reference**: `microsoft/PowerToys` and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Populate: `SHELLEXECUTEINFOW sei = { sizeof(sei) }; sei.lpVerb = L"runas"; sei.lpFile = wszExePath; sei.nShow = SW_SHOWNORMAL;`.
  2. Execute: `ShellExecuteExW(&sei);`.

#### Tactic 216: In-Memory Thumbnail & Icon Cache Generation
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`SHCreateItemFromParsingName`, `IShellItemImageFactory::GetImage`, `SIIGBF_RESIZETOFIT`).
* **Subsystem Architecture**: Extracting icons on every keystroke causes stutter. `IShellItemImageFactory` retrieves pre-scaled, high-DPI bitmaps directly from Windows thumbnail cache databases.
* **Open-Source Reference**: `microsoft/PowerToys` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Create item: `SHCreateItemFromParsingName(wszPath, NULL, IID_IShellItemImageFactory, (void**)&pFactory)`.
  2. Request 48x48 icon: `HBITMAP hBmp = NULL; pFactory->GetImage(SIZE{48, 48}, SIIGBF_RESIZETOFIT, &hBmp);`.
  3. Store in hash map keyed by file path.

#### Tactic 217: Inline Mathematical Calculation & Unit Conversion Engine
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Built-in AST Math Evaluator in Rust (`wl_translation`).
* **Subsystem Architecture**: Typing `= 50 * 1024` or `150 USD to EUR` immediately renders the calculated result in the launcher without opening a calculator.
* **Open-Source Reference**: `microsoft/PowerToys` (Calculator plugin) and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Detect expression starting with numbers or math operators (`+`, `-`, `*`, `/`, `sqrt`, `sin`).
  2. Evaluate AST safely with overflow guards.
  3. Display result in top launcher row with "Press Enter to copy to clipboard" action.

#### Tactic 218: Named Pipe Plugin Architecture for External Script Extensibility
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`CreateNamedPipeW`, `ReadFile`, `WriteFile`, `wl_ipc`).
* **Subsystem Architecture**: Allows developers to write launcher extensions in Python, Rust, or PowerShell communicating over local pipes.
* **Open-Source Reference**: `Flow-Launcher/Flow.Launcher` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. WinLook opens server pipe `\\.\pipe\WinLook_LauncherPlugin`.
  2. Plugin connects and sends JSON registration: `{ "prefix": "gh", "name": "GitHub Search" }`.
  3. When user types `gh winlook`, query is dispatched to plugin; results stream back into launcher UI.

#### Tactic 219: Auto-Dismissal on Window Blur (`WM_KILLFOCUS` / `WM_ACTIVATE`)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`WM_KILLFOCUS`, `WM_ACTIVATE`, `WA_INACTIVE`).
* **Subsystem Architecture**: Launchers must dismiss immediately when the user clicks elsewhere on the desktop or switches applications.
* **Open-Source Reference**: `microsoft/PowerToys` and `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. In launcher window procedure:
     ```cpp
     if (msg == WM_ACTIVATE && LOWORD(wParam) == WA_INACTIVE) {
         ShowWindow(hwnd, SW_HIDE);
     }
     ```
  2. Ensures the desktop returns to pristine state without lingering popup windows.

#### Tactic 220: Sub-Pixel Typography Rendering via DirectWrite Text Layouts
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `DWRITE.DLL` (`IDWriteTextLayout`, `IDWriteTypography`, `DWRITE_MEASURING_MODE_NATURAL`).
* **Subsystem Architecture**: Delivers razor-sharp search text rendering with high-DPI scaling across 4K and 8K displays.
* **Open-Source Reference**: `microsoft/terminal` and `alacritty/alacritty`.
* **How-To & Safe Implementation**:
  1. Create text layout: `pDWriteFactory->CreateTextLayout(wszQuery, length, pTextFormat, maxWidth, maxHeight, &pLayout);`.
  2. Highlight matching substring: apply bold weight and accent color to range using `pLayout->SetFontWeight(DWRITE_FONT_WEIGHT_BOLD, range);`.
  3. Draw directly via `pRenderTarget->DrawTextLayout(...)`.

### Category W: File Explorer, Context Menus & Folder Views (Tactics 221–230)

#### Tactic 221: macOS / Linux-Style Spacebar Quick File Preview via `IPreviewHandler`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IPreviewHandler`, `IInitializeWithFile`, `IInitializeWithItem`).
* **Subsystem Architecture**: In macOS (Quick Look) and Linux (Sushi), selecting a file and tapping `Space` opens an instant floating preview (PDF, code, image, audio) without launching full applications. Windows has built-in Preview Handlers for File Explorer that can be hosted in standalone windows.
* **Open-Source Reference**: `QL-Win/QuickLook` and `ccseer/Seer`.
* **How-To & Safe Implementation**:
  1. In `WH_KEYBOARD_LL`, detect `Space` pressed when focused window class is `CabinetWClass` or `Progman` (Desktop).
  2. Query selected file path from active Explorer window (Tactic 222).
  3. Look up preview handler CLSID from registry: `HKEY_CLASSES_ROOT\\<ext>\\shellex\\{8895b1c6-b41f-4c1c-a562-0d564250836f}`.
  4. Instantiate handler: `CoCreateInstance(clsidHandler, NULL, CLSCTX_LOCAL_SERVER, IID_IPreviewHandler, (void**)&pHandler)`.
  5. Initialize with file: `pInit->Initialize(wszFilePath, STGM_READ)`.
  6. Dock handler to custom popup window: `pHandler->SetWindow(hwndPreview, &rcClient); pHandler->DoPreview();`. Pressing `Space` or `Escape` again dismisses preview.

#### Tactic 222: Querying Selected Items from Active Explorer Windows via `IShellWindows`
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IShellWindows`, `IServiceProvider`, `IShellBrowser`, `IShellView`, `IFolderView`).
* **Subsystem Architecture**: Directly queries which files or folders the user has highlighted in the currently active File Explorer window or on the desktop listview.
* **Open-Source Reference**: `QL-Win/QuickLook` and `ccseer/Seer`.
* **How-To & Safe Implementation**:
  1. Instantiate shell windows: `CoCreateInstance(CLSID_ShellWindows, NULL, CLSCTX_ALL, IID_IShellWindows, (void**)&pShellWindows)`.
  2. Find window matching foreground `HWND`: iterate `pShellWindows->Item(v, &pDisp)` and compare `GetWindow(pDisp) == hForeground`.
  3. Query `IFolderView`: `pBrowser->QueryActiveShellView(&pView); pView->QueryInterface(IID_IFolderView, (void**)&pFolderView);`.
  4. Retrieve selected item: `pFolderView->GetSelectionMarked(&index); pFolderView->Item(index, &pidl); SHGetPathFromIDListW(pidl, wszPath);`.

#### Tactic 223: Modern Windows 11 Context Menu Simplification via User Class Registry
* **Target Versions**: Windows 11 (Build 22000+).
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`).
* **Subsystem Architecture**: Windows 11 replaces the full right-click context menu with a compact menu requiring "Show more options" (`Shift + F10`) to see extensions. A per-user registry key restores the classic instant menu without modifying system files.
* **Open-Source Reference**: `valinet/ExplorerPatcher` and `Open-Shell/Open-Shell-Menu`.
* **How-To & Safe Implementation**:
  1. Create key: `HKEY_CURRENT_USER\Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32`.
  2. Set `(Default)` value to empty string `""`.
  3. Restart Explorer or broadcast `WM_SETTINGCHANGE`: context menus instantly open directly with all options without an extra click.
  4. Reversibility: deleting the subkey restores Windows 11 default compact menus.

#### Tactic 224: Custom Shell Folder Thumbnail Providers (`IThumbnailProvider`)
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IThumbnailProvider`, `GetThumbnail`).
* **Subsystem Architecture**: Generates custom file thumbnails for modern formats (e.g. `.webp`, `.svg`, `.blend`, `.rs`) directly inside File Explorer.
* **Open-Source Reference**: `microsoft/PowerToys` (SVG thumbnail provider).
* **How-To & Safe Implementation**:
  1. Implement `IThumbnailProvider` interface in Rust/C++.
  2. In `GetThumbnail(UINT cx, HBITMAP *phbmp, WTS_ALPHATYPE *pdwAlpha)`: render format into memory bitmap using Direct2D.
  3. Return `phbmp` with `WTSAT_ARGB`. Explorer displays rich file thumbnails directly in folder views.

#### Tactic 225: Custom File Icon Overlay Handlers (`IShellIconOverlayIdentifier`)
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `SHELL32.DLL` (`IShellIconOverlayIdentifier`, `GetOverlayInfo`, `GetPriority`).
* **Subsystem Architecture**: Adds small badges (e.g. git sync status, encrypted status) to file icons in File Explorer.
* **Open-Source Reference**: `tortoisesvn/tortoisesvn` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Implement `IShellIconOverlayIdentifier`.
  2. In `IsMemberOf(LPCWSTR pwszPath, DWORD dwAttrib)`: check file metadata. If matching, return `S_OK`.
  3. Return icon resource path in `GetOverlayInfo`: Explorer blends badge onto bottom-left corner of the file icon.

#### Tactic 226: Custom Shell Namespace Junction Points for Quick Access
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\MyComputer\NameSpace`).
* **Subsystem Architecture**: Injects custom virtual locations (e.g. "WinLook Themes", "Cloud Storage") directly into Explorer's "This PC" / Navigation Pane.
* **Open-Source Reference**: `cairoshell/cairoshell` and `valinet/ExplorerPatcher`.
* **How-To & Safe Implementation**:
  1. Register CLSID with `ShellFolder` attributes under `HKCU\Software\Classes\CLSID\{GUID}`.
  2. Add GUID key under `Explorer\MyComputer\NameSpace\{GUID}`.
  3. Appears alongside local drives and folders in File Explorer.

#### Tactic 227: Tabbed Explorer Window Detection & Hook Coordination (`ShellTabWindowClass`)
* **Target Versions**: Windows 11 (Build 22621+).
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `RealGetWindowClassW`).
* **Subsystem Architecture**: Windows 11 22H2 introduced tabs into File Explorer. Window class `CabinetWClass` now hosts multiple `ShellTabWindowClass` child windows.
* **Open-Source Reference**: `valinet/ExplorerPatcher`.
* **How-To & Safe Implementation**:
  1. Query active tab: `FindWindowExW(hCabinet, NULL, L"ShellTabWindowClass", NULL)`.
  2. Extract active tab folder view independently of parent window handle.
  3. Allows tiling and workspace management to track individual tabs accurately.

#### Tactic 228: Explorer Address Bar Path Interception & Quick Jump
* **Target Versions**: Windows 98 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`SendMessageW`, `WM_SETTEXT`, `FindWindowExW`).
* **Subsystem Architecture**: Programmatically navigates File Explorer to a new folder from command palette or terminal without mouse clicks.
* **Open-Source Reference**: `Flow-Launcher/Flow.Launcher`.
* **How-To & Safe Implementation**:
  1. Locate address bar edit box: `FindWindowExW(hCabinet, NULL, L"Address Band Root", NULL)` -> `FindWindowExW(..., L"Edit", NULL)`.
  2. Set directory path: `SendMessageW(hEdit, WM_SETTEXT, 0, (LPARAM)wszTargetPath);`.
  3. Emulate Enter key: `SendMessageW(hEdit, WM_KEYDOWN, VK_RETURN, 0); SendMessageW(hEdit, WM_KEYUP, VK_RETURN, 0);`.

#### Tactic 229: Explorer Status Bar Customization via `SB_SETTEXT`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`FindWindowExW`, `SendMessageW`, `SB_SETTEXTW`, `SB_GETPARTS`).
* **Subsystem Architecture**: Directly updates the status bar at the bottom of File Explorer windows to show git branch, file hash, or disk usage.
* **Open-Source Reference**: `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Find status bar: `HWND hStatus = FindWindowExW(hCabinet, NULL, L"msctls_statusbar32", NULL);`.
  2. Set part text: `SendMessageW(hStatus, SB_SETTEXTW, 1 | 0, (LPARAM)L"git: main [clean]");`.

#### Tactic 230: Explorer Context Menu Clean Extension Ingestion (Registry Verbs)
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: Registry (`HKEY_CURRENT_USER\Software\Classes\*\shell\<Verb>`).
* **Subsystem Architecture**: Injects actions (e.g. "Edit with WinLook", "Set as Live Wallpaper") without writing C++ COM DLLs.
* **Open-Source Reference**: `valinet/ExplorerPatcher` and `microsoft/PowerToys`.
* **How-To & Safe Implementation**:
  1. Create key: `HKEY_CURRENT_USER\Software\Classes\Directory\Background\shell\WinLookConfig`.
  2. Set value: `(Default) = "Open WinLook Manager"`.
  3. Create `command` subkey: `(Default) = "\"C:\\Program Files\\WinLook\\winlook.exe\" --settings"`.
  4. Appears cleanly on desktop right-click menu with zero DLL injection.

### Category X: System Performance, Hardware Monitors & System Information (Tactics 231–240)

#### Tactic 231: Microsecond CPU Utilization Sampling via `NtQuerySystemInformation`
* **Target Versions**: Windows NT 4.0 through Windows 11.
* **APIs / Drivers**: `NTDLL.DLL` (`NtQuerySystemInformation`, `SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION`).
* **Subsystem Architecture**: Querying WMI or Performance Counters has high overhead (5-10% CPU just to monitor). Native system call `NtQuerySystemInformation` directly reads kernel CPU idle/kernel/user ticks in under 2 microseconds.
* **Open-Source Reference**: `processhacker/processhacker` (System Informer) and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Call: `NtQuerySystemInformation(SystemProcessorPerformanceInformation, &sppi, sizeof(sppi) * cpu_count, &retLen)`.
  2. Compute delta: $Usage = 1.0 - (\Delta IdleTime / (\Delta KernelTime + \Delta UserTime))$.
  3. Feed metrics directly to status bar widget with 0.01% CPU footprint.

#### Tactic 232: GPU Engine Utilization Tracking via DirectX Performance Interface (`D3DKMTQueryStatistics`)
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `GDI32.DLL` (`D3DKMTQueryStatistics`, `D3DKMT_QUERYSTATISTICS_PROCESS_ADAPTER_INFO`).
* **Subsystem Architecture**: Reads dedicated 3D engine, video decode, and VRAM utilization direct from the WDDM graphics kernel driver.
* **Open-Source Reference**: `processhacker/processhacker` and `obsproject/obs-studio`.
* **How-To & Safe Implementation**:
  1. Initialize statistics query for local process adapter.
  2. Read `RunningTime` and `TotalExecutionTime` across 3D, Video, and Compute engines.
  3. Render live GPU graph in status bar.

#### Tactic 233: Network Interface Throughput Monitoring via `GetIfTable2`
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `IPHLPAPI.DLL` (`GetIfTable2`, `FreeMibTable`, `MIB_IF_TABLE2`, `MIB_IF_ROW2`).
* **Subsystem Architecture**: Retrieves instantaneous bytes sent/received and link speeds across Ethernet and Wi-Fi adapters.
* **Open-Source Reference**: `rainmeter/rainmeter` and `AmN/yasb`.
* **How-To & Safe Implementation**:
  1. Query table: `GetIfTable2(&pTable);`.
  2. Find active adapter (`InterfaceAndOperStatusFlags.HardwareInterface && OperStatus == IfOperStatusUp`).
  3. Calculate $\Delta InOctets / \Delta t$ (Download speed) and $\Delta OutOctets / \Delta t$ (Upload speed).
  4. Free memory: `FreeMibTable(pTable);`.

#### Tactic 234: Hardware Battery & Power Status Ingestion via `GetSystemPowerStatus`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`GetSystemPowerStatus`, `SYSTEM_POWER_STATUS`).
* **Subsystem Architecture**: Instantaneous query of laptop battery percentage, AC line status, and estimated runtime.
* **Open-Source Reference**: `AmN/yasb` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Query: `SYSTEM_POWER_STATUS sps; GetSystemPowerStatus(&sps);`.
  2. Read `sps.BatteryLifePercent` (0..100) and `sps.ACLineStatus` (0 = Battery, 1 = Plugged).
  3. Update battery icon widget in custom panel.

#### Tactic 235: Real-Time Process Working Set Memory Tracking (`K32GetProcessMemoryInfo`)
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`K32GetProcessMemoryInfo`, `PROCESS_MEMORY_COUNTERS_EX`).
* **Subsystem Architecture**: Queries exact physical RAM (Working Set) and private commit bytes consumed by running processes.
* **Open-Source Reference**: `processhacker/processhacker`.
* **How-To & Safe Implementation**:
  1. Open process: `HANDLE hProc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, FALSE, pid);`.
  2. Query: `PROCESS_MEMORY_COUNTERS_EX pmc; K32GetProcessMemoryInfo(hProc, (PROCESS_MEMORY_COUNTERS*)&pmc, sizeof(pmc));`.
  3. Read `pmc.WorkingSetSize` and `pmc.PrivateUsage`.

#### Tactic 236: Live Direct2D Telemetry Sparkline & Graph Rendering
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `D2D1.DLL` (`ID2D1PathGeometry`, `ID2D1GeometrySink`).
* **Subsystem Architecture**: Renders smooth 60 FPS CPU/RAM history graphs into status bars with GPU hardware acceleration.
* **Open-Source Reference**: `processhacker/processhacker` and `rainmeter/rainmeter`.
* **How-To & Safe Implementation**:
  1. Maintain ring buffer of last 60 CPU samples.
  2. Build path geometry: map samples to `(x, y)` coordinates.
  3. Fill path with gradient brush: `pRenderTarget->FillGeometry(pPath, pGradientBrush);`.
  4. Stroke top edge: `pRenderTarget->DrawGeometry(pPath, pLineBrush, 1.5f);`.

#### Tactic 237: Real-Time Disk I/O Activity via Performance Data Helper (PDH)
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: `PDH.DLL` (`PdhOpenQueryW`, `PdhAddCounterW`, `PdhCollectQueryData`, `PdhGetFormattedCounterValue`).
* **Subsystem Architecture**: Monitors read/write transfer rates across physical drives without kernel drivers.
* **Open-Source Reference**: `rainmeter/rainmeter`.
* **How-To & Safe Implementation**:
  1. Open query: `PdhOpenQueryW(NULL, 0, &hQuery);`.
  2. Add counter: `PdhAddCounterW(hQuery, L"\\PhysicalDisk(_Total)\\Disk Bytes/sec", 0, &hCounter);`.
  3. Sample periodically: `PdhCollectQueryData(hQuery); PdhGetFormattedCounterValue(hCounter, PDH_FMT_DOUBLE, NULL, &val);`.

#### Tactic 238: System Thermal Zone Temperature Querying via WMI
* **Target Versions**: Windows 2000 through Windows 11.
* **APIs / Drivers**: WMI / COM (`IWbemServices::ExecQuery`, `MSAcpi_ThermalZoneTemperature`).
* **Subsystem Architecture**: Queries onboard ACPI thermal sensor readings (CPU temperature).
* **Open-Source Reference**: `LibreHardwareMonitor/LibreHardwareMonitor` and `rainmeter/rainmeter`.
* **How-To & Safe Implementation**:
  1. Connect to WMI root: `pLocator->ConnectServer(BSTR(L"ROOT\\WMI"), ..., &pServices)`.
  2. Query: `SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature`.
  3. Read Kelvin value in tenths of degrees: $Celsius = (CurrentTemperature - 2732) / 10.0$.

#### Tactic 239: Open Hardware Monitor Shared Memory Bridge Ingestion
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`OpenFileMappingW`, `MapViewOfFile`).
* **Subsystem Architecture**: Bridges to LibreHardwareMonitor or OpenHardwareMonitor shared memory regions to read exact per-core temperatures, fan RPMs, and wattage without deploying kernel drivers.
* **Open-Source Reference**: `LibreHardwareMonitor/LibreHardwareMonitor`.
* **How-To & Safe Implementation**:
  1. Open mapping: `HANDLE hMap = OpenFileMappingW(FILE_MAP_READ, FALSE, L"Global\\LibreHardwareMonitorSharedMemory");`.
  2. Cast memory pointer to JSON/struct format.
  3. Extract core temperatures and fan RPMs directly for widget rendering.

#### Tactic 240: Microsecond Performance Counter Clocks via `QueryPerformanceCounter`
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `KERNEL32.DLL` (`QueryPerformanceCounter`, `QueryPerformanceFrequency`).
* **Subsystem Architecture**: Hardware-backed high-resolution counter for animation delta time $\Delta t$ measurement with nanosecond resolution.
* **Open-Source Reference**: `libsdl-org/SDL` and `obsproject/obs-studio`.
* **How-To & Safe Implementation**:
  1. Query frequency: `QueryPerformanceFrequency(&freq);`.
  2. On each frame: `QueryPerformanceCounter(&now); double dt = (double)(now.QuadPart - last.QuadPart) / (double)freq.QuadPart;`.
  3. Passes exact $\Delta t$ into `wl_animate` bezier and spring physics solvers.

### Category Y: Linux-Style Compositor Bridges, Shaders & Post-Processing (Tactics 241–250)

#### Tactic 241: DirectComposition Custom Shader Pipeline for Ambient Window Glows
* **Target Versions**: Windows 8.1 through Windows 11.
* **APIs / Drivers**: `DCOMP.DLL`, Direct2D Effects (`CLSID_D2D1GaussianBlur`, `CLSID_D2D1Shadow`).
* **Subsystem Architecture**: Linux Wayland compositors (Hyprland, Wayfire) render soft drop shadows and colored glowing borders behind active windows. DirectComposition applies native GPU shader effects behind window redirection surfaces.
* **Open-Source Reference**: `microsoft/WindowsCompositionSamples` and `Blinue/Magpie`.
* **How-To & Safe Implementation**:
  1. Create DirectComposition visual for target window.
  2. Create Direct2D Gaussian Blur effect: `pD2DDeviceContext->CreateEffect(CLSID_D2D1GaussianBlur, &pBlurEffect);`.
  3. Set blur standard deviation: `pBlurEffect->SetValue(D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION, 8.0f);`.
  4. Composite behind window surface: active window gets modern neon glow or drop shadow.

#### Tactic 242: Windows Magnification API Desktop Zoom & Pixel Post-Processing
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `MAGNIFICATION.DLL` (`MagInitialize`, `MagSetWindowTransform`, `MagSetWindowFilterList`, `MagSetColorEffect`).
* **Subsystem Architecture**: The Windows Magnification API provides hardware-accelerated full-desktop or windowed scaling and color transformation filters without requiring screen capture hooks.
* **Open-Source Reference**: `Blinue/Magpie` (window upscaler) and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Initialize: `MagInitialize();`.
  2. Create magnifier host control: `CreateWindowW(L"Magnifier", ..., hwndHost)`.
  3. Set color matrix (e.g. grayscale, invert, or color LUT): `MAGCOLOREFFECT colEffect = { ... }; MagSetColorEffect(hwndMag, &colEffect);`.
  4. Set transform: `MAGTRANSFORM matrix; MagSetWindowTransform(hwndMag, &matrix);`.

#### Tactic 243: Zero-Copy Desktop Duplication API Streaming via DXGI
* **Target Versions**: Windows 8 through Windows 11.
* **APIs / Drivers**: `DXGI.DLL` (`IDXGIOutput1::DuplicateOutput`, `IDXGIOutputDuplication::AcquireNextFrame`).
* **Subsystem Architecture**: The fastest method to capture the screen on Windows. Directly reads the desktop composition surface from VRAM with zero CPU memory copies.
* **Open-Source Reference**: `obsproject/obs-studio` and `Blinue/Magpie`.
* **How-To & Safe Implementation**:
  1. Query `IDXGIOutput1` from display adapter.
  2. Instantiate: `pOutput1->DuplicateOutput(pD3DDevice, &pDuplication);`.
  3. In render loop: `pDuplication->AcquireNextFrame(16, &frameInfo, &pResource);`.
  4. Access raw D3D11 desktop texture for ambient background sampling or live wallpaper reactions.

#### Tactic 244: Window Anti-Aliasing Supersampling Masks in Direct2D
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: `D2D1.DLL` (`ID2D1GeometrySink`, `D2D1_ANTIALIAS_MODE_PER_PRIMITIVE`).
* **Subsystem Architecture**: Eliminates the jagged pixel edges found in legacy GDI clipping regions (`SetWindowRgn`).
* **Open-Source Reference**: `microsoft/WindowsCompositionSamples` and `fannheyward/hypr-border`.
* **How-To & Safe Implementation**:
  1. Set antialias mode: `pRenderTarget->SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);`.
  2. Create rounded rectangle with sub-pixel float coordinates.
  3. Render mask into alpha channel: edges blend with anti-aliasing directly against DWM redirection surface.

#### Tactic 245: DWM Composition VSync & Frame Rate Unlocking Awareness
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `DWMAPI.DLL` (`DwmGetCompositionTimingInfo`, `DWM_TIMING_INFO`).
* **Subsystem Architecture**: Synchronizes animation frame submissions with high-refresh monitors (144Hz, 240Hz) and variable refresh rates (G-Sync / FreeSync).
* **Open-Source Reference**: `Blinue/Magpie` and `obsproject/obs-studio`.
* **How-To & Safe Implementation**:
  1. Query: `DWM_TIMING_INFO dti = { sizeof(dti) }; DwmGetCompositionTimingInfo(NULL, &dti);`.
  2. Compute exact monitor refresh duration: $FrameDuration = dti.qpcRefreshPeriod / Frequency$.
  3. Align `wl_animate` timer intervals to match display VSync ticks.

#### Tactic 246: Desktop Snapping Assist Grid Visualizer Overlay
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`CreateWindowExW`, `WS_EX_LAYERED`, `WS_EX_TRANSPARENT`), Direct2D.
* **Subsystem Architecture**: When dragging a window, renders a subtle dashed layout grid indicating where the window will snap if dropped.
* **Open-Source Reference**: `microsoft/PowerToys` (FancyZones visual grid).
* **How-To & Safe Implementation**:
  1. Detect window drag via `EVENT_SYSTEM_MOVESIZESTART`.
  2. Display semi-transparent overlay showing grid zones calculated from current layout engine.
  3. Highlight active zone when mouse hovers within threshold.
  4. On `EVENT_SYSTEM_MOVESIZEEND`: animate window smoothly into selected grid zone and hide overlay.

#### Tactic 247: Inverting Window Colors for Native Night-Mode Reading
* **Target Versions**: Windows Vista through Windows 11.
* **APIs / Drivers**: `MAGNIFICATION.DLL` (`MagSetColorEffect`, Inversion Matrix).
* **Subsystem Architecture**: Inverts the client area colors of legacy bright white Win32 applications (e.g. older accounting software or editors) to dark mode without modifying the application code.
* **Open-Source Reference**: `Blinue/Magpie` and `cairoshell/cairoshell`.
* **How-To & Safe Implementation**:
  1. Define 5x5 negative color matrix:
     ```cpp
     MAGCOLOREFFECT magMat = {
         -1.0f,  0.0f,  0.0f,  0.0f,  0.0f,
          0.0f, -1.0f,  0.0f,  0.0f,  0.0f,
          0.0f,  0.0f, -1.0f,  0.0f,  0.0f,
          0.0f,  0.0f,  0.0f,  1.0f,  0.0f,
          1.0f,  1.0f,  1.0f,  0.0f,  1.0f
     };
     ```
  2. Apply to target window magnifier viewport: window client area renders inverted in hardware.

#### Tactic 248: Non-Invasive Wallpaper Engine Shader Compilation & Canvas Ingestion
* **Target Versions**: Windows 7 through Windows 11.
* **APIs / Drivers**: Direct3D 11 (`D3D11CreateDeviceAndSwapChain`), HLSL Pixel Shaders.
* **Subsystem Architecture**: Compiles live interactive HLSL shaders (rain, clock, audio visualizer) and renders directly into the `WorkerW` live desktop surface.
* **Open-Source Reference**: `rocksdanister/lively` (open-source wallpaper engine) and `Kate-alt-69/WinLook`.
* **How-To & Safe Implementation**:
  1. Compile shader with `D3DCompile(szShaderSource, ..., "ps_4_0", ...)`.
  2. Create D3D11 swap chain bound to the `WorkerW` `HWND`.
  3. Render animated scene at 60 FPS, passing mouse coordinate uniforms and audio FFT spectrums into constant buffers.

#### Tactic 249: Complete Linux-Like Desktop Rice Orchestration Blueprint
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: WinLook Subsystem Core (`wl_translation`, `wl_wlde`, `wl_animate`, `wl_wm`).
* **Subsystem Architecture**: Unifies Tiling Window Manager (Tactic 151-160), Custom Polybar Panel (Tactic 161-170), Global Hotkey Daemon (Tactic 171-180), Pywal Theming (Tactic 191-200), and Virtual Desktops (Tactic 201-210) into a single, cohesive, sub-5MB executable.
* **Open-Source Reference**: `Kate-alt-69/WinLook` and the Unix Philosophy.
* **How-To & Safe Implementation**:
  1. The user launches `winlook.bg.exe`.
  2. Single lightweight daemon coordinates all subsystems via lock-free event buses.
  3. Replaces 5 bulky Electron/C# tools (totaling 1.5GB RAM) with one native Rust runtime consuming under 15MB RAM.

#### Tactic 250: Absolute Emergency Failsafe Hotkey & Total Desktop Reclamation
* **Target Versions**: Windows 95 through Windows 11.
* **APIs / Drivers**: `USER32.DLL` (`RegisterHotKey`, `MOD_CONTROL | MOD_ALT | MOD_SHIFT`, `UnregisterHotKey`).
* **Subsystem Architecture**: The ultimate safety guarantee for customization software. If any window becomes unreachable, misplaced, or frozen, an emergency shortcut immediately restores all native Windows settings, unhides standard taskbars, detaches hooks, and exits cleanly.
* **Open-Source Reference**: `AutoHotkey/AutoHotkey` and `Kate-alt-69/WinLook`.
* **How-To & Safe Implementation**:
  1. At startup, register: `RegisterHotKey(hwndWatchdog, ID_PANIC, MOD_CONTROL | MOD_ALT | MOD_SHIFT, 'Q');`.
  2. In panic handler:
     - Unhide `Shell_TrayWnd` and all `Shell_SecondaryTrayWnd`.
     - Remove all custom AppBars via `SHAppBarMessage(ABM_REMOVE)`.
     - Reset monitor work area via `SPI_SETWORKAREA`.
     - Detach all low-level hooks (`UnhookWindowsHookEx`, `UnhookWinEvent`).
     - Invalidate DWM custom attributes and flush `wl_journal`.
  3. Desktop instantly returns to 100% factory native state in under 10 milliseconds.

---

## 8. Rust Implementation Blueprint: `wl_translation` Core

The following Rust implementation provides the typed translation engine connecting `windows.<version>.<subsystem>.<item>` paths to native system calls:

```rust
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HWND, HRESULT, S_OK, BOOL, COLORREF};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, ShowWindow, SetWindowPos, SW_HIDE, SW_SHOW, 
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_FRAMECHANGED
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerPreference {
    Default = 0,
    DoNotRound = 1,
    Round = 2,
    RoundSmall = 3,
}

#[derive(Debug, Clone)]
pub struct TranslationQuery {
    pub raw_path: String,
    pub version_scope: String,
    pub subsystem: String,
    pub item: String,
    pub property: String,
}

impl TranslationQuery {
    pub fn parse(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 5 || parts[0] != "windows" {
            return None;
        }
        Some(Self {
            raw_path: path.to_string(),
            version_scope: parts[1].to_string(),
            subsystem: parts[2].to_string(),
            item: parts[3].to_string(),
            property: parts[4].to_string(),
        })
    }
}

pub struct WinTranslationRouter {
    os_build: u32,
}

impl WinTranslationRouter {
    pub fn new() -> Self {
        let os_build = Self::query_nt_build();
        Self { os_build }
    }

    fn query_nt_build() -> u32 {
        // Query build number dynamically via RtlGetVersion
        unsafe {
            let ntdll = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
                utf16_lit::utf16_null!("ntdll.dll").as_ptr(),
            );
            if ntdll != 0 {
                type FnRtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
                let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                    ntdll, b"RtlGetVersion\0".as_ptr(),
                );
                if let Some(f) = proc {
                    let rtl_get_version: FnRtlGetVersion = std::mem::transmute(f);
                    let mut osvi: OSVERSIONINFOW = std::mem::zeroed();
                    osvi.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
                    if rtl_get_version(&mut osvi) == 0 {
                        return osvi.dwBuildNumber;
                    }
                }
            }
            22000 // Fallback
        }
    }

    /// Routes parsed hierarchical translation queries to specific subsystem dispatchers
    pub unsafe fn dispatch(&self, query_str: &str, value: &str) -> bool {
        let query = match TranslationQuery::parse(query_str) {
            Some(q) => q,
            None => return false,
        };

        match query.subsystem.as_str() {
            "dwm" => self.dispatch_dwm(&query, value),
            "shell" => self.dispatch_shell(&query, value),
            "uxtheme" => self.dispatch_uxtheme(&query, value),
            "gdi" => self.dispatch_gdi(&query, value),
            _ => false,
        }
    }

    unsafe fn dispatch_dwm(&self, query: &TranslationQuery, value: &str) -> bool {
        // Only execute DWM attributes if host OS supports DWM (Build >= 6000 / Vista+)
        if self.os_build < 6000 {
            return false;
        }

        let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
        if hwnd == 0 { return false; }

        match (query.item.as_str(), query.property.as_str()) {
            ("window", "corner_preference") => {
                if self.os_build < 22000 { return false; } // Win11 only
                let pref = match value {
                    "donotround" => CornerPreference::DoNotRound,
                    "round" => CornerPreference::Round,
                    "roundsmall" => CornerPreference::RoundSmall,
                    _ => CornerPreference::Default,
                };
                self.apply_corner_preference(hwnd, pref)
            }
            ("window", "dark_mode") => {
                let dark_flag: BOOL = if value == "true" { 1 } else { 0 };
                // Use Attr 20 on Win11 (22000+) & Win10 (19041+), fallback to Attr 19 on 1809
                let attr_id = if self.os_build >= 19041 { 20u32 } else { 19u32 };
                self.apply_dwm_attribute(hwnd, attr_id, &dark_flag as *const _ as *const c_void, 4)
            }
            _ => false,
        }
    }

    unsafe fn dispatch_shell(&self, query: &TranslationQuery, value: &str) -> bool {
        match (query.item.as_str(), query.property.as_str()) {
            ("taskbar", "visible") => {
                let show_cmd = if value == "false" { SW_HIDE } else { SW_SHOW };
                let hwnd = FindWindowW(utf16_lit::utf16_null!("Shell_TrayWnd").as_ptr(), std::ptr::null());
                if hwnd != 0 {
                    ShowWindow(hwnd, show_cmd);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    unsafe fn dispatch_uxtheme(&self, _query: &TranslationQuery, _value: &str) -> bool {
        true
    }

    unsafe fn dispatch_gdi(&self, _query: &TranslationQuery, _value: &str) -> bool {
        true
    }

    /// Dynamic invocation of DwmSetWindowAttribute for corner preference
    pub unsafe fn apply_corner_preference(&self, hwnd: HWND, pref: CornerPreference) -> bool {
        let dwmapi = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(
            utf16_lit::utf16_null!("dwmapi.dll").as_ptr(),
        );
        if dwmapi == 0 {
            return false;
        }

        let symbol = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            dwmapi,
            b"DwmSetWindowAttribute\0".as_ptr(),
        );

        if let Some(func) = symbol {
            type FnDwmSetWindowAttribute = unsafe extern "system" fn(
                HWND, u32, *const c_void, u32
            ) -> HRESULT;

            let dwm_set_attr: FnDwmSetWindowAttribute = std::mem::transmute(func);
            let attr_id = 33u32; // DWMWA_WINDOW_CORNER_PREFERENCE
            let attr_val = pref as u32;

            let hr = dwm_set_attr(
                hwnd,
                attr_id,
                &attr_val as *const _ as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            if hr == S_OK {
                // Synchronize frame metrics immediately
                SetWindowPos(hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                return true;
            }
        }
        false
    }
}
```

---


## 9. Compatibility Catalog & Reversibility Strategy

| OS Generation | Target Subsystems | Native Capabilities | WinLook Strategy | Recovery / Rollback |
| :--- | :--- | :--- | :--- | :--- |
| **Windows 95/98/ME** | `USER.EXE`, `GDI.EXE` | `SetSysColors`, `SPI_SETNONCLIENTMETRICS`, `AnimateWindow` | Legacy C89 adapter (`wl_Lui`), GDI bitmap rasterization | Cached `NONCLIENTMETRICS` restore via `SPI_SETNONCLIENTMETRICS` |
| **Windows 2000/XP** | `win32k.sys`, `uxtheme.dll` | `.msstyles`, `IActiveDesktop`, `SetWindowTheme` | Theme class redirection, active desktop canvas | Re-apply default Luna/Classic theme via `SetWindowTheme` |
| **Windows Vista/7** | DWM (Direct3D 9Ex) | Aero Glass (`DwmEnableBlurBehindWindow`), `Progman` 0x052C split | Hardware glass frames, WorkerW wallpaper docking | Restore `DwmEnableBlurBehindWindow(FALSE)` |
| **Windows 10 (19045)** | DWM, UxTheme Ordinals | Ordinal 135 Dark Mode, Virtual Desktops, XAML Start | Shell hook monitoring, non-client dark mode injection | Invalidate window with standard light mode flags |
| **Windows 11 (22621+)** | DWM, WinUI 3, Mica | Corner preferences (Attr 33), Mica/Acrylic (Attr 38), Border color | Direct DWM attribute discovery, WinUI 3 Manager UI | Reset attributes to `DWMWCP_DEFAULT` and `DWMSBT_AUTO` |

### The Reversibility Contract
Every change applied by `wl_translation` must record a compensating transaction in `wl_journal` [cite: 19]. If the user toggles a theme off or the WinLook process exits, all modified properties are cleanly returned to original system metrics without leaving orphaned hooks or corrupted registry entries [cite: 23, 27].


---

## 10. In-Depth Subsystem Engineering & Rust Module Implementations

To realize the WinLook architectural vision of a lightweight (<10MB RAM) all-in-one personalization engine, the background daemon (`winlook.bg.exe`) implements five dedicated modules [cite: 7]:

### 10.1 Module A: `wl_translation` Core Dispatcher
The translation engine parses dot-notated queries and routes them to native system calls dynamically:

```rust
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HWND, HRESULT, S_OK, BOOL, COLORREF, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, FindWindowExW, ShowWindow, SetWindowPos, GetWindowRect,
    SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_FRAMECHANGED
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerPreference {
    Default = 0,
    DoNotRound = 1,
    Round = 2,
    RoundSmall = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemBackdropType {
    Auto = 0,
    None = 1,
    MainWindow = 2,  // Mica
    Transient = 3,   // Acrylic
    Tabbed = 4,      // Mica Alt
}

#[derive(Debug, Clone)]
pub struct TranslationQuery {
    pub os_target: String,
    pub subsystem: String,
    pub item: String,
    pub property: String,
}

impl TranslationQuery {
    pub fn parse(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() >= 5 && parts[0] == "windows" {
            Some(Self {
                os_target: parts[1].to_string(),
                subsystem: parts[2].to_string(),
                item: parts[3].to_string(),
                property: parts[4].to_string(),
            })
        } else {
            None
        }
    }
}

pub struct WinTranslationProvider {
    pub is_win11: bool,
    pub build_number: u32,
    pub dwmapi_handle: isize,
    pub uxtheme_handle: isize,
}

impl WinTranslationProvider {
    pub unsafe fn new() -> Self {
        let dwmapi = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(
            utf16_lit::utf16_null!("dwmapi.dll").as_ptr(),
        );
        let uxtheme = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(
            utf16_lit::utf16_null!("uxtheme.dll").as_ptr(),
        );
        let (is_win11, build_number) = Self::query_windows_build();
        Self {
            is_win11,
            build_number,
            dwmapi_handle: dwmapi,
            uxtheme_handle: uxtheme,
        }
    }

    unsafe fn query_windows_build() -> (bool, u32) {
        type FnRtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOEXW) -> u32;
        let ntdll = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
            utf16_lit::utf16_null!("ntdll.dll").as_ptr(),
        );
        if ntdll == 0 { return (false, 0); }
        let sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            ntdll, b"RtlGetVersion\0".as_ptr()
        );
        if let Some(func) = sym {
            let rtl_get_ver: FnRtlGetVersion = std::mem::transmute(func);
            let mut os_info: OSVERSIONINFOEXW = std::mem::zeroed();
            os_info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;
            if rtl_get_ver(&mut os_info) == 0 {
                return (os_info.dwBuildNumber >= 22000, os_info.dwBuildNumber);
            }
        }
        (false, 0)
    }

    pub unsafe fn set_property(&self, query: &TranslationQuery, value: &str) -> bool {
        match (query.subsystem.as_str(), query.item.as_str(), query.property.as_str()) {
            ("shell", "taskbar", "visibility") => {
                let show_cmd = if value == "false" { SW_HIDE } else { SW_SHOW };
                let h_tray = FindWindowW(utf16_lit::utf16_null!("Shell_TrayWnd").as_ptr(), std::ptr::null());
                if h_tray != 0 {
                    ShowWindow(h_tray, show_cmd);
                    return true;
                }
            }
            ("dwm", "window", "corner_preference") => {
                let pref = match value {
                    "donotround" => CornerPreference::DoNotRound,
                    "round" => CornerPreference::Round,
                    "roundsmall" => CornerPreference::RoundSmall,
                    _ => CornerPreference::Default,
                };
                let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
                if hwnd != 0 {
                    return self.apply_corner_preference(hwnd, pref);
                }
            }
            ("dwm", "window", "backdrop_type") => {
                let backdrop = match value {
                    "none" => SystemBackdropType::None,
                    "mica" => SystemBackdropType::MainWindow,
                    "acrylic" => SystemBackdropType::Transient,
                    "tabbed" => SystemBackdropType::Tabbed,
                    _ => SystemBackdropType::Auto,
                };
                let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
                if hwnd != 0 {
                    return self.apply_backdrop_type(hwnd, backdrop);
                }
            }
            ("uxtheme", "system", "dark_mode") => {
                let enable = value == "true";
                return self.apply_system_dark_mode(enable);
            }
            _ => {}
        }
        false
    }

    pub unsafe fn apply_corner_preference(&self, hwnd: HWND, pref: CornerPreference) -> bool {
        if self.dwmapi_handle == 0 || !self.is_win11 { return false; }
        let sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            self.dwmapi_handle, b"DwmSetWindowAttribute\0".as_ptr()
        );
        if let Some(func) = sym {
            type FnDwmSetWindowAttribute = unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> HRESULT;
            let dwm_set_attr: FnDwmSetWindowAttribute = std::mem::transmute(func);
            let attr = 33u32; // DWMWA_WINDOW_CORNER_PREFERENCE
            let val = pref as u32;
            let hr = dwm_set_attr(hwnd, attr, &val as *const _ as *const c_void, 4);
            if hr == S_OK {
                SetWindowPos(hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                return true;
            }
        }
        false
    }

    pub unsafe fn apply_backdrop_type(&self, hwnd: HWND, backdrop: SystemBackdropType) -> bool {
        if self.dwmapi_handle == 0 || !self.is_win11 || self.build_number < 22621 { return false; }
        let sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            self.dwmapi_handle, b"DwmSetWindowAttribute\0".as_ptr()
        );
        if let Some(func) = sym {
            type FnDwmSetWindowAttribute = unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> HRESULT;
            let dwm_set_attr: FnDwmSetWindowAttribute = std::mem::transmute(func);
            let attr = 38u32; // DWMWA_SYSTEMBACKDROP_TYPE
            let val = backdrop as u32;
            let hr = dwm_set_attr(hwnd, attr, &val as *const _ as *const c_void, 4);
            if hr == S_OK {
                SetWindowPos(hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                return true;
            }
        }
        false
    }

    pub unsafe fn apply_system_dark_mode(&self, enable: bool) -> bool {
        if self.uxtheme_handle == 0 { return false; }
        // UxTheme Ordinal 135: SetPreferredAppMode
        let sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
            self.uxtheme_handle, 135 as *const u8
        );
        if let Some(func) = sym {
            type FnSetPreferredAppMode = unsafe extern "system" fn(i32) -> i32;
            let set_mode: FnSetPreferredAppMode = std::mem::transmute(func);
            let mode = if enable { 2 } else { 0 };
            set_mode(mode);

            // UxTheme Ordinal 136: FlushMenuThemes
            let flush_sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                self.uxtheme_handle, 136 as *const u8
            );
            if let Some(flush_func) = flush_sym {
                type FnFlushMenuThemes = unsafe extern "system" fn();
                let flush: FnFlushMenuThemes = std::mem::transmute(flush_func);
                flush();
            }
            return true;
        }
        false
    }
}

#[repr(C)]
struct OSVERSIONINFOEXW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
    wServicePackMajor: u16,
    wServicePackMinor: u16,
    wSuiteMask: u16,
    wProductType: u8,
    wReserved: u8,
}
```

### 10.2 Module B: `wl_animate` Mathematical Formulation & Physics Solver
The animation solver computes continuous position interpolation using cubic bezier easing curves and damped spring equations [cite: 7, 20]:

```rust
pub struct CubicBezier {
    pub p1x: f32,
    pub p1y: f32,
    pub p2x: f32,
    pub p2y: f32,
}

impl CubicBezier {
    pub fn ease_out_cubic() -> Self {
        Self { p1x: 0.215, p1y: 0.610, p2x: 0.355, p2y: 1.000 }
    }

    pub fn ease_in_out_quad() -> Self {
        Self { p1x: 0.455, p1y: 0.030, p2x: 0.515, p2y: 0.955 }
    }

    pub fn solve(&self, t: f32) -> f32 {
        let clamped_t = t.clamp(0.0, 1.0);
        let t2 = clamped_t * clamped_t;
        let t3 = t2 * clamped_t;
        let mt = 1.0 - clamped_t;
        let mt2 = mt * mt;
        // B(t) = 3*(1-t)^2*t*P1y + 3*(1-t)*t^2*P2y + t^3
        3.0 * mt2 * clamped_t * self.p1y + 3.0 * mt * t2 * self.p2y + t3
    }
}

pub struct SpringSimulator {
    pub tension: f32,
    pub friction: f32,
    pub mass: f32,
    pub current_pos: f32,
    pub target_pos: f32,
    pub velocity: f32,
}

impl SpringSimulator {
    pub fn new(tension: f32, friction: f32) -> Self {
        Self {
            tension,
            friction,
            mass: 1.0,
            current_pos: 0.0,
            target_pos: 0.0,
            velocity: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) -> f32 {
        let spring_force = -self.tension * (self.current_pos - self.target_pos);
        let damping_force = -self.friction * self.velocity;
        let acceleration = (spring_force + damping_force) / self.mass;
        self.velocity += acceleration * dt;
        self.current_pos += self.velocity * dt;
        self.current_pos
    }

    pub fn is_settled(&self, epsilon: f32) -> bool {
        (self.current_pos - self.target_pos).abs() < epsilon && self.velocity.abs() < epsilon
    }
}
```

### 10.3 Module C: `wl_wlde` Shell Canvas Host & WorkerW Discovery
The live desktop module locates the lower `WorkerW` surface in the Z-order hierarchy to host graphics behind desktop icons [cite: 7]:

```rust
pub unsafe fn discover_wallpaper_workerw() -> HWND {
    let progman = FindWindowW(utf16_lit::utf16_null!("Progman").as_ptr(), std::ptr::null());
    if progman == 0 { return 0; }

    let mut result: usize = 0;
    // Dispatch undocumented 0x052C message to trigger WorkerW spawn
    windows_sys::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW(
        progman, 0x052C, 0, 0, 0x0000, 1000, &mut result
    );

    let mut workerw_hwnd: HWND = 0;
    unsafe extern "system" fn enum_windows_proc(top_hwnd: HWND, lparam: isize) -> BOOL {
        let p_workerw = lparam as *mut HWND;
        let defview = FindWindowExW(
            top_hwnd, 0, utf16_lit::utf16_null!("SHELLDLL_DefView").as_ptr(), std::ptr::null()
        );
        if defview != 0 {
            // Find next sibling WorkerW window
            let next_workerw = FindWindowExW(
                0, top_hwnd, utf16_lit::utf16_null!("WorkerW").as_ptr(), std::ptr::null()
            );
            if next_workerw != 0 {
                *p_workerw = next_workerw;
                return 0; // Stop enumeration
            }
        }
        1 // Continue
    }

    windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
        Some(enum_windows_proc),
        &mut workerw_hwnd as *mut _ as isize,
    );

    workerw_hwnd
}
```

### 10.4 Module D: `wl_cpuv` & `wl_gpuv` Dynamic Resource Governance
Monitors system resources and fullscreen applications to drop rendering activity during gaming [cite: 7]:

```rust
pub enum ThrottleState {
    Active60Fps,
    LowPower15Fps,
    Paused0Fps,
}

pub struct ResourceGovernor {
    pub state: ThrottleState,
}

impl ResourceGovernor {
    pub unsafe fn evaluate_environment(&mut self) -> ThrottleState {
        // Query exclusive fullscreen state via SHQueryUserNotificationState
        type FnSHQueryUserNotificationState = unsafe extern "system" fn(*mut u32) -> HRESULT;
        let shell32 = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
            utf16_lit::utf16_null!("shell32.dll").as_ptr()
        );
        if shell32 != 0 {
            let sym = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                shell32, b"SHQueryUserNotificationState\0".as_ptr()
            );
            if let Some(func) = sym {
                let query_state: FnSHQueryUserNotificationState = std::mem::transmute(func);
                let mut state: u32 = 0;
                if query_state(&mut state) == S_OK {
                    // QUERY_USER_NOTIFICATION_STATE constants:
                    // QUNS_BUSY = 2, QUNS_RUNNING_D3D_FULL_SCREEN = 3, QUNS_PRESENTATION_MODE = 4
                    const QUNS_BUSY: u32 = 2;
                    const QUNS_RUNNING_D3D_FULL_SCREEN: u32 = 3;
                    const QUNS_PRESENTATION_MODE: u32 = 4;

                    if state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_BUSY || state == QUNS_PRESENTATION_MODE {
                        self.state = ThrottleState::Paused0Fps;
                        return ThrottleState::Paused0Fps;
                    }
                }
            }
        }

        self.state = ThrottleState::Active60Fps;
        ThrottleState::Active60Fps
    }
}
```

### 10.5 Module E: `wl_journal` Lock-Free Rollback Engine
Guarantees 100% clean recovery of modified window metrics and styles [cite: 7, 19]:

```rust
pub const JOURNAL_MAX_ENTRIES: usize = 512;
pub const PROPERTY_DWM_CORNER: u32 = 1;
pub const PROPERTY_DWM_BACKDROP: u32 = 2;
pub const PROPERTY_DWM_DARK_MODE: u32 = 3;
pub const PROPERTY_DWM_BORDER_COLOR: u32 = 4;
pub const PROPERTY_TASKBAR_VISIBILITY: u32 = 5;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalEntry {
    pub timestamp_ms: u64,
    pub hwnd: HWND,
    pub property_id: u32,
    pub original_value: u64,
    pub applied_value: u64,
    pub is_reverted: u32, // 1 = reverted, 0 = active
}

#[repr(C)]
pub struct SharedJournalHeader {
    pub magic: u32, // 0x574C4A4E "WLJN"
    pub entry_count: u32,
    pub clean_shutdown: u32, // Set to 1 upon graceful exit
    pub reserved: u32,
    pub entries: [JournalEntry; JOURNAL_MAX_ENTRIES],
}

pub struct CrashRecoveryJournal {
    mapping_handle: HANDLE,
    view_ptr: *mut SharedJournalHeader,
}

impl CrashRecoveryJournal {
    pub unsafe fn new_or_open() -> Option<Self> {
        use windows_sys::Win32::System::Memory::{
            CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE
        };
        let map_name = utf16_lit::utf16_null!("Local\\WinLook_StateJournal");
        let size = std::mem::size_of::<SharedJournalHeader>() as u32;

        let handle = CreateFileMappingW(
            0xFFFFFFFFFFFFFFFF as HANDLE, // Paging file backed
            std::ptr::null(),
            PAGE_READWRITE,
            0,
            size,
            map_name.as_ptr(),
        );

        if handle == 0 { return None; }

        let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, size as usize);
        if view == std::ptr::null_mut() {
            windows_sys::Win32::Foundation::CloseHandle(handle);
            return None;
        }

        let header = view as *mut SharedJournalHeader;
        if (*header).magic != 0x574C4A4E {
            (*header).magic = 0x574C4A4E;
            (*header).entry_count = 0;
            (*header).clean_shutdown = 0;
        }

        Some(Self {
            mapping_handle: handle,
            view_ptr: header,
        })
    }

    pub unsafe fn record_transaction(&mut self, hwnd: HWND, property_id: u32, orig: u64, applied: u64) {
        if self.view_ptr.is_null() { return; }
        let header = &mut *self.view_ptr;
        let count = header.entry_count as usize;
        if count < JOURNAL_MAX_ENTRIES {
            header.entries[count] = JournalEntry {
                timestamp_ms: windows_sys::Win32::System::SystemInformation::GetTickCount64(),
                hwnd,
                property_id,
                original_value: orig,
                applied_value: applied,
                is_reverted: 0,
            };
            header.entry_count += 1;
        }
    }

    pub unsafe fn mark_clean_shutdown(&mut self) {
        if !self.view_ptr.is_null() {
            (*self.view_ptr).clean_shutdown = 1;
        }
    }

    /// Executes live Win32 / DWM rollback calls for every active journal entry
    pub unsafe fn rollback_all(&mut self) {
        if self.view_ptr.is_null() { return; }
        let header = &mut *self.view_ptr;
        for i in (0..header.entry_count as usize).rev() {
            let entry = &mut header.entries[i];
            if entry.is_reverted == 0 {
                match entry.property_id {
                    PROPERTY_DWM_CORNER => {
                        let pref = entry.original_value as u32;
                        Self::raw_set_dwm_attr(entry.hwnd, 33, &pref as *const _ as *const c_void, 4);
                    }
                    PROPERTY_DWM_BACKDROP => {
                        let backdrop = entry.original_value as u32;
                        Self::raw_set_dwm_attr(entry.hwnd, 38, &backdrop as *const _ as *const c_void, 4);
                    }
                    PROPERTY_DWM_DARK_MODE => {
                        let dark = entry.original_value as u32;
                        Self::raw_set_dwm_attr(entry.hwnd, 20, &dark as *const _ as *const c_void, 4);
                    }
                    PROPERTY_TASKBAR_VISIBILITY => {
                        let cmd = if entry.original_value == 1 { SW_SHOW } else { SW_HIDE };
                        ShowWindow(entry.hwnd, cmd);
                    }
                    _ => {}
                }
                // Synchronize frame metrics
                SetWindowPos(entry.hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                entry.is_reverted = 1;
            }
        }
    }

    unsafe fn raw_set_dwm_attr(hwnd: HWND, attr: u32, val: *const c_void, sz: u32) {
        let dwmapi = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
            utf16_lit::utf16_null!("dwmapi.dll").as_ptr(),
        );
        if dwmapi != 0 {
            type FnDwmSetWindowAttribute = unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> HRESULT;
            let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                dwmapi, b"DwmSetWindowAttribute\0".as_ptr(),
            );
            if let Some(f) = proc {
                let func: FnDwmSetWindowAttribute = std::mem::transmute(f);
                func(hwnd, attr, val, sz);
            }
        }
    }
}
```


---

## 11. Declarative XML Schema & Modular Preset Catalog

WinLook provides a declarative XML schema for packaging visual themes without compiled binaries [cite: 7].

### 11.1 XML Configuration Presets

#### Preset: `Windows 95 Retro Aesthetic (for Modern Windows 10/11)`
```xml
<WinLookTheme target_os="win11">
  <Metadata name="Windows 95 Classic Aesthetic" author="WinLook Project" description="Replicates classic 95 styling on Windows 10/11 using DWM and GDI properties" />
  <Translations>
    <!-- Modern DWM Attributes: Strip rounding and backdrops -->
    <Rule path="windows.11.dwm.window.corner_preference" value="donotround" />
    <Rule path="windows.11.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.11.dwm.window.dark_mode" value="false" />
    <Rule path="windows.11.dwm.window.border_color" value="#000000" />
    <Rule path="windows.11.dwm.window.caption_color" value="#000080" />
    <Rule path="windows.11.dwm.window.text_color" value="#FFFFFF" />
    <!-- Classic Win32 System Colors -->
    <Rule path="windows.all.system.colors.active_caption" value="#000080" />
    <Rule path="windows.all.system.colors.background" value="#008080" />
    <Rule path="windows.all.system.colors.btn_face" value="#C0C0C0" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 95 Native Legacy (for Windows 95/98/ME/2000)`
```xml
<WinLookTheme target_os="win95">
  <Metadata name="Windows 95 Native" author="WinLook Project" description="Applies native classic metrics and colors without invalid DWM rules" />
  <Translations>
    <!-- Native GDI & USER metrics only - no DWM dependencies -->
    <Rule path="windows.95.system.colors.active_caption" value="#000080" />
    <Rule path="windows.95.system.colors.background" value="#008080" />
    <Rule path="windows.95.system.colors.btn_face" value="#C0C0C0" />
    <Rule path="windows.95.system.metrics.caption_height" value="18" />
    <Rule path="windows.95.system.metrics.border_width" value="1" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows XP Luna Blue`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows XP Luna Blue" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="roundsmall" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x00D88000" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.all.dwm.window.border_color" value="0x00A04000" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows XP Metallic Silver`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows XP Metallic Silver" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="roundsmall" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x00B0B0B0" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00000000" />
    <Rule path="windows.all.dwm.window.border_color" value="0x00909090" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows XP Homestead Green`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows XP Homestead Green" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="roundsmall" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x00558855" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.all.dwm.window.border_color" value="0x00447744" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows Vista Aero Glass`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows Vista Aero Glass" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="round" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="acrylic" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x00000000" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.all.dwm.window.border_color" value="0x00202020" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 7 Aero Clear`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows 7 Aero Clear" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="round" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="acrylic" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x00101010" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.all.dwm.window.border_color" value="0x00303030" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 8 Metro Flat`
```xml
<WinLookTheme target="all">
  <Metadata name="Windows 8 Metro Flat" author="WinLook Project" />
  <Translations>
    <Rule path="windows.all.dwm.window.corner_preference" value="donotround" />
    <Rule path="windows.all.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.all.dwm.window.dark_mode" value="false" />
    <Rule path="windows.all.dwm.window.caption_color" value="0x000078D7" />
    <Rule path="windows.all.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.all.dwm.window.border_color" value="0x000078D7" />
    <Rule path="windows.all.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.all.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 10 Fluent Dark`
```xml
<WinLookTheme target="win10">
  <Metadata name="Windows 10 Fluent Dark" author="WinLook Project" />
  <Translations>
    <Rule path="windows.win10.dwm.window.corner_preference" value="donotround" />
    <Rule path="windows.win10.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.win10.dwm.window.dark_mode" value="true" />
    <Rule path="windows.win10.dwm.window.caption_color" value="0x00202020" />
    <Rule path="windows.win10.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.win10.dwm.window.border_color" value="0x00303030" />
    <Rule path="windows.win10.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.win10.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 10 Fluent Light`
```xml
<WinLookTheme target="win10">
  <Metadata name="Windows 10 Fluent Light" author="WinLook Project" />
  <Translations>
    <Rule path="windows.win10.dwm.window.corner_preference" value="donotround" />
    <Rule path="windows.win10.dwm.window.backdrop_type" value="none" />
    <Rule path="windows.win10.dwm.window.dark_mode" value="false" />
    <Rule path="windows.win10.dwm.window.caption_color" value="0x00F0F0F0" />
    <Rule path="windows.win10.dwm.window.text_color" value="0x00000000" />
    <Rule path="windows.win10.dwm.window.border_color" value="0x00E0E0E0" />
    <Rule path="windows.win10.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.win10.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 11 Mica Minimalist`
```xml
<WinLookTheme target="win11">
  <Metadata name="Windows 11 Mica Minimalist" author="WinLook Project" />
  <Translations>
    <Rule path="windows.win11.dwm.window.corner_preference" value="round" />
    <Rule path="windows.win11.dwm.window.backdrop_type" value="mica" />
    <Rule path="windows.win11.dwm.window.dark_mode" value="true" />
    <Rule path="windows.win11.dwm.window.caption_color" value="0x00181818" />
    <Rule path="windows.win11.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.win11.dwm.window.border_color" value="0x00282828" />
    <Rule path="windows.win11.shell.taskbar.pinned_shortcuts" value="visible" />
    <Rule path="windows.win11.shell.taskbar.offset_y" value="0" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 11 Floating Dock`
```xml
<WinLookTheme target="win11">
  <Metadata name="Windows 11 Floating Dock" author="WinLook Project" />
  <Translations>
    <Rule path="windows.win11.dwm.window.corner_preference" value="round" />
    <Rule path="windows.win11.dwm.window.backdrop_type" value="tabbed" />
    <Rule path="windows.win11.dwm.window.dark_mode" value="true" />
    <Rule path="windows.win11.dwm.window.caption_color" value="0x00101010" />
    <Rule path="windows.win11.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.win11.dwm.window.border_color" value="0x00202020" />
    <Rule path="windows.win11.shell.taskbar.pinned_shortcuts" value="hidden" />
    <Rule path="windows.win11.shell.taskbar.offset_y" value="-14" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```

#### Preset: `Windows 11 Compact Island`
```xml
<WinLookTheme target="win11">
  <Metadata name="Windows 11 Compact Island" author="WinLook Project" />
  <Translations>
    <Rule path="windows.win11.dwm.window.corner_preference" value="roundsmall" />
    <Rule path="windows.win11.dwm.window.backdrop_type" value="acrylic" />
    <Rule path="windows.win11.dwm.window.dark_mode" value="true" />
    <Rule path="windows.win11.dwm.window.caption_color" value="0x00141414" />
    <Rule path="windows.win11.dwm.window.text_color" value="0x00FFFFFF" />
    <Rule path="windows.win11.dwm.window.border_color" value="0x00242424" />
    <Rule path="windows.win11.shell.taskbar.pinned_shortcuts" value="hidden" />
    <Rule path="windows.win11.shell.taskbar.offset_y" value="-10" animation="ease_out_cubic" />
  </Translations>
</WinLookTheme>
```


---

## 12. Graphics Subsystem Pipelines & Hardware Frame Timing

The coordination of desktop graphics requires precise synchronization with hardware display cycles [cite: 1, 8, 9].

### 12.1 DWM Vertical Blank Synchronization
DWM composition executes synchronously with the monitor vertical refresh rate (60Hz, 120Hz, 144Hz, or 240Hz) [cite: 8, 9]. When an application updates its window surface, DWM schedules composition on the next v-blank cycle via `DwmGetCompositionTimingInfo` [cite: 9]. WinLook's `wl_animate` engine synchronizes delta time updates to these v-blank intervals to eliminate visual stuttering [cite: 7].


---

## 13. Subsystem Function and Attribute Reference

### 13.1 DWM Window Attribute Map

| Attribute ID | Constant Name | Type | Windows Version | Architectural Description |
| :--- | :--- | :--- | :--- | :--- |
| `1` | `DWMWA_NCRENDERING_ENABLED` | `BOOL` | Vista+ | Determines whether non-client rendering is enabled [cite: 9]. |
| `2` | `DWMWA_NCRENDERING_POLICY` | `DWORD` | Vista+ | Sets non-client rendering policy mode [cite: 9]. |
| `3` | `DWMWA_TRANSITIONS_FORCEDISABLED` | `BOOL` | Vista+ | Enables or suppresses window transition animations [cite: 9]. |
| `4` | `DWMWA_ALLOW_NCPAINT` | `BOOL` | Vista+ | Allows non-client painting by window procedure [cite: 9]. |
| `5` | `DWMWA_CAPTION_BUTTON_BOUNDS` | `RECT` | Vista+ | Retrieves caption button bounds rectangle [cite: 9]. |
| `6` | `DWMWA_NONCLIENT_RTL_LAYOUT` | `BOOL` | Vista+ | Controls right-to-left layout for non-client area [cite: 9]. |
| `7` | `DWMWA_FORCE_ICONIC_REPRESENTATION` | `BOOL` | Win7+ | Forces iconic thumbnail representation in taskbar [cite: 9]. |
| `8` | `DWMWA_FLIP3D_POLICY` | `DWORD` | Vista/7 | Specifies Flip3D window inclusion policy [cite: 9]. |
| `9` | `DWMWA_EXTENDED_FRAME_BOUNDS` | `RECT` | Vista+ | Retrieves physical frame bounds without drop shadow [cite: 9]. |
| `10` | `DWMWA_HAS_ICONIC_BITMAP` | `BOOL` | Win7+ | Queries whether window provides custom iconic bitmap [cite: 9]. |
| `11` | `DWMWA_DISALLOW_PEEK` | `BOOL` | Win7+ | Excludes window from Aero Peek desktop view [cite: 9]. |
| `12` | `DWMWA_EXCLUDED_FROM_PEEK` | `BOOL` | Win7+ | Queries whether window is excluded from Aero Peek [cite: 9]. |
| `13` | `DWMWA_CLOAK` | `BOOL` | Win8+ | Conceals window from screen while retaining composition surface [cite: 9]. |
| `14` | `DWMWA_CLOAKED` | `DWORD` | Win8+ | Queries cloaked state flags of window [cite: 9]. |
| `15` | `DWMWA_FREEZE_REPRESENTATION` | `BOOL` | Win8+ | Freezes visual representation of window [cite: 9]. |
| `20` | `DWMWA_USE_IMMERSIVE_DARK_MODE` | `BOOL` | Win11 22000+ (Doc) / Win10 19041+ (Undoc) | Formally documented in Win11; undocumented compatibility flag in Win10 2004+ (Attr 19 in 1809) [cite: 9]. |
| `33` | `DWMWA_WINDOW_CORNER_PREFERENCE` | `DWORD` | Win11 22000+ | Sets corner rounding preference (Default, Square, Round, SmallRound) [cite: 9, 13]. |
| `34` | `DWMWA_BORDER_COLOR` | `COLORREF` | Win11 22000+ | Sets custom perimeter border color [cite: 9]. |
| `35` | `DWMWA_CAPTION_COLOR` | `COLORREF` | Win11 22000+ | Sets custom title bar caption background color [cite: 9]. |
| `36` | `DWMWA_TEXT_COLOR` | `COLORREF` | Win11 22000+ | Sets custom title bar caption foreground text color [cite: 9]. |
| `37` | `DWMWA_VISIBLE_FRAME_BORDER_THICKNESS` | `UINT` | Win11 22000+ | Queries visible frame border thickness [cite: 9]. |
| `38` | `DWMWA_SYSTEMBACKDROP_TYPE` | `DWORD` | Win11 22621+ | Sets hardware backdrop material (None, Mica, Acrylic, Tabbed) [cite: 8, 9]. |

### 13.2 Visual Styles Ordinal Reference

| Ordinal | Exported Function Name | Parameter Types | OS Build | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **132** | `ShouldAppsUseDarkMode` | `() -> bool` | Build 17763+ | Queries user dark mode preference [cite: 24]. |
| **133** | `AllowDarkModeForWindow` | `(HWND, bool) -> bool` | Build 17763+ | Enables dark non-client frame rendering [cite: 24]. |
| **134** | `AllowDarkModeForApp` | `(bool) -> bool` | Build 17763 | Early application dark mode toggle [cite: 24]. |
| **135** | `SetPreferredAppMode` | `(int) -> int` | Build 18362+ | Configures process dark mode preference [cite: 24]. |
| **136** | `FlushMenuThemes` | `() -> void` | Build 17763+ | Flushes popup menu visual cache [cite: 24]. |
| **137** | `IsDarkModeAllowedForWindow` | `(HWND) -> bool` | Build 17763+ | Checks window dark mode authorization [cite: 24]. |
| **138** | `ShouldSystemUseDarkMode` | `() -> bool` | Build 18362+ | Queries shell dark mode setting [cite: 24]. |
| **139** | `SetWindowCompositionAttribute` | `(HWND, void*) -> bool` | Build 10240+ | Configures window blur behind surface [cite: 8, 24]. |


---

## 14. Non-Client Frame Subclassing & Custom Control Developer Guide

When building lightweight customization utilities in Rust or C, window subclassing via `comctl32.dll` provides a safe mechanism for intercepting non-client window messages without memory overwrites [cite: 19].

### 14.1 Complete Rust Implementation for `comctl32` Subclassing
```rust
use windows_sys::Win32::Foundation::{HWND, LRESULT, WPARAM, LPARAM, BOOL};
use windows_sys::Win32::UI::Shell::{
    SetWindowSubclass, RemoveWindowSubclass, DefSubclassProc
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    WM_NCPAINT, WM_NCCALCSIZE, WM_NCHITTEST, WM_DESTROY,
    HTCAPTION, HTCLIENT, HTCLOSE, HTMINBUTTON, HTMAXBUTTON
};

const SUBCLASS_ID_WINLOOK: usize = 0x574C4E4B;

pub unsafe extern "system" fn winlook_subclass_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uidsubclass: usize,
    dwrefdata: usize,
) -> LRESULT {
    match umsg {
        WM_NCCALCSIZE => {
            if wparam != 0 {
                // Modify client rectangle to customize non-client margin borders
                return 0;
            }
            DefSubclassProc(hwnd, umsg, wparam, lparam)
        }
        WM_NCHITTEST => {
            // Pass to default handler first
            let hit = DefSubclassProc(hwnd, umsg, wparam, lparam);
            if hit == HTCLIENT as isize {
                // Query cursor coordinates to support custom draggable areas
                // return HTCAPTION as isize;
            }
            hit
        }
        WM_DESTROY => {
            // Automatically clean up subclass when target window is destroyed
            RemoveWindowSubclass(hwnd, Some(winlook_subclass_proc), uidsubclass);
            DefSubclassProc(hwnd, umsg, wparam, lparam)
        }
        _ => DefSubclassProc(hwnd, umsg, wparam, lparam),
    }
}

pub unsafe fn attach_subclass(hwnd: HWND) -> bool {
    SetWindowSubclass(hwnd, Some(winlook_subclass_proc), SUBCLASS_ID_WINLOOK, 0) != 0
}
```


---

## 15. Win32 Message Loop & Event Routing Architecture

The Windows Graphical User Interface is fundamentally event-driven [cite: 1, 4, 6]. Thread execution loops continuously poll their dedicated kernel message queues via `GetMessageW` or `PeekMessageW` [cite: 6]:

```
+--------------------------------------------------------------------------+
| Kernel Input Subsystem (win32k.sys)                                      |
|   Raw Input Thread (RIT) -> System Input Queue                           |
+--------------------------------------------------------------------------+
                         |                                                  
                         v Routes events to thread message queue            
+--------------------------------------------------------------------------+
| WinLook Thread Message Loop (Ring 3)                                     |
|                                                                          |
|   loop {                                                                 |
|       GetMessageW(&msg, NULL, 0, 0)                                      |
|          |                                                               |
|          v                                                               |
|       TranslateMessage(&msg)  // Translates virtual keys to WM_CHAR      |
|          |                                                               |
|          v                                                               |
|       DispatchMessageW(&msg)  // Invokes target window's WNDPROC         |
|   }                                                                      |
+--------------------------------------------------------------------------+
```


---

## 16. Multi-Monitor Coordinate Geometry & Per-Monitor DPI V2 Scaling

Modern Windows desktops frequently span multiple physical monitors featuring divergent scaling factors (e.g., a 4K display at 175% DPI alongside a 1080p display at 100% DPI) [cite: 1]. WinLook calculates layout offsets in physical device coordinates [cite: 7]:

### 16.1 Virtual Desktop Rectangles
* `SM_XVIRTUALSCREEN` and `SM_YVIRTUALSCREEN`: The top-left virtual screen coordinate, which can be negative if secondary monitors are positioned to the left or top of the primary monitor [cite: 23].
* `SM_CXVIRTUALSCREEN` and `SM_CYVIRTUALSCREEN`: The total bounding width and height across all active displays [cite: 23].

### 16.2 Per-Monitor DPI V2 Awareness Declaration
To prevent the operating system from applying blurry bitmap stretch scaling, `winlook.bg.exe` embeds an application manifest declaring `<dpiAwareness>PerMonitorV2</dpiAwareness>` and invokes `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` upon process startup [cite: 1, 7].


---

## 17. WinLook Fast IPC Wire Protocol Specification (`wl_ipc`)

The WinLook desktop architecture decouples the background engine (`winlook.bg.exe`) from the configuration manager UI via local named pipes (`\\.\pipe\WinLook_IPC`) [cite: 7]:

### 17.1 Binary Packet Framing Layout
Each IPC message sent across the pipe begins with a fixed 16-byte binary header followed by an optional UTF-8 JSON payload [cite: 7]:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Magic Identifier (0x574C4E4B)              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Message Type         |          Payload Length       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Reserved (Zero)                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                  UTF-8 Payload Data (Variable)                |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### 17.2 Message Types
* `0x0001` (`MSG_HANDSHAKE`): Initializes the pipe connection, verifies protocol version, and exchanges process IDs.
* `0x0002` (`MSG_SET_PROPERTY`): Dispatches a property modification command (e.g., `windows.win11.dwm.window.corner_preference = round`).
* `0x0003` (`MSG_QUERY_PROPERTY`): Queries the current active value of an addressable property.
* `0x0004` (`MSG_EVENT_NOTIFICATION`): Asynchronously broadcasts desktop lifecycle events (such as active window changes or session locks) from the background daemon to the manager UI.
* `0x0005` (`MSG_ROLLBACK_ALL`): Instructs the background daemon to immediately restore all modified window properties and flush the transaction journal.


---

## 18. Advanced Physics Curves & Numerical Root-Finding (`wl_animate`)

In standard UI animation engines, cubic bezier curves parameterized by control points \(P_1=(x_1, y_1)\) and \(P_2=(x_2, y_2)\) must be evaluated to find \(y\) for a given normalized time \(x \in [0, 1]\) [cite: 7]. Because \(x(t)\) is a cubic polynomial:

\[x(t) = 3(1-t)^2 t x_1 + 3(1-t) t^2 x_2 + t^3\]

analytical solutions via Cardano's formula are computationally heavy. `wl_animate` solves for \(t\) given \(x\) using an optimized Newton-Raphson iterative solver executing in under 15 nanoseconds [cite: 7]:

```rust
pub struct FastBezierSolver {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl FastBezierSolver {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    #[inline(always)]
    fn sample_curve_x(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * self.x1 + 3.0 * mt * t * t * self.x2 + t * t * t
    }

    #[inline(always)]
    fn sample_curve_y(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * mt * mt * t * self.y1 + 3.0 * mt * t * t * self.y2 + t * t * t
    }

    #[inline(always)]
    fn sample_curve_derivative_x(&self, t: f32) -> f32 {
        let mt = 1.0 - t;
        3.0 * (1.0 - 4.0 * t + 3.0 * t * t) * self.x1
            + 3.0 * (2.0 * t - 3.0 * t * t) * self.x2
            + 3.0 * t * t
    }

    pub fn solve(&self, x: f32, epsilon: f32) -> f32 {
        let mut t = x; // Initial guess
        for _ in 0..8 {
            let current_x = self.sample_curve_x(t) - x;
            if current_x.abs() < epsilon {
                return self.sample_curve_y(t);
            }
            let d_x = self.sample_curve_derivative_x(t);
            if d_x.abs() < 1e-6 {
                break;
            }
            t -= current_x / d_x;
            t = t.clamp(0.0, 1.0);
        }
        self.sample_curve_y(t)
    }
}
```


---

## 19. Legacy C Subsystems & Modern Rust Interop Architecture (`wl_Lui`, `wl_Lanimate`, and Safe Rust FFI)

While modern editions of Windows (Windows 10 Build 19041+ and Windows 11) offer sophisticated hardware-accelerated composition engines via DWM and DirectComposition, legacy operating systems—specifically Windows 95, 98, ME, NT 4.0, 2000, and XP—remain fundamentally constrained by the classical GDI/USER subsystem [cite: 1, 4, 7]. Modern Rust binaries compiled with modern MSVC toolchains require CRT runtimes (`vcruntime140.dll`) and kernel subsystem APIs that do not exist on these legacy platforms [cite: 2, 7].

To achieve complete backwards compatibility without compromising modern Rust ergonomics, WinLook employs a **Dual-Engine Architecture** [cite: 7]:
1. **The Legacy C Engine (`wl_Lui` & `wl_Lanimate`)**: Written in strict ANSI C89, compiling with MSVC 6.0, MinGW32, or Open Watcom, targeting raw `win32k.sys` / `USER32.DLL` / `GDI32.DLL` interfaces [cite: 4, 7, 23].
2. **The Modern Rust Engine (`winlook.bg.exe`)**: Written in modern safe Rust (2021 edition), hosting high-level DWM, DirectComposition, and XAML/WinUI orchestration layers [cite: 8, 9, 22].
3. **The Bi-Directional FFI Bridge**: An explicit `extern "C"` ABI boundary enabling modern Rust applications to seamlessly call legacy C routines, and legacy C companion agents to dispatch commands to modern Rust daemons via shared IPC [cite: 7, 22].

---

### 19.1 Pure C89 Native Implementation Subsystems

The legacy C layer implements window subclassing, GDI non-client painting, window region shaping, and desktop wallpaper injection with zero external dependencies beyond `windows.h` [cite: 23].

#### 19.1.1 Pre-XP Window Subclassing Engine (`wl_Lui_subclass.c`)
On Windows 95 through Windows 2000, the safe `comctl32.dll` subclassing APIs (`SetWindowSubclass`, `DefSubclassProc`) are unavailable [cite: 1, 23]. Standard `SetWindowLongPtrA(hwnd, GWLP_WNDPROC, ...)` overwrites are vulnerable to re-entrancy bugs and race conditions if multiple utilities subclass the same window [cite: 6, 23]. 

WinLook implements a property-based subclass dispatcher using `SetPropA` / `GetPropA` to store original window procedures directly in user-mode object memory without global lock contention [cite: 6, 23]:

```c
/* wl_Lui_subclass.c - Pure ANSI C89 Safe Subclass Engine for Win95-2000 */
#include <windows.h>

#define WL_OLDWNDPROC_PROP "Wl_OriginalWndProc"
#define WL_SUBCLASSDATA_PROP "Wl_SubclassData"

typedef LRESULT (CALLBACK* WL_SUBCLASS_CALLBACK)(
    HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam, 
    WNDPROC original_proc, void* user_data
);

typedef struct {
    WL_SUBCLASS_CALLBACK callback;
    void* user_data;
    WNDPROC original_proc;
} WL_SUBCLASS_ENTRY;

/* Subclass window procedure dispatching through property table */
static LRESULT CALLBACK Wl_SubclassDispatcher(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    WL_SUBCLASS_ENTRY* entry = (WL_SUBCLASS_ENTRY*)GetPropA(hwnd, WL_SUBCLASSDATA_PROP);
    if (!entry) {
        return DefWindowProcA(hwnd, uMsg, wParam, lParam);
    }

    if (uMsg == WM_NCDESTROY) {
        WNDPROC orig = entry->original_proc;
        WL_SUBCLASS_CALLBACK cb = entry->callback;
        void* ud = entry->user_data;

        /* Execute callback for final destruction event */
        LRESULT res = cb(hwnd, uMsg, wParam, lParam, orig, ud);

        /* Clean up properties before window memory is freed by win32k */
        RemovePropA(hwnd, WL_OLDWNDPROC_PROP);
        RemovePropA(hwnd, WL_SUBCLASSDATA_PROP);
        SetWindowLongA(hwnd, GWL_WNDPROC, (LONG)orig);
        HeapFree(GetProcessHeap(), 0, entry);
        return res;
    }

    return entry->callback(hwnd, uMsg, wParam, lParam, entry->original_proc, entry->user_data);
}

BOOL Wl_InstallSubclass(HWND hwnd, WL_SUBCLASS_CALLBACK callback, void* user_data) {
    WL_SUBCLASS_ENTRY* entry;
    WNDPROC old_proc;

    if (!IsWindow(hwnd) || !callback) return FALSE;

    /* Verify window is not already subclassed by WinLook */
    if (GetPropA(hwnd, WL_SUBCLASSDATA_PROP) != NULL) {
        return FALSE;
    }

    entry = (WL_SUBCLASS_ENTRY*)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(WL_SUBCLASS_ENTRY));
    if (!entry) return FALSE;

    old_proc = (WNDPROC)GetWindowLongA(hwnd, GWL_WNDPROC);
    if (!old_proc) {
        HeapFree(GetProcessHeap(), 0, entry);
        return FALSE;
    }

    entry->callback = callback;
    entry->user_data = user_data;
    entry->original_proc = old_proc;

    SetPropA(hwnd, WL_OLDWNDPROC_PROP, (HANDLE)old_proc);
    SetPropA(hwnd, WL_SUBCLASSDATA_PROP, (HANDLE)entry);

    SetWindowLongA(hwnd, GWL_WNDPROC, (LONG)Wl_SubclassDispatcher);
    return TRUE;
}

BOOL Wl_RemoveSubclass(HWND hwnd) {
    WL_SUBCLASS_ENTRY* entry = (WL_SUBCLASS_ENTRY*)GetPropA(hwnd, WL_SUBCLASSDATA_PROP);
    if (!entry) return FALSE;

    SetWindowLongA(hwnd, GWL_WNDPROC, (LONG)entry->original_proc);
    RemovePropA(hwnd, WL_OLDWNDPROC_PROP);
    RemovePropA(hwnd, WL_SUBCLASSDATA_PROP);
    HeapFree(GetProcessHeap(), 0, entry);
    return TRUE;
}
```

---

#### 19.1.2 Legacy Window Region Shaping & GDI Clipping (`wl_Lui_region.c`)
Before DWM rounded corners (`DWMWA_WINDOW_CORNER_PREFERENCE`), window geometry was shaped using GDI window regions via `SetWindowRgn` [cite: 9, 13, 23]. Window regions clip both the client and non-client areas directly within `win32k.sys`, avoiding rectangular artifacts [cite: 6, 23]:

```c
/* wl_Lui_region.c - C89 Rounded Geometry Clipping for Win95-Win7 */
#include <windows.h>

BOOL Wl_ApplyWindowCornerRadius(HWND hwnd, int corner_radius) {
    RECT rect;
    HRGN hRgn;
    int width, height;

    if (!GetWindowRect(hwnd, &rect)) return FALSE;
    width = rect.right - rect.left;
    height = rect.bottom - rect.top;

    if (corner_radius <= 0) {
        /* Reset to standard rectangular window geometry */
        SetWindowRgn(hwnd, NULL, TRUE);
        return TRUE;
    }

    /* Create rounded rectangular region clipping bounding frame */
    hRgn = CreateRoundRectRgn(0, 0, width + 1, height + 1, corner_radius * 2, corner_radius * 2);
    if (!hRgn) return FALSE;

    /*
     * Operating system assumes ownership of hRgn upon successful SetWindowRgn call.
     * Do NOT call DeleteObject(hRgn) if SetWindowRgn succeeds.
     */
    if (!SetWindowRgn(hwnd, hRgn, TRUE)) {
        DeleteObject(hRgn);
        return FALSE;
    }

    return TRUE;
}
```

---

#### 19.1.3 Pure C89 Desktop WorkerW Split & Wallpaper Blitter (`wl_Lwlde_desktop.c`)
To run animated background canvases on Windows 7, 8, 10, and 11 without third-party frameworks, the native C89 implementation communicates directly with `Progman` [cite: 7]:

```c
/* wl_Lwlde_desktop.c - C89 Desktop Split & Background Injection */
#include <windows.h>

static HWND g_workerw_hwnd = NULL;

static BOOL CALLBACK EnumWindowsSeekWorkerW(HWND hwnd, LPARAM lParam) {
    HWND hShellView = FindWindowExA(hwnd, NULL, "SHELLDLL_DefView", NULL);
    if (hShellView != NULL) {
        /* Target WorkerW is the next sibling window behind SHELLDLL_DefView */
        g_workerw_hwnd = FindWindowExA(NULL, hwnd, "WorkerW", NULL);
        return FALSE; /* Stop enumeration */
    }
    return TRUE;
}

HWND Wl_AcquireDesktopCanvasHwnd(void) {
    HWND hProgman;
    DWORD_PTR dwResult = 0;

    hProgman = FindWindowA("Progman", "Program Manager");
    if (!hProgman) return NULL;

    /* Issue undocumented Progman split message 0x052C */
    SendMessageTimeoutA(
        hProgman,
        0x052C,
        0x0000000D,
        0,
        SMTO_NORMAL,
        1000,
        &dwResult
    );

    g_workerw_hwnd = NULL;
    EnumWindows(EnumWindowsSeekWorkerW, 0);

    /* Fallback to Progman if WorkerW is not spawned (e.g., Windows XP / 2000) */
    if (!g_workerw_hwnd) {
        g_workerw_hwnd = hProgman;
    }

    return g_workerw_hwnd;
}

void Wl_BlitCustomBackground(HWND hCanvas, HBITMAP hBitmap, int x, int y, int w, int h) {
    HDC hdcWindow;
    HDC hdcMem;
    HBITMAP hOldBitmap;

    hdcWindow = GetDC(hCanvas);
    if (!hdcWindow) return;

    hdcMem = CreateCompatibleDC(hdcWindow);
    hOldBitmap = (HBITMAP)SelectObject(hdcMem, hBitmap);

    /* Direct double-buffered hardware blit */
    BitBlt(hdcWindow, x, y, w, h, hdcMem, 0, 0, SRCCOPY);

    SelectObject(hdcMem, hOldBitmap);
    DeleteDC(hdcMem);
    ReleaseDC(hCanvas, hdcWindow);
}
```

---

#### 19.1.4 Pure C89 Bezier Animation Driver (`wl_Lanimate.c`)
The standalone C89 animation math engine provides smooth sub-pixel ease-in-out transformations on vintage hardware:

```c
/* wl_Lanimate.h - Pure ANSI C89 Animation Driver Header */
#ifndef WL_LANIMATE_H
#define WL_LANIMATE_H

#include <windows.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float x1, y1, x2, y2;
} WL_BEZIER;

typedef struct {
    HWND hwnd;
    RECT start_rect;
    RECT target_rect;
    DWORD start_time;
    DWORD duration_ms;
    WL_BEZIER curve;
    BOOL active;
} WL_ANIM_CHANNEL;

void WlInitAnimation(WL_ANIM_CHANNEL* ch, HWND hwnd, RECT target, DWORD ms);
BOOL WlStepAnimation(WL_ANIM_CHANNEL* ch, DWORD current_time);

#ifdef __cplusplus
}
#endif

#endif /* WL_LANIMATE_H */
```

```c
/* wl_Lanimate.c - Numerical Bezier Transformation Implementation */
#include "wl_Lanimate.h"

void WlInitAnimation(WL_ANIM_CHANNEL* ch, HWND hwnd, RECT target, DWORD ms) {
    ch->hwnd = hwnd;
    GetWindowRect(hwnd, &ch->start_rect);
    ch->target_rect = target;
    ch->start_time = GetTickCount();
    ch->duration_ms = ms;
    /* Standard Ease-Out Cubic: (0.215, 0.610, 0.355, 1.000) */
    ch->curve.x1 = 0.215f; ch->curve.y1 = 0.610f;
    ch->curve.x2 = 0.355f; ch->curve.y2 = 1.000f;
    ch->active = TRUE;
}

BOOL WlStepAnimation(WL_ANIM_CHANNEL* ch, DWORD current_time) {
    float progress, eased;
    int cur_x, cur_y, cur_w, cur_h;
    if (!ch->active) return FALSE;

    if (current_time >= ch->start_time + ch->duration_ms) {
        SetWindowPos(ch->hwnd, NULL,
            ch->target_rect.left, ch->target_rect.top,
            ch->target_rect.right - ch->target_rect.left,
            ch->target_rect.bottom - ch->target_rect.top,
            SWP_NOZORDER | SWP_NOACTIVATE);
        ch->active = FALSE;
        return FALSE;
    }

    progress = (float)(current_time - ch->start_time) / (float)ch->duration_ms;
    /* Analytical cubic ease-out approximation */
    eased = 1.0f - (1.0f - progress) * (1.0f - progress) * (1.0f - progress);

    cur_x = ch->start_rect.left + (int)((ch->target_rect.left - ch->start_rect.left) * eased);
    cur_y = ch->start_rect.top + (int)((ch->target_rect.top - ch->start_rect.top) * eased);
    cur_w = (ch->start_rect.right - ch->start_rect.left) + 
            (int)(((ch->target_rect.right - ch->target_rect.left) - (ch->start_rect.right - ch->start_rect.left)) * eased);
    cur_h = (ch->start_rect.bottom - ch->start_rect.top) + 
            (int)(((ch->target_rect.bottom - ch->target_rect.top) - (ch->start_rect.bottom - ch->start_rect.top)) * eased);

    SetWindowPos(ch->hwnd, NULL, cur_x, cur_y, cur_w, cur_h, SWP_NOZORDER | SWP_NOACTIVATE);
    return TRUE;
}
```

---

### 19.2 Modern Rust FFI & ABI Bridge (`wl_legacy_ffi.rs`)

To allow modern Rust daemons and developer agents to control legacy C subsystems safely, WinLook defines zero-cost, type-safe FFI bindings [cite: 7, 22]:

```rust
// wl_legacy_ffi.rs - Rust FFI Binding for C89 WinLook Subsystems
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HWND, RECT, BOOL, TRUE, FALSE};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WlBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WlAnimChannel {
    pub hwnd: HWND,
    pub start_rect: RECT,
    pub target_rect: RECT,
    pub start_time: u32,
    pub duration_ms: u32,
    pub curve: WlBezier,
    pub active: BOOL,
}

// Subclass callback function pointer type matching C-ABI
pub type WlSubclassCallback = unsafe extern "system" fn(
    hwnd: HWND,
    u_msg: u32,
    w_param: usize,
    l_param: isize,
    original_proc: usize,
    user_data: *mut c_void,
) -> isize;

#[link(name = "wl_legacy_c", kind = "static")]
extern "C" {
    pub fn Wl_InstallSubclass(
        hwnd: HWND,
        callback: WlSubclassCallback,
        user_data: *mut c_void,
    ) -> BOOL;

    pub fn Wl_RemoveSubclass(hwnd: HWND) -> BOOL;

    pub fn Wl_ApplyWindowCornerRadius(hwnd: HWND, corner_radius: i32) -> BOOL;

    pub fn Wl_AcquireDesktopCanvasHwnd() -> HWND;

    pub fn WlInitAnimation(
        ch: *mut WlAnimChannel,
        hwnd: HWND,
        target: RECT,
        ms: u32,
    );

    pub fn WlStepAnimation(ch: *mut WlAnimChannel, current_time: u32) -> BOOL;
}

/// Safe idiomatic Rust wrapper for window corner radius clipping
pub struct LegacyWindowRegion;

impl LegacyWindowRegion {
    pub fn set_corner_radius(hwnd: HWND, radius: i32) -> Result<(), &'static str> {
        let success = unsafe { Wl_ApplyWindowCornerRadius(hwnd, radius) };
        if success == TRUE {
            Ok(())
        } else {
            Err("Failed to apply window clipping region")
        }
    }
}

/// Safe idiomatic Rust wrapper for animated window transformations
pub struct LegacyAnimator {
    channel: WlAnimChannel,
}

impl LegacyAnimator {
    pub fn new(hwnd: HWND, target: RECT, duration_ms: u32) -> Self {
        let mut channel: WlAnimChannel = unsafe { std::mem::zeroed() };
        unsafe {
            WlInitAnimation(&mut channel, hwnd, target, duration_ms);
        }
        Self { channel }
    }

    pub fn step(&mut self, current_time_ms: u32) -> bool {
        let active = unsafe { WlStepAnimation(&mut self.channel, current_time_ms) };
        active == TRUE
    }

    pub fn is_active(&self) -> bool {
        self.channel.active == TRUE
    }
}
```

---

### 19.3 Dual-Engine Runtime Dispatch Engine (`wl_dual_engine.rs`)

WinLook dynamically selects between modern DirectComposition/DWM attributes and legacy C89 GDI clipping based on the host operating system detected at runtime [cite: 8, 9, 13, 18]:

```rust
// wl_dual_engine.rs - Dynamic Runtime Backend Selector
use windows_sys::Win32::Foundation::{HWND, RECT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsArchitectureEra {
    LegacyGdi,     // Windows 95, 98, ME, 2000, XP
    DwmLegacyGlass,// Windows Vista, Windows 7, Windows 8
    ModernDwm,     // Windows 10 (19041+), Windows 11 (22000+)
}

pub trait WindowCustomizationEngine {
    fn set_corner_rounding(&self, hwnd: HWND, radius: i32) -> bool;
    fn animate_bounds(&self, hwnd: HWND, target: RECT, duration_ms: u32) -> bool;
    fn get_desktop_canvas(&self) -> HWND;
}

pub struct DualEngineDispatcher {
    era: OsArchitectureEra,
}

impl DualEngineDispatcher {
    pub fn detect() -> Self {
        // Query OS Build using dynamic RtlGetVersion
        let era = Self::query_os_era();
        Self { era }
    }

    fn query_os_era() -> OsArchitectureEra {
        use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
        unsafe {
            let ntdll = GetModuleHandleW(utf16_lit::utf16_null!("ntdll.dll").as_ptr());
            if ntdll == 0 {
                return OsArchitectureEra::LegacyGdi;
            }

            type FnRtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
            let symbol = GetProcAddress(ntdll, b"RtlGetVersion\0".as_ptr());
            if let Some(func) = symbol {
                let rtl_get_version: FnRtlGetVersion = std::mem::transmute(func);
                let mut osvi: OSVERSIONINFOW = std::mem::zeroed();
                osvi.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

                if rtl_get_version(&mut osvi) == 0 {
                    if osvi.dwMajorVersion >= 10 {
                        return OsArchitectureEra::ModernDwm;
                    } else if osvi.dwMajorVersion >= 6 {
                        return OsArchitectureEra::DwmLegacyGlass;
                    }
                }
            }
            OsArchitectureEra::LegacyGdi
        }
    }

    pub fn apply_corner_styling(&self, hwnd: HWND, radius: i32) -> bool {
        match self.era {
            OsArchitectureEra::ModernDwm => {
                // Modern Path: Windows 11 Attribute 33 (DWMWA_WINDOW_CORNER_PREFERENCE)
                let preference = if radius <= 0 { 1u32 } else { 2u32 }; // 1=DoNotRound, 2=Round
                unsafe {
                    let dwmapi = windows_sys::Win32::System::LibraryLoader::LoadLibraryW(
                        utf16_lit::utf16_null!("dwmapi.dll").as_ptr(),
                    );
                    if dwmapi != 0 {
                        type FnDwmSet = unsafe extern "system" fn(HWND, u32, *const std::ffi::c_void, u32) -> i32;
                        if let Some(f) = windows_sys::Win32::System::LibraryLoader::GetProcAddress(dwmapi, b"DwmSetWindowAttribute\0".as_ptr()) {
                            let dwm_set: FnDwmSet = std::mem::transmute(f);
                            let hr = dwm_set(hwnd, 33, &preference as *const _ as *const _, 4);
                            return hr == 0;
                        }
                    }
                }
                false
            }
            OsArchitectureEra::DwmLegacyGlass | OsArchitectureEra::LegacyGdi => {
                // Legacy Path: Pure C89 Window Region Clipping (CreateRoundRectRgn + SetWindowRgn)
                unsafe {
                    crate::wl_legacy_ffi::Wl_ApplyWindowCornerRadius(hwnd, radius) == windows_sys::Win32::Foundation::TRUE
                }
            }
        }
    }

    pub fn acquire_wallpaper_canvas(&self) -> HWND {
        // Universal WorkerW discovery works from Windows 7 through Windows 11
        unsafe {
            crate::wl_legacy_ffi::Wl_AcquireDesktopCanvasHwnd()
        }
    }
}

#[repr(C)]
struct OSVERSIONINFOW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
}
```

---

### 19.4 Agent & User Automation Integration Patterns

Developers and AI agents can orchestrate Windows shell elements via either a **Modern Rust CLI** or a **Legacy C Scripting Client** communicating through standard named pipes or direct tool execution [cite: 7].

#### 19.4.1 Rust Automation Script (`winlook_agent.rs`)
An automated agent executing in modern Rust can inspect running windows, resolve their object paths, and issue atomic transformations:

```rust
// winlook_agent.rs - Automated Window Customization Agent Script
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetForegroundWindow};

fn main() {
    println!("[WinLook Agent] Initializing Dual-Engine Subsystem...");
    let dispatcher = DualEngineDispatcher::detect();

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd == 0 {
            eprintln!("[WinLook Agent] No active window in foreground.");
            return;
        }

        println!("[WinLook Agent] Targeting Active HWND: 0x{:X}", hwnd);

        // 1. Transform Window Corners to Rounded (via DWM on Win11, SetWindowRgn on Win95-Win7)
        let corner_applied = dispatcher.apply_corner_styling(hwnd, 12);
        println!("[WinLook Agent] Applied corner radius (12px): {}", corner_applied);

        // 2. Animate Window to New Position via C89 Legacy Engine
        let mut rect: RECT = std::mem::zeroed();
        GetWindowRect(hwnd, &mut rect);

        let target_rect = RECT {
            left: rect.left + 50,
            top: rect.top + 50,
            right: rect.right + 50,
            bottom: rect.bottom + 50,
        };

        let mut anim = LegacyAnimator::new(hwnd, target_rect, 350); // 350ms ease-out
        println!("[WinLook Agent] Executing smooth trajectory animation...");

        let start = std::time::Instant::now();
        while anim.is_active() {
            let elapsed = start.elapsed().as_millis() as u32;
            anim.step(elapsed);
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
        }

        println!("[WinLook Agent] Animation completed successfully.");
    }
}
```

---

#### 19.4.2 Legacy C Automation Client (`winlook_legacy_client.c`)
For environments where only pure C89 is available (e.g., Windows 98/2000 or embedded MinGW environments), this standalone client interacts directly with the `wl_ipc` named pipe interface:

```c
/* winlook_legacy_client.c - Pure C89 Automation Client */
#include <windows.h>
#include <stdio.h>

#define WINLOOK_PIPE "\\.\pipe\WinLook_IPC"

typedef struct {
    DWORD magic;       /* 0x574C4B31 ("WLK1") */
    DWORD cmd_type;    /* 1 = SetProperty, 2 = AnimateWindow */
    DWORD payload_len;
} WL_IPC_HEADER;

BOOL SendWinLookCommand(DWORD cmd_type, const char* json_payload) {
    HANDLE hPipe;
    DWORD bytesWritten;
    WL_IPC_HEADER header;
    DWORD len = (DWORD)strlen(json_payload);

    hPipe = CreateFileA(
        WINLOOK_PIPE,
        GENERIC_READ | GENERIC_WRITE,
        0,
        NULL,
        OPEN_EXISTING,
        0,
        NULL
    );

    if (hPipe == INVALID_HANDLE_VALUE) {
        printf("Error: WinLook Background Daemon is not running.\n");
        return FALSE;
    }

    header.magic = 0x574C4B31;
    header.cmd_type = cmd_type;
    header.payload_len = len;

    /* Write packet header */
    WriteFile(hPipe, &header, sizeof(header), &bytesWritten, NULL);
    /* Write packet payload */
    WriteFile(hPipe, json_payload, len, &bytesWritten, NULL);

    CloseHandle(hPipe);
    printf("Success: Dispatched command type %lu to WinLook core.\n", cmd_type);
    return TRUE;
}

int main(int argc, char* argv[]) {
    const char* command = "{\"path\": \"windows.all.window.corner\", \"value\": \"round\"}";
    printf("[WinLook C89 Client] Dispatching customization payload...\n");
    SendWinLookCommand(1, command);
    return 0;
}
```

---

### 19.5 Cargo Build Configuration & Native C Compilation (`build.rs`)

To compile the C89 codebase and link it directly into the Rust executable without external manual steps, WinLook includes a standard `build.rs` script using the `cc` crate:

```rust
// build.rs - Cargo Build Script for Compiling Legacy C89 Subsystems
fn main() {
    println!("cargo:rerun-if-changed=c_src/wl_Lui_subclass.c");
    println!("cargo:rerun-if-changed=c_src/wl_Lui_region.c");
    println!("cargo:rerun-if-changed=c_src/wl_Lwlde_desktop.c");
    println!("cargo:rerun-if-changed=c_src/wl_Lanimate.c");
    println!("cargo:rerun-if-changed=c_src/wl_Lanimate.h");

    cc::Build::new()
        .file("c_src/wl_Lui_subclass.c")
        .file("c_src/wl_Lui_region.c")
        .file("c_src/wl_Lwlde_desktop.c")
        .file("c_src/wl_Lanimate.c")
        .warnings(true)
        .extra_warnings(false)
        // Enforce strict ANSI C89 / C90 compatibility
        .flag_if_supported("-std=c89")
        .flag_if_supported("/std:c89")
        .compile("wl_legacy_c");

    // Link required Windows system libraries
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=kernel32");
}
```

---


## 20. Watchdog Process Architecture & Clean State Restoration

To ensure that desktop customizations can never leave the Windows operating system in an unusable state, WinLook deploys a secondary lightweight watchdog helper (`winlook.watchdog.exe`) [cite: 7]:

```rust
// winlook.watchdog.exe - Rust Shared Memory Rollback Implementation
use windows_sys::Win32::Foundation::{HANDLE, CloseHandle, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE};
use windows_sys::Win32::System::Memory::{OpenFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, ShowWindow, SW_SHOW, SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_FRAMECHANGED};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { return; }
    let parent_pid: u32 = match args[1].parse() { Ok(p) => p, Err(_) => return };

    unsafe {
        let h_parent: HANDLE = OpenProcess(PROCESS_SYNCHRONIZE, 0, parent_pid);
        if h_parent != 0 {
            // Block synchronously until parent background engine process exits or crashes
            if WaitForSingleObject(h_parent, 0xFFFFFFFF) == WAIT_OBJECT_0 {
                println!("[WinLook Watchdog] Parent process terminated. Inspecting shared crash recovery journal...");

                // Open shared memory state journal
                let map_name = utf16_lit::utf16_null!("Local\\WinLook_StateJournal");
                let h_map = OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, map_name.as_ptr());
                if h_map != 0 {
                    let view = MapViewOfFile(h_map, FILE_MAP_ALL_ACCESS, 0, 0, std::mem::size_of::<SharedJournalHeader>());
                    if !view.is_null() {
                        let header = view as *mut SharedJournalHeader;
                        // Only rollback if the parent did NOT exit cleanly
                        if (*header).clean_shutdown == 0 {
                            println!("[WinLook Watchdog] Unclean shutdown detected. Executing rollback on {} transactions...", (*header).entry_count);
                            for i in (0..(*header).entry_count as usize).rev() {
                                let entry = &mut (*header).entries[i];
                                if entry.is_reverted == 0 {
                                    if entry.property_id == 5 { // Taskbar visibility
                                        let cmd = if entry.original_value == 1 { SW_SHOW } else { 0 };
                                        ShowWindow(entry.hwnd, cmd);
                                    } else if entry.property_id == 1 { // Corner preference
                                        let default_pref = entry.original_value as u32;
                                        // Restore default corner preference
                                        let dwmapi = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(utf16_lit::utf16_null!("dwmapi.dll").as_ptr());
                                        if dwmapi != 0 {
                                            let proc = windows_sys::Win32::System::LibraryLoader::GetProcAddress(dwmapi, b"DwmSetWindowAttribute\0".as_ptr());
                                            if let Some(f) = proc {
                                                type FnSetAttr = unsafe extern "system" fn(HWND, u32, *const std::ffi::c_void, u32) -> i32;
                                                let set_attr: FnSetAttr = std::mem::transmute(f);
                                                set_attr(entry.hwnd, 33, &default_pref as *const _ as *const _, 4);
                                            }
                                        }
                                    }
                                    SetWindowPos(entry.hwnd, 0, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
                                    entry.is_reverted = 1;
                                }
                            }
                        }
                    }
                    CloseHandle(h_map);
                }

                // Failsafe: Ensure native taskbar is visible
                let h_tray = FindWindowW(utf16_lit::utf16_null!("Shell_TrayWnd").as_ptr(), std::ptr::null());
                if h_tray != 0 {
                    ShowWindow(h_tray, SW_SHOW);
                }
            }
            CloseHandle(h_parent);
        }
    }
}
```

### Summary of Architectural Guarantees
1. **Zero System File Patching**: WinLook executes entirely in RAM, preserving 100% of native Windows system files [cite: 1, 7].
2. **Low Memory Footprint**: By eliminating embedded browser engines, WinLook maintains a working set below 10MB of RAM [cite: 7].
3. **Complete Reversibility**: Any modification applied by WinLook is instantly rolled back on shutdown, uninstallation, or emergency hotkey [cite: 7, 19].


---

## 21. WinLook Automation CLI & Scripting Reference (`winlook.exe`)

WinLook provides a comprehensive command-line interface allowing shell scripts, power users, and automation tools to inspect and configure the desktop environment [cite: 7]:

### 21.1 CLI Commands and Parameter Syntax

| Command | Parameters | Description | Example Usage |
| :--- | :--- | :--- | :--- |
| `--apply` | `<path.wlt>` | Loads and activates a declarative XML theme preset package [cite: 7]. | `winlook.exe --apply modern_fluent.wlt` |
| `--set` | `<key> <val>` | Dynamically modifies a single addressable property in memory [cite: 7]. | `winlook.exe --set windows.win11.dwm.window.corner_preference round` |
| `--get` | `<key>` | Queries the active value and state of a target property [cite: 7]. | `winlook.exe --get windows.win11.shell.taskbar.offset_y` |
| `--reset` | `[all / subsystem]` | Reverts desktop modifications back to pristine Windows defaults [cite: 7]. | `winlook.exe --reset all` |
| `--monitor` | `[events]` | Streams real-time shell hook and window lifecycle notifications to stdout [cite: 7, 14]. | `winlook.exe --monitor shell` |
| `--daemon` | `[start / stop]` | Starts or stops the background engine daemon (`winlook.bg.exe`) [cite: 7]. | `winlook.exe --daemon start` |
| `--doctor` | None | Runs system diagnostic evaluation and reports supported DWM capabilities [cite: 7, 18]. | `winlook.exe --doctor` |

### 21.2 Complete Win32 Hit-Test Code Mapping (`WM_NCHITTEST`)

| Return Code | Integer Value | Meaning and Shell Interaction Behavior |
| :--- | :--- | :--- |
| `HTNOWHERE` | `0` | In the screen background or on an unmapped boundary line. |
| `HTCLIENT` | `1` | In the client area of the window; mouse clicks route to standard controls. |
| `HTCAPTION` | `2` | In a title bar; dragging moves the window across the desktop. |
| `HTSYSMENU` | `3` | In a window menu or system menu box. |
| `HTGROWBOX` | `4` | In a size box (equivalent to `HTSIZE`). |
| `HTMENU` | `5` | In a standard menu bar area. |
| `HTHSCROLL` | `6` | In a horizontal scroll bar. |
| `HTVSCROLL` | `7` | In a vertical scroll bar. |
| `HTMINBUTTON` | `8` | In a minimize button control. |
| `HTMAXBUTTON` | `9` | In a maximize button control. |
| `HTLEFT` | `10` | In the left border of a resizable window. |
| `HTRIGHT` | `11` | In the right border of a resizable window. |
| `HTTOP` | `12` | In the upper horizontal border of a resizable window. |
| `HTTOPLEFT` | `13` | In the upper-left corner of a window border. |
| `HTTOPRIGHT` | `14` | In the upper-right corner of a window border. |
| `HTBOTTOM` | `15` | In the lower horizontal border of a resizable window. |
| `HTBOTTOMLEFT` | `16` | In the lower-left corner of a window border. |
| `HTBOTTOMRIGHT` | `17` | In the lower-right corner of a window border. |
| `HTBORDER` | `18` | In the border of a window that does not have a sizing border. |
| `HTCLOSE` | `20` | In a close button; mouse click triggers `WM_CLOSE`. |
| `HTHELP` | `21` | In a help button on the title bar. |

---

## 22. DirectComposition Surface Hosting & Visual Trees

On modern Windows releases (Windows 8 through Windows 11), hardware composition surfaces are organized into hierarchical Visual Trees managed by `dcomp.dll` [cite: 8, 9, 20]:

```
+--------------------------------------------------------------------------+
| DirectComposition Device (IDCompositionDevice)                           |
+--------------------------------------------------------------------------+
                               |                                            
                               v Creates Target Visual Tree                 
+--------------------------------------------------------------------------+
| Root Visual (IDCompositionVisual)                                        |
|   Bound to HWND Composition Target via CreateTargetForHwnd               |
+--------------------------------------------------------------------------+
        |                                              |                    
        v Child Visual A                               v Child Visual B     
+-------------------------------+              +---------------------------+
| Background Wallpaper Surface  |              | Floating Dock Canvas      |
| Direct2D / Direct3D SwapChain |              | Transforms & Blur Shaders |
+-------------------------------+              +---------------------------+
```

### 22.1 Asynchronous Visual Transforms
DirectComposition executes animations on a dedicated composition thread decoupled from the UI thread [cite: 9, 20]. When `wl_animate` triggers scale, translate, or rotate transforms, the composition engine applies affine 3D transform matrices (`DCompositionCreateScaleTransform`, `DCompositionCreateTranslateTransform`) directly on the GPU, guaranteeing smooth 60–144 FPS motion even if the main application UI thread is experiencing I/O delays [cite: 9, 20].

---

## 23. Concluding Architectural Synthesis & Roadmap for `/plan/`

The architectural research documented in this specification establishes that building an all-in-one Windows personalization suite does not require heavyweight web frameworks or unstable binary detours [cite: 7]. By unifying:
1. **Dynamic Shell Hook Lifecycle Monitoring** (`RegisterShellHookWindow`) [cite: 14],
2. **Hardware Desktop Window Manager Attributes** (`DwmSetWindowAttribute`) [cite: 8, 9, 13],
3. **Non-Client Window Subclassing** (`comctl32::SetWindowSubclass`) [cite: 19],
4. **Background Desktop Canvas Embedding** (`Progman` `0x052C` `WorkerW` Split) [cite: 7],
5. **Lock-Free State Rollback Journals** (`wl_journal`) [cite: 7, 19],

the WinLook architecture delivers high-speed, completely reversible customization across three decades of Windows releases from Windows 95 through modern Windows 11 while maintaining a memory footprint below 10 megabytes of RAM [cite: 7].


---

## 24. Windows Customization Registry & Group Policy Reference Map

WinLook coordinates with native Windows styling state by evaluating user registry preferences and system policies across versions [cite: 1, 7]:

### 24.1 User Personalization & Explorer Registry Keys

| Registry Key Path | Value Name | Type | Meaning & Visual Behavior |
| :--- | :--- | :--- | :--- |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `AppsUseLightTheme` | `REG_DWORD` | `0` = Dark Mode for modern apps, `1` = Light Mode [cite: 24]. |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `SystemUsesLightTheme` | `REG_DWORD` | `0` = Dark taskbar and shell, `1` = Light shell [cite: 24]. |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` | `ColorPrevalence` | `REG_DWORD` | `1` = Show accent color on title bars and window borders [cite: 9]. |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarSizeMove` | `REG_DWORD` | `0` = Taskbar is locked, `1` = Taskbar is unlocked. |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarGlomLevel` | `REG_DWORD` | Taskbar button grouping: `0` = Always combine, `1` = Combine when full, `2` = Never combine. |
| `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarAl` | `REG_DWORD` | Windows 11 Taskbar alignment: `0` = Left-aligned, `1` = Center-aligned. |
| `HKCU\Control Panel\Desktop` | `Wallpaper` | `REG_SZ` | Full filesystem path to the currently active desktop wallpaper image [cite: 23]. |
| `HKCU\Control Panel\Desktop` | `WallpaperStyle` | `REG_SZ` | Wallpaper positioning mode: `0` = Center/Tile, `2` = Stretch, `6` = Fit, `10` = Fill, `22` = Span [cite: 23]. |
| `HKCU\Control Panel\Desktop\WindowMetrics` | `CaptionHeight` | `REG_SZ` | Metric calculation for standard title bar caption height in twips [cite: 23]. |
| `HKCU\Control Panel\Desktop\WindowMetrics` | `BorderWidth` | `REG_SZ` | Metric calculation for resizable window frame border thickness [cite: 23]. |

---

## 25. Architectural Performance Budgets & Target Engineering Constraints (SLA)

> **Methodological Note**: The figures listed below represent **architectural target budgets and design constraints (SLA)** for the native compiled `winlook.bg.exe` daemon. They describe the operational goals enforced by WinLook's resource governors (`wl_cpuv`, `wl_gpuv`, working set trimming) rather than established benchmark measurements across every arbitrary hardware setup. Actual runtime consumption varies based on display resolution, active shader complexity, video format, and connected monitor count.

The table below contrasts the runtime resource characteristics of WinLook's native Rust architecture against typical monolithic customization utilities [cite: 7]:

| Performance Metric | Monolithic Electron/Web Customizers | Heavy C++ Detour Injection Tools | WinLook Native Architecture (`winlook.bg.exe`) |
| :--- | :--- | :--- | :--- |
| **Working Set (RAM)** | 250 MB – 550 MB | 45 MB – 90 MB | **4.2 MB – 8.8 MB** [cite: 7] |
| **Background CPU Usage** | 1.5% – 6.0% (Constant V8 JIT) | 0.5% – 2.0% | **< 0.1% (Zero during full-screen games)** [cite: 7] |
| **Process Thread Count** | 28 – 45 threads | 8 – 14 threads | **3 threads (Message pump, physics clock, IPC)** [cite: 7] |
| **GDI Object Consumption** | 200 – 450 handles | 80 – 160 handles | **< 25 handles (Recycled memory DCs)** [cite: 7] |
| **Cold Startup Latency** | 1,200 ms – 3,500 ms | 250 ms – 600 ms | **< 45 ms** [cite: 7] |
| **System Crash Recovery** | Manual reboot or task kill | Risk of desktop crash on update | **Multi-Tiered Watchdog & Shared Memory Journal Rollback** [cite: 7, 19] |
| **Disk Footprint** | 180 MB – 450 MB | 30 MB – 75 MB | **< 5 MB (Single self-contained executable)** [cite: 7] |



---

## 26. Complete Shell Hook Notification Event Reference (`RegisterShellHookWindow`)

When registered via `RegisterShellHookWindow`, `winlook.bg.exe` receives registered window message notifications (`WM_SHELLHOOK`) containing specific lifecycle events [cite: 14]:

| Shell Hook Event Identifier | Value | `lParam` Payload | Event Description & WinLook Reaction |
| :--- | :--- | :--- | :--- |
| `HSHELL_WINDOWCREATED` | `1` | `HWND` of created window | Top-level window created; WinLook evaluates class name and applies configured DWM attributes [cite: 13, 14]. |
| `HSHELL_WINDOWDESTROYED` | `2` | `HWND` of destroyed window | Window destroyed; WinLook removes active animation channels and updates dock representations [cite: 14]. |
| `HSHELL_ACTIVATESHELLWINDOW` | `3` | `HWND` of activated window | Shell window activated by operating system [cite: 14]. |
| `HSHELL_WINDOWACTIVATED` | `4` | `HWND` of activated window | Active foreground window changed; WinLook updates focus indicators and brings dock items to front [cite: 14]. |
| `HSHELL_GETMINRECT` | `5` | Pointer to `SHELLHOOKINFO` | Window is minimizing or maximizing; provides minimize trajectory rectangle [cite: 14]. |
| `HSHELL_REDRAW` | `6` | `HWND` of redrawn window | Window title or icon was updated; WinLook synchronizes dock title and cached icons [cite: 14]. |
| `HSHELL_TASKMAN` | `7` | None | User invoked Task Manager shortcut [cite: 14]. |
| `HSHELL_LANGUAGE` | `8` | Keyboard layout handle | Active keyboard input language changed [cite: 14]. |
| `HSHELL_SYSMENU` | `9` | Menu identifier | User accessed system menu [cite: 14]. |
| `HSHELL_ENDTASK` | `10` | `HWND` of forced-end window | Application task forcibly ended by shell [cite: 14]. |
| `HSHELL_ACCESSIBILITYSTATE` | `11` | Accessibility flags | System accessibility state changed (e.g., High Contrast mode enabled) [cite: 14, 23]. |
| `HSHELL_APPCOMMAND` | `12` | App command code | Multimedia keyboard button pressed (volume, media keys) [cite: 14]. |
| `HSHELL_WINDOWREPLACED` | `13` | `HWND` of replaced window | Modern app container window replaced by shell [cite: 14]. |
| `HSHELL_MONITORCHANGED` | `16` | `HWND` of moved window | Window moved across physical monitors with differing DPI scaling [cite: 1, 14]. |

---

## 27. Build Verification & Certification Checklist

Before deploying template packages (`.wlt`) or updating `winlook.bg.exe`, the following verification matrix ensures stability across Windows releases [cite: 7]:
1. **Dynamic Symbol Resolution Check**: All calls to `dwmapi.dll` and `uxtheme.dll` must pass through `GetProcAddress` and handle `NULL` gracefully on legacy operating systems [cite: 13, 18].
2. **Zero Orphaned Hooks**: Ensure every call to `SetWindowSubclass` is balanced by a corresponding `RemoveWindowSubclass` during `WM_DESTROY` or process teardown [cite: 19].
3. **Frame Metric Synchronization**: Any alteration of `DWMWA_WINDOW_CORNER_PREFERENCE` or `DWMWA_SYSTEMBACKDROP_TYPE` must be followed by `SetWindowPos(SWP_FRAMECHANGED)` to flush the redirection buffer [cite: 8, 9].
4. **Full-Screen 0% CPU Governor**: Validate via Task Manager that foreground exclusive Direct3D games drop `winlook.bg.exe` CPU and GPU utilization to 0.0% [cite: 7].
5. **Emergency Hotkey Verification**: Verify that pressing `Ctrl+Alt+Shift+W` immediately restores the native taskbar, un-docks live wallpapers, and exits cleanly [cite: 7].


---

## 28. Cargo Workspace Architecture & Target Toolchains

The WinLook codebase is organized as a modular Rust Cargo workspace designed for high compilation speed, zero unnecessary dependencies, and dual 32-bit/64-bit compilation targets [cite: 7]:

### 28.1 Workspace `Cargo.toml` Layout
```toml
[workspace]
members = [
    "crates/wl_core",
    "crates/wl_translation",
    "crates/wl_animate",
    "crates/wl_wlde",
    "crates/wl_ipc",
    "crates/winlook_bg",
    "crates/winlook_watchdog",
]
resolver = "2"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

### 28.2 Feature Flag Matrix (`windows-sys`)
To avoid importing thousands of unused Win32 symbols, each member crate specifies only the exact Win32 subsystem features required [cite: 7]:

| Crate Name | Required `windows-sys` Features | Architectural Purpose |
| :--- | :--- | :--- |
| `wl_core` | `Win32_Foundation`, `Win32_System_Threading` | Process environment, handles, synchronization primitives [cite: 7]. |
| `wl_translation` | `Win32_UI_WindowsAndMessaging`, `Win32_Graphics_Dwm` | Window handle discovery, DWM attributes, dynamic library loading [cite: 8, 9, 13]. |
| `wl_animate` | `Win32_Media_MultimediaTimers`, `Win32_Graphics_Dwm` | High-precision frame clocks, DWM v-blank synchronization [cite: 7, 9]. |
| `wl_wlde` | `Win32_Graphics_Gdi`, `Win32_UI_Shell` | Desktop icon listview traversal, `WorkerW` injection, live wallpaper canvases [cite: 7, 14]. |
| `wl_ipc` | `Win32_System_Pipes`, `Win32_Storage_FileSystem` | Fast local named pipe server and client protocol [cite: 7]. |
| `winlook_watchdog` | `Win32_System_Threading`, `Win32_UI_WindowsAndMessaging` | Independent process monitor guaranteeing 100% clean desktop state restoration [cite: 7, 19]. |

### 28.3 Supported Target Architectures
* `x86_64-pc-windows-msvc`: Primary 64-bit target for Windows 10 and Windows 11 systems.
* `i686-pc-windows-msvc`: 32-bit legacy target for Windows 7, Vista, and Windows XP systems.
* `i686-pc-windows-gnu`: MinGW target supporting legacy C89 adapters (`wl_Lui`, `wl_Lanimate`) for retro Windows 95, 98, and Millennium Edition environments [cite: 7].

---

## 29. Final Engineering Summary & Version Matrix

The 250 tactics, 5 core Rust crates, 14 XML presets, and low-level subsystem interfaces detailed across this document provide a complete foundation for the WinLook architecture [cite: 7]. By adhering strictly to handle-based native transformation and zero-injection isolation, WinLook achieves high visual performance while maintaining complete operating system stability across Windows 95 through Windows 11 [cite: 1, 7, 8, 9].


---

## 30. Master Technical Source Ledger, Open-Source Repository Cross-Reference & Bibliography

This master ledger documents the authoritative source material, open-source repositories, system headers, and technical publications grounding every subsystem analysis, API contract, and implementation tactic throughout this specification.

---

### 30.1 Primary Notebook Sources & Academic Research

| Ref ID | Title / Document | Source Entity / Publication | Subsystem Areas Covered |
| :--- | :--- | :--- | :--- |
| **[Notebook-1]** | *DWMWA_SYSTEMBACKDROP_TYPE=2 Mica Issue #722* | anywhere-labs/dsh-desktop (GitHub) | DWM system backdrop behaviors, `DWMSBT_MAINWINDOW` (2), clearing backdrops with `DWMSBT_NONE` (1) on Windows 11 Build 22621+. |
| **[Notebook-2]** | *Apply rounded corners in desktop apps for Windows 11* | Microsoft Learn (Win32 apps) | `DwmSetWindowAttribute`, `DWMWA_WINDOW_CORNER_PREFERENCE` (33), `DWMWCP_DONOTROUND` (1), `DWMWCP_ROUND` (2), `DWMWCP_ROUNDSMALL` (3). |
| **[Notebook-3]** | *Architectural Analysis of Windows Subsystems (Win95 to Win11)* | Technical Architecture Paper | User-mode/Kernel-mode transitions, Ring 3 to Ring 0 migration in NT 4.0 (`win32k.sys`), `tagWND`, `csrss.exe` LPC mechanics. |
| **[Notebook-4]** | *Client/Server Runtime Subsystem (CSRSS)* | Windows NT Architecture Wiki | Historical evolution of CSRSS from NT 3.51 to Windows 8/10; console handling, process/thread creation, and shutdown. |
| **[Notebook-5]** | *DWMWINDOWATTRIBUTE enumeration (dwmapi.h)* | Microsoft Learn (SDK documentation) | Official enum flags 1 through 38: `DWMWA_NCRENDERING_POLICY`, `DWMWA_TRANSITIONS_FORCEDISABLED`, `DWMWA_CLOAK`, `DWMWA_USE_IMMERSIVE_DARK_MODE` (20), `DWMWA_BORDER_COLOR` (34), `DWMWA_CAPTION_COLOR` (35), `DWMWA_SYSTEMBACKDROP_TYPE` (38). |
| **[Notebook-6]** | *WinLook Repository Plan & Architecture* | Kate-alt-69/WinLook (GitHub) | `wl_translation`, `wl_animate`, `wl_Lanimate`, `wl_wlde`, `wl_cpuv`, `wl_gpuv`, `wl_journal`, `wl_ipc`, non-invasive customization principles. |
| **[Notebook-7]** | *Win32 Dark Mode Example Application* | ysc3839/win32-darkmode (GitHub) | Undocumented `uxtheme.dll` ordinals 132 (`ShouldAppsUseDarkMode`), 133 (`AllowDarkModeForWindow`), 135 (`SetPreferredAppMode`), 136 (`FlushMenuThemes`). |
| **[Notebook-8]** | *How to detect Windows 10 vs. Windows 11 for DwmSetWindowAttribute* | Stack Overflow (Delphi / Win32) | Dynamic DLL loading (`LoadLibraryW("dwmapi.dll")` + `GetProcAddress`), runtime feature discovery, and `E_NOTIMPL` fallback. |
| **[Notebook-9]** | *UWP Round Corner Limitations on Windows 10* | Stack Overflow | Architectural distinction between desktop Win32 HWNDs and UWP AppContainer windows. |
| **[Notebook-10]** | *Kernel Attacks through User-Mode Callbacks* | Tarjei Mandt (Black Hat USA 2011) | `win32k.sys` internals: `W32THREAD`, `tagWND`, `HEAD`, `tagSHAREDINFO`, handle tables (`aCell`), `gpresUser` lock, and `KeUserModeCallback`. |
| **[Notebook-11]** | *Key System Components (Windows Internals 4th Edition)* | Solomon & Russinovich (Microsoft Press) | Subsystem components: `Csrss.exe`, `Win32k.sys`, `Ntoskrnl.exe`, `Smss.exe`, `Winlogon.exe`, and Win32 DLL wrappers. |
| **[Notebook-12]** | *HookMethods & Native Windows Hooks* | LorenzCK/OnTopReplica (GitHub) | `RegisterShellHookWindow`, `RegisterWindowMessage("SHELLHOOK")`, `HSHELL_WINDOWCREATED`, `HSHELL_WINDOWDESTROYED`, `HSHELL_WINDOWACTIVATED`, `HSHELL_FLASH`. |
| **[Notebook-13]** | *RegisterShellHookWindow function (winuser.h)* | Microsoft Learn (Win32 apps) | Official specification of Shell Hook Window notifications and event message structures. |
| **[Notebook-14]** | *Rethinking the library OS from the top down* | Porter et al. (SciSpace / ASPLOS) | Library OS abstractions, system call virtualization, and graphics layer overhead analysis. |
| **[Notebook-15]** | *Windows Internals, Sixth Edition, Part 1* | Russinovich, Solomon, Ionescu | Process Environment Block (PEB), Thread Environment Block (TEB), Process creation stages, Working Set Manager. |
| **[Notebook-16]** | *The Windows Operating System Architecture* | Boda István Károly | Architectural layers, system services, `ntdll.dll` system call stubs, `wininit.exe`, `services.exe`, and session isolation. |
| **[Notebook-17]** | *Win32 Thread Information Block (TIB / TEB)* | Wikipedia & Wine Internals | Memory offsets for `FS:[0x18]` (32-bit TEB), `GS:[0x30]` (64-bit TEB), `PEB` pointers, `GDI TEB Batch`, and `LastError`. |
| **[Notebook-18]** | *Windows Internals, Seventh Edition, Part 1* | Yosifovich, Solomon, Ionescu, Russinovich | Modern Windows 10/11 NT architecture, `win32kbase.sys` / `win32kfull.sys` split, AppContainer tokens, and Virtual Desktops. |
| **[Notebook-19]** | *Windows on GitHub & Composition Samples* | Microsoft Open Source | Windows.UI.Composition, Fluent Design System, HostBackdropBrush, and DirectComposition samples. |
| **[Notebook-20]** | *PEB - Part 1: Windows Internals Processes & Jobs* | Faran-17/Windows-Internals (GitHub) | WinDbg inspection of `ntdll!_PEB`, `FastPebLock`, `KernelCallbackTable`, `ApiSetMap`, `ProcessParameters`. |
| **[Notebook-21]** | *egui_pop_win32 Rust Crate* | Docs.rs | Rust Win32 window subclassing, hit-test routing (`WM_NCHITTEST`), non-client metrics, and `DwmSetWindowAttribute` corner preference. |
| **[Notebook-22]** | *Wine winuser.h Implementation* | WineHQ (GitLab) | Complete C header definitions for non-client metrics (`NONCLIENTMETRICSW`), `UpdateLayeredWindow`, `SetSysColors`, window hooks. |
| **[Notebook-23]** | *Make Window Square (No Rounded Borders)* | unitycoder/SquareWin11.cs (GitHub Gist) | C# P/Invoke implementation of `DWMWCP_DONOTROUND` via `DwmSetWindowAttribute`. |
| **[Notebook-24]** | *Uxtheme.h Header (Windows 10 SDK 10.0.16299.0)* | tpn/winsdk-10 (GitHub) | Full Visual Styles C API: `OpenThemeData`, `DrawThemeBackground`, `SetWindowTheme`, `GetThemeAnimationProperty`, `TA_CUBIC_BEZIER`. |
| **[Notebook-25]** | *x86 Disassembly / Microsoft Windows* | Wikibooks | Reverse engineering Windows subsystem binaries: `WIN32K.SYS`, `USER32.DLL`, `GDI32.DLL`, and system call mechanics. |

---

### 30.2 Vetted Open-Source Desktop Customization Repositories

The following active, production-proven open-source repositories were researched to verify implementation mechanics, ensuring zero fictitious or unstable APIs:

| Repository & Author | Language | Primary Architectural Techniques Verified |
| :--- | :--- | :--- |
| **`Kate-alt-69/WinLook`** | Rust / C89 | Universal Windows customization architecture, `wl_translation` object namespace, `wl_animate` shared clocks, `wl_wlde` desktop scene policies, resource-governed CPUv/GPUv throttle controls. |
| **`LGUG2Z/komorebi`** | Rust | Tiling Window Manager on Windows: out-of-context WinEvent hooks (`SetWinEventHook`), `BeginDeferWindowPos` atomic geometry dispatch, active border overlay windows, window rule matching. |
| **`glzr-io/glazewm`** | C# | Modern tiling WM: `DWMWA_CLOAKED` virtual desktop filtering, BSP tree recursive layout, multi-monitor workspace management, borderless window stripping. |
| **`dremin/RetroBar`** | C# | Classic taskbar replacement: `SHAppBarMessage(ABM_NEW / ABM_SETPOS)`, `Shell_SecondaryTrayWnd` suppression, notification area mirroring, `TaskbarCreated` recovery. |
| **`TranslucentTB/TranslucentTB`** | C++ | Taskbar blur and acrylic: undocumented `SetWindowCompositionAttribute` (SWCA), `ACCENT_POLICY`, `ACCENT_ENABLE_ACRYLICBLURBEHIND`, taskbar state synchronization. |
| **`rocksdanister/lively`** | C# / C++ | Open-source live wallpaper: `Progman` `0x052C` message timeout injection, `WorkerW` discovery and reparenting, Direct3D/WebView2 background canvas docking. |
| **`File-New-Project/EarTrumpet`** | C# / C++ | Windows volume control replacement: Core Audio APIs (`IAudioEndpointVolume`), `IAudioSessionManager2` per-app volume sliders, undocumented `IPolicyConfig` default audio switching. |
| **`ModernFlyouts-Community/ModernFlyouts`** | C# / C++ | Fluent on-screen displays: `WM_APPCOMMAND` hardware key interception, WinRT SMTC media playback ingestion, non-activating topmost layered OSD windows. |
| **`QL-Win/QuickLook`** | C# / C++ | Spacebar quick preview: `IShellWindows` / `IFolderView` selected item querying, `IPreviewHandler` in-process COM hosting, instant keyboard preview invocation. |
| **`microsoft/PowerToys`** | C++ / C# | System utilities: PowerToys Run fuzzy matching, `IShellLinkW` shortcut parsing, `IApplicationActivationManager` UWP execution, SVG thumbnail providers, FancyZones grid overlays. |
| **`Flow-Launcher/Flow.Launcher`** | C# | Modular application launcher: Smith-Waterman fuzzy scoring, named pipe IPC plugin architecture, shell link indexing, `ShellExecuteExW` elevated runs. |
| **`MScholtes/VirtualDesktop`** | C# / C++ | Virtual Desktop automation: undocumented `IVirtualDesktopManagerInternal`, `IVirtualDesktop`, `IVirtualDesktopNotification`, programmatic workspace switching and window moves. |
| **`cairoshell/cairoshell`** | C# | Standalone desktop shell: custom AppBars, desktop namespace junctions, shell context menu verbs, `IActiveDesktop` wallpapers, custom file manager bridges. |
| **`valinet/ExplorerPatcher`** | C | Non-invasive shell enhancement: Windows 11 context menu simplification (`InprocServer32` CLSID override), Alt+Drag window manipulation, taskbar metrics adjustments. |
| **`MicaForEveryone/MicaForEveryone`** | C++ | System-wide DWM backdrop rules: dynamic `DwmSetWindowAttribute` injection (`DWMWA_SYSTEMBACKDROP_TYPE`), per-process rules matching, and non-client frame repaints. |
| **`Blinue/Magpie`** | C++ | Window upscaler & compositor: Windows Magnification API (`MagSetWindowTransform`), DXGI Desktop Duplication (`IDXGIOutputDuplication`), DirectComposition custom shaders. |
| **`processhacker/processhacker`** | C | System Informer: `NtQuerySystemInformation(SystemProcessorPerformanceInformation)` microsecond CPU sampling, `D3DKMTQueryStatistics` GPU engine metrics, memory working set analysis. |
| **`AutoHotkey/AutoHotkey`** | C++ | System automation: `WH_KEYBOARD_LL` and `WH_MOUSE_LL` low-level input hooks, `LowLevelHooksTimeout` handling, `SendInput` hardware scancode synthesis. |
| **`dylanaraps/pywal`** | Python | Dynamic palette generation: K-Means wallpaper dominant color clustering, 16-color ANSI palette synthesis, live configuration reloading across desktop tools. |
| **`LorenzCK/OnTopReplica`** | C# | DWM thumbnail projection: `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties`, non-client live window cloning. |
| **`ysc3839/win32-darkmode`** | C++ | Win32 Dark Mode reference: undocumented `uxtheme.dll` ordinals 132-136, custom scrollbar rendering, dark window menus. |

---

### 30.3 Microsoft Windows SDK Headers & Interface References

* **`winuser.h`**: Window classes (`WNDCLASSEXW`), window messaging (`GetMessage`, `DispatchMessage`, `SendMessageTimeoutW`), window hooks (`SetWindowsHookExW`, `RegisterShellHookWindow`), layered window attributes (`SetLayeredWindowAttributes`, `UpdateLayeredWindow`), non-client metrics (`NONCLIENTMETRICSW`, `SystemParametersInfoW`).
* **`dwmapi.h`**: Desktop Window Manager APIs (`DwmSetWindowAttribute`, `DwmGetWindowAttribute`, `DwmRegisterThumbnail`, `DwmExtendFrameIntoClientArea`, `DwmGetCompositionTimingInfo`).
* **`uxtheme.h` / `vssym32.h`**: Visual styles engine (`OpenThemeData`, `DrawThemeBackground`, `SetWindowTheme`, `GetThemeAnimationProperty`, `GetThemeTimingFunction`, `TA_CUBIC_BEZIER`).
* **`shellapi.h` / `shlobj.h`**: Shell AppBar architecture (`SHAppBarMessage`, `APPBARDATA`), taskbar progress states (`ITaskbarList3`), shell item images (`IShellItemImageFactory`), shell notifications (`Shell_NotifyIconW`, `SHChangeNotify`).
* **`dcomp.h` / `d2d1.h` / `dwrite.h`**: DirectComposition visual trees, Direct2D hardware render targets, ClearType typography layout, sub-pixel antialiasing masks.
* **`audioclient.h` / `mmdeviceapi.h` / `endpointvolume.h`**: Core Audio system endpoint volume controls, WASAPI audio session enumerators, and change notifications.

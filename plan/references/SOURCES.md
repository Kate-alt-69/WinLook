# WinLook Technical Source Ledger

Last verified: 2026-11-13. These citations support platform or upstream-project claims; they do not prove that WinLook code has been implemented or tested.

## Rust / UI runtime / hosted toolchains

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| RUST-001 | [Rust MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html) | Standard `*-pc-windows-msvc` target requires Windows 10+ client installs | Does not mean every Win10 API is available; verify imported APIs independently |
| RUST-002 | [Rust Windows 7 MSVC target](https://doc.rust-lang.org/rustc/platform-support/win7-windows-msvc.html) | Win7 MSVC triples exist as Tier 3 and are not shipped as prebuilt rustup components | Requires custom Rust build / build-std; not an automatic stable installation |
| TAURI-001 | [Tauri 2.12 announcement](https://v2.tauri.app/blog/tauri-2.12/) | Tauri 2.12 raises MSRV to 1.90 and drops official Windows 7 support | Use a different backend for the Win7 baseline |
| TAURI-002 | [Tauri WebView versions](https://tauri.app/reference/webview-versions/) | Tauri uses platform WebViews, including system WebView2 on Windows | Runtime/system package requirements remain platform-dependent |
| BUILD-011 | [GitHub Actions runner images](https://github.com/actions/runner-images) | `windows-2022` is the VS 2022 image; `windows-2025-vs2026` is the VS 2026 image | Image contents change; validate exact VS/SDK components in the workflow |
| BUILD-012 | [Windows 2022 hosted image inventory](https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md) | VS 2022 hosted image includes Windows SDK 10.0.19041 and matching compiler tools | Current image inventory checked 2026-11-13; versions can change |
| BUILD-013 | [Windows 2025 VS 2026 hosted image inventory](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-VS2026-Readme.md) | Dedicated hosted image provides VS 2026 and Windows 11 SDK 26100 | Current image inventory checked 2026-11-13; versions can change |

## Microsoft Windows / Win32

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| MSAA-001 | [Active Accessibility supported platforms](https://learn.microsoft.com/en-us/windows/win32/winauto/supported-platforms) | MSAA versions included with Win95/98/Me/NT4/2000/XP/Vista/7; Win95 requires 1.3 RDK | Does not prove every WinEvent API exists in each base OS |
| WIN-001 | [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook) | Minimum client Windows 2000; Active Accessibility 1.3 RDK for Windows 95 and NT4 SP6+ | RDK route requires separate installation/testing |
| WIN-002 | [AnimateWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-animatewindow) | Show/hide effects; minimum client Windows 2000 | Not available as a Win95 dependency |
| DWM-001 | [DwmIsCompositionEnabled](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmiscompositionenabled) | DWM API minimum Vista; query composition state | Windows 8+ always on; manifest compatibility affects return behaviour |
| DWM-002 | [Windows 8 DWM always on](https://learn.microsoft.com/en-us/windows/compatibility/desktop-window-manager-is-always-on) | DWM cannot be disabled by apps/users in Windows 8; composition is core | Windows 8 platform-specific documentation |
| DCOMP-001 | [IDCompositionDesktopDevice](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nn-dcomp-idcompositiondesktopdevice) | DirectComposition Win32 desktop interface; minimum Windows 8.1 | Does not imply this interface exists on Windows 8 |
| VDESK-001 | [IVirtualDesktopManager](https://learn.microsoft.com/en-us/windows/desktop/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager) | Windows 10 interface for window virtual desktop interaction | Not a complete create/switch API |
| VDESK-002 | [MoveWindowToDesktop](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ivirtualdesktopmanager-movewindowtodesktop) | Moving top-level HWND to a desktop; minimum Windows 10 | Method-specific contract |
| MIT-001 | [PROCESS_MITIGATION_DYNAMIC_CODE_POLICY](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy) | ProhibitDynamicCode prevents dynamic code generation or executable-code modification; minimum Windows 8.1 | One mitigation type; query actual target |
| SVC-001 | [Service changes for Windows Vista](https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista) | Session 0 reserved for services/non-interactive processes from Vista | Critical to BGW/Service/UI placement |
| WTS-001 | [WTSRegisterSessionNotification](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification) | Registers a window for session-change notifications delivered as WM_WTSSESSION_CHANGE | Minimum client Vista; unregister before destroying the registered window |
| WTS-002 | [WM_WTSSESSION_CHANGE](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession_change) | Message delivered to windows registered for session-change events | Not a universal Win95-to-Win11 event source |
| SESSION-001 | [Desktops](https://learn.microsoft.com/en-us/windows/desktop/winstation/desktops) | Default, ScreenSaver and Winlogon desktop roles; Winlogon desktop access is restricted | Security boundary reference; does not provide WinLook access |
| SHELL-001 | [Windows Shell interfaces](https://learn.microsoft.com/en-us/windows/win32/shell/interfaces) | Overview of Shell COM interfaces for file/desktop integration | Each interface has its own minimum OS and behaviour |
| SHELL-002 | [Shell links and shortcuts](https://learn.microsoft.com/en-us/windows/win32/shell/links) | Shell link model and shortcut integration | Does not imply a universal running-taskbar icon replacement API |
| DWM-003 | [DWM overview](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview) | DWM compositor architecture | Vista+; retain fallback on older systems |
| DWM-004 | [DWM window attributes](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute) | Supported DWM window attributes | Attribute support differs by Windows release |

## Audio and media

| ID | Source | Claim supported | Scope / limit |
|---|---|---|---|
| AUDIO-001 | [IAudioSessionManager](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager) | Core Audio session manager baseline | Vista minimum; distinct from IAudioSessionManager2 |
| AUDIO-002 | [IAudioSessionManager2](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager2) | Audio-session enumeration and notifications | Windows 7 minimum |
| AUDIO-003 | [ISimpleAudioVolume](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-isimpleaudiovolume) | Shared-mode audio-session volume and mute | Does not control exclusive-mode streams |
| AUDIO-004 | [IAudioEndpointVolume](https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume) | Audio endpoint master volume/mute | Vista minimum |
| AUDIO-005 | [IAudioSessionManager2::GetSessionEnumerator](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nf-audiopolicy-iaudiosessionmanager2-getsessionenumerator) | Creates a session enumerator for the current audio session manager | The enumerator may miss sessions created around enumeration time; maintain the live list using session notifications |
| MEDIA-001 | [GSMTC session manager](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager) | System media-session discovery and change events | Windows 10 version 1809 / build 17763 |
| MEDIA-002 | [GSMTC session](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssession) | Source app ID, transport requests, timeline and events | Provider may omit metadata or decline commands |
| MEDIA-003 | [GSMTC media properties](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmediaproperties) | Audio/video type, metadata and provider thumbnail | Source-session supplied; completeness is not guaranteed |
| MEDIA-004 | [GSMTC timeline properties](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessiontimelineproperties) | Position and seek timeline information | Use only when exposed by the session |
| MEDIA-005 | [GSMTC session manager](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager) | Access to the system media-session manager | Declares the `globalMediaControl` capability requirement; Windows 10 version 1809+ |

## File Explorer and Shell customization

| ID | Source | Claim supported | Scope / limit |
|---|---|---|---|
| SHELL-003 | [Shell namespace extensions](https://learn.microsoft.com/en-us/windows/win32/shell/nse-works) | Namespace extension provides its own virtual folder; its view is highly customizable while tree-view control is limited | Not a global Explorer skin |
| SHELL-004 | [Implementing a folder view](https://learn.microsoft.com/en-us/windows/win32/lwef/nse-folderview) | IShellView supplies a child view and can contribute folder-specific menu/toolbar/status behaviour | Custom view trade-offs versus default DefView must be evaluated |
| SHELL-005 | [IShellView](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ishellview) | Namespace folder-view interface | Windows XP minimum |
| SHELL-006 | [File Explorer context-menu integration](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer) | IExplorerCommand registration and modern context-menu requirements | Modern menu registration has package identity/manifest requirements |
| SHELL-007 | [Windows 11 context-menu developer guidance](https://blogs.windows.com/windowsdeveloper/2021/07/19/extending-the-context-menu-and-share-dialog-in-windows-11/) | Describes Windows 11 modern context-menu grouping and developer extension direction | Not a universal menu chrome customization API |
| SHELL-008 | [Windhawk File Explorer Styler](https://windhawksoftware.com/mods/windows-11-file-explorer-styler/) | Third-party demonstration of selected Explorer surface styling | Feasibility example only; not a Microsoft API or universal support guarantee |
| SHELL-009 | [IExplorerCommand](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-iexplorercommand) | Explorer command-extension interface | Documented minimum client is Windows Vista; API minimum is separate from Windows 11 package-identity/manifest registration |

## Language and Windows target support

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| TOOLCHAIN-001 | [Rust MSVC target support](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html) | Official Rust/MSVC Windows target support floor | Windows 10+ for client installs; this is toolchain documentation, not a WinLook runtime test |
| TOOLCHAIN-002 | [Rust GNU target support](https://doc.rust-lang.org/rustc/platform-support/windows-gnu.html) | Current documentation state for Rust GNU Windows targets | Page is a stub with no target maintainers and no supported old-client minimum; not evidence of pre-Windows-10 support |


| CPU-001 | [Windows thread-pool creation](https://learn.microsoft.com/en-us/windows/win32/api/threadpoolapiset/nf-threadpoolapiset-createthreadpool) | A Windows thread pool can be configured with explicit minimum/maximum thread counts and has its own lifecycle | Windows Vista+; actual scheduling remains owned by Windows |
| CPU-002 | [Job Object CPU rate control](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information) | Job Objects expose weight-based, hard-cap and min/max CPU-rate control modes | Hard-cap can stop all threads in a job for the rest of a scheduling interval; use only for WinLook-owned helper jobs, not as a default UI throttle |
| GPU-001 | [DXGI_QUERY_VIDEO_MEMORY_INFO](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info) | OS reports per-process GPU memory budget, usage, available reservation and current reservation | The budget is the target; exceeding it can cause stutter/eviction pressure |
| GPU-002 | [IDXGIAdapter3::QueryVideoMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo) | Budget query is per node and local/non-local memory segment | Adapter/node/segment identity must remain separate |
| GPU-003 | [IDXGIAdapter3::SetVideoMemoryReservation](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-setvideomemoryreservation) | Reservation communicates an application's minimum memory need to the OS | A hint, not a guaranteed private VRAM partition or an increased budget |
| GPU-004 | [Video-memory budget-change notification](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-registervideomemorybudgetchangenotificationevent) | DXGI can signal a CPU event when the video-memory budget changes | Unregister notification handles during adapter/device shutdown |
| GPU-005 | [D3D12 residency](https://learn.microsoft.com/en-us/windows/win32/direct3d12/residency) | DXGI reservation is a minimum-working-set hint and does not increase the application's normal budget | Reservation may not be available when the app is not foreground; still handle budget and residency changes |

## Vector graphics and SVG

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| SVG-001 | [SVG 2 Paths](https://www.w3.org/TR/SVG/paths.html) | Path geometry, subpaths, current-point behavior and the path-data grammar | Parser must implement repeated groups, relative coordinates, arcs and specified error behavior |
| SVG-002 | [SVG 2 Rendering Model](https://www.w3.org/TR/SVG/render.html) | Overall/group opacity is distinct from fill-opacity, stroke-opacity and stop-opacity | Group opacity must be applied at the group compositing boundary |
| SVG-003 | [Direct2D SVG support](https://learn.microsoft.com/en-us/windows/win32/direct2d/svg-support) | Windows 10 Creators Update and newer Direct2D can render a documented subset of SVG 1.1 elements and presentation attributes | Not full SVG; unsupported elements/attributes are ignored by Direct2D, so WinLook needs its own feature report and fallback policy |
| VECTOR-001 | [Direct2D geometries overview](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-geometries-overview) | Direct2D provides path, primitive, composite and transformed geometry; geometry can be reused and used for hit/clip regions | Device-independent geometry must remain separate from device-dependent renderer resources |
| COMPOSITION-001 | [DirectComposition best practices](https://learn.microsoft.com/en-us/windows/win32/directcomp/best-practices-for-directcomposition) | Device loss requires validation/recovery; create only needed visuals and remove hidden visuals instead of setting opacity to zero | Microsoft recommends Windows.UI.Composition instead of DirectComposition for Windows 10 app composition; validate the chosen Windows 10 path |

## Legacy toolchain

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| OW-001 | [Open Watcom Linker Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/lguide.html) | Distinct Win95 x86 and NT/Win32 targets | Toolchain target availability is not runtime proof |

## Third-party project references

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WINDHAWK-001 | [Windhawk injection targets](https://github.com/ramensoftware/windhawk/wiki/Injection-targets-and-critical-system-processes) | Injection targeting, early-load rationale, exclusion lists and compatibility tradeoffs | Design reference only; do not copy its defaults without review |
| WINDHAWK-002 | [Windhawk repository](https://github.com/ramensoftware/windhawk) | Upstream project source | Pin specific tag/commit before relying on implementation internals |
| HYPR-001 | [Hyprland](https://hypr.land/) | Window manager/compositor project identity | Inspiration only, not Windows compatibility evidence |
| HYPR-002 | [Hyprland window rules](https://wiki.hypr.land/Configuring/Basics/Window-Rules/) | Window rule concepts | Do not copy syntax as WinLook contract |
| HYPR-003 | [Hyprland workspace rules](https://wiki.hypr.land/Configuring/Workspace-Rules/) | Workspace rule concepts | Design reference only |
| RAIN-001 | [Rainmeter documentation](https://docs.rainmeter.net/) | Rainmeter configuration/runtime concepts | Parsing .rmskin is not full runtime compatibility |
| RAIN-002 | [Rainmeter source](https://github.com/rainmeter/rainmeter) | Upstream implementation | GPL-2.0 project; respect project and asset licensing; pin commit when citing internals |
| RAIN-003 | [Distributing Skins (.rmskin)](https://docs.rainmeter.net/manual/distributing-skins/) | .rmskin is a modified ZIP with package metadata/integrity data; renaming a ZIP is not equivalent | Rainmeter package specification; inspect version-specific metadata |
| RAIN-004 | [Installing Skins](https://docs.rainmeter.net/manual/installing-skins/) | Skin packages may contain skins, layouts and architecture-specific plugins | Does not imply WinLook implements the Rainmeter runtime |

## Ledger rules

For every new source record its exact claim, publisher, affected version, last-verified date and confidence. Separate documentation-verified facts from runtime tests. Pin a commit/tag when describing upstream internals. Do not promote an inferred behaviour into fact without evidence.

## Open research

Win9x toolchain/runtime combinations; safe desktop icon suppression/recovery; process creation/injection compatibility; legacy display APIs; desktop wallpaper placement without Explorer modification; modern DWM/DirectComposition differences; WLWM workspace APIs beyond the public IVirtualDesktopManager interface; OS-specific lock/session event mechanisms for pre-Vista systems; exact Rainmeter package metadata and runtime emulation scope.

## UI design references

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| UI-001 | [WinToys screenshots](https://freesoft-100.com/screen/ss-wintoys.html) | Example of a Windows utility organized around a persistent navigation menu and focused category pages | Visual/information-architecture reference only; not a source to copy and not evidence of WinLook implementation |
| UI-002 | [WinToys feature overview and screenshots](https://www.computerbild.de/artikel/Tipps-Windows-Wintoys-Kostenloses-Tuning-Tool-fuer-Windows-11-und-10-045b07f-35673881.html) | Illustrates grouped Apps, Services, Performance, Health and Optimization areas in a single utility | Third-party overview; UI direction reference only |
| UI-003 | [Microsoft Windows app design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/) | Windows guidance for navigation, commands, content hierarchy, spacing and control grouping | General design guidance; does not select WinLook's UI framework or guarantee legacy API availability |



## Modern WinUI / Rust UI research

| ID | Source | Claim supported | Scope / limit |
|---|---|---|---|
| UI-001 | [WinUI 3 overview](https://learn.microsoft.com/en-us/windows/apps/winui/winui3/) | WinUI 3 is Microsoft's native desktop UI framework with Fluent controls and XAML programming model | Windows App SDK; not a Win95 option; minimum OS is documented separately |
| UI-002 | [Windows developer FAQ](https://learn.microsoft.com/en-au/windows/apps/get-started/windows-developer-faq) | Windows App SDK supports desktop apps down to Windows 10 version 1809 (build 17763); Microsoft recommends WinUI 3 for new general-purpose Windows desktop apps | Exact API support still varies by OS/build; does not certify WinLook performance |
| UI-003 | [windows-reactor documentation](https://github.com/microsoft/windows-rs/blob/master/docs/crates/windows-reactor.md) | Rust library backed by native WinUI 3; declarative components/state; docs cover NavigationView, controls, UI Automation modifiers, virtualization, dialogs and composition | Upstream library; API and release versions must be pinned and tested |
| UI-004 | [windows-reactor README](https://github.com/microsoft/windows-rs/blob/master/crates/libs/reactor/readme.md) | Typed Rust component/message model and native WinUI controls | Upstream example, not WinLook code |
| UI-005 | [windows-reactor-setup guide](https://github.com/microsoft/windows-rs/blob/master/docs/crates/windows-reactor-setup.md) | Framework-dependent and self-contained Windows App SDK deployment options; setup helper is a build dependency for self-contained staging | Packaging choice affects install prerequisites and output size; measure before release |
| UI-006 | [Rust for Windows August 2026 release notes](https://github.com/microsoft/windows-rs/issues/4867) | Windows Reactor stable 0.100 release family uses Rust 1.95 MSRV and initially targets Windows App SDK 2.4.0 | Stable release facts; this is not the preview API used by the current prototype |
| UI-008 | [windows-reactor-pre 0.107.1](https://docs.rs/windows-reactor-pre/0.107.1/windows_reactor/) | Preview API used by the current WinLook shell; exposes the newer NavigationView/TitleBar/component APIs | Explicitly preview; public APIs may change. Keep the dependency behind wl_ui and pin exactly |
| UI-007 | [windows-rs dependency pins](https://github.com/microsoft/windows-rs/blob/master/docs/dependencies.md) | Current upstream development metadata/runtime pin is Windows App SDK 2.5.1 | Current upstream source may be newer than the published 0.100.0 crate; pin matching crate/setup/runtime versions together |

Implementation consequence: prototype Win10/Win11 Manager UI with the published Windows Reactor release family, but treat cold start, idle CPU, private bytes, navigation latency, accessibility and low-end hardware behaviour as WinLook-specific acceptance tests. The Win95 C UI remains a separate native compatibility implementation.

## Build, workflow and security tooling research — 2026-11-11

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| BUILD-001 | [Open Watcom 2.0 C/C++ Getting Started](https://open-watcom.github.io/open-watcom-v2-wikidocs/c_readme.html) | Open Watcom 2.0 supports 32-bit Windows 95/98/Me targets and Linux host development | Establishes target availability, not WinLook compatibility or an unattended installer CLI |
| BUILD-002 | [Open Watcom C/C++ User Guide](https://open-watcom.github.io/open-watcom-v2-wikidocs/cguide.html) | WCC386 is the 32-bit x86 compiler; command-line target options and compiler invocation are documented | Exact WinLook Makefile/compiler flags still need a tested project |
| BUILD-003 | [Official Open Watcom 2026-11-01 release assets](https://github.com/open-watcom/open-watcom-v2/releases/tag/2026-11-01-Build) | Official Linux x64 installer asset and release SHA-256 metadata are published | WinLook pins the asset digest. The public guide does not establish the -s/--install-dir flags; the Python runner does not execute guessed options |
| BUILD-004 | [actionlint configuration](https://github.com/rhysd/actionlint/blob/main/docs/config.md) | Custom self-hosted runner labels are configured in .github/actionlint.yaml under self-hosted-runner.labels | The workflow branch declares only the WinLook-specific open-watcom label |
| BUILD-005 | [CodeQL supported languages](https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/) | CodeQL supports C/C++, Rust, Python and GitHub Actions workflow analysis | Source scanning is not a substitute for compilation, tests, fuzzing or runtime validation |
| BUILD-006 | [CodeQL build modes for compiled languages](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages) | C/C++ and Rust support build-mode none without a complete project build | Useful for early source scanning; completeness can differ from a full extracted build |
| BUILD-007 | [Gitleaks usage](https://github.com/gitleaks/gitleaks/blob/master/README.md) | Current v8 command surface uses git, dir and stdin scanning modes; git scans repository history | WinLook scans full reachable history with redacted results and a pinned binary |
| BUILD-008 | [Gitleaks v8.30.1 release](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) | Official release asset and SHA-256 digest are available | WinLook verifies the Linux x64 archive digest before extraction |
| BUILD-009 | [actions/checkout v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1), [setup-python v7.1.0](https://github.com/actions/setup-python/releases/tag/v7.1.0), [cache v6.1.0](https://github.com/actions/cache/releases/tag/v6.1.0), [upload-artifact v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2), [download-artifact v8.0.2](https://github.com/actions/download-artifact/releases/tag/v8.0.2) | Current official GitHub Actions release versions used by this workflow branch, checked 2026-11-11 | Each use is pinned to its resolved commit SHA in the YAML; hosted runner compatibility still applies |
| BUILD-010 | [CodeQL Action v4.39.3](https://github.com/github/codeql-action/releases/tag/v4.39.3), [Dependency Review v5.0.0](https://github.com/actions/dependency-review-action/releases/tag/v5.0.0) | Current security action versions used by the branch, checked 2026-11-11 | Pinned commit SHAs are recorded in security.yml; CodeQL build-mode none is source analysis, not a compiled build |

Implementation consequence: C and Rust share the build/ root but keep outputs and compiler caches distinct. Cargo outputs use build/rust/, Cargo's registry/Git and optional sccache use build/.rcache/, CMake outputs use build/c/, and ccache uses build/.Ccache/. This directory arrangement is a WinLook repository contract, not a requirement imposed by the upstream tools.


## UI theme and styling implementation

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| UI-GDI-001 | [CreateSolidBrush](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createsolidbrush) | Creates native solid brushes for Win32 control/window surfaces; brush resources are released with DeleteObject | GDI; the API page currently lists Windows 2000 Professional as the minimum supported client; older OS runtime compatibility must be validated separately |
| UI-GDI-002 | [CreateFontW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createfontw) | Creates a logical GDI font from size, weight and family parameters | Actual font substitution depends on installed fonts; renderer uses the target platform's native font mapper |
| UI-GDI-003 | [SetTextColor](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-settextcolor), [SetBkColor](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setbkcolor) | Sets text and background colors on a device context | GDI drawing properties; not a general CSS/style engine |
| UI-GDI-004 | [FillRect](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-fillrect) | Fills the client background using a logical brush | User32; uses a supplied GDI brush handle |
| UI-GDI-005 | [DeleteObject](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deleteobject) | Releases GDI brushes and fonts owned by the backend | Do not delete objects while selected into a device context |
| TAURI-003 | [Tauri 2.12.5 WebviewWindowBuilder initialization_script](https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindowBuilder.html#method.initialization_script) | Initialization script executes after the global object exists and before HTML parsing/other page scripts | Main-frame script; host frontend and permissions remain application-owned |


## WLDE desktop presentation research — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-001 | [IDesktopWallpaper::SetWallpaper](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-idesktopwallpaper-setwallpaper) | The documented Windows desktop wallpaper interface accepts a monitor ID and static wallpaper path | Windows 8+ desktop API; setting a file path is not an animated frame-surface or decoder API |
| WLDE-002 | [IDesktopWallpaper interface](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper) | Provides per-monitor desktop wallpaper management and monitor geometry information | Establishes static wallpaper management/topology access, not a supported custom surface behind desktop icons |
| WLDE-003 | [Windows.UI.Composition visual layer](https://learn.microsoft.com/en-us/windows/apps/develop/ui/visual-layer/) | Provides visual composition primitives for application UI | App visual composition alone does not establish a desktop wallpaper-layer attachment or Explorer icon ordering contract |

Implementation consequence: keep static wallpaper path management and animated desktop presentation as separate adapter capabilities. Do not treat a composition API as proof of desktop-layer placement, and do not depend on undocumented WorkerW/Progman parenting without an independently reviewed/recoverable OS contract.


## Wallpaper Store and Wallpaper Engine integration — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WSTORE-001 | [Wallpaper Engine command-line controls](https://help.wallpaperengine.io/en/functionality/cli.html) | An already-running Wallpaper Engine instance can receive -control openWallpaper -file to open a project.json or supported media path; an optional zero-based -monitor selects a monitor | The executable is wallpaper32.exe or wallpaper64.exe; this is an external-app bridge, not a native WLDE renderer |
| WSTORE-002 | [Wallpaper Engine Steam Workshop backup guide](https://help.wallpaperengine.io/en/steam/backup.html) | Steam Workshop files for Wallpaper Engine use app ID 431960 in steamapps/workshop/content/431960 | The bridge reads this directory only; Steam retains subscription/download ownership, and the directory must not be manually deleted |
| WSTORE-003 | [Openverse documentation](https://docs.openverse.org/) and [Terms of Service](https://docs.openverse.org/terms_of_service) | Openverse is a search engine/API for openly licensed media; clients must respect API limits and content-level license/attribution conditions | Openverse says it does not verify license claims; clients must check the source work and must not scrape the catalogue |
| WSTORE-004 | [Wikimedia Commons reuse licenses](https://commons.wikimedia.org/wiki/Commons:Reusing_content_outside_Wikimedia/licenses/en) | Reuse terms are listed on each file's description page; CC-BY and CC-BY-SA require attribution, with share-alike conditions for CC-BY-SA derivatives | Licensing is item-specific; follow the full file-page license |
| WSTORE-005 | [NASA Images and Media Guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) | NASA media is generally reusable under stated conditions, but third-party copyrighted media may be included and NASA identifiers/endorsement are restricted | Check the credit/rights notice on every item; do not imply NASA endorses WinLook |

Implementation consequence: keep provider browsing separate from asset installation. A provider-level Free marker is discovery metadata, not a blanket right to copy/redistribute the media. Preserve creator/source/license/attribution in any later library importer and never mirror Steam Workshop data into the WinLook catalogue without rights to do so.


## Win32 monitor snapshot adapter — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-004 | [EnumDisplayMonitors](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors) | Enumerates monitor handles and calls a host-supplied callback; the null HDC/null clipping-rectangle form covers the virtual desktop | The API can include invisible mirroring pseudo-monitors; filter them separately |
| WLDE-005 | [EnumDisplayDevicesW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw), [DISPLAY_DEVICEW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-display_devicew) | Enumerates display devices; desktop-attached state can be selected and the mirroring-driver flag identifies pseudo devices | Device-name strings identify the current GDI display device, not a permanent hardware identity |
| WLDE-006 | [MONITORINFO](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfo), [MONITORINFOEXW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfoexw) | Provides monitor/work-area rectangles in virtual-screen coordinates, primary-display flag and the GDI monitor device name | Secondary monitor coordinates may be negative; set cbSize to the full structure size before GetMonitorInfo |

Implementation consequence: keep enumeration synchronous and bounded, then let the host event loop refresh snapshots on display/device topology notifications. Do not persist GDI names as durable monitor identities and do not introduce a polling thread.


## WLDE adaptive icons and Explorer Desktop Bridge — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-007 | [Android adaptive icon design](https://developer.android.com/develop/ui/compose/system/icon_design_adaptive) | Adaptive icons use separate foreground/background layers, may provide a monochrome layer for themed icons, and reserve an inner safe region within a larger masked tile | WinLook borrows the presentation model; this is not an Android runtime dependency or an exact Android resource format |
| WLDE-008 | [Working with Shell Extensions](https://learn.microsoft.com/en-us/windows/win32/shell/shell-exts) | Shell extension handlers include shortcut-menu, icon/thumbnail/preview and other Shell integration types; handlers are in-process COM DLLs loaded into Explorer or another Shell host | A crash/hang in an extension can crash/hang the hosting Shell process; threading model and per-monitor DPI require care |
| WLDE-009 | [Windows Shell Namespace Extensions](https://learn.microsoft.com/en-us/windows/win32/shell/nse-works), [IShellFolder::CreateViewObject](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellfolder-createviewobject) | Namespace extensions can supply custom folder views and Shell objects for their own virtual namespace | This does not make a namespace extension a universal replacement for the existing desktop or arbitrary filesystem folder views |
| WLDE-010 | [Debugging with the Shell](https://learn.microsoft.com/en-us/windows/win32/shell/debugging-with-the-shell) | Windows supports a separate Explorer desktop-process mode to isolate Shell extension testing from the active desktop/taskbar process | Development/testing aid only; verify exact target-version behaviour |

Implementation consequence: BGW always owns and initializes the Explorer Desktop Bridge when Explorer owns the desktop. The implementation uses version-specific Shell interfaces and controlled runtime hooks only where required. The Explorer executable on disk is never patched or replaced; DWM remains intact. A failure must restore native icon presentation and fall back without modifying user items.


## WLDE Explorer desktop-host discovery — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-011 | [GetShellWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getshellwindow), [GetWindowThreadProcessId](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid) | The Shell desktop window can be resolved and mapped to its creating process/thread | Both are documented from Windows 2000; this only identifies the Shell desktop window owner, not every Explorer folder window |
| WLDE-012 | [QueryFullProcessImageNameW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew) | Returns the full process executable path for a queried process | Vista+ API; do not import as a baseline for Windows 2000/XP or older |
| WLDE-013 | [ProcessIdToSessionId](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-processidtosessionid) | Resolves the Remote Desktop Services session associated with a PID | Vista+ API; compare against BGW's own session |
| WLDE-014 | [GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes) | Returns process creation time as FILETIME | Available from XP; creation time helps distinguish PID reuse |

Implementation consequence: the modern Rust probe is Vista+ because it depends on QueryFullProcessImageNameW and ProcessIdToSessionId. Older Shell generations require a separate classic adapter. PID alone is not a stable Explorer identity; pair it with session and process creation time.


## WLDE desktop-host process discovery — 2026-11-14

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-015 | [Process Security and Access Rights](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights), [ProcessIdToSessionId](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-processidtosessionid), [QueryFullProcessImageNameW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew), [GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes) | The adapter must open the target with PROCESS_QUERY_INFORMATION for session lookup; this right also implies the limited-query right needed by image-path lookup, and permits reading process creation time | Query-only access; it does not request VM read/write, thread creation, termination or process modification. The composed probe remains Vista+ because ProcessIdToSessionId and QueryFullProcessImageNameW are Vista+ |


## WLDE BGW host-monitor loop — 2026-11-15

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-016 | [WaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject), [Process Security and Access Rights](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights) | A process handle can be waited on until it becomes signaled; the handle must include SYNCHRONIZE | The BGW monitor thread does not create windows or own a message loop. The wait is bounded to five seconds so it can revalidate the current Shell desktop owner |
| WLDE-017 | [OutputDebugStringW](https://learn.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-outputdebugstringw) | Emits a string to the system debugger when one is active | Used only as initial prototype diagnostics; not a persistent log or guaranteed user-visible status channel |

The monitor requests PROCESS_QUERY_INFORMATION for process/session/time queries
and SYNCHRONIZE for process waiting. It does not request memory-read/write,
thread-creation, termination or process-modification access. While no
compatible Explorer-owned desktop is available, BGW retries discovery every
two seconds. It drops the old process handle before rediscovery.


## WLDE adaptive application-icon composition — 2026-11-15

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-018 | [Creating Shell Extension Handlers](https://learn.microsoft.com/en-us/windows/win32/shell/handlers), [How to Create Icon Handlers](https://learn.microsoft.com/en-us/windows/win32/shell/how-to-create-icon-handlers) | The Shell supports per-file icon handlers using IExtractIconA/W. The handler is asked for icon location and can return an icon handle when extraction is needed | This describes supported Shell integration points; it does not provide a universal custom desktop-view hook or prove every shortcut can be customized independently |
| WLDE-019 | [Working with Shell Extensions](https://learn.microsoft.com/en-us/windows/win32/shell/shell-exts), [Registering Shell Extension Handlers](https://learn.microsoft.com/en-us/windows/win32/shell/reg-shell-exts) | Shell extension handlers execute in-process in Explorer/other Shell hosts; a crashing or hanging extension can crash/hang the host. Shell view extensions must use Apartment threading and handle per-monitor DPI | Future native adapters must remain short/bounded, use the correct COM apartment and fail closed; the portable compositor itself does not execute inside Explorer |
| WLDE-020 | [About Icons](https://learn.microsoft.com/en-us/windows/win32/menurc/about-icons) | Shell icon lists expose different logical sizes and DPI-aware icon sizes; the extra-large desktop list scales with DPI while the 256 px jumbo list is fixed | The adapter chooses output dimensions for the target Shell list/monitor DPI and caches by DPI/output size; renderer output is bounded to 512 px |

The Rust renderer accepts caller-decoded RGBA image views and returns a bounded,
cacheable app-icon bitmap. The C89 renderer writes to a caller-owned buffer and
does not allocate. Neither implementation opens icon files, registers Shell
handlers, mutates shortcut/executable resources or changes Explorer state.


## WLDE read-only desktop filesystem inventory — 2026-11-15

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-021 | [KNOWNFOLDERID reference](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid) | `FOLDERID_Desktop` identifies the current user's Desktop folder; `FOLDERID_PublicDesktop` identifies the common Public Desktop folder. The documented GUIDs are used instead of hardcoded profile paths | These are filesystem Known Folders. They do not enumerate virtual Shell namespace objects or represent Explorer's active view/filter state |
| WLDE-022 | [SHGetKnownFolderPath](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath), [Known Folders overview](https://learn.microsoft.com/en-us/windows/win32/shell/known-folders) | Resolves the current user's Known Folder path, replacing the older SHGetFolderPath API; the returned path buffer must be released with CoTaskMemFree | Vista+ API. This modern Rust adapter uses it for Win10/11. Legacy C adapters still need generation-specific APIs |

This first inventory is one-level, read-only filesystem enumeration. Opaque path hashes are transient Group keys, not security credentials. Exact Shell desktop membership still requires an IShellFolder/IFolderView-backed adapter.


## WLDE filesystem Desktop change notifications — 2026-11-15

| ID | Source | Claim supported | Scope / caveat |
|---|---|---|---|
| WLDE-023 | [FindFirstChangeNotificationW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstchangenotificationw), [Obtaining Directory Change Notifications](https://learn.microsoft.com/en-us/windows/win32/fileio/obtaining-directory-change-notifications) | Opens a waitable notification handle for matching changes inside one directory; filters include file/directory names, attributes, size, last-write and creation. It does not identify which entry changed, so the host must re-enumerate. The handle is waited on and then re-armed | Non-recursive watcher only. The API does not report changes to the watched directory object itself. Minimum supported client is Windows XP, so this modern Rust adapter is for Win10/11; older C targets need a generation-specific watcher. Remote file systems may not reliably deliver notifications |
| WLDE-024 | [FindNextChangeNotification](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findnextchangenotification), [FindCloseChangeNotification](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findclosechangenotification), [WaitForMultipleObjects](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitformultipleobjects) | FindNext re-arms a signaled notification for a later wait; a successful re-arm must be followed by a wait before another re-arm. FindClose releases the handle. WaitForMultipleObjects can wait on multiple change-notification handles and returns the signaled object's index | BGW waits on at most two handles with a finite timeout. UNC roots retain a five-second snapshot fallback because remote notification delivery is not guaranteed. Long local/UNC paths use the documented extended-length prefix when the default MAX_PATH limit is exceeded |

The watcher is scoped to the current User/Public Desktop Known Folders, sets
bWatchSubtree=FALSE, and never examines file contents. Each signal causes a
fresh bounded one-level snapshot; it does not attempt to infer an exact file
operation from the coarse notification.

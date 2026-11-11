# WinLook Technical Source Ledger

Last verified: 2026-11-11. These citations support platform or upstream-project claims; they do not prove that WinLook code has been implemented or tested.

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
| UI-006 | [Rust for Windows August 2026 release notes](https://github.com/microsoft/windows-rs/issues/4867) | Windows Reactor 0.100 release family uses Rust 1.95 MSRV and initially targets Windows App SDK 2.4.0 | Release-family facts; do not mix setup/runtime metadata versions |
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
| BUILD-009 | [actions/checkout v7](https://github.com/actions/checkout/tree/v7), [setup-python v7](https://github.com/actions/setup-python/tree/v7), [cache v4](https://github.com/actions/cache/tree/v4), [upload-artifact v4](https://github.com/actions/upload-artifact/tree/v4), [download-artifact v4](https://github.com/actions/download-artifact/tree/v4) | Workflow action versions referenced in the staged workflow suite | Each use is pinned to the resolved commit SHA recorded in the YAML file, not a moving tag |

Implementation consequence: C and Rust share the build/ root but keep outputs and compiler caches distinct. Cargo outputs use build/rust/, Cargo's registry/Git and optional sccache use build/.rcache/, CMake outputs use build/c/, and ccache uses build/.Ccache/. This directory arrangement is a WinLook repository contract, not a requirement imposed by the upstream tools.

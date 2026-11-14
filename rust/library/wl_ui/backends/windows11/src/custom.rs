use std::rc::Rc;
use windows_reactor::*;

use wl_ui_core::{UiApplication, UiColor, UiColorScheme, UiError, UiMetrics, UiPage, UiPalette};
use wl_ui_win32::{browse_file_from, enumerate_monitors, open_url};
use wl_wlde::{store_provider, StoreProviderId, WallpaperEngineInstall, WallpaperTarget};

#[derive(Clone)]
enum Message {
    PaneOpenChanged(bool),
    SearchChanged(String),
    SelectedTagChanged(Option<String>),
    TogglePane,
    Navigate(String),
    LocateWallpaperEngine,
    BrowseWallpaper,
    OpenProvider(StoreProviderId),
    RefreshDisplays,
}

struct CustomShell {
    application: UiApplication,
    selected_page: String,
    pane_open: bool,
    search: String,
    wallpaper_engine: Option<WallpaperEngineInstall>,
    store_status: String,
    monitor_status: String,
}

impl CustomShell {
    fn selected_page(&self) -> &UiPage {
        self.application
            .pages()
            .iter()
            .find(|page| page.id() == self.selected_page)
            .unwrap_or(&self.application.pages()[0])
    }

    fn matches_search(page: &UiPage, query: &str) -> bool {
        let query = query.trim().to_ascii_lowercase();
        query.is_empty()
            || page.title().to_ascii_lowercase().contains(&query)
            || page.description().to_ascii_lowercase().contains(&query)
            || page
                .navigation_label()
                .to_ascii_lowercase()
                .contains(&query)
    }

    fn engine_status(&self) -> String {
        match &self.wallpaper_engine {
            Some(install) => format!("Detected: {}", install.executable().display()),
            None => "Wallpaper Engine was not detected. Use Browse to select wallpaper64.exe or wallpaper32.exe.".to_string(),
        }
    }

    fn refresh_displays(&mut self) {
        self.monitor_status = monitor_snapshot_status();
    }

    fn locate_wallpaper_engine(&mut self) {
        match browse_file_from(
            "Locate Wallpaper Engine",
            "Wallpaper Engine executable",
            "wallpaper64.exe;wallpaper32.exe",
            None,
        ) {
            Ok(Some(path)) => match WallpaperEngineInstall::from_executable(path) {
                Ok(install) => {
                    self.store_status = format!(
                        "Wallpaper Engine located: {}",
                        install.executable().display()
                    );
                    self.wallpaper_engine = Some(install);
                }
                Err(error) => self.store_status = error.to_string(),
            },
            Ok(None) => {}
            Err(error) => self.store_status = error.to_string(),
        }
    }

    fn browse_wallpaper(&mut self) {
        let Some(install) = self.wallpaper_engine.as_ref() else {
            self.store_status =
                "Locate Wallpaper Engine first, then browse to a local wallpaper file.".to_string();
            return;
        };
        let initial_directory = install.workshop_directory().filter(|path| path.is_dir());
        match browse_file_from(
            "Browse Wallpaper Engine wallpapers",
            "Wallpaper Engine project or media",
            "project.json;*.mp4;*.webm;*.avi;*.mov;*.m4v;*.wmv;*.gif;*.png;*.jpg;*.jpeg;*.webp;*.bmp",
            initial_directory.as_deref(),
        ) {
            Ok(Some(path)) => match WallpaperTarget::new(path) {
                Ok(target) => match install.open_wallpaper(&target, None) {
                    Ok(_child) => {
                        self.store_status = "The openWallpaper command was sent. Wallpaper Engine should already be running; Steam still manages Workshop subscriptions and downloads.".to_string();
                    }
                    Err(error) => self.store_status = error.to_string(),
                },
                Err(error) => self.store_status = error.to_string(),
            },
            Ok(None) => {}
            Err(error) => self.store_status = error.to_string(),
        }
    }
}

fn native_color(color: UiColor) -> Color {
    Color::argb(color.alpha, color.red, color.green, color.blue)
}

fn monitor_snapshot_status() -> String {
    match enumerate_monitors() {
        Ok(topology) => {
            let primary = topology
                .primary()
                .map(|monitor| monitor.id().to_string())
                .unwrap_or_else(|| "not reported".to_string());
            format!(
                "{} attached display(s) in the latest snapshot. Primary: {}. Hot-plug refresh is not wired to the Manager message loop yet.",
                topology.len(),
                primary
            )
        }
        Err(error) => format!("Display snapshot unavailable: {error}"),
    }
}

fn info_card(
    title: &str,
    detail: &str,
    status: &str,
    status_color: UiColor,
    palette: &UiPalette,
    metrics: &UiMetrics,
) -> View {
    let content = StackPanel::new()
        .spacing(f64::from(metrics.spacing_unit))
        .children((
            TextBlock::new()
                .text(status)
                .font_size(12.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(native_color(status_color)),
            TextBlock::new()
                .text(title)
                .font_size(18.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .foreground(native_color(palette.text_primary)),
            TextBlock::new()
                .text(detail)
                .font_size(14.0)
                .line_height(20.0)
                .foreground(native_color(palette.text_secondary))
                .text_wrapping(TextWrapping::Wrap),
        ));

    Border::new()
        .background(native_color(palette.surface_elevated))
        .border_brush(native_color(palette.border))
        .border_thickness(Thickness::uniform(f64::from(metrics.border_width)))
        .corner_radius(CornerRadius::uniform(f64::from(metrics.radius_lg)))
        .padding(Thickness::uniform(f64::from(metrics.content_padding)))
        .content(content)
        .into()
}

fn window_theme(scheme: UiColorScheme) -> WindowTheme {
    match scheme {
        UiColorScheme::Light => WindowTheme::Light,
        UiColorScheme::Dark => WindowTheme::Dark,
        UiColorScheme::System | UiColorScheme::HighContrast => WindowTheme::System,
    }
}

fn navigation_item(page: &UiPage, selected: bool) -> KeyedView {
    let item = NavigationViewItem::new()
        .tag(page.id())
        .is_selected(selected)
        .selects_on_invoked(true)
        .content(page.navigation_label());

    KeyedView::new(page.id().to_string(), item)
}

impl Component for CustomShell {
    type Message = Message;
    type Input = UiApplication;

    fn create(input: &UiApplication, _: &ComponentContext<Self>) -> Self {
        let wallpaper_engine = WallpaperEngineInstall::discover();
        let store_status = match &wallpaper_engine {
            Some(install) => format!(
                "Wallpaper Engine detected at {}",
                install.executable().display()
            ),
            None => "Wallpaper Engine not detected. Use Browse for Wallpaper Engine to locate wallpaper64.exe or wallpaper32.exe.".to_string(),
        };
        Self {
            application: input.clone(),
            selected_page: input.initial_page_id().to_string(),
            pane_open: true,
            search: String::new(),
            wallpaper_engine,
            store_status,
            monitor_status: monitor_snapshot_status(),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::PaneOpenChanged(value) => self.pane_open = value,
            Message::SearchChanged(value) => self.search = value,
            Message::SelectedTagChanged(tag) => {
                if let Some(id) = tag {
                    if self.application.pages().iter().any(|page| page.id() == id) {
                        self.selected_page = id;
                    }
                }
            }
            Message::TogglePane => self.pane_open = !self.pane_open,
            Message::Navigate(page_id) => {
                if self
                    .application
                    .pages()
                    .iter()
                    .any(|page| page.id() == page_id)
                {
                    self.selected_page = page_id;
                }
            }
            Message::LocateWallpaperEngine => self.locate_wallpaper_engine(),
            Message::BrowseWallpaper => self.browse_wallpaper(),
            Message::OpenProvider(id) => {
                let provider = store_provider(id);
                self.store_status = match open_url(provider.browse_url) {
                    Ok(()) => format!("Opened {} in your browser.", provider.name),
                    Err(error) => error.to_string(),
                };
            }
            Message::RefreshDisplays => self.refresh_displays(),
        }
    }

    fn view(&self, _: &UiApplication, context: &mut ViewContext<Self>) -> View {
        let window = self.application.window_config();
        let theme = self.application.theme_ref();
        let palette = theme.palette();
        let typography = theme.typography_tokens();
        let metrics = theme.metrics_tokens();
        let page = self.selected_page();

        context.window_title(format!("{} — {}", window.title(), page.title()));
        context.window_visuals(
            WindowVisuals::new()
                .client_size(window.width() as f64, window.height() as f64)
                .theme(window_theme(theme.color_scheme())),
        );

        let visible_pages: Vec<&UiPage> = self
            .application
            .pages()
            .iter()
            .filter(|item| Self::matches_search(item, &self.search))
            .collect();
        let menu_items: Vec<KeyedView> = visible_pages
            .iter()
            .map(|item| navigation_item(item, item.id() == self.selected_page))
            .collect();

        let page_actions: View = match page.id() {
            "home" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "WinLook Desktop Engine",
                        "Your desktop workspace, wallpaper sources and presentation controls. The native renderer is still being built; this page reports only features that are actually present.",
                        "WLDE · MODULE OVERVIEW",
                        palette.accent,
                        palette,
                        metrics,
                    ),
                    StackPanel::new()
                        .spacing(f64::from(metrics.spacing_unit * 2.0))
                        .children((
                            info_card(
                                "Display topology",
                                &self.monitor_status,
                                "WIN32 SNAPSHOT",
                                palette.info,
                                palette,
                                metrics,
                            ),
                            info_card(
                                "Resource policy",
                                "The Rust and C89 policy controllers are ready. System telemetry sampling is not connected, so live percentages are intentionally not shown.",
                                "POLICY CORE READY",
                                palette.success,
                                palette,
                                metrics,
                            ),
                            info_card(
                                "Wallpaper presentation",
                                "Wallpaper Engine hand-off is available. Native static/animated desktop rendering, decoding and GPU recovery remain future gates.",
                                "RENDERER IN PROGRESS",
                                palette.warning,
                                palette,
                                metrics,
                            ),
                        )),
                    StackPanel::new()
                        .spacing(f64::from(metrics.spacing_unit))
                        .children((
                            Button::new()
                                .on_click(context.message(Message::Navigate("wallpaper".to_string())))
                                .content("Open Wallpaper"),
                            Button::new()
                                .on_click(context.message(Message::Navigate("displays".to_string())))
                                .content("View Displays"),
                            Button::new()
                                .on_click(context.message(Message::Navigate("wallpaper-performance".to_string())))
                                .content("Resource & Performance"),
                        )),
                ))
                .into(),
            "wallpaper" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Wallpaper workspace",
                        "Manage providers, your local collection, display topology and WLDE playback policy from one place.",
                        "WORKSPACE READY",
                        palette.accent,
                        palette,
                        metrics,
                    ),
                    StackPanel::new()
                        .spacing(f64::from(metrics.spacing_unit))
                        .children((
                            Button::new()
                                .on_click(context.message(Message::Navigate("wallpaper-store".to_string())))
                                .content("Open Wallpaper Store"),
                            Button::new()
                                .on_click(context.message(Message::Navigate("wallpaper-library".to_string())))
                                .content("Open Wallpaper Library"),
                            Button::new()
                                .on_click(context.message(Message::Navigate("displays".to_string())))
                                .content("Configure Displays"),
                        )),
                    StackPanel::new()
                        .spacing(f64::from(metrics.spacing_unit * 2.0))
                        .children((
                            info_card(
                                "Resource & Performance",
                                "CPU, GPU, RAM and VRAM pressure policy with hysteresis. Platform telemetry is the remaining integration step.",
                                "POLICY CORE READY",
                                palette.success,
                                palette,
                                metrics,
                            ),
                            info_card(
                                "Widgets and Desktop Groups",
                                "Widget hosting needs a restricted runtime. Desktop Groups stay disabled until native Shell items can be restored after crashes and uninstall.",
                                "CAPABILITY GATED",
                                palette.warning,
                                palette,
                                metrics,
                            ),
                        )),
                ))
                .into(),
            "wallpaper-store" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Wallpaper Engine",
                        "Connect to an existing installation, browse its Workshop catalogue and hand downloaded local projects/media to the running application.",
                        "EXTERNAL PROVIDER",
                        palette.accent,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::Navigate("wallpaper-store-wallpaper-engine".to_string())))
                        .content("Open Wallpaper Engine"),
                    info_card(
                        "Free catalogues",
                        "Browse Openverse, Wikimedia Commons and NASA. Free access does not replace each asset's license, creator and attribution requirements.",
                        "THREE SOURCES",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::Navigate("wallpaper-store-free".to_string())))
                        .content("Browse Free Catalogues"),
                    info_card(
                        "Wallpaper Library",
                        "The offline library and collection UI is present as a destination; durable import, previews and collection management are the next slice.",
                        "LIBRARY UI SCAFFOLD",
                        palette.info,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::Navigate("wallpaper-library".to_string())))
                        .content("Open Wallpaper Library"),
                ))
                .into(),
            "wallpaper-store-wallpaper-engine" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Wallpaper Engine connection",
                        &self.engine_status(),
                        if self.wallpaper_engine.is_some() { "INSTALLATION DETECTED" } else { "NEEDS SETUP" },
                        if self.wallpaper_engine.is_some() { palette.success } else { palette.warning },
                        palette,
                        metrics,
                    ),
                    StackPanel::new()
                        .spacing(f64::from(metrics.spacing_unit))
                        .children((
                            Button::new()
                                .on_click(context.message(Message::LocateWallpaperEngine))
                                .content("Locate Wallpaper Engine"),
                            Button::new()
                                .is_enabled(self.wallpaper_engine.is_some())
                                .on_click(context.message(Message::BrowseWallpaper))
                                .content("Browse Wallpaper"),
                            Button::new()
                                .on_click(context.message(Message::OpenProvider(StoreProviderId::WallpaperEngine)))
                                .content("Open Steam Workshop"),
                        )),
                    info_card(
                        "Steam ownership and downloads",
                        "Steam remains responsible for accounts, subscriptions, purchases, downloads and updates. WinLook does not unpack, modify or redistribute Workshop packages. Wallpaper Engine should already be running before hand-off.",
                        "PROVIDER-HOSTED",
                        palette.info,
                        palette,
                        metrics,
                    ),
                    TextBlock::new()
                        .text(self.store_status.clone())
                        .foreground(native_color(palette.text_secondary))
                        .text_wrapping(TextWrapping::Wrap),
                ))
                .into(),
            "wallpaper-store-free" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Openverse",
                        "Search openly licensed images, then verify the license and attribution against the original source.",
                        "FREE / LICENSE CHECK",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::Openverse)))
                        .content("Browse Openverse"),
                    info_card(
                        "Wikimedia Commons",
                        "Licenses and attribution vary per file. Always follow the original file page's reuse requirements.",
                        "FREE / PER-FILE RIGHTS",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::WikimediaCommons)))
                        .content("Browse Wikimedia Commons"),
                    info_card(
                        "NASA Image and Video Library",
                        "NASA media is generally reusable, but check credits and third-party copyright notices for each item.",
                        "FREE / CREDIT CHECK",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::NasaImages)))
                        .content("Browse NASA Images"),
                    TextBlock::new()
                        .text(self.store_status.clone())
                        .foreground(native_color(palette.text_secondary)),
                ))
                .into(),
            "wallpaper-library" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Your offline wallpaper collection",
                        "This destination is reserved for imported wallpapers, collections, source records and preview assets. Durable import and collection management are not implemented yet.",
                        "EMPTY LIBRARY · IMPORT PENDING",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::Navigate("wallpaper-store".to_string())))
                        .content("Find Wallpapers in the Store"),
                    Button::new()
                        .on_click(context.message(Message::Navigate("wallpaper-store-wallpaper-engine".to_string())))
                        .content("Browse Wallpaper Engine Files"),
                ))
                .into(),
            "displays" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Attached display snapshot",
                        &self.monitor_status,
                        "MANUAL SNAPSHOT",
                        palette.info,
                        palette,
                        metrics,
                    ),
                    Button::new()
                        .on_click(context.message(Message::RefreshDisplays))
                        .content("Refresh Monitor Snapshot"),
                    info_card(
                        "Per-monitor wallpaper placement",
                        "The shared topology and Fill/Fit/Stretch/Center placement contracts are ready. Live hot-plug events, DPI/orientation reconciliation and per-monitor wallpaper assignment are still being integrated.",
                        "TOPOLOGY API READY",
                        palette.success,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "wallpaper-performance" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Resource pressure controller",
                        "The portable Rust and C89 controllers evaluate CPU, GPU, RAM and VRAM independently. Sustained pressure enters Optimize or Suspend; recovery uses separate consecutive windows.",
                        "POLICY CORE READY",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Live system telemetry",
                        "No OS sampling provider is connected yet. Live CPU/GPU/RAM/VRAM values are intentionally unavailable instead of being guessed or represented by fake zeroes.",
                        "TELEMETRY NOT CONNECTED",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Playback policy",
                        "Normal uses the configured scene cadence, Optimize requests 30 FPS, and Suspend stops animation requests while preserving the last presented frame. Decoder and GPU device-loss recovery remain open.",
                        "SCENE POLICY INTEGRATED",
                        palette.info,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "icons" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "WLDE adaptive application tiles",
                        "Application shortcuts can use a consistent tile with separate foreground and background artwork, an optional monochrome layer, a selectable mask and a safe-zone scale. The original executable, shortcut and source artwork are not rewritten.",
                        "PORTABLE ICON CONTRACT READY",
                        palette.accent,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Source-preserving icon cleanup",
                        "WLDE normalizes how application artwork is presented; it does not silently replace vendor artwork, download lookalike icons or edit installed application files. Native icon extraction and the visual renderer remain adapter work.",
                        "SOURCE ASSETS PRESERVED",
                        palette.info,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Right-click actions",
                        "The portable menu contract distinguishes apps, files, folders and Groups. Apps can expose Customize Icon; all item types can join a Group; grouped items can be removed from that Group without deleting their originals.",
                        "CONTEXT MENU MODEL READY",
                        palette.success,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "wallpaper-widgets" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Widget runtime",
                        "The shared scene clock and capability model are ready, but no untrusted widget code is executed by the current WLDE host.",
                        "RUNTIME NOT ENABLED",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Permissions and isolation",
                        "Filesystem, network, input, audio and process capabilities must be explicit and isolated before third-party widgets can run.",
                        "SECURITY GATE",
                        palette.info,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "wallpaper-groups" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Virtual desktop Groups",
                        "A Group can collect application shortcuts, files and folders by opaque item reference. It is a presentation container, not a physical directory: the original Shell items, targets and filesystem locations stay untouched.",
                        "PORTABLE GROUP MODEL READY",
                        palette.accent,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Group-friendly right-click",
                        "The context-menu contract offers Open, Add to Group, Remove from Group, Properties and app-only Customize Icon actions. The host must still delegate native open/properties verbs to the Shell.",
                        "NATIVE MENU INTEGRATION PENDING",
                        palette.info,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Crash restoration guard",
                        "Desktop Groups remain unavailable for live presentation until the OS adapter can restore original icon visibility and placement after a crash, disable, session exit and uninstall.",
                        "PRESENTATION DISABLED BY SAFETY",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "wallpaper-rainmeter" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Rainmeter compatibility",
                        "RMSKIN package parsing is not equivalent to running Rainmeter skins. Measures, meters, variables, bangs, plugins and licensing still need a reviewed compatibility subset.",
                        "RESEARCH / RUNTIME PENDING",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Safe import boundary",
                        "No skin scripts or third-party plugins are executed by WLDE today. Any future support must use bounded extraction and explicit permissions.",
                        "NO CODE EXECUTION",
                        palette.info,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            "game-mode" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    info_card(
                        "Game Mode profiles",
                        "The bounded SHA-256 profile resolver is ready. Exact user profiles override automatic system/fullscreen rules; Windows signal collection and profile persistence are still pending.",
                        "PROFILE RESOLVER READY",
                        palette.success,
                        palette,
                        metrics,
                    ),
                    info_card(
                        "Foreground priority",
                        "The host must supply the foreground executable fingerprint, fullscreen state and optional OS Game Mode signal. No filename-only detection or background process scan is performed by the portable core.",
                        "HOST SIGNALS PENDING",
                        palette.warning,
                        palette,
                        metrics,
                    ),
                ))
                .into(),
            _ => info_card(
                "Workspace section",
                "This page is part of the shared WinLook Manager model. Its native controls are being integrated in a separate subsystem slice.",
                "MODULE PLACEHOLDER",
                palette.info,
                palette,
                metrics,
            ),
        };

        let navigation = NavigationView::new()
            .pane_display_mode(NavigationViewPaneDisplayMode::Left)
            .is_pane_toggle_button_visible(false)
            .is_back_button_visible(NavigationViewBackButtonVisible::Collapsed)
            .is_settings_visible(false)
            .always_show_header(false)
            .pane_title(window.title())
            .is_pane_open(self.pane_open)
            .on_is_pane_open_changed(context.callback(Message::PaneOpenChanged))
            .on_selected_tag_changed(context.callback(|tag: Option<Rc<str>>| {
                Message::SelectedTagChanged(tag.map(|value| value.to_string()))
            }))
            .grid_row(1)
            .keyed_menu_items(menu_items)
            .content(
                ScrollViewer::new().content(
                    Border::new()
                        .background(native_color(palette.surface))
                        .border_brush(native_color(palette.border))
                        .border_thickness(Thickness::uniform(f64::from(metrics.border_width)))
                        .corner_radius(CornerRadius::uniform(f64::from(metrics.radius_md)))
                        .padding(Thickness::uniform(f64::from(metrics.content_padding)))
                        .content(
                            StackPanel::new()
                                .spacing(f64::from(metrics.spacing_unit * 3.0))
                                .children((
                                    TextBlock::new()
                                        .text(page.title())
                                        .font_family(&typography.font_family)
                                        .font_size(f64::from(typography.size_xl))
                                        .font_weight(FontWeight::SEMI_BOLD)
                                        .foreground(native_color(palette.text_primary)),
                                    TextBlock::new()
                                        .text(page.description())
                                        .font_family(&typography.font_family)
                                        .font_size(f64::from(typography.size_md))
                                        .line_height(f64::from(
                                            typography.line_height * typography.size_md,
                                        ))
                                        .foreground(native_color(palette.text_secondary))
                                        .text_wrapping(TextWrapping::Wrap),
                                    page_actions,
                                )),
                        ),
                ),
            );

        let title_bar = TitleBar::new()
            .preferred_height(WindowTitleBarHeight::Tall)
            .title(window.title())
            .subtitle(page.title())
            .is_pane_toggle_button_visible(true)
            .on_pane_toggle_requested(context.message(Message::TogglePane))
            .grid_row(0)
            .content(
                TextBox::new()
                    .text(self.search.clone())
                    .background(native_color(palette.surface))
                    .border_brush(native_color(palette.border))
                    .border_thickness(Thickness::uniform(f64::from(metrics.border_width)))
                    .placeholder_text("Search pages")
                    .on_text_changed(
                        context
                            .callback(|value: Rc<str>| Message::SearchChanged(value.to_string())),
                    ),
            );

        Grid::new()
            .background(native_color(palette.window))
            .rows([GridLength::Auto, GridLength::Star(1.0)])
            .children((title_bar, navigation))
            .into()
    }
}

pub fn run(application: UiApplication) -> Result<(), UiError> {
    App::run_component::<CustomShell>(application)
        .map_err(|error| UiError::Backend(format!("Windows Reactor failed to start: {error:?}")))
}

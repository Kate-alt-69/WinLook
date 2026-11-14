use std::rc::Rc;
use windows_reactor::*;

use wl_ui_core::{UiApplication, UiColor, UiColorScheme, UiError, UiPage};
use wl_ui_win32::{browse_file_from, open_url};
use wl_wlde::{
    store_provider, StoreProviderId, WallpaperEngineInstall, WallpaperTarget,
};

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
}

struct CustomShell {
    application: UiApplication,
    selected_page: String,
    pane_open: bool,
    search: String,
    wallpaper_engine: Option<WallpaperEngineInstall>,
    store_status: String,
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
            || page.navigation_label().to_ascii_lowercase().contains(&query)
    }

    fn engine_status(&self) -> String {
        match &self.wallpaper_engine {
            Some(install) => format!("Detected: {}", install.executable().display()),
            None => "Wallpaper Engine was not detected. Use Browse to select wallpaper64.exe or wallpaper32.exe.".to_string(),
        }
    }

    fn locate_wallpaper_engine(&mut self) {
        match browse_file(
            "Locate Wallpaper Engine",
            "Wallpaper Engine executable",
            "wallpaper64.exe;wallpaper32.exe",
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
        let initial_directory = install
            .workshop_directory()
            .filter(|path| path.is_dir());
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
                if self.application.pages().iter().any(|page| page.id() == page_id) {
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
            "wallpaper" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    TextBlock::new().text(
                        "Manage desktop wallpapers, open the integrated Wallpaper Store, or organize your local library.",
                    ),
                    Button::new()
                        .on_click(context.message(Message::Navigate(
                            "wallpaper-store".to_string(),
                        )))
                        .content("Open Wallpaper Store"),
                    Button::new()
                        .on_click(context.message(Message::Navigate(
                            "wallpaper-library".to_string(),
                        )))
                        .content("Open Wallpaper Library"),
                ))
                .into(),
            "wallpaper-store" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    Button::new()
                        .on_click(context.message(Message::Navigate(
                            "wallpaper-store-wallpaper-engine".to_string(),
                        )))
                        .content("Wallpaper Engine"),
                    Button::new()
                        .on_click(context.message(Message::Navigate(
                            "wallpaper-store-free".to_string(),
                        )))
                        .content("Free Catalogues"),
                    Button::new()
                        .on_click(context.message(Message::Navigate(
                            "wallpaper-library".to_string(),
                        )))
                        .content("Wallpaper Library"),
                ))
                .into(),
            "wallpaper-store-wallpaper-engine" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    TextBlock::new()
                        .text("Wallpaper Engine connection")
                        .font_weight(FontWeight::SEMI_BOLD)
                        .foreground(native_color(palette.text_primary)),
                    TextBlock::new().text(self.engine_status()),
                    Button::new()
                        .on_click(context.message(Message::LocateWallpaperEngine))
                        .content("Browse for Wallpaper Engine..."),
                    Button::new()
                        .is_enabled(self.wallpaper_engine.is_some())
                        .on_click(context.message(Message::BrowseWallpaper))
                        .content("Browse Wallpaper..."),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(
                            StoreProviderId::WallpaperEngine,
                        )))
                        .content("Open Steam Workshop"),
                    TextBlock::new().text(
                        "Steam handles subscriptions, purchases, downloads and updates. Select a local project.json or supported media file. Wallpaper Engine should already be running.",
                    ),
                    TextBlock::new()
                        .text(self.store_status.clone())
                        .foreground(native_color(palette.text_secondary)),
                ))
                .into(),
            "wallpaper-store-free" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    TextBlock::new().text(
                        "Free catalogues. Check each item's license and creator attribution before using or sharing it.",
                    ),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::Openverse)))
                        .content("Browse Openverse"),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::WikimediaCommons)))
                        .content("Browse Wikimedia Commons"),
                    Button::new()
                        .on_click(context.message(Message::OpenProvider(StoreProviderId::NasaImages)))
                        .content("Browse NASA Images"),
                    TextBlock::new().text(
                        "Openverse: confirm each result's license and attribution. Wikimedia Commons: licenses vary by file. NASA: check third-party notices and media credits.",
                    ),
                    TextBlock::new()
                        .text(self.store_status.clone())
                        .foreground(native_color(palette.text_secondary)),
                ))
                .into(),
            "wallpaper-library" => StackPanel::new()
                .spacing(f64::from(metrics.spacing_unit * 2.0))
                .children((
                    TextBlock::new().text(
                        "The local library is designed to keep imported wallpapers available offline. Import, collections and backup management are the next implementation slice.",
                    ),
                ))
                .into(),
            _ => View::empty(),
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
                                        .line_height(f64::from(typography.line_height * typography.size_md))
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

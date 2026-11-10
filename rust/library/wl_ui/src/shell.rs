use std::rc::Rc;
use windows_reactor::*;

#[path = "pages.rs"]
mod pages;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageId {
    Home,
    Appearance,
    AudioMedia,
    FileExplorer,
    Icons,
    Wallpaper,
    WindowManager,
    Displays,
    Notifications,
    Templates,
    Extensions,
    Compatibility,
    Diagnostics,
    Settings,
}

impl PageId {
    pub const ALL: [Self; 14] = [
        Self::Home,
        Self::Appearance,
        Self::AudioMedia,
        Self::FileExplorer,
        Self::Icons,
        Self::Wallpaper,
        Self::WindowManager,
        Self::Displays,
        Self::Notifications,
        Self::Templates,
        Self::Extensions,
        Self::Compatibility,
        Self::Diagnostics,
        Self::Settings,
    ];

    pub const fn tag(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Appearance => "appearance",
            Self::AudioMedia => "audio-media",
            Self::FileExplorer => "file-explorer",
            Self::Icons => "icons",
            Self::Wallpaper => "wallpaper",
            Self::WindowManager => "window-manager",
            Self::Displays => "displays",
            Self::Notifications => "notifications",
            Self::Templates => "templates",
            Self::Extensions => "extensions",
            Self::Compatibility => "compatibility",
            Self::Diagnostics => "diagnostics",
            Self::Settings => "settings",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Appearance => "Appearance",
            Self::AudioMedia => "Audio & Media",
            Self::FileExplorer => "File Explorer",
            Self::Icons => "Icons",
            Self::Wallpaper => "Wallpaper",
            Self::WindowManager => "Window Manager",
            Self::Displays => "Displays",
            Self::Notifications => "Notifications",
            Self::Templates => "Templates",
            Self::Extensions => "Extensions",
            Self::Compatibility => "Compatibility",
            Self::Diagnostics => "Diagnostics",
            Self::Settings => "Settings",
        }
    }

    pub const fn summary(self) -> &'static str {
        match self {
            Self::Home => "A quick overview of WinLook and the background runtime.",
            Self::Appearance => "Themes, colors, visual preferences and appearance profiles.",
            Self::AudioMedia => "Audio-session controls and system media playback information.",
            Self::FileExplorer => "File Explorer presentation and supported Shell integrations.",
            Self::Icons => "Desktop and system icon presentation.",
            Self::Wallpaper => "Wallpapers, desktop scenes, widgets and playback.",
            Self::WindowManager => "Window layouts, rules, workspaces and compatibility.",
            Self::Displays => "Monitor topology, display settings and per-display capabilities.",
            Self::Notifications => "WinLook notifications, popups and on-screen display.",
            Self::Templates => "Importable WinLook configurations and reusable presets.",
            Self::Extensions => "Optional extensions, capabilities and permissions.",
            Self::Compatibility => "Windows build, API capabilities and verified feature support.",
            Self::Diagnostics => "BGW health, subsystem state and resource-governor diagnostics.",
            Self::Settings => "Manager preferences, startup behaviour and application settings.",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|page| page.tag() == tag)
    }

    pub fn matches_search(self, query: &str) -> bool {
        let query = query.trim();
        query.is_empty()
            || self
                .title()
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
            || self
                .summary()
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
    }
}

#[derive(Clone)]
enum Message {
    Back,
    CycleTheme,
    Navigate(PageId),
    PaneOpenChanged(bool),
    SearchChanged(String),
    SelectedTagChanged(Option<String>),
    TogglePane,
}

struct ManagerShell {
    history: Vec<PageId>,
    page: PageId,
    pane_open: bool,
    search: String,
    theme: WindowTheme,
}

fn theme_name(theme: WindowTheme) -> &'static str {
    match theme {
        WindowTheme::System => "System",
        WindowTheme::Light => "Light",
        WindowTheme::Dark => "Dark",
    }
}

fn navigation_item(page: PageId, selected: bool) -> KeyedView {
    let mut item = NavigationViewItem::new()
        .tag(page.tag())
        .is_selected(selected)
        .selects_on_invoked(true)
        .content(page.title());

    item = match page {
        PageId::Home => item.icon(Symbol::Home),
        PageId::Settings => item.icon(Symbol::Setting),
        _ => item,
    };

    KeyedView::new(page.tag().to_string(), item)
}

impl ManagerShell {
    fn navigate(&mut self, page: PageId) {
        if page != self.page {
            self.history.push(self.page);
            self.page = page;
        }
    }

    fn go_back(&mut self) {
        if let Some(page) = self.history.pop() {
            self.page = page;
        }
    }
}

impl Component for ManagerShell {
    type Message = Message;
    type Input = ();

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            history: Vec::new(),
            page: PageId::Home,
            pane_open: true,
            search: String::new(),
            theme: WindowTheme::System,
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Back => self.go_back(),
            Message::CycleTheme => {
                self.theme = match self.theme {
                    WindowTheme::System => WindowTheme::Light,
                    WindowTheme::Light => WindowTheme::Dark,
                    WindowTheme::Dark => WindowTheme::System,
                };
            }
            Message::Navigate(page) => self.navigate(page),
            Message::PaneOpenChanged(value) => self.pane_open = value,
            Message::SearchChanged(value) => self.search = value,
            Message::SelectedTagChanged(tag) => {
                if let Some(page) = tag.and_then(|tag| PageId::from_tag(&tag)) {
                    self.navigate(page);
                }
            }
            Message::TogglePane => self.pane_open = !self.pane_open,
        }
    }

    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title(format!("WinLook — {}", self.page.title()));
        context.window_visuals(
            WindowVisuals::new()
                .theme(self.theme)
                .client_size(1280.0, 820.0),
        );

        let on_navigate = context.callback(Message::Navigate);
        let page_view = pages::render(self.page, on_navigate);
        let menu_items: Vec<KeyedView> = PageId::ALL
            .iter()
            .copied()
            .filter(|page| page.matches_search(&self.search))
            .map(|page| navigation_item(page, page == self.page))
            .collect();

        let navigation = NavigationView::new()
            .pane_display_mode(NavigationViewPaneDisplayMode::Left)
            .is_pane_toggle_button_visible(false)
            .is_back_button_visible(NavigationViewBackButtonVisible::Collapsed)
            .is_settings_visible(false)
            .always_show_header(false)
            .pane_title("WinLook")
            .is_pane_open(self.pane_open)
            .on_is_pane_open_changed(context.callback(Message::PaneOpenChanged))
            .on_selected_tag_changed(context.callback(|tag: Option<Rc<str>>| {
                Message::SelectedTagChanged(tag.map(|tag| tag.to_string()))
            }))
            .grid_row(1)
            .keyed_menu_items(menu_items)
            .content(Grid::new().keyed_children([keyed(self.page.tag(), page_view)]));

        let title_bar = TitleBar::new()
            .preferred_height(WindowTitleBarHeight::Tall)
            .title("WinLook")
            .subtitle(self.page.title())
            .is_back_button_visible(true)
            .is_back_button_enabled(!self.history.is_empty())
            .is_pane_toggle_button_visible(true)
            .on_back_requested(context.message(Message::Back))
            .on_pane_toggle_requested(context.message(Message::TogglePane))
            .grid_row(0)
            .content(
                TextBox::new()
                    .text(self.search.clone())
                    .placeholder_text("Search WinLook settings")
                    .on_text_changed(
                        context
                            .callback(|value: Rc<str>| Message::SearchChanged(value.to_string())),
                    ),
            )
            .right_header(
                Button::new()
                    .on_click(context.message(Message::CycleTheme))
                    .content(format!("Theme: {}", theme_name(self.theme))),
            );

        Grid::new()
            .rows([GridLength::Auto, GridLength::Star(1.0)])
            .children((title_bar, navigation))
            .into()
    }
}

pub fn run() {
    App::run_component::<ManagerShell>(()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_tags_are_unique_and_round_trip() {
        let mut tags = PageId::ALL
            .iter()
            .map(|page| page.tag())
            .collect::<Vec<_>>();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), PageId::ALL.len());

        for page in PageId::ALL {
            assert_eq!(PageId::from_tag(page.tag()), Some(page));
        }
    }

    #[test]
    fn unknown_page_tags_are_rejected() {
        assert_eq!(PageId::from_tag("not-a-page"), None);
    }

    #[test]
    fn search_matches_titles_and_summaries() {
        assert!(PageId::AudioMedia.matches_search("audio"));
        assert!(PageId::WindowManager.matches_search("workspaces"));
        assert!(!PageId::WindowManager.matches_search("printer"));
        assert!(PageId::Settings.matches_search("   "));
    }
}

use std::rc::Rc;
use windows_reactor::*;

use wl_ui_core::{UiApplication, UiError, UiPage};

#[derive(Clone)]
enum Message {
    PaneOpenChanged(bool),
    SearchChanged(String),
    SelectedTagChanged(Option<String>),
    TogglePane,
}

struct CustomShell {
    application: UiApplication,
    selected_page: String,
    pane_open: bool,
    search: String,
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
    }
}

fn navigation_item(page: &UiPage, selected: bool) -> KeyedView {
    let item = NavigationViewItem::new()
        .tag(page.id())
        .is_selected(selected)
        .selects_on_invoked(true)
        .content(page.title());

    KeyedView::new(page.id().to_string(), item)
}

impl Component for CustomShell {
    type Message = Message;
    type Input = UiApplication;

    fn create(input: &UiApplication, _: &ComponentContext<Self>) -> Self {
        Self {
            application: input.clone(),
            selected_page: input.initial_page_id().to_string(),
            pane_open: true,
            search: String::new(),
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
        }
    }

    fn view(&self, _: &UiApplication, context: &mut ViewContext<Self>) -> View {
        let window = self.application.window_config();
        let page = self.selected_page();

        context.window_title(format!("{} — {}", window.title(), page.title()));
        context.window_visuals(
            WindowVisuals::new()
                .client_size(window.width() as f64, window.height() as f64),
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
                    StackPanel::new()
                        .spacing(14.0)
                        .children((
                            TextBlock::new().text(page.title()).font_size(28.0),
                            TextBlock::new()
                                .text(page.description())
                                .text_wrapping(TextWrapping::Wrap),
                            TextBlock::new()
                                .text("This page is supplied by the application's shared wl_ui model.")
                                .text_wrapping(TextWrapping::Wrap),
                        )),
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
                    .placeholder_text("Search pages")
                    .on_text_changed(
                        context.callback(|value: Rc<str>| {
                            Message::SearchChanged(value.to_string())
                        }),
                    ),
            );

        Grid::new()
            .rows([GridLength::Auto, GridLength::Star(1.0)])
            .children((title_bar, navigation))
            .into()
    }
}

pub fn run(application: UiApplication) -> Result<(), UiError> {
    App::run_component::<CustomShell>(application)
        .map_err(|error| UiError::Backend(format!("Windows Reactor failed to start: {error:?}")))
}

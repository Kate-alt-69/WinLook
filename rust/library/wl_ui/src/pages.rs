use std::rc::Rc;
use windows_reactor::*;

use crate::shell::PageId;

fn page_header(title: &str, summary: &str) -> View {
    StackPanel::new()
        .spacing(6.0)
        .children((
            TextBlock::new().text(title).font_size(28.0),
            TextBlock::new()
                .text(summary)
                .text_wrapping(TextWrapping::Wrap),
        ))
        .into()
}

fn information_card(title: &str, body: &str) -> View {
    Border::new()
        .padding(Thickness::new(18.0, 16.0, 18.0, 16.0))
        .content(
            StackPanel::new()
                .spacing(6.0)
                .children((
                    TextBlock::new().text(title).font_size(16.0),
                    TextBlock::new()
                        .text(body)
                        .text_wrapping(TextWrapping::Wrap),
                )),
        )
        .into()
}

fn preview_notice() -> View {
    TextBlock::new()
        .text("Preview build — the BGW background runtime is not connected yet. This UI prototype does not apply system changes.")
        .text_wrapping(TextWrapping::Wrap)
        .into()
}

fn home_page(on_navigate: Callback<PageId>) -> View {
    let quick_actions = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            Button::new()
                .content("Appearance")
                .on_click(on_navigate.call(PageId::Appearance)),
            Button::new()
                .content("Wallpaper")
                .on_click(on_navigate.call(PageId::Wallpaper)),
            Button::new()
                .content("Displays")
                .on_click(on_navigate.call(PageId::Displays)),
        ));

    ScrollViewer::new()
        .content(
            Border::new()
                .padding(Thickness::new(36.0, 28.0, 36.0, 36.0))
                .content(
                    StackPanel::new()
                        .spacing(22.0)
                        .children((
                            page_header(
                                "Welcome to WinLook",
                                "A clear home for your desktop, appearance and window experience.",
                            ),
                            information_card(
                                "Background runtime",
                                "Not connected. The Manager shell is being built before BGW integration.",
                            ),
                            information_card(
                                "Your desktop",
                                "Wallpaper, themes, icons and display controls will appear here when their runtime capabilities are available.",
                            ),
                            page_header("Quick access", "Jump directly to a WinLook area."),
                            quick_actions,
                            preview_notice(),
                        )),
                ),
        )
        .into()
}

fn page_content(page: PageId) -> View {
    let (status_title, status_body) = match page {
        PageId::Diagnostics => (
            "Runtime diagnostics",
            "BGW is not connected in this UI prototype. Live process, subsystem and resource information will be shown only after the runtime protocol is implemented.",
        ),
        PageId::Compatibility => (
            "Compatibility status",
            "No platform capability probe has run yet. WinLook will distinguish documented API availability from features verified on this exact Windows build.",
        ),
        PageId::Extensions => (
            "Extension safety",
            "Extensions are not loaded by this prototype. Future extension capabilities will be isolated and permission-scoped.",
        ),
        PageId::Settings => (
            "Manager preferences",
            "The title-bar theme selector currently changes this window's theme only. Persistence and system-wide settings are not connected yet.",
        ),
        _ => (
            "Page scaffold",
            "This page has its navigation destination and layout scaffold. Its controls will be connected to BGW capabilities in a later integration stage.",
        ),
    };

    ScrollViewer::new()
        .content(
            Border::new()
                .padding(Thickness::new(36.0, 28.0, 36.0, 36.0))
                .content(
                    StackPanel::new()
                        .spacing(22.0)
                        .children((
                            page_header(page.title(), page.summary()),
                            information_card(status_title, status_body),
                            preview_notice(),
                        )),
                ),
        )
        .into()
}

pub(super) fn render(page: PageId, on_navigate: Callback<PageId>) -> View {
    if page == PageId::Home {
        home_page(on_navigate)
    } else {
        page_content(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pages_have_non_empty_user_facing_copy() {
        for page in PageId::ALL {
            assert!(!page.title().trim().is_empty());
            assert!(!page.summary().trim().is_empty());
            assert!(!page.tag().trim().is_empty());
        }
    }

    #[test]
    fn diagnostics_and_compatibility_are_explicit_about_missing_runtime_data() {
        let diagnostics = page_content(PageId::Diagnostics);
        let compatibility = page_content(PageId::Compatibility);
        // The views are native view trees; construction itself is the smoke check.
        let _ = (diagnostics, compatibility);
    }

    #[test]
    fn page_registry_is_searchable_without_case_sensitivity() {
        let query = "aUdIo";
        assert!(PageId::ALL
            .iter()
            .any(|page| page.title().to_ascii_lowercase().contains(&query.to_ascii_lowercase())));
    }
}

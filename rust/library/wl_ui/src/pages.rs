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
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(1.0)
        .corner_radius(8.0)
        .padding(Thickness::new(18.0, 16.0, 18.0, 16.0))
        .content(
            StackPanel::new().spacing(6.0).children((
                TextBlock::new()
                    .text(title)
                    .font_size(16.0)
                    .font_weight(FontWeight::SEMI_BOLD),
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
    let appearance = on_navigate.clone();
    let wallpaper = on_navigate.clone();
    let displays = on_navigate;
    let quick_actions = StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(8.0)
        .children((
            Button::new()
                .content("Appearance")
                .on_click(move || appearance.call(PageId::Appearance)),
            Button::new()
                .content("Wallpaper")
                .on_click(move || wallpaper.call(PageId::Wallpaper)),
            Button::new()
                .content("Displays")
                .on_click(move || displays.call(PageId::Displays)),
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

fn page_settings(page: PageId) -> &'static [(&'static str, &'static str)] {
    match page {
        PageId::Appearance => &[
            ("Theme and colours", "Choose the window theme and configure accent colours. The live theme selector is in the title bar."),
            ("Transparency effects", "Configure supported transparency and material effects after the platform capability probe is available."),
            ("Appearance profiles", "Save and switch between reusable visual profiles."),
        ],
        PageId::AudioMedia => &[
            ("Audio sessions", "View supported audio sessions and route WinLook-managed audio controls."),
            ("Media playback", "Display the active media session and supported transport controls."),
            ("Volume overlay", "Configure the WinLook volume and media on-screen display."),
        ],
        PageId::FileExplorer => &[
            ("Explorer appearance", "Configure supported File Explorer presentation options."),
            ("Context menu", "Manage optional context-menu integrations and their permissions."),
            ("Folder presentation", "Choose supported folder and navigation presentation preferences."),
        ],
        PageId::Icons => &[
            ("Desktop icons", "Configure supported desktop icon presentation."),
            ("Icon library", "Browse and select installed icon packs."),
            ("System icons", "Review which system icon locations are supported on this Windows build."),
        ],
        PageId::Wallpaper => &[
            ("Wallpaper library", "Import and organize still images, videos and desktop scenes."),
            ("Playback", "Configure playback, pause behaviour and per-display wallpaper assignment."),
            ("Widgets", "Manage desktop widgets and their update/resource policies."),
        ],
        PageId::WindowManager => &[
            ("Window rules", "Create rules for supported window placement and behaviour."),
            ("Workspaces", "Configure workspace layouts and restore points."),
            ("Compatibility", "Review optional target integrations and their safety status."),
        ],
        PageId::Displays => &[
            ("Connected displays", "View monitor topology and the capabilities reported by the display adapter."),
            ("Display profiles", "Save per-monitor arrangements and supported display preferences."),
            ("Per-display wallpaper", "Assign desktop scenes to individual displays."),
        ],
        PageId::Notifications => &[
            ("On-screen display", "Configure WinLook notification and OSD presentation."),
            ("Notification style", "Choose placement, duration and visual presentation."),
            ("Quiet hours", "Configure when optional WinLook notifications should be suppressed."),
        ],
        PageId::Templates => &[
            ("Template library", "Browse reusable WinLook configurations and presets."),
            ("Import configuration", "Review and validate a configuration before applying it."),
            ("Export configuration", "Export a selected profile without including private runtime state."),
        ],
        PageId::Extensions => &[
            ("Installed extensions", "Review installed extensions and their declared capabilities."),
            ("Permissions", "Inspect and revoke extension permissions."),
            ("Extension sources", "Configure trusted extension sources and signature requirements."),
        ],
        PageId::Compatibility => &[
            ("Windows capabilities", "Probe the current Windows build and available platform APIs."),
            ("Feature compatibility", "Distinguish documented support from features verified on this machine."),
            ("Legacy adapters", "Review compatibility adapters and their tested OS ranges."),
        ],
        PageId::Diagnostics => &[
            ("BGW runtime", "Inspect background runtime health, protocol version and reconnect state."),
            ("CPUv and GPUv", "Review WinLook-owned resource reservations, pressure and recovery events."),
            ("Event log", "Inspect structured WinLook diagnostics without exposing unrelated process data."),
        ],
        PageId::Settings => &[
            ("Startup", "Configure whether the BGW background runtime starts with the Windows user session."),
            ("Updates", "Review application and library update channels."),
            ("Privacy", "Configure diagnostics collection and local data retention."),
        ],
        PageId::Home => &[],
    }
}

fn setting_card(title: &str, body: &str) -> View {
    Border::new()
        .background(ThemeBrush::CardBackground)
        .border_brush(ThemeBrush::CardStroke)
        .border_thickness(1.0)
        .corner_radius(8.0)
        .padding(Thickness::new(18.0, 16.0, 18.0, 16.0))
        .content(
            StackPanel::new()
                .spacing(8.0)
                .children((
                    TextBlock::new()
                        .text(title)
                        .font_size(16.0)
                        .font_weight(FontWeight::SEMI_BOLD),
                    TextBlock::new()
                        .text(body)
                        .text_wrapping(TextWrapping::Wrap),
                    ToggleSwitch::new()
                        .header("Unavailable until BGW is connected")
                        .is_enabled(false),
                )),
        )
        .into()
}

fn page_content(page: PageId) -> View {
    let mut children = vec![
        page_header(page.title(), page.summary()),
        TextBlock::new()
            .text("Settings preview")
            .font_size(20.0)
            .font_weight(FontWeight::SEMI_BOLD)
            .into(),
    ];

    for (title, body) in page_settings(page) {
        children.push(setting_card(title, body));
    }

    if page == PageId::Diagnostics {
        children.push(information_card(
            "Current prototype state",
            "No BGW process, IPC channel or live system telemetry is connected. This page intentionally does not fabricate runtime measurements.",
        ));
    }

    if page == PageId::Settings {
        children.push(information_card(
            "Window theme",
            "Use the Theme selector in the title bar to switch this Manager window between System, Light and Dark. This preview preference is not persisted yet.",
        ));
    }

    children.push(preview_notice());

    ScrollViewer::new()
        .content(
            Border::new()
                .padding(Thickness::new(36.0, 28.0, 36.0, 36.0))
                .content(StackPanel::new().spacing(18.0).children(children)),
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
    fn every_non_home_page_has_a_scaffolded_settings_section() {
        for page in PageId::ALL {
            if page != PageId::Home {
                assert!(!page_settings(page).is_empty(), "missing page sections for {}", page.tag());
            }
        }
    }

    #[test]
    fn page_registry_is_searchable_without_case_sensitivity() {
        let query = "aUdIo";
        assert!(PageId::ALL.iter().any(|page| {
            page.title()
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
        }));
    }
}

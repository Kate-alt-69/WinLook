//! Provider metadata for WLDE's integrated Wallpaper Store.
//!
//! The initial Store is a provider hub. Browser providers own their catalogue,
//! item downloads and terms. Steam owns Wallpaper Engine Workshop subscriptions
//! and delivery; WLDE opens the official Workshop and can hand local files to
//! an already-installed Wallpaper Engine process.

/// Stable identifier for a built-in Store provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoreProviderId {
    WallpaperEngine,
    Openverse,
    WikimediaCommons,
    NasaImages,
}

/// How a provider opens from the Manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreProviderAccess {
    Browser,
    InstalledApplication,
}

/// Cost model advertised by a provider. Item-level terms still apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorePriceModel {
    Free,
    Mixed,
}

/// A built-in provider entry shown in Wallpaper Store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreProvider {
    pub id: StoreProviderId,
    pub name: &'static str,
    pub description: &'static str,
    pub browse_url: &'static str,
    pub access: StoreProviderAccess,
    pub price_model: StorePriceModel,
    pub rights_note: &'static str,
}

/// Built-in providers whose current terms support the advertised workflow.
///
/// This list intentionally excludes image sites that prohibit redistributing
/// unmodified content as standalone wallpapers or prohibit competing wallpaper
/// catalogues. Provider availability is not a blanket license for every item.
pub const STORE_PROVIDERS: [StoreProvider; 4] = [
    StoreProvider {
        id: StoreProviderId::WallpaperEngine,
        name: "Wallpaper Engine",
        description: "Browse Steam Workshop and use wallpapers already downloaded by Steam.",
        browse_url: "https://steamcommunity.com/workshop/browse/?appid=431960",
        access: StoreProviderAccess::InstalledApplication,
        price_model: StorePriceModel::Mixed,
        rights_note: "Steam manages subscriptions and purchases. Follow each creator's license; do not unpack or redistribute protected packages.",
    },
    StoreProvider {
        id: StoreProviderId::Openverse,
        name: "Openverse",
        description: "Search openly licensed images from multiple sources.",
        browse_url: "https://openverse.org/search/image?q=wallpaper",
        access: StoreProviderAccess::Browser,
        price_model: StorePriceModel::Free,
        rights_note: "Check the individual result's license, creator, source and attribution requirements before importing or sharing.",
    },
    StoreProvider {
        id: StoreProviderId::WikimediaCommons,
        name: "Wikimedia Commons",
        description: "Browse freely licensed and public-domain media.",
        browse_url: "https://commons.wikimedia.org/wiki/Special:MediaSearch?type=image&search=wallpaper",
        access: StoreProviderAccess::Browser,
        price_model: StorePriceModel::Free,
        rights_note: "Licenses vary by file. Preserve attribution and follow the exact file page's license terms.",
    },
    StoreProvider {
        id: StoreProviderId::NasaImages,
        name: "NASA Image and Video Library",
        description: "Explore NASA imagery and public science media.",
        browse_url: "https://images.nasa.gov/search?q=wallpaper",
        access: StoreProviderAccess::Browser,
        price_model: StorePriceModel::Free,
        rights_note: "NASA media is generally reusable, but check each item's credit and third-party rights notices; NASA endorsement is not implied.",
    },
];

/// Returns the built-in provider list in Manager display order.
pub fn store_providers() -> &'static [StoreProvider] {
    &STORE_PROVIDERS
}

/// Finds a provider by its stable identifier.
pub fn store_provider(id: StoreProviderId) -> &'static StoreProvider {
    STORE_PROVIDERS
        .iter()
        .find(|provider| provider.id == id)
        .expect("every StoreProviderId has a built-in provider")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_free_catalogues_are_marked_free_and_have_item_level_rights_notes() {
        let free = store_providers()
            .iter()
            .filter(|provider| provider.price_model == StorePriceModel::Free)
            .collect::<Vec<_>>();

        assert_eq!(free.len(), 3);
        assert!(free
            .iter()
            .all(|provider| provider.browse_url.starts_with("https://")));
        assert!(free.iter().all(|provider| !provider.rights_note.is_empty()));
    }

    #[test]
    fn wallpaper_engine_is_a_mixed_price_external_provider() {
        let provider = store_provider(StoreProviderId::WallpaperEngine);
        assert_eq!(provider.price_model, StorePriceModel::Mixed);
        assert_eq!(provider.access, StoreProviderAccess::InstalledApplication);
        assert!(provider.browse_url.contains("appid=431960"));
    }
}

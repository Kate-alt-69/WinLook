//! Bounded, memory-only Shell icon acquisition for BGW.
//!
//! One dedicated worker performs Shell extraction and RGBA rasterization.
//! The BGW host-monitor loop only submits bounded work and drains results; it
//! never waits for an icon handler. Paths and pixel buffers are never persisted.

use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread;

use wl_wlde::AdaptiveIconImageView;
use wl_wlde_win32::{
    extract_desktop_shell_icon, rasterize_desktop_shell_icon_rgba, DesktopInventoryItem,
    DesktopItemIdentityStability, DesktopShellIconError, DesktopShellIconRgba,
    DesktopShellIconSize,
};

const WORK_QUEUE_CAPACITY: usize = 16;
const RESULT_QUEUE_CAPACITY: usize = 16;
const MAX_ICON_CACHE_BYTES: usize = 8 * 1024 * 1024;
const MAX_ICON_CACHE_ENTRIES: usize = 256;
const COINIT_APARTMENTTHREADED: u32 = 0x2;

#[cfg(windows)]
#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut std::ffi::c_void, concurrency_model: u32) -> i32;
    fn CoUninitialize();
}

#[cfg(windows)]
struct ShellComApartment;

#[cfg(windows)]
impl ShellComApartment {
    fn initialize() -> Result<Self, i32> {
        let result = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };
        if result < 0 {
            Err(result)
        } else {
            // S_OK and S_FALSE both require a balancing CoUninitialize call.
            Ok(Self)
        }
    }
}

#[cfg(windows)]
impl Drop for ShellComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[cfg(not(windows))]
struct ShellComApartment;

#[cfg(not(windows))]
impl ShellComApartment {
    fn initialize() -> Result<Self, i32> {
        Ok(Self)
    }
}

/// Rendering context used to distinguish cached Shell icon variants.
///
/// The initial BGW host uses 96 DPI and theme generation 0 until dedicated
/// DPI and appearance-change notifications are connected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DesktopIconContext {
    size: DesktopShellIconSize,
    dpi: u32,
    theme_generation: u64,
}

impl DesktopIconContext {
    pub fn new(size: DesktopShellIconSize, dpi: u32, theme_generation: u64) -> Option<Self> {
        (dpi != 0).then_some(Self {
            size,
            dpi,
            theme_generation,
        })
    }

    pub const fn initial_bgw() -> Self {
        Self {
            size: DesktopShellIconSize::Large,
            dpi: 96,
            theme_generation: 0,
        }
    }
}

impl Default for DesktopIconContext {
    fn default() -> Self {
        Self::initial_bgw()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DesktopIconCacheKey {
    item_id: String,
    identity_stability: DesktopItemIdentityStability,
    fallback_generation: u64,
    context: DesktopIconContext,
}

impl DesktopIconCacheKey {
    fn from_item(
        item: &DesktopInventoryItem,
        context: DesktopIconContext,
        fallback_generation: u64,
    ) -> Self {
        let identity_stability = item.identity_stability();
        Self {
            item_id: item.item().id().as_str().to_owned(),
            identity_stability,
            fallback_generation: if identity_stability == DesktopItemIdentityStability::PathFallback {
                fallback_generation
            } else {
                0
            },
            context,
        }
    }

    #[cfg(test)]
    fn for_test(
        item_id: &str,
        identity_stability: DesktopItemIdentityStability,
        context: DesktopIconContext,
    ) -> Self {
        Self::for_test_with_fallback_generation(item_id, identity_stability, context, 0)
    }

    #[cfg(test)]
    fn for_test_with_fallback_generation(
        item_id: &str,
        identity_stability: DesktopItemIdentityStability,
        context: DesktopIconContext,
        fallback_generation: u64,
    ) -> Self {
        Self {
            item_id: item_id.to_owned(),
            identity_stability,
            fallback_generation: if identity_stability == DesktopItemIdentityStability::PathFallback {
                fallback_generation
            } else {
                0
            },
            context,
        }
    }
}

struct CacheEntry {
    image: DesktopShellIconRgba,
    bytes: usize,
    last_used: u64,
}

/// BGW-owned LRU cache with hard byte and entry limits.
struct DesktopIconCache {
    entries: HashMap<DesktopIconCacheKey, CacheEntry>,
    used_bytes: usize,
    max_bytes: usize,
    max_entries: usize,
    access_clock: u64,
}

impl DesktopIconCache {
    fn new(max_bytes: usize, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            used_bytes: 0,
            max_bytes,
            max_entries,
            access_clock: 0,
        }
    }

    fn contains(&self, key: &DesktopIconCacheKey) -> bool {
        self.entries.contains_key(key)
    }

    fn get(&mut self, key: &DesktopIconCacheKey) -> Option<&DesktopShellIconRgba> {
        self.access_clock = self.access_clock.wrapping_add(1);
        let entry = self.entries.get_mut(key)?;
        entry.last_used = self.access_clock;
        Some(&entry.image)
    }

    fn insert(&mut self, key: DesktopIconCacheKey, image: DesktopShellIconRgba) -> bool {
        let bytes = image.pixels().len();
        if self.max_entries == 0 || bytes > self.max_bytes {
            return false;
        }

        if let Some(previous) = self.entries.remove(&key) {
            self.used_bytes = self.used_bytes.saturating_sub(previous.bytes);
        }

        self.access_clock = self.access_clock.wrapping_add(1);
        self.used_bytes = self.used_bytes.saturating_add(bytes);
        self.entries.insert(
            key,
            CacheEntry {
                image,
                bytes,
                last_used: self.access_clock,
            },
        );

        while self.entries.len() > self.max_entries || self.used_bytes > self.max_bytes {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone());
            let Some(oldest) = oldest else {
                break;
            };
            if let Some(removed) = self.entries.remove(&oldest) {
                self.used_bytes = self.used_bytes.saturating_sub(removed.bytes);
            }
        }
        true
    }

    fn retain_identities(
        &mut self,
        current: &HashSet<(String, DesktopItemIdentityStability)>,
    ) {
        self.entries.retain(|key, _| {
            current.contains(&(key.item_id.clone(), key.identity_stability))
        });
        self.recount_bytes();
    }

    fn retain_filesystem_backed(&mut self) {
        self.entries.retain(|key, _| {
            key.identity_stability == DesktopItemIdentityStability::FilesystemFileId
        });
        self.recount_bytes();
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.used_bytes = 0;
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn recount_bytes(&mut self) {
        self.used_bytes = self.entries.values().map(|entry| entry.bytes).sum();
    }
}

struct IconWork {
    generation: u64,
    key: DesktopIconCacheKey,
    item: DesktopInventoryItem,
}

struct IconWorkResult {
    generation: u64,
    key: DesktopIconCacheKey,
    result: Result<DesktopShellIconRgba, DesktopShellIconError>,
}

/// Aggregate worker results without exposing item paths or labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DesktopIconPollStats {
    pub completed: usize,
    pub failed: usize,
    pub stale_discarded: usize,
}

/// Single-worker Shell icon pipeline owned by BGW.
///
/// Shell handlers execute on one named worker thread. Both channels are
/// bounded; a slow handler cannot create unbounded threads, work items or
/// pixel allocations. Polling and cache access are non-blocking.
pub struct DesktopIconPipeline {
    context: DesktopIconContext,
    generation: u64,
    fallback_generation: u64,
    current_identities: HashSet<(String, DesktopItemIdentityStability)>,
    failed: HashSet<DesktopIconCacheKey>,
    pending: HashSet<(u64, DesktopIconCacheKey)>,
    request_tx: SyncSender<IconWork>,
    result_rx: Receiver<IconWorkResult>,
    cache: DesktopIconCache,
    worker_available: bool,
}

impl DesktopIconPipeline {
    pub fn new(context: DesktopIconContext) -> io::Result<Self> {
        let (request_tx, request_rx) = mpsc::sync_channel::<IconWork>(WORK_QUEUE_CAPACITY);
        let (result_tx, result_rx) =
            mpsc::sync_channel::<IconWorkResult>(RESULT_QUEUE_CAPACITY);
        let (startup_tx, startup_rx) = mpsc::sync_channel::<Result<(), i32>>(1);

        thread::Builder::new()
            .name("winlook-wlde-shell-icons".to_owned())
            .spawn(move || {
                let _com_apartment = match ShellComApartment::initialize() {
                    Ok(apartment) => {
                        let _ = startup_tx.send(Ok(()));
                        apartment
                    }
                    Err(code) => {
                        let _ = startup_tx.send(Err(code));
                        return;
                    }
                };

                while let Ok(work) = request_rx.recv() {
                    let result = extract_desktop_shell_icon(&work.item, work.key.context.size)
                        .and_then(|icon| rasterize_desktop_shell_icon_rgba(&icon));
                    if result_tx
                        .send(IconWorkResult {
                            generation: work.generation,
                            key: work.key,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })?;

        match startup_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(code)) => return Err(io::Error::from_raw_os_error(code)),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Shell icon worker exited before COM initialization completed",
                ));
            }
        }

        Ok(Self {
            context,
            generation: 0,
            fallback_generation: 0,
            current_identities: HashSet::new(),
            failed: HashSet::new(),
            pending: HashSet::new(),
            request_tx,
            result_rx,
            cache: DesktopIconCache::new(MAX_ICON_CACHE_BYTES, MAX_ICON_CACHE_ENTRIES),
            worker_available: true,
        })
    }

    /// Starts a new verified Explorer-host session and discards old pixels.
    ///
    /// An already-running or queued old-host request may finish, but its
    /// generation prevents the result from entering the new session's cache.
    /// Old pending keys stay tracked until poll drains their stale results.
    pub fn begin_host_session(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.fallback_generation = 0;
        self.current_identities.clear();
        self.failed.clear();
        self.cache.clear();
    }

    /// Drops path-fallback cache entries after a filesystem inventory refresh.
    ///
    /// Stable single-link filesystem identities may survive a same-volume
    /// rename. Path-derived IDs cannot distinguish replacement at the same
    /// path, so their icons are conservatively reacquired on every refresh.
    /// A fallback-only epoch invalidates old fallback results without
    /// discarding valid in-flight filesystem-ID requests.
    pub fn invalidate_path_fallbacks(&mut self) {
        self.fallback_generation = self.fallback_generation.wrapping_add(1);
        self.cache.retain_filesystem_backed();
        self.failed.retain(|key| {
            key.identity_stability == DesktopItemIdentityStability::FilesystemFileId
        });
    }

    /// Changes DPI/theme/size policy and invalidates every cached variant.
    pub fn set_context(&mut self, context: DesktopIconContext) {
        if self.context == context {
            return;
        }
        self.context = context;
        self.generation = self.generation.wrapping_add(1);
        self.fallback_generation = 0;
        self.failed.clear();
        self.cache.clear();
    }

    /// Queues missing icons without blocking the BGW host-monitor loop.
    ///
    /// The queue is bounded. If it fills, remaining inventory entries are
    /// retried after the next poll rather than spawning additional workers.
    pub fn request_missing(&mut self, inventory: &[DesktopInventoryItem]) -> usize {
        self.current_identities = inventory
            .iter()
            .map(|item| {
                (
                    item.item().id().as_str().to_owned(),
                    item.identity_stability(),
                )
            })
            .collect();
        self.cache.retain_identities(&self.current_identities);
        let context = self.context;
        let current_identities = &self.current_identities;
        self.failed.retain(|key| {
            key.context == context
                && current_identities.contains(&(key.item_id.clone(), key.identity_stability))
        });

        if !self.worker_available {
            return 0;
        }

        let mut requested = 0;
        for item in inventory {
            let key = DesktopIconCacheKey::from_item(item, self.context, self.fallback_generation);
            let pending_key = (self.generation, key.clone());
            if self.cache.contains(&key)
                || self.failed.contains(&key)
                || self.pending.contains(&pending_key)
            {
                continue;
            }

            match self.request_tx.try_send(IconWork {
                generation: self.generation,
                key: key.clone(),
                item: item.clone(),
            }) {
                Ok(()) => {
                    self.pending.insert(pending_key);
                    requested += 1;
                }
                Err(TrySendError::Full(_)) => break,
                Err(TrySendError::Disconnected(_)) => {
                    self.worker_available = false;
                    break;
                }
            }
        }
        requested
    }

    /// Drains completed results without waiting for Shell extraction.
    pub fn poll(&mut self) -> DesktopIconPollStats {
        let mut stats = DesktopIconPollStats::default();
        loop {
            match self.result_rx.try_recv() {
                Ok(response) => {
                    self.pending
                        .remove(&(response.generation, response.key.clone()));
                    let fallback_is_current = response.key.identity_stability
                        != DesktopItemIdentityStability::PathFallback
                        || response.key.fallback_generation == self.fallback_generation;
                    let is_current = response.generation == self.generation
                        && response.key.context == self.context
                        && fallback_is_current
                        && self.current_identities.contains(&(
                            response.key.item_id.clone(),
                            response.key.identity_stability,
                        ));
                    if !is_current {
                        stats.stale_discarded += 1;
                        continue;
                    }

                    match response.result {
                        Ok(image) => {
                            if self.cache.insert(response.key, image) {
                                stats.completed += 1;
                            } else {
                                stats.failed += 1;
                            }
                        }
                        Err(_) => {
                            self.failed.insert(response.key);
                            stats.failed += 1;
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.worker_available = false;
                    break;
                }
            }
        }
        stats
    }

    /// Returns a cached image for immediate BGW rendering.
    pub fn get(&mut self, item: &DesktopInventoryItem) -> Option<&DesktopShellIconRgba> {
        let key = DesktopIconCacheKey::from_item(item, self.context, self.fallback_generation);
        self.cache.get(&key)
    }

    /// Borrows a cached icon as WLDE compositor input without a pixel copy.
    pub fn adaptive_image_view(
        &mut self,
        item: &DesktopInventoryItem,
    ) -> Option<AdaptiveIconImageView<'_>> {
        self.get(item)?.as_adaptive_image_view().ok()
    }

    pub fn cached_entries(&self) -> usize {
        self.cache.len()
    }

    pub fn cached_bytes(&self) -> usize {
        self.cache.used_bytes()
    }

    pub fn worker_available(&self) -> bool {
        self.worker_available
    }

    pub fn has_pending_work(&self) -> bool {
        self.worker_available && !self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(dpi: u32, theme_generation: u64) -> DesktopIconContext {
        DesktopIconContext::new(DesktopShellIconSize::Large, dpi, theme_generation).unwrap()
    }

    fn key(
        item_id: &str,
        identity_stability: DesktopItemIdentityStability,
        dpi: u32,
        theme_generation: u64,
    ) -> DesktopIconCacheKey {
        DesktopIconCacheKey::for_test(
            item_id,
            identity_stability,
            context(dpi, theme_generation),
        )
    }

    fn image(red: u8) -> DesktopShellIconRgba {
        DesktopShellIconRgba::from_pixels(1, 1, vec![red, 20, 30, 255]).unwrap()
    }

    #[test]
    fn cache_keys_isolate_dpi_theme_and_identity_stability() {
        let mut cache = DesktopIconCache::new(1024, 8);
        let stable = key(
            "desktop:file:stable",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        assert!(cache.insert(stable.clone(), image(12)));
        assert_eq!(cache.get(&stable).unwrap().pixels()[0], 12);
        assert!(cache
            .get(&key(
                "desktop:file:stable",
                DesktopItemIdentityStability::FilesystemFileId,
                144,
                0,
            ))
            .is_none());
        assert!(cache
            .get(&key(
                "desktop:file:stable",
                DesktopItemIdentityStability::FilesystemFileId,
                96,
                1,
            ))
            .is_none());
        let small_icon_key = DesktopIconCacheKey::for_test(
            "desktop:file:stable",
            DesktopItemIdentityStability::FilesystemFileId,
            DesktopIconContext::new(DesktopShellIconSize::Small, 96, 0).unwrap(),
        );
        assert!(cache.get(&small_icon_key).is_none());
        assert!(cache
            .get(&key(
                "desktop:file:stable",
                DesktopItemIdentityStability::PathFallback,
                96,
                0,
            ))
            .is_none());
    }

    #[test]
    fn cache_enforces_byte_and_entry_limits_with_lru_eviction() {
        let mut cache = DesktopIconCache::new(8, 2);
        let first = key(
            "desktop:file:first",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        let second = key(
            "desktop:file:second",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        let third = key(
            "desktop:file:third",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );

        assert!(cache.insert(first.clone(), image(1)));
        assert!(cache.insert(second.clone(), image(2)));
        assert_eq!(cache.get(&first).unwrap().pixels()[0], 1);
        assert!(cache.insert(third.clone(), image(3)));

        assert!(cache.get(&first).is_some());
        assert!(cache.get(&second).is_none());
        assert!(cache.get(&third).is_some());
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.used_bytes(), 8);
        assert!(!cache.insert(
            key(
                "desktop:file:large",
                DesktopItemIdentityStability::FilesystemFileId,
                96,
                0,
            ),
            DesktopShellIconRgba::from_pixels(2, 1, vec![0; 8]).unwrap(),
        ));
    }

    #[test]
    fn inventory_reconciliation_drops_renamed_fallback_ids_but_keeps_file_ids() {
        let mut cache = DesktopIconCache::new(1024, 8);
        let file_id = key(
            "desktop:file:stable",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        let old_fallback = key(
            "desktop:path:old-name",
            DesktopItemIdentityStability::PathFallback,
            96,
            0,
        );
        assert!(cache.insert(file_id.clone(), image(1)));
        assert!(cache.insert(old_fallback.clone(), image(2)));

        let current = HashSet::from([
            (
                "desktop:file:stable".to_owned(),
                DesktopItemIdentityStability::FilesystemFileId,
            ),
            (
                "desktop:path:new-name".to_owned(),
                DesktopItemIdentityStability::PathFallback,
            ),
        ]);
        cache.retain_identities(&current);

        assert!(cache.contains(&file_id));
        assert!(!cache.contains(&old_fallback));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn path_fallback_epoch_changes_without_changing_filesystem_id_keys() {
        let fallback_old = DesktopIconCacheKey::for_test_with_fallback_generation(
            "desktop:path:notes",
            DesktopItemIdentityStability::PathFallback,
            context(96, 0),
            0,
        );
        let fallback_new = DesktopIconCacheKey::for_test_with_fallback_generation(
            "desktop:path:notes",
            DesktopItemIdentityStability::PathFallback,
            context(96, 0),
            1,
        );
        let filesystem_old = DesktopIconCacheKey::for_test_with_fallback_generation(
            "desktop:file:notes",
            DesktopItemIdentityStability::FilesystemFileId,
            context(96, 0),
            0,
        );
        let filesystem_new = DesktopIconCacheKey::for_test_with_fallback_generation(
            "desktop:file:notes",
            DesktopItemIdentityStability::FilesystemFileId,
            context(96, 0),
            1,
        );

        assert_ne!(fallback_old, fallback_new);
        assert_eq!(filesystem_old, filesystem_new);
    }

    #[test]
    fn path_fallback_invalidation_keeps_only_filesystem_backed_icons() {
        let mut cache = DesktopIconCache::new(1024, 8);
        let file_id = key(
            "desktop:file:stable",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        let fallback = key(
            "desktop:path:notes",
            DesktopItemIdentityStability::PathFallback,
            96,
            0,
        );
        assert!(cache.insert(file_id.clone(), image(1)));
        assert!(cache.insert(fallback.clone(), image(2)));

        cache.retain_filesystem_backed();

        assert!(cache.contains(&file_id));
        assert!(!cache.contains(&fallback));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.used_bytes(), 4);
    }

    #[test]
    fn cache_rejects_images_larger_than_its_memory_budget() {
        let mut cache = DesktopIconCache::new(3, 2);
        let cache_key = key(
            "desktop:file:one",
            DesktopItemIdentityStability::FilesystemFileId,
            96,
            0,
        );
        assert!(!cache.insert(cache_key, image(1)));
        assert_eq!(cache.used_bytes(), 0);
        assert_eq!(cache.len(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn shell_worker_initializes_com_before_accepting_requests() {
        let pipeline = DesktopIconPipeline::new(context(96, 0))
            .expect("initialize the dedicated Shell worker COM apartment");
        assert!(pipeline.worker_available());
        drop(pipeline);
    }

    #[test]
    fn zero_dpi_context_is_rejected() {
        assert_eq!(
            DesktopIconContext::new(DesktopShellIconSize::Large, 0, 0),
            None
        );
    }
}

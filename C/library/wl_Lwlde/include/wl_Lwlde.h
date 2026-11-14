#ifndef WL_LWLDE_H
#define WL_LWLDE_H

/*
 * Portable WinLook Desktop Engine scene/policy core.
 * C89-compatible and allocation-free in scene/geometry hot paths.
 * Platform presentation, media decoding and OS lifecycle belong in adapters.
 */
#include "wl_Lanimate.h"
#include "wl_Lcpuv.h"
#include "wl_Lgpuv.h"

#define WL_LWDE_VERSION_MAJOR 0
#define WL_LWDE_VERSION_MINOR 1
#define WL_LWDE_VERSION_PATCH 0
#define WL_LWDE_MAX_MONITORS 16U
#define WL_LWDE_MONITOR_ID_CAPACITY 64U
#define WL_LWDE_EXECUTABLE_SHA256_SIZE 32U
#define WL_LWDE_MAX_GAME_MODE_PROFILES 64U

typedef enum wl_Lwlde_result {
    WL_LWDE_OK = 0,
    WL_LWDE_NO_FRAME = 1,
    WL_LWDE_INVALID_ARGUMENT = 2,
    WL_LWDE_INVALID_GEOMETRY = 3,
    WL_LWDE_INVALID_MONITOR = 4,
    WL_LWDE_DUPLICATE_MONITOR = 5,
    WL_LWDE_TOO_MANY_MONITORS = 6,
    WL_LWDE_INTERNAL_ERROR = 7,
    WL_LWDE_INVALID_RESOURCE_POLICY = 8,
    WL_LWDE_INVALID_RESOURCE_SAMPLE = 9,
    WL_LWDE_TOO_MANY_GAME_MODE_PROFILES = 10,
    WL_LWDE_DUPLICATE_GAME_MODE_PROFILE = 11,
    WL_LWDE_INVALID_GAME_MODE_POLICY = 12,
    WL_LWDE_INVALID_GAME_MODE_PROFILE = 13,
    WL_LWDE_INVALID_GAME_MODE_SIGNALS = 14,
    WL_LWDE_INVALID_DESKTOP_GROUP = 15,
    WL_LWDE_TOO_MANY_DESKTOP_GROUPS = 16,
    WL_LWDE_INVALID_APP_ICON = 17,
    WL_LWDE_INVALID_ICON_PIXELS = 18,
    WL_LWDE_INVALID_ICON_OUTPUT = 19
} wl_Lwlde_result;

typedef struct wl_Lwlde_rect {
    double x;
    double y;
    double width;
    double height;
} wl_Lwlde_rect;

typedef struct wl_Lwlde_size {
    double width;
    double height;
} wl_Lwlde_size;

typedef struct wl_Lwlde_placement {
    wl_Lwlde_rect destination;
    wl_Lwlde_rect source_crop;
} wl_Lwlde_placement;

typedef enum wl_Lwlde_scale_mode {
    WL_LWDE_SCALE_FILL = 0,
    WL_LWDE_SCALE_FIT = 1,
    WL_LWDE_SCALE_STRETCH = 2,
    WL_LWDE_SCALE_CENTER = 3
} wl_Lwlde_scale_mode;

/* IDs are adapter-supplied and copied into this fixed-size record. */
typedef struct wl_Lwlde_monitor {
    char id[WL_LWDE_MONITOR_ID_CAPACITY];
    wl_Lwlde_rect bounds;
    wl_Lwlde_rect work_area;
    unsigned int primary;
} wl_Lwlde_monitor;

/* Caller supplies storage. Replacement is validated before the active list changes. */
typedef struct wl_Lwlde_topology {
    wl_Lwlde_monitor *monitors;
    unsigned int count;
    unsigned int capacity;
} wl_Lwlde_topology;

wl_Lwlde_result wl_Lwlde_topology_init(
    wl_Lwlde_topology *topology,
    wl_Lwlde_monitor *storage,
    unsigned int capacity
);
wl_Lwlde_result wl_Lwlde_topology_replace(
    wl_Lwlde_topology *topology,
    const wl_Lwlde_monitor *monitors,
    unsigned int count
);
const wl_Lwlde_monitor *wl_Lwlde_topology_get(
    const wl_Lwlde_topology *topology,
    unsigned int index
);

/* source_crop coordinates are normalized to [0, 1] in source-image space. */
wl_Lwlde_result wl_Lwlde_compute_placement(
    const wl_Lwlde_size *source,
    const wl_Lwlde_rect *viewport,
    wl_Lwlde_scale_mode mode,
    wl_Lwlde_placement *placement_out
);

typedef enum wl_Lwlde_visibility {
    WL_LWDE_VISIBILITY_VISIBLE = 0,
    WL_LWDE_VISIBILITY_CONTRIBUTING = 1,
    WL_LWDE_VISIBILITY_OCCLUDED = 2,
    WL_LWDE_VISIBILITY_UNKNOWN = 3
} wl_Lwlde_visibility;

typedef enum wl_Lwlde_game_mode {
    WL_LWDE_GAME_NORMAL = 0,
    WL_LWDE_GAME_OPTIMIZE = 1,
    WL_LWDE_GAME_SUSPEND = 2
} wl_Lwlde_game_mode;

/* The digest is supplied by the host; filenames are not app identities. */
typedef struct wl_Lwlde_game_mode_profile {
    unsigned char executable_sha256[WL_LWDE_EXECUTABLE_SHA256_SIZE];
    wl_Lwlde_game_mode mode;
} wl_Lwlde_game_mode_profile;

typedef struct wl_Lwlde_game_mode_policy {
    unsigned int fullscreen_enabled;
    wl_Lwlde_game_mode fullscreen_mode;
    unsigned int system_game_mode_enabled;
    wl_Lwlde_game_mode system_game_mode;
} wl_Lwlde_game_mode_policy;

typedef struct wl_Lwlde_game_mode_signals {
    unsigned char executable_sha256[WL_LWDE_EXECUTABLE_SHA256_SIZE];
    unsigned int is_foreground;
    unsigned int is_fullscreen;
    unsigned int system_game_mode_active;
} wl_Lwlde_game_mode_signals;

/* No OS process lookup or file hashing occurs in this portable resolver. */
wl_Lwlde_result wl_Lwlde_game_mode_resolve(
    const wl_Lwlde_game_mode_signals *signals,
    const wl_Lwlde_game_mode_policy *policy,
    const wl_Lwlde_game_mode_profile *profiles,
    unsigned int profile_count,
    wl_Lwlde_game_mode *mode_out
);

typedef enum wl_Lwlde_power_state {
    WL_LWDE_POWER_NORMAL = 0,
    WL_LWDE_POWER_SAVER = 1,
    WL_LWDE_POWER_SUSPENDED = 2
} wl_Lwlde_power_state;

typedef enum wl_Lwlde_frame_action {
    WL_LWDE_FRAME_STATIC_ONLY = 0,
    WL_LWDE_FRAME_ANIMATE = 1,
    WL_LWDE_FRAME_HOLD_LAST = 2,
    WL_LWDE_FRAME_SUSPENDED = 3
} wl_Lwlde_frame_action;

typedef enum wl_Lwlde_resource_pressure {
    WL_LWDE_RESOURCE_NORMAL = 0,
    WL_LWDE_RESOURCE_OPTIMIZE = 1,
    WL_LWDE_RESOURCE_SUSPEND = 2
} wl_Lwlde_resource_pressure;

typedef struct wl_Lwlde_resource_sample {
    unsigned int cpu_percent;
    unsigned int gpu_percent;
    unsigned int ram_percent;
    unsigned int vram_percent;
} wl_Lwlde_resource_sample;

typedef struct wl_Lwlde_resource_policy {
    unsigned int optimize_at_percent;
    unsigned int suspend_at_percent;
    unsigned int recovery_below_percent;
    unsigned int enter_windows;
    unsigned int recovery_windows;
} wl_Lwlde_resource_policy;

typedef struct wl_Lwlde_resource_controller {
    wl_Lwlde_resource_policy policy;
    wl_Lwlde_resource_pressure pressure;
    unsigned int optimize_streak;
    unsigned int suspend_streak;
    unsigned int recovery_streak;
    unsigned int initialized;
} wl_Lwlde_resource_controller;

/* One scene owns one wl_Lanimate clock shared by all of its animation tracks. */
typedef struct wl_Lwlde_scene {
    wl_Lwlde_visibility visibility;
    wl_Lwlde_game_mode game_mode;
    wl_Lwlde_power_state power_state;
    unsigned int active_animations;
    wl_Lwlde_resource_pressure resource_pressure;
    wl_Lanimate_frame_clock frame_clock;
    unsigned int clock_paused;
    unsigned int initialized;
} wl_Lwlde_scene;

wl_Lwlde_result wl_Lwlde_scene_init(wl_Lwlde_scene *scene);
wl_Lwlde_result wl_Lwlde_scene_set_conditions(
    wl_Lwlde_scene *scene,
    wl_Lwlde_visibility visibility,
    wl_Lwlde_game_mode game_mode,
    wl_Lwlde_power_state power_state,
    unsigned int active_animations
);
wl_Lwlde_result wl_Lwlde_scene_set_game_mode(
    wl_Lwlde_scene *scene,
    wl_Lwlde_game_mode game_mode
);
wl_Lwlde_result wl_Lwlde_scene_set_resource_pressure(
    wl_Lwlde_scene *scene,
    wl_Lwlde_resource_pressure pressure
);
wl_Lwlde_frame_action wl_Lwlde_scene_frame_action(const wl_Lwlde_scene *scene);
unsigned int wl_Lwlde_scene_recommended_fps(const wl_Lwlde_scene *scene);
/* Returns NO_FRAME while idle/occluded/suspended or before the next cadence. */
wl_Lwlde_result wl_Lwlde_scene_tick(
    wl_Lwlde_scene *scene,
    double monotonic_now_ms,
    wl_Lanimate_frame_tick *tick_out
);

typedef enum wl_Lwlde_content_kind {
    WL_LWDE_CONTENT_STATIC_IMAGE = 0,
    WL_LWDE_CONTENT_ANIMATED = 1,
    WL_LWDE_CONTENT_INTERACTIVE = 2
} wl_Lwlde_content_kind;

typedef struct wl_Lwlde_presentation_capabilities {
    unsigned int static_wallpaper;
    unsigned int animated_wallpaper;
    unsigned int interactive_content;
    unsigned int visibility_feedback;
    unsigned int virtual_groups;
    unsigned int crash_safe_group_restore;
} wl_Lwlde_presentation_capabilities;

int wl_Lwlde_content_supported(
    const wl_Lwlde_presentation_capabilities *capabilities,
    wl_Lwlde_content_kind kind
);
/* Groups are enabled only when both display and crash-safe restore are proven. */
int wl_Lwlde_groups_available(
    const wl_Lwlde_presentation_capabilities *capabilities
);

/*
 * Provider metadata for the integrated Wallpaper Store.
 * These records contain links and licensing guidance only; they do not download
 * content or override Steam/provider subscriptions or item-level terms.
 */
#define WL_LWDE_STORE_PROVIDER_COUNT 4U

typedef enum wl_Lwlde_store_provider_id {
    WL_LWDE_STORE_WALLPAPER_ENGINE = 0,
    WL_LWDE_STORE_OPENVERSE = 1,
    WL_LWDE_STORE_WIKIMEDIA_COMMONS = 2,
    WL_LWDE_STORE_NASA_IMAGES = 3
} wl_Lwlde_store_provider_id;

typedef enum wl_Lwlde_store_access {
    WL_LWDE_STORE_ACCESS_BROWSER = 0,
    WL_LWDE_STORE_ACCESS_INSTALLED_APPLICATION = 1
} wl_Lwlde_store_access;

typedef enum wl_Lwlde_store_price_model {
    WL_LWDE_STORE_PRICE_FREE = 0,
    WL_LWDE_STORE_PRICE_MIXED = 1
} wl_Lwlde_store_price_model;

typedef struct wl_Lwlde_store_provider {
    wl_Lwlde_store_provider_id id;
    const char *name;
    const char *description;
    const char *browse_url;
    wl_Lwlde_store_access access;
    wl_Lwlde_store_price_model price_model;
    const char *rights_note;
} wl_Lwlde_store_provider;

/* Returns one immutable built-in provider record, or NULL for an invalid index. */
const wl_Lwlde_store_provider *wl_Lwlde_store_provider_get(unsigned int index);

/* Non-owning references to the existing CPUv/GPUv governors. Caller serializes access. */
typedef struct wl_Lwlde_resource_context {
    wl_Lcpuv_governor *cpu;
    wl_Lgpuv_governor *gpu;
} wl_Lwlde_resource_context;

wl_Lwlde_result wl_Lwlde_resource_context_init(
    wl_Lwlde_resource_context *context,
    wl_Lcpuv_governor *cpu,
    wl_Lgpuv_governor *gpu
);
wl_Lwlde_result wl_Lwlde_resource_controller_init(
    wl_Lwlde_resource_controller *controller,
    const wl_Lwlde_resource_policy *policy
);
wl_Lwlde_result wl_Lwlde_resource_controller_update(
    wl_Lwlde_resource_controller *controller,
    const wl_Lwlde_resource_sample *sample,
    wl_Lwlde_resource_pressure *pressure_out
);


/*
 * Desktop groups are virtual references. Item IDs are supplied by the host;
 * these APIs never move, rename, hide or delete filesystem/Shell items.
 */
#define WL_LWDE_MAX_DESKTOP_GROUPS 64U
#define WL_LWDE_MAX_DESKTOP_GROUP_MEMBERS 128U
#define WL_LWDE_DESKTOP_ID_CAPACITY 128U
#define WL_LWDE_DESKTOP_LABEL_CAPACITY 256U
#define WL_LWDE_ICON_ASSET_CAPACITY 256U
#define WL_LWDE_DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT 61U

typedef enum wl_Lwlde_desktop_item_kind {
    WL_LWDE_DESKTOP_FILE = 0,
    WL_LWDE_DESKTOP_FOLDER = 1,
    WL_LWDE_DESKTOP_APPLICATION = 2
} wl_Lwlde_desktop_item_kind;

typedef struct wl_Lwlde_desktop_item_ref {
    char id[WL_LWDE_DESKTOP_ID_CAPACITY];
    char label[WL_LWDE_DESKTOP_LABEL_CAPACITY];
    wl_Lwlde_desktop_item_kind kind;
} wl_Lwlde_desktop_item_ref;

/* member_ids is caller-owned and must outlive the group record in the set. */
typedef struct wl_Lwlde_desktop_group {
    char id[WL_LWDE_DESKTOP_ID_CAPACITY];
    char label[WL_LWDE_DESKTOP_LABEL_CAPACITY];
    const char *const *member_ids;
    unsigned int member_count;
} wl_Lwlde_desktop_group;

/* Caller supplies bounded storage; replacement validates fully before commit. */
typedef struct wl_Lwlde_desktop_groups {
    wl_Lwlde_desktop_group *groups;
    unsigned int count;
    unsigned int capacity;
} wl_Lwlde_desktop_groups;

wl_Lwlde_result wl_Lwlde_desktop_groups_init(
    wl_Lwlde_desktop_groups *groups,
    wl_Lwlde_desktop_group *storage,
    unsigned int capacity
);
wl_Lwlde_result wl_Lwlde_desktop_groups_replace(
    wl_Lwlde_desktop_groups *groups,
    const wl_Lwlde_desktop_group *replacement,
    unsigned int replacement_count,
    const wl_Lwlde_desktop_item_ref *available_items,
    unsigned int available_item_count
);
const wl_Lwlde_desktop_group *wl_Lwlde_desktop_groups_find(
    const wl_Lwlde_desktop_groups *groups,
    const char *group_id
);
int wl_Lwlde_desktop_group_contains(
    const wl_Lwlde_desktop_group *group,
    const char *item_id
);

typedef enum wl_Lwlde_app_icon_mask {
    WL_LWDE_APP_ICON_CIRCLE = 0,
    WL_LWDE_APP_ICON_ROUNDED_SQUARE = 1,
    WL_LWDE_APP_ICON_SQUIRCLE = 2
} wl_Lwlde_app_icon_mask;

/*
 * Foreground/background/optional monochrome are opaque asset references.
 * Rendering applies the mask and safe zone without rewriting source assets.
 */
typedef struct wl_Lwlde_adaptive_app_icon {
    const char *application_id;
    const char *label;
    const char *foreground_asset;
    const char *background_asset;
    const char *monochrome_asset;
    wl_Lwlde_app_icon_mask mask;
    unsigned int safe_zone_percent;
} wl_Lwlde_adaptive_app_icon;

wl_Lwlde_result wl_Lwlde_adaptive_app_icon_validate(
    const wl_Lwlde_adaptive_app_icon *icon
);

#define WL_LWDE_MAX_ADAPTIVE_ICON_SOURCE_DIMENSION 4096U
#define WL_LWDE_MIN_ADAPTIVE_ICON_OUTPUT_SIZE 16U
#define WL_LWDE_MAX_ADAPTIVE_ICON_OUTPUT_SIZE 512U

/* Tightly packed, row-major, straight-alpha RGBA8 pixels. */
typedef struct wl_Lwlde_rgba_image_view {
    const unsigned char *pixels;
    unsigned int width;
    unsigned int height;
    unsigned int pixel_length;
} wl_Lwlde_rgba_image_view;

/*
 * Decoded pixel views are supplied by the platform adapter. The caller owns
 * output_rgba and must provide at least output_size * output_size * 4 bytes.
 * The output must not overlap either source pixel buffer. No allocation or
 * source-file mutation occurs in this function.
 */
wl_Lwlde_result wl_Lwlde_adaptive_app_icon_render_rgba(
    const wl_Lwlde_adaptive_app_icon *icon,
    const wl_Lwlde_rgba_image_view *background,
    const wl_Lwlde_rgba_image_view *foreground,
    unsigned int output_size,
    unsigned char *output_rgba,
    unsigned int output_capacity
);


typedef enum wl_Lwlde_desktop_context_target {
    WL_LWDE_CONTEXT_FILE = 0,
    WL_LWDE_CONTEXT_FOLDER = 1,
    WL_LWDE_CONTEXT_APPLICATION = 2,
    WL_LWDE_CONTEXT_GROUP = 3,
    WL_LWDE_CONTEXT_DESKTOP_BACKGROUND = 4
} wl_Lwlde_desktop_context_target;

typedef enum wl_Lwlde_desktop_context_action {
    WL_LWDE_CONTEXT_OPEN = 0,
    WL_LWDE_CONTEXT_ADD_TO_GROUP = 1,
    WL_LWDE_CONTEXT_REMOVE_FROM_GROUP = 2,
    WL_LWDE_CONTEXT_CUSTOMIZE_APPLICATION_ICON = 3,
    WL_LWDE_CONTEXT_OPEN_GROUP = 4,
    WL_LWDE_CONTEXT_ADD_GROUP_MEMBERS = 5,
    WL_LWDE_CONTEXT_RENAME_GROUP = 6,
    WL_LWDE_CONTEXT_UNGROUP = 7,
    WL_LWDE_CONTEXT_PROPERTIES = 8,
    WL_LWDE_CONTEXT_CREATE_GROUP = 9,
    WL_LWDE_CONTEXT_OPEN_DESKTOP_SETTINGS = 10
} wl_Lwlde_desktop_context_action;

/*
 * Returns the required action count when actions_out is NULL. If a buffer is
 * supplied, it must have action_capacity slots; insufficient capacity returns
 * zero without writing any actions. Native Open/Properties remain Shell-owned.
 * Desktop-background entries provide WLDE Create Group and Desktop Settings;
 * native View, Sort, Refresh, New, wallpaper and Display Settings remain Shell-owned.
 */
unsigned int wl_Lwlde_desktop_context_actions(
    wl_Lwlde_desktop_context_target target,
    unsigned int is_group_member,
    wl_Lwlde_desktop_context_action *actions_out,
    unsigned int action_capacity
);

#endif /* WL_LWLDE_H */

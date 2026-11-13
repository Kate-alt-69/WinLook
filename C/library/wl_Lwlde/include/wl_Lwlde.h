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

typedef enum wl_Lwlde_result {
    WL_LWDE_OK = 0,
    WL_LWDE_NO_FRAME = 1,
    WL_LWDE_INVALID_ARGUMENT = 2,
    WL_LWDE_INVALID_GEOMETRY = 3,
    WL_LWDE_INVALID_MONITOR = 4,
    WL_LWDE_DUPLICATE_MONITOR = 5,
    WL_LWDE_TOO_MANY_MONITORS = 6,
    WL_LWDE_INTERNAL_ERROR = 7
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

/* One scene owns one wl_Lanimate clock shared by all of its animation tracks. */
typedef struct wl_Lwlde_scene {
    wl_Lwlde_visibility visibility;
    wl_Lwlde_game_mode game_mode;
    wl_Lwlde_power_state power_state;
    unsigned int active_animations;
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

#endif /* WL_LWLDE_H */

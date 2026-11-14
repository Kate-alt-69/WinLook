#include "wl_Lwlde.h"

#include <float.h>
#include <string.h>

static int wl_Lwlde_finite(double value)
{
    return value == value && value <= DBL_MAX && value >= -DBL_MAX;
}

static int wl_Lwlde_valid_rect(const wl_Lwlde_rect *rect)
{
    return rect != 0 && wl_Lwlde_finite(rect->x) && wl_Lwlde_finite(rect->y) &&
        wl_Lwlde_finite(rect->width) && wl_Lwlde_finite(rect->height) &&
        rect->width > 0.0 && rect->height > 0.0;
}

static int wl_Lwlde_valid_monitor(const wl_Lwlde_monitor *monitor)
{
    unsigned int i;
    int terminated;
    if (monitor == 0 || monitor->primary > 1U ||
        !wl_Lwlde_valid_rect(&monitor->bounds) ||
        !wl_Lwlde_valid_rect(&monitor->work_area)) return 0;
    terminated = 0;
    for (i = 0U; i < WL_LWDE_MONITOR_ID_CAPACITY; ++i) {
        if (monitor->id[i] == '\0') {
            terminated = i > 0U;
            break;
        }
    }
    return terminated;
}

wl_Lwlde_result wl_Lwlde_topology_init(wl_Lwlde_topology *topology,
    wl_Lwlde_monitor *storage, unsigned int capacity)
{
    unsigned int i;
    if (topology == 0 || storage == 0 || capacity == 0U) return WL_LWDE_INVALID_ARGUMENT;
    topology->monitors = storage;
    topology->count = 0U;
    topology->capacity = capacity < WL_LWDE_MAX_MONITORS ? capacity : WL_LWDE_MAX_MONITORS;
    for (i = 0U; i < topology->capacity; ++i) {
        storage[i].id[0] = '\0';
        storage[i].primary = 0U;
    }
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_topology_replace(wl_Lwlde_topology *topology,
    const wl_Lwlde_monitor *monitors, unsigned int count)
{
    unsigned int i;
    unsigned int j;
    unsigned int primary_count;
    if (topology == 0 || topology->monitors == 0 || (count > 0U && monitors == 0))
        return WL_LWDE_INVALID_ARGUMENT;
    if (count > topology->capacity || count > WL_LWDE_MAX_MONITORS)
        return WL_LWDE_TOO_MANY_MONITORS;
    primary_count = 0U;
    for (i = 0U; i < count; ++i) {
        if (!wl_Lwlde_valid_monitor(&monitors[i])) return WL_LWDE_INVALID_MONITOR;
        if (monitors[i].primary) ++primary_count;
        for (j = 0U; j < i; ++j) {
            if (strcmp(monitors[i].id, monitors[j].id) == 0)
                return WL_LWDE_DUPLICATE_MONITOR;
        }
    }
    if (primary_count > 1U) return WL_LWDE_INVALID_MONITOR;
    if (count > 0U) memcpy(topology->monitors, monitors, count * sizeof(*monitors));
    for (i = count; i < topology->count; ++i) topology->monitors[i].id[0] = '\0';
    topology->count = count;
    return WL_LWDE_OK;
}

const wl_Lwlde_monitor *wl_Lwlde_topology_get(const wl_Lwlde_topology *topology,
    unsigned int index)
{
    if (topology == 0 || topology->monitors == 0 || index >= topology->count) return 0;
    return &topology->monitors[index];
}

wl_Lwlde_result wl_Lwlde_compute_placement(const wl_Lwlde_size *source,
    const wl_Lwlde_rect *viewport, wl_Lwlde_scale_mode mode, wl_Lwlde_placement *out)
{
    double source_ratio;
    double viewport_ratio;
    double crop_ratio;
    double width;
    double height;
    if (source == 0 || out == 0 || !wl_Lwlde_finite(source->width) ||
        !wl_Lwlde_finite(source->height) || source->width <= 0.0 || source->height <= 0.0 ||
        !wl_Lwlde_valid_rect(viewport) || (int)mode < (int)WL_LWDE_SCALE_FILL ||
        (int)mode > (int)WL_LWDE_SCALE_CENTER) return WL_LWDE_INVALID_GEOMETRY;

    out->destination = *viewport;
    out->source_crop.x = 0.0;
    out->source_crop.y = 0.0;
    out->source_crop.width = 1.0;
    out->source_crop.height = 1.0;
    source_ratio = source->width / source->height;
    viewport_ratio = viewport->width / viewport->height;
    if (!wl_Lwlde_finite(source_ratio) || !wl_Lwlde_finite(viewport_ratio) ||
        source_ratio <= 0.0 || viewport_ratio <= 0.0) return WL_LWDE_INVALID_GEOMETRY;

    if (mode == WL_LWDE_SCALE_FILL) {
        if (source_ratio > viewport_ratio) {
            crop_ratio = viewport_ratio / source_ratio;
            out->source_crop.width = crop_ratio;
            out->source_crop.x = (1.0 - crop_ratio) / 2.0;
        } else if (source_ratio < viewport_ratio) {
            crop_ratio = source_ratio / viewport_ratio;
            out->source_crop.height = crop_ratio;
            out->source_crop.y = (1.0 - crop_ratio) / 2.0;
        }
    } else if (mode == WL_LWDE_SCALE_FIT) {
        if (source_ratio > viewport_ratio) {
            width = viewport->width;
            height = width / source_ratio;
        } else {
            height = viewport->height;
            width = height * source_ratio;
        }
        out->destination.x = viewport->x + (viewport->width - width) / 2.0;
        out->destination.y = viewport->y + (viewport->height - height) / 2.0;
        out->destination.width = width;
        out->destination.height = height;
    } else if (mode == WL_LWDE_SCALE_CENTER) {
        out->destination.x = viewport->x + (viewport->width - source->width) / 2.0;
        out->destination.y = viewport->y + (viewport->height - source->height) / 2.0;
        out->destination.width = source->width;
        out->destination.height = source->height;
    }
    if (!wl_Lwlde_valid_rect(&out->destination) ||
        !wl_Lwlde_valid_rect(&out->source_crop)) return WL_LWDE_INVALID_GEOMETRY;
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_scene_init(wl_Lwlde_scene *scene)
{
    wl_Lanimate_result result;
    if (scene == 0) return WL_LWDE_INVALID_ARGUMENT;
    scene->visibility = WL_LWDE_VISIBILITY_UNKNOWN;
    scene->game_mode = WL_LWDE_GAME_NORMAL;
    scene->power_state = WL_LWDE_POWER_NORMAL;
    scene->active_animations = 0U;
    scene->resource_pressure = WL_LWDE_RESOURCE_NORMAL;
    scene->clock_paused = 0U;
    scene->initialized = 0U;
    result = wl_Lanimate_frame_clock_init(&scene->frame_clock,
        WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS, WL_LANIMATE_FRAME_DEFAULT_MAXIMUM_DELTA_MS);
    if (result != WL_LANIMATE_OK) return WL_LWDE_INTERNAL_ERROR;
    scene->initialized = 1U;
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_scene_set_conditions(wl_Lwlde_scene *scene,
    wl_Lwlde_visibility visibility, wl_Lwlde_game_mode game_mode,
    wl_Lwlde_power_state power_state, unsigned int active_animations)
{
    if (scene == 0 || !scene->initialized ||
        (int)visibility < (int)WL_LWDE_VISIBILITY_VISIBLE ||
        (int)visibility > (int)WL_LWDE_VISIBILITY_UNKNOWN ||
        (int)game_mode < (int)WL_LWDE_GAME_NORMAL ||
        (int)game_mode > (int)WL_LWDE_GAME_SUSPEND ||
        (int)power_state < (int)WL_LWDE_POWER_NORMAL ||
        (int)power_state > (int)WL_LWDE_POWER_SUSPENDED) return WL_LWDE_INVALID_ARGUMENT;
    scene->visibility = visibility;
    scene->game_mode = game_mode;
    scene->power_state = power_state;
    scene->active_animations = active_animations;
    return WL_LWDE_OK;
}

static int wl_Lwlde_valid_game_mode(wl_Lwlde_game_mode mode)
{
    return (int)mode >= (int)WL_LWDE_GAME_NORMAL &&
        (int)mode <= (int)WL_LWDE_GAME_SUSPEND;
}

wl_Lwlde_result wl_Lwlde_game_mode_resolve(
    const wl_Lwlde_game_mode_signals *signals,
    const wl_Lwlde_game_mode_policy *policy,
    const wl_Lwlde_game_mode_profile *profiles,
    unsigned int profile_count,
    wl_Lwlde_game_mode *mode_out)
{
    unsigned int i;
    unsigned int j;
    if (signals == 0 || policy == 0 || mode_out == 0 ||
        (profile_count > 0U && profiles == 0)) return WL_LWDE_INVALID_ARGUMENT;
    if (profile_count > WL_LWDE_MAX_GAME_MODE_PROFILES)
        return WL_LWDE_TOO_MANY_GAME_MODE_PROFILES;
    if (policy->fullscreen_enabled > 1U || policy->system_game_mode_enabled > 1U ||
        !wl_Lwlde_valid_game_mode(policy->fullscreen_mode) ||
        !wl_Lwlde_valid_game_mode(policy->system_game_mode))
        return WL_LWDE_INVALID_GAME_MODE_POLICY;
    if (signals->is_foreground > 1U || signals->is_fullscreen > 1U ||
        signals->system_game_mode_active > 1U)
        return WL_LWDE_INVALID_GAME_MODE_SIGNALS;

    for (i = 0U; i < profile_count; ++i) {
        if (!wl_Lwlde_valid_game_mode(profiles[i].mode))
            return WL_LWDE_INVALID_GAME_MODE_PROFILE;
        for (j = 0U; j < i; ++j) {
            if (memcmp(profiles[i].executable_sha256,
                profiles[j].executable_sha256,
                WL_LWDE_EXECUTABLE_SHA256_SIZE) == 0)
                return WL_LWDE_DUPLICATE_GAME_MODE_PROFILE;
        }
    }

    *mode_out = WL_LWDE_GAME_NORMAL;
    if (signals->is_foreground == 0U) return WL_LWDE_OK;

    for (i = 0U; i < profile_count; ++i) {
        if (memcmp(signals->executable_sha256, profiles[i].executable_sha256,
            WL_LWDE_EXECUTABLE_SHA256_SIZE) == 0) {
            *mode_out = profiles[i].mode;
            return WL_LWDE_OK;
        }
    }

    if (signals->system_game_mode_active != 0U &&
        policy->system_game_mode_enabled != 0U) {
        *mode_out = policy->system_game_mode;
        return WL_LWDE_OK;
    }
    if (signals->is_fullscreen != 0U && policy->fullscreen_enabled != 0U) {
        *mode_out = policy->fullscreen_mode;
        return WL_LWDE_OK;
    }
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_scene_set_game_mode(wl_Lwlde_scene *scene,
    wl_Lwlde_game_mode game_mode)
{
    if (scene == 0 || !scene->initialized || !wl_Lwlde_valid_game_mode(game_mode))
        return WL_LWDE_INVALID_ARGUMENT;
    scene->game_mode = game_mode;
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_scene_set_resource_pressure(wl_Lwlde_scene *scene,
    wl_Lwlde_resource_pressure pressure)
{
    if (scene == 0 || !scene->initialized ||
        (int)pressure < (int)WL_LWDE_RESOURCE_NORMAL ||
        (int)pressure > (int)WL_LWDE_RESOURCE_SUSPEND) return WL_LWDE_INVALID_ARGUMENT;
    scene->resource_pressure = pressure;
    return WL_LWDE_OK;
}

wl_Lwlde_frame_action wl_Lwlde_scene_frame_action(const wl_Lwlde_scene *scene)
{
    if (scene == 0 || !scene->initialized) return WL_LWDE_FRAME_SUSPENDED;
    if (scene->resource_pressure == WL_LWDE_RESOURCE_SUSPEND ||
        scene->game_mode == WL_LWDE_GAME_SUSPEND ||
        scene->power_state == WL_LWDE_POWER_SUSPENDED) return WL_LWDE_FRAME_SUSPENDED;
    if (scene->visibility == WL_LWDE_VISIBILITY_OCCLUDED) return WL_LWDE_FRAME_HOLD_LAST;
    if (scene->active_animations == 0U) return WL_LWDE_FRAME_STATIC_ONLY;
    return WL_LWDE_FRAME_ANIMATE;
}

unsigned int wl_Lwlde_scene_recommended_fps(const wl_Lwlde_scene *scene)
{
    wl_Lwlde_frame_action action;
    if (scene == 0 || !scene->initialized) return 0U;
    action = wl_Lwlde_scene_frame_action(scene);
    if (action != WL_LWDE_FRAME_ANIMATE) return 0U;
    if (scene->resource_pressure == WL_LWDE_RESOURCE_OPTIMIZE ||
        scene->game_mode == WL_LWDE_GAME_OPTIMIZE ||
        scene->power_state == WL_LWDE_POWER_SAVER) return 30U;
    return 60U;
}

wl_Lwlde_result wl_Lwlde_scene_tick(wl_Lwlde_scene *scene,
    double now_ms, wl_Lanimate_frame_tick *tick)
{
    double target_interval;
    double interval_difference;
    unsigned int fps;
    wl_Lanimate_result result;
    if (scene == 0 || tick == 0 || !scene->initialized || !wl_Lwlde_finite(now_ms))
        return WL_LWDE_INVALID_ARGUMENT;
    fps = wl_Lwlde_scene_recommended_fps(scene);
    if (fps == 0U) {
        if (!scene->clock_paused) wl_Lanimate_frame_clock_pause(&scene->frame_clock);
        scene->clock_paused = 1U;
        return WL_LWDE_NO_FRAME;
    }
    target_interval = fps == 30U ? 33.333330 : WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS;
    interval_difference = scene->frame_clock.target_interval_ms - target_interval;
    if (interval_difference < 0.0) interval_difference = -interval_difference;
    if (interval_difference > 0.001) {
        result = wl_Lanimate_frame_clock_init(&scene->frame_clock, target_interval,
            WL_LANIMATE_FRAME_DEFAULT_MAXIMUM_DELTA_MS);
        if (result != WL_LANIMATE_OK) return WL_LWDE_INTERNAL_ERROR;
        scene->clock_paused = 1U;
    }
    if (scene->clock_paused) {
        result = wl_Lanimate_frame_clock_resume(&scene->frame_clock, now_ms);
        if (result != WL_LANIMATE_OK) return WL_LWDE_INTERNAL_ERROR;
        scene->clock_paused = 0U;
        return WL_LWDE_NO_FRAME;
    }
    result = wl_Lanimate_frame_clock_tick(&scene->frame_clock, now_ms, tick);
    if (result == WL_LANIMATE_NO_FRAME) return WL_LWDE_NO_FRAME;
    return result == WL_LANIMATE_OK ? WL_LWDE_OK : WL_LWDE_INTERNAL_ERROR;
}

int wl_Lwlde_content_supported(const wl_Lwlde_presentation_capabilities *caps,
    wl_Lwlde_content_kind kind)
{
    if (caps == 0) return 0;
    if (kind == WL_LWDE_CONTENT_STATIC_IMAGE) return caps->static_wallpaper != 0U;
    if (kind == WL_LWDE_CONTENT_ANIMATED) return caps->animated_wallpaper != 0U;
    if (kind == WL_LWDE_CONTENT_INTERACTIVE) return caps->interactive_content != 0U;
    return 0;
}

int wl_Lwlde_groups_available(const wl_Lwlde_presentation_capabilities *caps)
{
    return caps != 0 && caps->virtual_groups != 0U &&
        caps->crash_safe_group_restore != 0U;
}

static const wl_Lwlde_store_provider wl_Lwlde_store_providers[WL_LWDE_STORE_PROVIDER_COUNT] = {
    {
        WL_LWDE_STORE_WALLPAPER_ENGINE,
        "Wallpaper Engine",
        "Browse Steam Workshop and use wallpapers already downloaded by Steam.",
        "https://steamcommunity.com/workshop/browse/?appid=431960",
        WL_LWDE_STORE_ACCESS_INSTALLED_APPLICATION,
        WL_LWDE_STORE_PRICE_MIXED,
        "Steam manages subscriptions and purchases. Follow each creator's license; do not unpack or redistribute protected packages."
    },
    {
        WL_LWDE_STORE_OPENVERSE,
        "Openverse",
        "Search openly licensed images from multiple sources.",
        "https://openverse.org/search/image?q=wallpaper",
        WL_LWDE_STORE_ACCESS_BROWSER,
        WL_LWDE_STORE_PRICE_FREE,
        "Check the individual result's license, creator, source and attribution requirements before importing or sharing."
    },
    {
        WL_LWDE_STORE_WIKIMEDIA_COMMONS,
        "Wikimedia Commons",
        "Browse freely licensed and public-domain media.",
        "https://commons.wikimedia.org/wiki/Special:MediaSearch?type=image&search=wallpaper",
        WL_LWDE_STORE_ACCESS_BROWSER,
        WL_LWDE_STORE_PRICE_FREE,
        "Licenses vary by file. Preserve attribution and follow the exact file page's license terms."
    },
    {
        WL_LWDE_STORE_NASA_IMAGES,
        "NASA Image and Video Library",
        "Explore NASA imagery and public science media.",
        "https://images.nasa.gov/search?q=wallpaper",
        WL_LWDE_STORE_ACCESS_BROWSER,
        WL_LWDE_STORE_PRICE_FREE,
        "NASA media is generally reusable, but check each item's credit and third-party rights notices; NASA endorsement is not implied."
    }
};

const wl_Lwlde_store_provider *wl_Lwlde_store_provider_get(unsigned int index)
{
    if (index >= WL_LWDE_STORE_PROVIDER_COUNT) return 0;
    return &wl_Lwlde_store_providers[index];
}

wl_Lwlde_result wl_Lwlde_resource_context_init(wl_Lwlde_resource_context *context,
    wl_Lcpuv_governor *cpu, wl_Lgpuv_governor *gpu)
{
    if (context == 0 || cpu == 0 || gpu == 0) return WL_LWDE_INVALID_ARGUMENT;
    context->cpu = cpu;
    context->gpu = gpu;
    return WL_LWDE_OK;
}

static unsigned int wl_Lwlde_resource_advance(unsigned int current, unsigned int required)
{
    return current < required ? current + 1U : required;
}

static unsigned int wl_Lwlde_resource_peak(const wl_Lwlde_resource_sample *sample)
{
    unsigned int peak;
    peak = sample->cpu_percent;
    if (sample->gpu_percent > peak) peak = sample->gpu_percent;
    if (sample->ram_percent > peak) peak = sample->ram_percent;
    if (sample->vram_percent > peak) peak = sample->vram_percent;
    return peak;
}

wl_Lwlde_result wl_Lwlde_resource_controller_init(
    wl_Lwlde_resource_controller *controller,
    const wl_Lwlde_resource_policy *policy)
{
    if (controller == 0 || policy == 0) return WL_LWDE_INVALID_ARGUMENT;
    if (policy->recovery_below_percent >= policy->optimize_at_percent ||
        policy->optimize_at_percent >= policy->suspend_at_percent ||
        policy->suspend_at_percent > 100U ||
        policy->enter_windows == 0U || policy->recovery_windows == 0U)
        return WL_LWDE_INVALID_RESOURCE_POLICY;

    controller->policy = *policy;
    controller->pressure = WL_LWDE_RESOURCE_NORMAL;
    controller->optimize_streak = 0U;
    controller->suspend_streak = 0U;
    controller->recovery_streak = 0U;
    controller->initialized = 1U;
    return WL_LWDE_OK;
}

wl_Lwlde_result wl_Lwlde_resource_controller_update(
    wl_Lwlde_resource_controller *controller,
    const wl_Lwlde_resource_sample *sample,
    wl_Lwlde_resource_pressure *pressure_out)
{
    unsigned int peak;
    if (controller == 0 || sample == 0 || pressure_out == 0 ||
        controller->initialized == 0U) return WL_LWDE_INVALID_ARGUMENT;
    if (sample->cpu_percent > 100U || sample->gpu_percent > 100U ||
        sample->ram_percent > 100U || sample->vram_percent > 100U)
        return WL_LWDE_INVALID_RESOURCE_SAMPLE;

    peak = wl_Lwlde_resource_peak(sample);
    if (controller->pressure == WL_LWDE_RESOURCE_SUSPEND) {
        if (peak < controller->policy.recovery_below_percent) {
            controller->recovery_streak = wl_Lwlde_resource_advance(
                controller->recovery_streak, controller->policy.recovery_windows);
            if (controller->recovery_streak >= controller->policy.recovery_windows) {
                controller->pressure = WL_LWDE_RESOURCE_NORMAL;
                controller->optimize_streak = 0U;
                controller->suspend_streak = 0U;
                controller->recovery_streak = 0U;
            }
        } else {
            controller->recovery_streak = 0U;
        }
        *pressure_out = controller->pressure;
        return WL_LWDE_OK;
    }

    if (controller->pressure == WL_LWDE_RESOURCE_OPTIMIZE) {
        if (peak < controller->policy.recovery_below_percent) {
            controller->recovery_streak = wl_Lwlde_resource_advance(
                controller->recovery_streak, controller->policy.recovery_windows);
            if (controller->recovery_streak >= controller->policy.recovery_windows) {
                controller->pressure = WL_LWDE_RESOURCE_NORMAL;
                controller->optimize_streak = 0U;
                controller->suspend_streak = 0U;
                controller->recovery_streak = 0U;
                *pressure_out = controller->pressure;
                return WL_LWDE_OK;
            }
        } else {
            controller->recovery_streak = 0U;
        }
    } else {
        controller->recovery_streak = 0U;
    }

    if (peak >= controller->policy.suspend_at_percent) {
        controller->suspend_streak = wl_Lwlde_resource_advance(
            controller->suspend_streak, controller->policy.enter_windows);
        controller->optimize_streak = 0U;
        if (controller->suspend_streak >= controller->policy.enter_windows) {
            controller->pressure = WL_LWDE_RESOURCE_SUSPEND;
            controller->optimize_streak = 0U;
            controller->suspend_streak = 0U;
            controller->recovery_streak = 0U;
        }
    } else if (peak >= controller->policy.optimize_at_percent) {
        controller->optimize_streak = wl_Lwlde_resource_advance(
            controller->optimize_streak, controller->policy.enter_windows);
        controller->suspend_streak = 0U;
        if (controller->optimize_streak >= controller->policy.enter_windows) {
            controller->pressure = WL_LWDE_RESOURCE_OPTIMIZE;
            controller->optimize_streak = 0U;
        }
    } else {
        controller->optimize_streak = 0U;
        controller->suspend_streak = 0U;
    }

    *pressure_out = controller->pressure;
    return WL_LWDE_OK;
}

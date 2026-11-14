#include "wl_Lwlde.h"

#include <assert.h>
#include <stdio.h>
#include <string.h>

static wl_Lwlde_monitor monitor_make(const char *id, double x, int primary)
{
    wl_Lwlde_monitor monitor;
    size_t length;
    length = strlen(id);
    assert(length < WL_LWDE_MONITOR_ID_CAPACITY);
    memset(&monitor, 0, sizeof(monitor));
    memcpy(monitor.id, id, length + 1U);
    monitor.bounds.x = x;
    monitor.bounds.y = 0.0;
    monitor.bounds.width = 1920.0;
    monitor.bounds.height = 1080.0;
    monitor.work_area = monitor.bounds;
    monitor.primary = (unsigned int)primary;
    return monitor;
}

int main(void)
{
    wl_Lwlde_monitor storage[4];
    wl_Lwlde_monitor inputs[2];
    wl_Lwlde_topology topology;
    wl_Lwlde_rect viewport;
    wl_Lwlde_size source;
    wl_Lwlde_placement placement;
    wl_Lwlde_scene scene;
    wl_Lanimate_frame_tick tick;
    wl_Lwlde_presentation_capabilities caps;
    const wl_Lwlde_store_provider *provider;
    wl_Lwlde_resource_policy resource_policy;
    wl_Lwlde_resource_controller resource_controller;
    wl_Lwlde_resource_sample resource_sample;
    wl_Lwlde_resource_pressure resource_pressure;
    wl_Lwlde_game_mode_profile game_profiles[2];
    wl_Lwlde_game_mode_policy game_policy;
    wl_Lwlde_game_mode_signals game_signals;
    wl_Lwlde_game_mode resolved_game_mode;
    unsigned int i;

    assert(wl_Lwlde_topology_init(&topology, storage, 4U) == WL_LWDE_OK);
    inputs[0] = monitor_make("DISPLAY-A", 0.0, 1);
    inputs[1] = monitor_make("DISPLAY-B", 1920.0, 0);
    assert(wl_Lwlde_topology_replace(&topology, inputs, 2U) == WL_LWDE_OK);
    assert(topology.count == 2U);
    assert(strcmp(wl_Lwlde_topology_get(&topology, 1U)->id, "DISPLAY-B") == 0);

    inputs[1] = inputs[0];
    assert(wl_Lwlde_topology_replace(&topology, inputs, 2U) == WL_LWDE_DUPLICATE_MONITOR);
    inputs[1] = monitor_make("DISPLAY-B", 1920.0, 1);
    assert(wl_Lwlde_topology_replace(&topology, inputs, 2U) == WL_LWDE_INVALID_MONITOR);

    viewport.x = -1920.0;
    viewport.y = 0.0;
    viewport.width = 1920.0;
    viewport.height = 1080.0;
    source.width = 4000.0;
    source.height = 1000.0;
    assert(wl_Lwlde_compute_placement(&source, &viewport, WL_LWDE_SCALE_FILL, &placement) == WL_LWDE_OK);
    assert(placement.destination.x == -1920.0);
    assert(placement.source_crop.width < 1.0);
    assert(wl_Lwlde_compute_placement(&source, &viewport, WL_LWDE_SCALE_FIT, &placement) == WL_LWDE_OK);
    assert(placement.destination.height < 1080.0);
    assert(placement.source_crop.width == 1.0);

    assert(wl_Lwlde_scene_init(&scene) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_set_conditions(&scene, WL_LWDE_VISIBILITY_UNKNOWN,
        WL_LWDE_GAME_NORMAL, WL_LWDE_POWER_NORMAL, 1U) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_frame_action(&scene) == WL_LWDE_FRAME_ANIMATE);
    assert(wl_Lwlde_scene_recommended_fps(&scene) == 60U);
    assert(wl_Lwlde_scene_tick(&scene, 0.0, &tick) == WL_LWDE_OK);
    assert(tick.delta_ms == 0.0);
    assert(wl_Lwlde_scene_tick(&scene, 17.0, &tick) == WL_LWDE_OK);

    assert(wl_Lwlde_scene_set_conditions(&scene, WL_LWDE_VISIBILITY_OCCLUDED,
        WL_LWDE_GAME_NORMAL, WL_LWDE_POWER_NORMAL, 1U) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_frame_action(&scene) == WL_LWDE_FRAME_HOLD_LAST);
    assert(wl_Lwlde_scene_tick(&scene, 34.0, &tick) == WL_LWDE_NO_FRAME);

    assert(wl_Lwlde_scene_set_conditions(&scene, WL_LWDE_VISIBILITY_VISIBLE,
        WL_LWDE_GAME_OPTIMIZE, WL_LWDE_POWER_NORMAL, 1U) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_recommended_fps(&scene) == 30U);
    assert(wl_Lwlde_scene_tick(&scene, 50.0, &tick) == WL_LWDE_NO_FRAME);
    assert(wl_Lwlde_scene_tick(&scene, 84.0, &tick) == WL_LWDE_OK);

    assert(wl_Lwlde_scene_set_conditions(&scene, WL_LWDE_VISIBILITY_VISIBLE,
        WL_LWDE_GAME_SUSPEND, WL_LWDE_POWER_NORMAL, 1U) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_frame_action(&scene) == WL_LWDE_FRAME_SUSPENDED);
    assert(wl_Lwlde_scene_set_conditions(&scene, WL_LWDE_VISIBILITY_VISIBLE,
        WL_LWDE_GAME_NORMAL, WL_LWDE_POWER_NORMAL, 1U) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_set_resource_pressure(&scene, WL_LWDE_RESOURCE_OPTIMIZE) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_recommended_fps(&scene) == 30U);
    assert(wl_Lwlde_scene_set_resource_pressure(&scene, WL_LWDE_RESOURCE_SUSPEND) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_frame_action(&scene) == WL_LWDE_FRAME_SUSPENDED);
    assert(wl_Lwlde_scene_set_resource_pressure(&scene, WL_LWDE_RESOURCE_NORMAL) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_set_game_mode(&scene, WL_LWDE_GAME_OPTIMIZE) == WL_LWDE_OK);
    assert(wl_Lwlde_scene_recommended_fps(&scene) == 30U);

    memset(game_profiles, 0, sizeof(game_profiles));
    memset(&game_policy, 0, sizeof(game_policy));
    memset(&game_signals, 0, sizeof(game_signals));
    for (i = 0U; i < WL_LWDE_EXECUTABLE_SHA256_SIZE; ++i) {
        game_profiles[0].executable_sha256[i] = 0xA1U;
        game_profiles[1].executable_sha256[i] = 0xB2U;
    }
    game_profiles[0].mode = WL_LWDE_GAME_NORMAL;
    game_profiles[1].mode = WL_LWDE_GAME_SUSPEND;
    game_policy.fullscreen_enabled = 1U;
    game_policy.fullscreen_mode = WL_LWDE_GAME_SUSPEND;
    game_policy.system_game_mode_enabled = 1U;
    game_policy.system_game_mode = WL_LWDE_GAME_OPTIMIZE;
    game_signals.is_foreground = 1U;
    game_signals.is_fullscreen = 1U;
    game_signals.system_game_mode_active = 1U;
    memset(game_signals.executable_sha256, 0xA1, WL_LWDE_EXECUTABLE_SHA256_SIZE);
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_OK);
    assert(resolved_game_mode == WL_LWDE_GAME_NORMAL);
    memset(game_signals.executable_sha256, 0xB2, WL_LWDE_EXECUTABLE_SHA256_SIZE);
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_OK);
    assert(resolved_game_mode == WL_LWDE_GAME_SUSPEND);
    memset(game_signals.executable_sha256, 0xC3, WL_LWDE_EXECUTABLE_SHA256_SIZE);
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_OK);
    assert(resolved_game_mode == WL_LWDE_GAME_OPTIMIZE);
    game_signals.system_game_mode_active = 0U;
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_OK);
    assert(resolved_game_mode == WL_LWDE_GAME_SUSPEND);
    game_signals.is_foreground = 0U;
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_OK);
    assert(resolved_game_mode == WL_LWDE_GAME_NORMAL);
    game_signals.is_foreground = 1U;
    memcpy(game_profiles[1].executable_sha256, game_profiles[0].executable_sha256,
        WL_LWDE_EXECUTABLE_SHA256_SIZE);
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 2U,
        &resolved_game_mode) == WL_LWDE_DUPLICATE_GAME_MODE_PROFILE);
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles,
        WL_LWDE_MAX_GAME_MODE_PROFILES + 1U, &resolved_game_mode) ==
        WL_LWDE_TOO_MANY_GAME_MODE_PROFILES);
    game_signals.is_foreground = 2U;
    assert(wl_Lwlde_game_mode_resolve(&game_signals, &game_policy, game_profiles, 1U,
        &resolved_game_mode) == WL_LWDE_INVALID_GAME_MODE_SIGNALS);

    resource_policy.optimize_at_percent = 75U;
    resource_policy.suspend_at_percent = 90U;
    resource_policy.recovery_below_percent = 65U;
    resource_policy.enter_windows = 3U;
    resource_policy.recovery_windows = 4U;
    assert(wl_Lwlde_resource_controller_init(&resource_controller, &resource_policy) == WL_LWDE_OK);
    resource_sample.cpu_percent = 12U;
    resource_sample.gpu_percent = 20U;
    resource_sample.ram_percent = 91U;
    resource_sample.vram_percent = 10U;
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(resource_pressure == WL_LWDE_RESOURCE_NORMAL);
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(resource_pressure == WL_LWDE_RESOURCE_NORMAL);
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(resource_pressure == WL_LWDE_RESOURCE_SUSPEND);
    resource_sample.cpu_percent = 40U;
    resource_sample.gpu_percent = 50U;
    resource_sample.ram_percent = 60U;
    resource_sample.vram_percent = 64U;
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(resource_pressure == WL_LWDE_RESOURCE_SUSPEND);
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_OK);
    assert(resource_pressure == WL_LWDE_RESOURCE_NORMAL);
    resource_sample.gpu_percent = 101U;
    assert(wl_Lwlde_resource_controller_update(&resource_controller, &resource_sample,
        &resource_pressure) == WL_LWDE_INVALID_RESOURCE_SAMPLE);
    resource_policy.suspend_at_percent = 70U;
    assert(wl_Lwlde_resource_controller_init(&resource_controller, &resource_policy) ==
        WL_LWDE_INVALID_RESOURCE_POLICY);

    memset(&caps, 0, sizeof(caps));
    caps.static_wallpaper = 1U;
    caps.virtual_groups = 1U;
    assert(wl_Lwlde_content_supported(&caps, WL_LWDE_CONTENT_STATIC_IMAGE));
    assert(!wl_Lwlde_content_supported(&caps, WL_LWDE_CONTENT_INTERACTIVE));
    assert(!wl_Lwlde_groups_available(&caps));
    caps.crash_safe_group_restore = 1U;
    assert(wl_Lwlde_groups_available(&caps));

    provider = wl_Lwlde_store_provider_get(WL_LWDE_STORE_WALLPAPER_ENGINE);
    assert(provider != 0);
    assert(strcmp(provider->name, "Wallpaper Engine") == 0);
    assert(provider->access == WL_LWDE_STORE_ACCESS_INSTALLED_APPLICATION);
    assert(provider->price_model == WL_LWDE_STORE_PRICE_MIXED);
    assert(strstr(provider->browse_url, "appid=431960") != 0);

    provider = wl_Lwlde_store_provider_get(WL_LWDE_STORE_OPENVERSE);
    assert(provider != 0);
    assert(provider->price_model == WL_LWDE_STORE_PRICE_FREE);
    assert(strstr(provider->rights_note, "license") != 0);

    provider = wl_Lwlde_store_provider_get(WL_LWDE_STORE_WIKIMEDIA_COMMONS);
    assert(provider != 0);
    assert(provider->price_model == WL_LWDE_STORE_PRICE_FREE);

    provider = wl_Lwlde_store_provider_get(WL_LWDE_STORE_NASA_IMAGES);
    assert(provider != 0);
    assert(provider->price_model == WL_LWDE_STORE_PRICE_FREE);
    assert(wl_Lwlde_store_provider_get(WL_LWDE_STORE_PROVIDER_COUNT) == 0);

    puts("wl_Lwlde C89 core tests passed");
    return 0;
}

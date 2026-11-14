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

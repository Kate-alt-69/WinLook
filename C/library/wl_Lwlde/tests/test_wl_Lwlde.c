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


    {
        wl_Lwlde_desktop_item_ref desktop_items[3];
        wl_Lwlde_desktop_group group_storage[2];
        wl_Lwlde_desktop_groups desktop_groups;
        wl_Lwlde_desktop_group group_input[1];
        const char *group_members[2];
        wl_Lwlde_adaptive_app_icon app_icon;
        wl_Lwlde_desktop_context_action context_actions[5];
        unsigned int context_count;

        memset(desktop_items, 0, sizeof(desktop_items));
        strcpy(desktop_items[0].id, "app:editor");
        strcpy(desktop_items[0].label, "Editor");
        desktop_items[0].kind = WL_LWDE_DESKTOP_APPLICATION;
        strcpy(desktop_items[1].id, "file:notes");
        strcpy(desktop_items[1].label, "Notes.txt");
        desktop_items[1].kind = WL_LWDE_DESKTOP_FILE;
        strcpy(desktop_items[2].id, "folder:assets");
        strcpy(desktop_items[2].label, "Assets");
        desktop_items[2].kind = WL_LWDE_DESKTOP_FOLDER;

        group_members[0] = "app:editor";
        group_members[1] = "folder:assets";
        memset(group_input, 0, sizeof(group_input));
        strcpy(group_input[0].id, "group:work");
        strcpy(group_input[0].label, "Work");
        group_input[0].member_ids = group_members;
        group_input[0].member_count = 2U;

        assert(wl_Lwlde_desktop_groups_init(&desktop_groups, group_storage, 2U) ==
            WL_LWDE_OK);
        assert(wl_Lwlde_desktop_groups_replace(&desktop_groups, group_input, 1U,
            desktop_items, 3U) == WL_LWDE_OK);
        assert(desktop_groups.count == 1U);
        assert(strcmp(desktop_groups.groups[0].id, "group:work") == 0);
        assert(wl_Lwlde_desktop_groups_find(&desktop_groups, "group:work") ==
            &desktop_groups.groups[0]);
        assert(wl_Lwlde_desktop_groups_find(&desktop_groups, "group:missing") == 0);
        assert(wl_Lwlde_desktop_group_contains(&desktop_groups.groups[0],
            "app:editor") == 1);
        assert(wl_Lwlde_desktop_group_contains(&desktop_groups.groups[0],
            "file:notes") == 0);

        group_members[1] = "file:missing";
        assert(wl_Lwlde_desktop_groups_replace(&desktop_groups, group_input, 1U,
            desktop_items, 3U) == WL_LWDE_INVALID_DESKTOP_GROUP);
        assert(desktop_groups.count == 1U);
        assert(strcmp(desktop_groups.groups[0].id, "group:work") == 0);
        group_members[1] = "folder:assets";

        memset(&app_icon, 0, sizeof(app_icon));
        app_icon.application_id = "app:editor";
        app_icon.label = "Editor";
        app_icon.foreground_asset = "asset:editor-foreground";
        app_icon.background_asset = "asset:editor-background";
        app_icon.monochrome_asset = "asset:editor-mono";
        app_icon.mask = WL_LWDE_APP_ICON_SQUIRCLE;
        app_icon.safe_zone_percent = WL_LWDE_DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT;
        assert(wl_Lwlde_adaptive_app_icon_validate(&app_icon) == WL_LWDE_OK);
        app_icon.safe_zone_percent = 90U;
        assert(wl_Lwlde_adaptive_app_icon_validate(&app_icon) == WL_LWDE_INVALID_APP_ICON);
        app_icon.safe_zone_percent = WL_LWDE_DEFAULT_ADAPTIVE_ICON_SAFE_ZONE_PERCENT;
        {
            unsigned char background_pixels[4] = { 255U, 0U, 0U, 255U };
            unsigned char foreground_pixels[4] = { 0U, 0U, 255U, 255U };
            unsigned char rendered_pixels[32U * 32U * 4U];
            wl_Lwlde_rgba_image_view background_view;
            wl_Lwlde_rgba_image_view foreground_view;
            unsigned int center_offset;

            background_view.pixels = background_pixels;
            background_view.width = 1U;
            background_view.height = 1U;
            background_view.pixel_length = 4U;
            foreground_view.pixels = foreground_pixels;
            foreground_view.width = 1U;
            foreground_view.height = 1U;
            foreground_view.pixel_length = 4U;
            memset(rendered_pixels, 0xAA, sizeof(rendered_pixels));
            assert(wl_Lwlde_adaptive_app_icon_render_rgba(&app_icon,
                &background_view, &foreground_view, 32U, rendered_pixels,
                (unsigned int)sizeof(rendered_pixels)) == WL_LWDE_OK);
            center_offset = (16U * 32U + 16U) * 4U;
            assert(rendered_pixels[center_offset] == 0U);
            assert(rendered_pixels[center_offset + 1U] == 0U);
            assert(rendered_pixels[center_offset + 2U] == 255U);
            assert(rendered_pixels[center_offset + 3U] == 255U);
            assert(rendered_pixels[3U] == 0U);
            assert(background_pixels[0] == 255U && foreground_pixels[2] == 255U);

            background_view.pixel_length = 3U;
            assert(wl_Lwlde_adaptive_app_icon_render_rgba(&app_icon,
                &background_view, &foreground_view, 32U, rendered_pixels,
                (unsigned int)sizeof(rendered_pixels)) == WL_LWDE_INVALID_ICON_PIXELS);
            background_view.pixel_length = 4U;
            assert(wl_Lwlde_adaptive_app_icon_render_rgba(&app_icon,
                &background_view, &foreground_view, 32U, rendered_pixels, 8U) ==
                WL_LWDE_INVALID_ICON_OUTPUT);
        }

        context_count = wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_APPLICATION, 0U, 0, 0U);
        assert(context_count == 4U);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_APPLICATION, 0U, context_actions, 3U) == 0U);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_APPLICATION, 0U, context_actions, 5U) == 4U);
        assert(context_actions[2] == WL_LWDE_CONTEXT_CUSTOMIZE_APPLICATION_ICON);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_APPLICATION, 1U, context_actions, 5U) == 5U);
        assert(context_actions[2] == WL_LWDE_CONTEXT_REMOVE_FROM_GROUP);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_FILE, 0U, context_actions, 5U) == 3U);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_GROUP, 0U, context_actions, 5U) == 5U);
        assert(context_actions[3] == WL_LWDE_CONTEXT_UNGROUP);
        context_count = wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_DESKTOP_BACKGROUND, 0U, 0, 0U);
        assert(context_count == 2U);
        assert(wl_Lwlde_desktop_context_actions(
            WL_LWDE_CONTEXT_DESKTOP_BACKGROUND, 0U, context_actions, 5U) == 2U);
        assert(context_actions[0] == WL_LWDE_CONTEXT_CREATE_GROUP);
        assert(context_actions[1] == WL_LWDE_CONTEXT_OPEN_DESKTOP_SETTINGS);
    }

    puts("wl_Lwlde C89 core tests passed");
    return 0;
}

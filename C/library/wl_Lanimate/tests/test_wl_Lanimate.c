#include <assert.h>
#include <math.h>

#include "wl_Lanimate.h"

static int near_value(double actual, double expected)
{
    return fabs(actual - expected) < 0.000001;
}

static void test_tween_lifecycle(void)
{
    wl_Lanimate_tween tween;
    wl_Lanimate_tween_init(&tween, 0.0, 10.0, 100.0, WL_LANIMATE_LINEAR);
    assert(tween.state == WL_LANIMATE_PENDING);
    assert(near_value(wl_Lanimate_tween_progress(&tween), 0.0));
    assert(near_value(wl_Lanimate_tween_advance(&tween, 25.0), 2.5));
    assert(tween.state == WL_LANIMATE_RUNNING);
    wl_Lanimate_tween_pause(&tween);
    assert(tween.state == WL_LANIMATE_PAUSED);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 50.0), 2.5));
    wl_Lanimate_tween_resume(&tween);
    assert(tween.state == WL_LANIMATE_RUNNING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 75.0), 10.0));
    assert(tween.state == WL_LANIMATE_FINISHED);
    assert(near_value(wl_Lanimate_tween_progress(&tween), 1.0));

    wl_Lanimate_tween_init(&tween, 0.0, 1.0, 100.0, WL_LANIMATE_LINEAR);
    wl_Lanimate_tween_advance(&tween, 10.0);
    wl_Lanimate_tween_cancel(&tween);
    assert(tween.state == WL_LANIMATE_CANCELLED);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 90.0), 0.1));
    wl_Lanimate_tween_restart(&tween);
    assert(tween.state == WL_LANIMATE_PENDING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 50.0), 0.5));
}

static void test_tween_delay(void)
{
    wl_Lanimate_tween tween;
    wl_Lanimate_tween_init_delayed(&tween, 0.0, 10.0, 100.0, 50.0,
                                   WL_LANIMATE_LINEAR);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 25.0), 0.0));
    assert(tween.state == WL_LANIMATE_PENDING);
    wl_Lanimate_tween_pause(&tween);
    assert(tween.state == WL_LANIMATE_PAUSED);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 500.0), 0.0));
    wl_Lanimate_tween_resume(&tween);
    assert(tween.state == WL_LANIMATE_PENDING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 25.0), 0.0));
    assert(tween.state == WL_LANIMATE_RUNNING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 50.0), 5.0));
    assert(near_value(wl_Lanimate_tween_advance(&tween, 50.0), 10.0));
    assert(tween.state == WL_LANIMATE_FINISHED);

    wl_Lanimate_tween_init_delayed(&tween, 1.0, 4.0, 0.0, 25.0,
                                   WL_LANIMATE_LINEAR);
    assert(near_value(wl_Lanimate_tween_value(&tween), 1.0));
    assert(near_value(wl_Lanimate_tween_advance(&tween, 24.0), 1.0));
    assert(tween.state == WL_LANIMATE_PENDING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 1.0), 4.0));
    assert(tween.state == WL_LANIMATE_FINISHED);
}

static void test_easing_and_values(void)
{
    wl_Lanimate_point2 p0;
    wl_Lanimate_point2 p1;
    wl_Lanimate_point2 p;
    wl_Lanimate_rgba c0;
    wl_Lanimate_rgba c1;
    wl_Lanimate_rgba c;

    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_IN_OUT_CUBIC, -1.0), 0.0));
    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_IN_OUT_CUBIC, 2.0), 1.0));
    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_LINEAR, 0.5), 0.5));
    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_OUT_QUART, 0.5), 0.9375));
    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_IN_OUT_SINE, 0.5), 0.5));

    p0.x = 0.0; p0.y = 10.0;
    p1.x = 100.0; p1.y = 30.0;
    p = wl_Lanimate_point2_lerp(p0, p1, 0.5);
    assert(near_value(p.x, 50.0));
    assert(near_value(p.y, 20.0));

    c0.r = 0.0; c0.g = 10.0; c0.b = 20.0; c0.a = 30.0;
    c1.r = 100.0; c1.g = 110.0; c1.b = 120.0; c1.a = 130.0;
    c = wl_Lanimate_rgba_lerp(c0, c1, 0.5);
    assert(near_value(c.r, 50.0));
    assert(near_value(c.g, 60.0));
    assert(near_value(c.b, 70.0));
    assert(near_value(c.a, 80.0));
}

static void test_keyframe_track(void)
{
    wl_Lanimate_keyframe frames[3];
    wl_Lanimate_track track;
    wl_Lanimate_result result;

    frames[0].at_ms = 0.0;
    frames[0].value = 0.0;
    frames[0].easing_to_next = WL_LANIMATE_LINEAR;
    frames[1].at_ms = 100.0;
    frames[1].value = 20.0;
    frames[1].easing_to_next = WL_LANIMATE_EASE_IN_QUAD;
    frames[2].at_ms = 200.0;
    frames[2].value = 40.0;
    frames[2].easing_to_next = WL_LANIMATE_LINEAR;

    result = wl_Lanimate_track_init(&track, frames, 3);
    assert(result == WL_LANIMATE_OK);
    assert(near_value(wl_Lanimate_track_sample(&track, -1.0), 0.0));
    assert(near_value(wl_Lanimate_track_sample(&track, 50.0), 10.0));
    assert(near_value(wl_Lanimate_track_sample(&track, 150.0), 25.0));
    assert(near_value(wl_Lanimate_track_sample(&track, 250.0), 40.0));

    frames[1].at_ms = 0.0;
    result = wl_Lanimate_track_init(&track, frames, 3);
    assert(result == WL_LANIMATE_INVALID_KEYFRAME_ORDER);
    assert(track.keyframes == 0);

    frames[1].at_ms = -1.0;
    result = wl_Lanimate_track_init(&track, frames, 3);
    assert(result == WL_LANIMATE_INVALID_KEYFRAME_TIME);
    assert(track.keyframes == 0);

    assert(wl_Lanimate_track_init(&track, frames, 0) == WL_LANIMATE_INVALID_ARGUMENT);
    assert(wl_Lanimate_track_init(&track, 0, 2) == WL_LANIMATE_INVALID_ARGUMENT);
    assert(wl_Lanimate_track_sample(&track, 10.0) == 0.0);
}

int main(void)
{
    test_tween_lifecycle();
    test_tween_delay();
    test_easing_and_values();
    test_keyframe_track();
    return 0;
}

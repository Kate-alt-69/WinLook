#include <assert.h>
#include <math.h>
#include "wl_Lanimate.h"

static int near_value(double actual, double expected)
{
    return fabs(actual - expected) < 0.000001;
}

int main(void)
{
    wl_Lanimate_tween tween;
    wl_Lanimate_point2 p0;
    wl_Lanimate_point2 p1;
    wl_Lanimate_point2 p;
    wl_Lanimate_keyframe frames[2];
    wl_Lanimate_track track;

    wl_Lanimate_tween_init(&tween, 0.0, 10.0, 100.0, WL_LANIMATE_LINEAR);
    assert(tween.state == WL_LANIMATE_PENDING);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 25.0), 2.5));
    wl_Lanimate_tween_pause(&tween);
    assert(tween.state == WL_LANIMATE_PAUSED);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 50.0), 2.5));
    wl_Lanimate_tween_resume(&tween);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 75.0), 10.0));
    assert(tween.state == WL_LANIMATE_FINISHED);

    wl_Lanimate_tween_init(&tween, 1.0, 4.0, 0.0, WL_LANIMATE_LINEAR);
    assert(tween.state == WL_LANIMATE_FINISHED);
    assert(near_value(wl_Lanimate_tween_value(&tween), 4.0));

    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_IN_OUT_CUBIC, -1.0), 0.0));
    assert(near_value(wl_Lanimate_ease(WL_LANIMATE_EASE_IN_OUT_CUBIC, 2.0), 1.0));

    p0.x = 0.0; p0.y = 10.0;
    p1.x = 100.0; p1.y = 30.0;
    p = wl_Lanimate_point2_lerp(p0, p1, 0.5);
    assert(near_value(p.x, 50.0));
    assert(near_value(p.y, 20.0));

    wl_Lanimate_tween_init(&tween, 0.0, 1.0, 100.0, WL_LANIMATE_LINEAR);
    wl_Lanimate_tween_advance(&tween, 10.0);
    wl_Lanimate_tween_cancel(&tween);
    assert(tween.state == WL_LANIMATE_CANCELLED);
    assert(near_value(wl_Lanimate_tween_advance(&tween, 90.0), 0.1));

    frames[0].at_ms = 0.0;
    frames[0].value = 0.0;
    frames[0].easing_to_next = WL_LANIMATE_LINEAR;
    frames[1].at_ms = 100.0;
    frames[1].value = 20.0;
    frames[1].easing_to_next = WL_LANIMATE_LINEAR;
    assert(wl_Lanimate_track_init(&track, frames, 2) == WL_LANIMATE_OK);
    assert(near_value(wl_Lanimate_track_sample(&track, 25.0), 5.0));
    assert(near_value(wl_Lanimate_track_sample(&track, 150.0), 20.0));

    frames[1].at_ms = 0.0;
    assert(wl_Lanimate_track_init(&track, frames, 2) == WL_LANIMATE_INVALID_KEYFRAME_ORDER);

    return 0;
}

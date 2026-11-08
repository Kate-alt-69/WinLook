#include "wl_Lanimate.h"

static double wl_Lanimate_clamp01(double value)
{
    if (value != value) return 0.0;
    if (value < 0.0) return 0.0;
    if (value > 1.0) return 1.0;
    return value;
}

static double wl_Lanimate_lerp(double from_value, double to_value, double progress)
{
    return from_value + (to_value - from_value) * wl_Lanimate_clamp01(progress);
}

wl_Lanimate_result wl_Lanimate_track_init(wl_Lanimate_track *track,
                                           const wl_Lanimate_keyframe *keyframes,
                                           unsigned int count)
{
    unsigned int i;
    if (track == 0 || keyframes == 0 || count == 0) return WL_LANIMATE_INVALID_ARGUMENT;
    for (i = 1; i < count; ++i) {
        if (keyframes[i - 1].at_ms >= keyframes[i].at_ms) {
            track->keyframes = 0;
            track->count = 0;
            return WL_LANIMATE_INVALID_KEYFRAME_ORDER;
        }
    }
    track->keyframes = keyframes;
    track->count = count;
    return WL_LANIMATE_OK;
}

double wl_Lanimate_track_sample(const wl_Lanimate_track *track, double at_ms)
{
    unsigned int i;
    const wl_Lanimate_keyframe *left;
    const wl_Lanimate_keyframe *right;
    double span;
    double progress;
    if (track == 0 || track->keyframes == 0 || track->count == 0) return 0.0;
    if (at_ms <= track->keyframes[0].at_ms) return track->keyframes[0].value;
    if (at_ms >= track->keyframes[track->count - 1].at_ms) return track->keyframes[track->count - 1].value;
    for (i = 0; i + 1 < track->count; ++i) {
        left = &track->keyframes[i];
        right = &track->keyframes[i + 1];
        if (at_ms <= right->at_ms) {
            span = right->at_ms - left->at_ms;
            progress = span <= 0.0 ? 1.0 : (at_ms - left->at_ms) / span;
            return wl_Lanimate_lerp(left->value, right->value,
                                    wl_Lanimate_ease(left->easing_to_next, progress));
        }
    }
    return track->keyframes[track->count - 1].value;
}

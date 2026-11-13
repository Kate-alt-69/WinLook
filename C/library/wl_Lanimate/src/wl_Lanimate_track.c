#include "wl_Lanimate.h"

#include <float.h>

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

static int wl_Lanimate_valid_timestamp(double value)
{
    return value >= 0.0 && value <= DBL_MAX;
}

wl_Lanimate_result wl_Lanimate_track_init(wl_Lanimate_track *track,
                                           const wl_Lanimate_keyframe *keyframes,
                                           unsigned int count)
{
    unsigned int i;
    if (track == 0) return WL_LANIMATE_INVALID_ARGUMENT;
    track->keyframes = 0;
    track->count = 0;
    if (keyframes == 0 || count == 0U) return WL_LANIMATE_INVALID_ARGUMENT;
    for (i = 0; i < count; ++i) {
        if (!wl_Lanimate_valid_timestamp(keyframes[i].at_ms)) {
            return WL_LANIMATE_INVALID_KEYFRAME_TIME;
        }
        if (i > 0U && keyframes[i - 1U].at_ms >= keyframes[i].at_ms) {
            return WL_LANIMATE_INVALID_KEYFRAME_ORDER;
        }
    }
    track->keyframes = keyframes;
    track->count = count;
    return WL_LANIMATE_OK;
}

void wl_Lanimate_track_cursor_reset(wl_Lanimate_track_cursor *cursor)
{
    if (cursor != 0) cursor->segment_index = 0U;
}

double wl_Lanimate_track_sample_cached(
    const wl_Lanimate_track *track,
    wl_Lanimate_track_cursor *cursor,
    double at_ms
)
{
    unsigned int left_index;
    unsigned int low;
    unsigned int high;
    unsigned int middle;
    const wl_Lanimate_keyframe *left;
    const wl_Lanimate_keyframe *right;
    double span;
    double progress;

    if (track == 0 || track->keyframes == 0 || track->count == 0U) return 0.0;
    if (at_ms != at_ms || at_ms <= track->keyframes[0].at_ms) {
        if (cursor != 0) cursor->segment_index = 0U;
        return track->keyframes[0].value;
    }
    if (at_ms >= track->keyframes[track->count - 1U].at_ms) {
        if (cursor != 0) cursor->segment_index = track->count - 2U;
        return track->keyframes[track->count - 1U].value;
    }

    left_index = 0U;
    if (cursor != 0 && cursor->segment_index < track->count - 1U) {
        left_index = cursor->segment_index;
    }
    left = &track->keyframes[left_index];
    right = &track->keyframes[left_index + 1U];
    if (at_ms < left->at_ms || at_ms > right->at_ms) {
        low = 0U;
        high = track->count - 1U;
        while (high - low > 1U) {
            middle = low + (high - low) / 2U;
            if (at_ms < track->keyframes[middle].at_ms) high = middle;
            else low = middle;
        }
        left_index = low;
    }

    if (cursor != 0) cursor->segment_index = left_index;
    left = &track->keyframes[left_index];
    right = &track->keyframes[left_index + 1U];
    span = right->at_ms - left->at_ms;
    progress = span <= 0.0 ? 1.0 : (at_ms - left->at_ms) / span;
    return wl_Lanimate_lerp(left->value, right->value,
                            wl_Lanimate_ease(left->easing_to_next, progress));
}

double wl_Lanimate_track_sample(const wl_Lanimate_track *track, double at_ms)
{
    wl_Lanimate_track_cursor cursor;
    cursor.segment_index = 0U;
    return wl_Lanimate_track_sample_cached(track, &cursor, at_ms);
}

#include "wl_Lanimate.h"

#include <float.h>
#include <math.h>

#define WL_LANIMATE_MAX_TIME_MS 3600000.0
#define WL_LANIMATE_MAX_ITERATIONS 1000U
#define WL_LANIMATE_MAX_KEYFRAMES 64U

static int wl_Lanimate_direction_valid(wl_Lanimate_direction direction)
{
    return direction >= WL_LANIMATE_DIRECTION_NORMAL
        && direction <= WL_LANIMATE_DIRECTION_ALTERNATE_REVERSE;
}

static int wl_Lanimate_fill_valid(wl_Lanimate_fill_mode fill_mode)
{
    return fill_mode >= WL_LANIMATE_FILL_NONE
        && fill_mode <= WL_LANIMATE_FILL_BOTH;
}

static int wl_Lanimate_is_reverse(wl_Lanimate_direction direction, unsigned int iteration)
{
    switch (direction) {
    case WL_LANIMATE_DIRECTION_REVERSE:
        return 1;
    case WL_LANIMATE_DIRECTION_ALTERNATE:
        return (iteration % 2U) != 0U;
    case WL_LANIMATE_DIRECTION_ALTERNATE_REVERSE:
        return (iteration % 2U) == 0U;
    case WL_LANIMATE_DIRECTION_NORMAL:
    default:
        return 0;
    }
}

static void wl_Lanimate_animation_refresh_state(wl_Lanimate_animation *animation)
{
    double active_elapsed;
    double total_duration;

    if (animation->elapsed_ms < animation->delay_ms) {
        animation->state = WL_LANIMATE_PENDING;
        return;
    }

    active_elapsed = animation->elapsed_ms - animation->delay_ms;
    if (animation->iterations != 0U) {
        total_duration = animation->duration_ms * (double)animation->iterations;
        if (active_elapsed >= total_duration) {
            animation->state = WL_LANIMATE_FINISHED;
            return;
        }
    }
    animation->state = WL_LANIMATE_RUNNING;
}

wl_Lanimate_result wl_Lanimate_animation_init(
    wl_Lanimate_animation *animation,
    const wl_Lanimate_track *track,
    double duration_ms,
    double delay_ms,
    unsigned int iterations,
    wl_Lanimate_direction direction,
    wl_Lanimate_fill_mode fill_mode
)
{
    wl_Lanimate_track validated_track;
    wl_Lanimate_result result;

    if (animation == 0 || track == 0 || track->keyframes == 0
        || track->count < 2U || track->count > WL_LANIMATE_MAX_KEYFRAMES
        || !(duration_ms > 0.0) || duration_ms > WL_LANIMATE_MAX_TIME_MS
        || !(delay_ms >= 0.0) || delay_ms > WL_LANIMATE_MAX_TIME_MS
        || iterations > WL_LANIMATE_MAX_ITERATIONS
        || !wl_Lanimate_direction_valid(direction)
        || !wl_Lanimate_fill_valid(fill_mode)) {
        return WL_LANIMATE_INVALID_ARGUMENT;
    }

    result = wl_Lanimate_track_init(&validated_track, track->keyframes, track->count);
    if (result != WL_LANIMATE_OK) return result;

    animation->track = validated_track;
    animation->cursor.segment_index = 0U;
    animation->duration_ms = duration_ms;
    animation->delay_ms = delay_ms;
    animation->elapsed_ms = 0.0;
    animation->iterations = iterations;
    animation->direction = direction;
    animation->fill_mode = fill_mode;
    animation->state = WL_LANIMATE_PENDING;
    return WL_LANIMATE_OK;
}

wl_Lanimate_result wl_Lanimate_animation_sample(
    wl_Lanimate_animation *animation,
    double *value_out
)
{
    double active_elapsed;
    double cycle;
    double fraction;
    double progress;
    double start_ms;
    double end_ms;
    double sample_ms;
    double whole_iteration;
    unsigned int iteration;
    int finished;
    int reverse;

    if (animation == 0 || value_out == 0 || animation->track.keyframes == 0
        || animation->track.count < 2U) {
        return WL_LANIMATE_INVALID_ARGUMENT;
    }
    *value_out = 0.0;

    start_ms = animation->track.keyframes[0].at_ms;
    end_ms = animation->track.keyframes[animation->track.count - 1U].at_ms;

    if (animation->elapsed_ms < animation->delay_ms) {
        if (animation->fill_mode != WL_LANIMATE_FILL_BACKWARDS
            && animation->fill_mode != WL_LANIMATE_FILL_BOTH) {
            return WL_LANIMATE_NO_VALUE;
        }
        reverse = wl_Lanimate_is_reverse(animation->direction, 0U);
        sample_ms = reverse ? end_ms : start_ms;
        *value_out = wl_Lanimate_track_sample_cached(
            &animation->track, &animation->cursor, sample_ms
        );
        return WL_LANIMATE_OK;
    }

    active_elapsed = animation->elapsed_ms - animation->delay_ms;
    finished = animation->state == WL_LANIMATE_FINISHED;
    if (animation->iterations != 0U
        && active_elapsed >= animation->duration_ms * (double)animation->iterations) {
        finished = 1;
    }

    if (finished && animation->fill_mode != WL_LANIMATE_FILL_FORWARDS
        && animation->fill_mode != WL_LANIMATE_FILL_BOTH) {
        return WL_LANIMATE_NO_VALUE;
    }

    if (finished) {
        iteration = animation->iterations == 0U ? 0U : animation->iterations - 1U;
        progress = 1.0;
    } else {
        cycle = active_elapsed / animation->duration_ms;
        fraction = modf(cycle, &whole_iteration);
        progress = fraction;
        iteration = fmod(whole_iteration, 2.0) >= 1.0 ? 1U : 0U;
    }

    reverse = wl_Lanimate_is_reverse(animation->direction, iteration);
    if (reverse) progress = 1.0 - progress;

    sample_ms = start_ms + (end_ms - start_ms) * progress;
    *value_out = wl_Lanimate_track_sample_cached(
        &animation->track, &animation->cursor, sample_ms
    );
    return WL_LANIMATE_OK;
}

wl_Lanimate_result wl_Lanimate_animation_advance(
    wl_Lanimate_animation *animation,
    double delta_ms,
    double *value_out
)
{
    if (animation == 0 || value_out == 0) return WL_LANIMATE_INVALID_ARGUMENT;

    if (animation->state == WL_LANIMATE_PENDING
        || animation->state == WL_LANIMATE_RUNNING) {
        if (delta_ms == delta_ms && delta_ms > 0.0 && delta_ms <= DBL_MAX) {
            if (delta_ms > DBL_MAX - animation->elapsed_ms) {
                animation->elapsed_ms = DBL_MAX;
            } else {
                animation->elapsed_ms += delta_ms;
            }
        }
        wl_Lanimate_animation_refresh_state(animation);
    }
    return wl_Lanimate_animation_sample(animation, value_out);
}

void wl_Lanimate_animation_pause(wl_Lanimate_animation *animation)
{
    if (animation != 0 && (animation->state == WL_LANIMATE_PENDING
        || animation->state == WL_LANIMATE_RUNNING)) {
        animation->state = WL_LANIMATE_PAUSED;
    }
}

void wl_Lanimate_animation_resume(wl_Lanimate_animation *animation)
{
    if (animation != 0 && animation->state == WL_LANIMATE_PAUSED) {
        wl_Lanimate_animation_refresh_state(animation);
    }
}

void wl_Lanimate_animation_cancel(wl_Lanimate_animation *animation)
{
    if (animation != 0 && animation->state != WL_LANIMATE_FINISHED) {
        animation->state = WL_LANIMATE_CANCELLED;
    }
}

void wl_Lanimate_animation_restart(wl_Lanimate_animation *animation)
{
    if (animation == 0) return;
    animation->elapsed_ms = 0.0;
    animation->cursor.segment_index = 0U;
    animation->state = WL_LANIMATE_PENDING;
}

#include "wl_Lanimate.h"

#include <stdio.h>
#include <time.h>

#define WL_LANIMATE_BENCHMARK_ANIMATIONS 1024U
#define WL_LANIMATE_BENCHMARK_FRAMES 600U

static wl_Lanimate_keyframe wl_Lanimate_benchmark_frames[3];
static wl_Lanimate_animation wl_Lanimate_benchmark_animations[
    WL_LANIMATE_BENCHMARK_ANIMATIONS
];

int main(void)
{
    wl_Lanimate_track track;
    wl_Lanimate_frame_clock frame_clock;
    wl_Lanimate_frame_tick frame;
    wl_Lanimate_result result;
    unsigned int i;
    unsigned int frame_index;
    unsigned int emitted_frames;
    double now_ms;
    double value;
    double checksum;
    clock_t started;
    clock_t finished;
    clock_t elapsed_ticks;
    double elapsed_ms;
    double nanoseconds_per_sample;

    wl_Lanimate_benchmark_frames[0].at_ms = 0.0;
    wl_Lanimate_benchmark_frames[0].value = 0.0;
    wl_Lanimate_benchmark_frames[0].easing_to_next = WL_LANIMATE_EASE_OUT_SINE;
    wl_Lanimate_benchmark_frames[1].at_ms = 180.0;
    wl_Lanimate_benchmark_frames[1].value = 1.0;
    wl_Lanimate_benchmark_frames[1].easing_to_next = WL_LANIMATE_EASE_OUT_CUBIC;
    wl_Lanimate_benchmark_frames[2].at_ms = 360.0;
    wl_Lanimate_benchmark_frames[2].value = 0.0;
    wl_Lanimate_benchmark_frames[2].easing_to_next = WL_LANIMATE_EASE_IN_OUT_SINE;

    result = wl_Lanimate_track_init(
        &track, wl_Lanimate_benchmark_frames, 3U
    );
    if (result != WL_LANIMATE_OK) {
        fprintf(stderr, "Could not initialize the benchmark track: %d\n", (int)result);
        return 1;
    }

    for (i = 0U; i < WL_LANIMATE_BENCHMARK_ANIMATIONS; ++i) {
        result = wl_Lanimate_animation_init(
            &wl_Lanimate_benchmark_animations[i],
            &track,
            360.0,
            0.0,
            0U,
            WL_LANIMATE_DIRECTION_ALTERNATE,
            WL_LANIMATE_FILL_BOTH
        );
        if (result != WL_LANIMATE_OK) {
            fprintf(stderr, "Could not initialize animation %u: %d\n", i, (int)result);
            return 2;
        }
    }

    result = wl_Lanimate_frame_clock_init(
        &frame_clock,
        WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS,
        WL_LANIMATE_FRAME_DEFAULT_MAXIMUM_DELTA_MS
    );
    if (result != WL_LANIMATE_OK) {
        fprintf(stderr, "Could not initialize the benchmark frame clock: %d\n", (int)result);
        return 3;
    }

    now_ms = 0.0;
    result = wl_Lanimate_frame_clock_tick(&frame_clock, now_ms, &frame);
    if (result != WL_LANIMATE_OK) return 4;

    checksum = 0.0;
    emitted_frames = 0U;
    started = clock();
    for (frame_index = 0U; frame_index < WL_LANIMATE_BENCHMARK_FRAMES; ++frame_index) {
        /* Host refresh is 16.666667 ms; target cadence is slightly below it. */
        now_ms += WL_LANIMATE_FRAME_60_FPS_INTERVAL_MS + 0.000002;
        result = wl_Lanimate_frame_clock_tick(&frame_clock, now_ms, &frame);
        if (result == WL_LANIMATE_NO_FRAME) continue;
        if (result != WL_LANIMATE_OK) {
            fprintf(stderr, "Frame clock failed at frame %u: %d\n", frame_index, (int)result);
            return 5;
        }
        emitted_frames++;

        for (i = 0U; i < WL_LANIMATE_BENCHMARK_ANIMATIONS; ++i) {
            result = wl_Lanimate_animation_advance(
                &wl_Lanimate_benchmark_animations[i],
                frame.delta_ms,
                &value
            );
            if (result != WL_LANIMATE_OK) {
                fprintf(stderr, "Animation sample failed at frame %u: %d\n",
                        frame_index, (int)result);
                return 6;
            }
            checksum += value;
        }
    }
    finished = clock();

    if (started == (clock_t)-1 || finished == (clock_t)-1
        || finished < started || CLOCKS_PER_SEC <= 0) {
        fprintf(stderr, "The C runtime clock is unavailable for this benchmark.\n");
        return 7;
    }

    if (emitted_frames != WL_LANIMATE_BENCHMARK_FRAMES) {
        fprintf(stderr, "Expected %u emitted frames, received %u.\n",
                WL_LANIMATE_BENCHMARK_FRAMES, emitted_frames);
        return 8;
    }

    elapsed_ticks = finished - started;
    elapsed_ms = ((double)elapsed_ticks * 1000.0) / (double)CLOCKS_PER_SEC;
    nanoseconds_per_sample =
        ((double)elapsed_ticks * 1000000000.0)
        / ((double)CLOCKS_PER_SEC
           * (double)WL_LANIMATE_BENCHMARK_ANIMATIONS
           * (double)WL_LANIMATE_BENCHMARK_FRAMES);

    printf("Animations: %u\n", WL_LANIMATE_BENCHMARK_ANIMATIONS);
    printf("Frames: %u at 60 FPS\n", emitted_frames);
    printf("Total process CPU time: %.3f ms\n", elapsed_ms);
    printf("Average process CPU time per frame: %.3f us\n",
           elapsed_ms * 1000.0 / (double)WL_LANIMATE_BENCHMARK_FRAMES);
    printf("Average process CPU time per animation sample: %.1f ns\n", nanoseconds_per_sample);
    printf("Checksum: %.6f\n", checksum);
    printf("No timers, threads, renderer or GPU calls are created by this benchmark.\n");
    return 0;
}

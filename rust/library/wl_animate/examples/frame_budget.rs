use std::time::{Duration, Instant};
use wl_animate::{
    Animation, Direction, Easing, FillMode, FrameClock, IterationCount, Keyframe, KeyframeTrack,
};

fn main() {
    const ANIMATION_COUNT: usize = 1024;
    const FRAME_COUNT: usize = 600;
    let track = KeyframeTrack::new(vec![
        Keyframe::new(Duration::ZERO, 0.0_f64, Easing::EaseOutSine),
        Keyframe::new(Duration::from_millis(180), 1.0_f64, Easing::EaseOutCubic),
        Keyframe::new(Duration::from_millis(360), 0.0_f64, Easing::EaseInOutSine),
    ])
    .expect("valid sample track");

    let mut animations = (0..ANIMATION_COUNT)
        .map(|_| {
            Animation::new(
                track.clone(),
                Duration::from_millis(360),
                Duration::ZERO,
                IterationCount::Infinite,
                Direction::Alternate,
                FillMode::Both,
            )
            .expect("valid animation")
        })
        .collect::<Vec<_>>();

    let frame_delta = Duration::from_nanos(16_666_667);
    let mut frame_clock = FrameClock::sixty_fps();
    let mut host_timestamp = Duration::ZERO;
    let _initial_tick = frame_clock
        .tick(host_timestamp)
        .expect("the initial clock tick establishes the baseline");
    let started = Instant::now();
    let mut checksum = 0.0_f64;
    let mut emitted_frames = 0_usize;
    for _ in 0..FRAME_COUNT {
        host_timestamp = host_timestamp.saturating_add(frame_delta);
        if let Some(frame) = frame_clock.tick(host_timestamp) {
            emitted_frames += 1;
            for animation in &mut animations {
                if let Some(value) = animation.advance(frame.delta) {
                    checksum += value;
                }
            }
        }
    }

    assert_eq!(
        emitted_frames, FRAME_COUNT,
        "the 60 FPS benchmark clock must emit every expected host frame"
    );

    let elapsed = started.elapsed();
    let microseconds_per_frame = elapsed.as_secs_f64() * 1_000_000.0 / FRAME_COUNT as f64;
    let nanoseconds_per_sample = elapsed.as_nanos() as f64 / (ANIMATION_COUNT * FRAME_COUNT) as f64;

    println!("Animations: {ANIMATION_COUNT}");
    println!("Frames: {emitted_frames} at 60 FPS");
    println!("Total CPU time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
    println!(
        "Average update time per frame: {:.3} us",
        microseconds_per_frame
    );
    println!(
        "Average time per animation sample: {:.1} ns",
        nanoseconds_per_sample
    );
    println!("Checksum: {:.6}", checksum);
    println!("No timers, threads, renderer or GPU calls are created by this benchmark.");
}

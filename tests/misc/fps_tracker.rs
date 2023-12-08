//local shortcuts
use bevy_kot::prelude::*;

//third-party shortcuts

//standard shortcuts
use std::time::Duration;


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn check_expected(tracker: &FpsTracker, expected_avg_delta_secs: f32, expected_fps: u16) -> bool
{
    if tracker.average_delta_seconds() != expected_avg_delta_secs
    { println!("avg delta: {}", tracker.average_delta_seconds()); return false; }

    if tracker.fps() != expected_fps
    { println!("fps: {}", tracker.fps()); return false; }

    return true;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn fps_tracker_basic()
{
    // 1. zero records
    let mut tracker = FpsTracker::new(0);

    assert!(check_expected(&tracker, 0f32, 0u16));
    tracker.update(0.1, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0f32, 0u16));

    // 2. one record
    let mut tracker = FpsTracker::new(1);

    assert!(check_expected(&tracker, 0f32, 0u16));
    tracker.update(0.1, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0.1, 10u16));
    tracker.update(0.2, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0.2, 5u16));

    // 3. two records
    let mut tracker = FpsTracker::new(2);

    assert!(check_expected(&tracker, 0f32, 0u16));
    tracker.update(0.05, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0.05, 20u16));
    tracker.update(0.15, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0.1, 10u16));
    tracker.update(0.10, Duration::from_secs(1));
    assert!(check_expected(&tracker, 0.25/2.0, 8u16));
}

//-------------------------------------------------------------------------------------------------------------------

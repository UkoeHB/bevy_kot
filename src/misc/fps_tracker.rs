//! FPS tracker utility
//!
//! PRECONDITION: plugin dependencies
//! - bevy::time::TimePlugin
//!

//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_fn_plugin::bevy_plugin;

//standard shortcuts
use std::collections::VecDeque;
use std::time::Duration;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: pass by value on construction
const FPS_TRACKER_NUM_RECORDS : u8  = 30;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Update FPS tracker with new time.
fn update_fps_tracker(mut tracker: ResMut<FPSTracker>, time: Res<Time>)
{
    tracker.update(time.delta_seconds(), time.elapsed());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// FPS tracker
#[derive(Resource)]
pub struct FPSTracker
{
    max_records     : u8,
    delta_record_ns : VecDeque<u64>,
    total_delta_ns  : u64,
    previous_time   : Duration,
    current_time    : Duration
}

impl FPSTracker
{
    /// make new tracker
    pub fn new(max: u8) -> FPSTracker
    {
        let mut new_tracker = 
            FPSTracker{
                    max_records     : max,
                    delta_record_ns : VecDeque::new(),
                    total_delta_ns  : 0u64,
                    previous_time   : Duration::from_secs(0),
                    current_time    : Duration::from_secs(0)
                };
        new_tracker.delta_record_ns.reserve((max + 1) as usize);
        return new_tracker
    }

    /// average delta recorded (in seconds)
    pub fn average_delta_seconds(&self) -> f32
    {
        (self.average_delta_nanoseconds() as f32) / 1_000_000_000.0
    }

    /// average delta recorded (in nanoseconds)
    pub fn average_delta_nanoseconds(&self) -> u64
    {
        match self.delta_record_ns.len()
        {
            0           => 0u64,
            num_records => self.total_delta_ns / (num_records as u64)
        }
    }

    /// FPS estimate
    pub fn fps(&self) -> u16
    {
        match (1_000_000_000u64).checked_div(self.average_delta_nanoseconds())
        {
            Some(rate) => rate as u16,
            None       => 0
        }
    }

    pub fn previous_time(&self) -> Duration { self.previous_time }
    pub fn current_time(&self)  -> Duration { self.current_time  }

    /// update the tracker with a new time
    pub fn update(self: &mut FPSTracker, delta: f32, current_time: Duration)
    {
        // 1. add new record
        let delta_ns = (delta * 1_000_000_000.0) as u64;
        self.delta_record_ns.push_back(delta_ns);
        self.total_delta_ns += delta_ns;

        // 2. update current time
        self.previous_time = self.current_time;
        self.current_time  = current_time;

        // 3. remove excess records
        while self.delta_record_ns.len() > (self.max_records as usize)
        {
            self.total_delta_ns -= self.delta_record_ns.get(0).unwrap();
            self.delta_record_ns.pop_front();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, PartialEq, Eq, Debug, Hash, Clone)]
pub struct FPSTrackerSet;

/// Tracks FPS. Use `FPSTracker` Resource to access the fps with `tracker.fps()`.
#[bevy_plugin]
pub fn FPSTrackerPlugin(app: &mut App)
{
    app
        .insert_resource::<FPSTracker>(FPSTracker::new(FPS_TRACKER_NUM_RECORDS))
        .configure_set(First, FPSTrackerSet.after(bevy::time::TimeSystem))
        .add_systems(First, update_fps_tracker.in_set(FPSTrackerSet));
}

//-------------------------------------------------------------------------------------------------------------------

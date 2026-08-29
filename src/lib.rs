use bevy_app::{App, First, Plugin};
use bevy_ecs::{
    change_detection::{Res, ResMut},
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::Local,
    world::World,
};
use bevy_log::warn;
use bevy_time::{TimeReceiver, TimeSystems, TimeUpdateStrategy};
use std::time::{Duration, Instant};

pub use time_smoothing::TimeSmoothing;

mod time_smoothing;

/// Smooths Bevy's frame delta time.
///
/// Runs before Bevy's time systems each frame and sets [`TimeUpdateStrategy::ManualDuration`] to
/// the smoothed delta time.
///
/// See [`TimeSmoothing::new`] for configuration.
pub struct TimeSmoothingPlugin {
    pub window_size: usize,
    pub time_constant: f64,
}

impl Default for TimeSmoothingPlugin {
    fn default() -> Self {
        Self {
            window_size: 7,
            time_constant: 0.1,
        }
    }
}

impl Plugin for TimeSmoothingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeSmoothing::new(self.window_size, self.time_constant))
            .add_systems(
                First,
                (take_time_receiver, update_time_update_strategy)
                    .chain()
                    .before(TimeSystems),
            );
    }
}

// Workaround to ensure this is the only receiver reading from it.
#[derive(Resource)]
struct TimeReceiverHolder(TimeReceiver);

fn take_time_receiver(world: &mut World) {
    if let Some(TimeReceiver(receiver)) = world.remove_resource::<TimeReceiver>() {
        world.insert_resource(TimeReceiverHolder(TimeReceiver(receiver)));
    }
}

fn update_time_update_strategy(
    mut time_smoothing: ResMut<TimeSmoothing>,
    mut update_strategy: ResMut<TimeUpdateStrategy>,
    time_recv: Option<Res<TimeReceiverHolder>>,
    mut has_received_time: Local<bool>,
    mut last_instant: Local<Option<Instant>>,
) {
    let sent_time = match time_recv.map(|res| res.0.0.try_recv()) {
        Some(Ok(new_time)) => {
            *has_received_time = true;
            Some(new_time)
        }
        Some(Err(_)) => {
            if *has_received_time {
                warn!(
                    "time_system did not receive the time from the render world! \
                    Calculations depending on the time may be incorrect.",
                );
            }
            None
        }
        None => None,
    };

    let instant = sent_time.unwrap_or_else(Instant::now);

    let smoothed_delta = if let Some(last_instant) = *last_instant {
        let delta = instant - last_instant;
        Duration::from_secs_f64(time_smoothing.update(delta.as_secs_f64()))
    } else {
        Duration::ZERO
    };

    *update_strategy = TimeUpdateStrategy::ManualDuration(smoothed_delta);

    *last_instant = Some(instant);
}

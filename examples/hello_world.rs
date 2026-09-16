use bevy::prelude::*;
use bevy_mod_time_smoothing::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TimeSmoothingPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, log_delta_time)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
    commands.spawn(Sprite::sized(Vec2::splat(210.0)));
}

fn log_delta_time(time: Res<Time>, time_smoothing: Res<TimeSmoothing>) {
    info!(
        "raw: {:.6} | smoothed: {:.6}",
        time_smoothing.raw_delta().as_secs_f64(),
        time.delta_secs_f64(),
    );
}

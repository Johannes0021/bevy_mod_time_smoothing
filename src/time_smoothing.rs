use std::num::NonZeroUsize;
use std::time::Duration;

use bevy_ecs::resource::Resource;

//==================================================================================================
// TimeSmoothingConfig
//==================================================================================================

#[derive(Debug, Clone, Copy)]
pub struct TimeSmoothingConfig {
    /// Number of samples used to calculate the average.
    pub average_count: NonZeroUsize,
    /// Number of samples ignored on each side of the sorted samples.
    pub ignore_side_count: usize,
    /// Controls how quickly the smoothed value reacts.
    pub time_constant: f64,
}

impl Default for TimeSmoothingConfig {
    fn default() -> Self {
        Self {
            average_count: NonZeroUsize::new(6).unwrap(),
            ignore_side_count: 2,
            time_constant: 0.1,
        }
    }
}

impl TimeSmoothingConfig {
    pub fn window_size(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.average_count.get() + (self.ignore_side_count * 2)).unwrap()
    }
}

//==================================================================================================
// TimeSmoothing
//==================================================================================================

#[derive(Resource, Debug, Clone)]
pub struct TimeSmoothing {
    samples: Vec<Duration>,
    sorted_samples: Vec<Duration>,
    config: TimeSmoothingConfig,
    smoothed: Duration,
}

impl TimeSmoothing {
    pub fn new(config: TimeSmoothingConfig) -> Self {
        let window_size = config.window_size();

        Self {
            samples: Vec::with_capacity(window_size.get()),
            sorted_samples: Vec::with_capacity(window_size.get()),
            config,
            smoothed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, delta: Duration) -> Duration {
        self.samples.push(delta);

        if self.samples.len() > self.config.window_size().get() {
            self.samples.remove(0);
        }

        self.sorted_samples.clear();
        self.sorted_samples.extend(self.samples.iter().copied());
        self.sorted_samples.sort_unstable();

        let (start, end) = if self.sorted_samples.len() >= self.config.window_size().get() {
            (
                self.config.ignore_side_count,
                self.config.ignore_side_count + self.config.average_count.get(),
            )
        } else {
            (0, self.sorted_samples.len())
        };

        let average = self.sorted_samples[start..end]
            .iter()
            .copied()
            .sum::<Duration>()
            / (end - start) as u32;

        if self.config.time_constant == 0.0 {
            self.smoothed = average;
        } else {
            let delta_secs = average.as_secs_f64();
            let alpha = 1.0 - (-delta_secs / self.config.time_constant).exp();

            let current = self.smoothed.as_secs_f64();
            let target = average.as_secs_f64();

            self.smoothed = Duration::from_secs_f64(current + (alpha * (target - current)));
        }

        self.smoothed
    }

    pub fn smoothed_delta(&self) -> Duration {
        self.smoothed
    }

    pub fn raw_delta(&self) -> Duration {
        self.samples.last().copied().unwrap_or_default()
    }

    pub fn samples(&self) -> &[Duration] {
        &self.samples
    }

    pub fn sorted_samples(&self) -> &[Duration] {
        &self.sorted_samples
    }

    pub fn config(&self) -> &TimeSmoothingConfig {
        &self.config
    }
}

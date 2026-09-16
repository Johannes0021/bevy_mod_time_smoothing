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
    /// If set, intervals above this threshold are used as the raw smoothed delta time for the
    /// current frame and are not added to the smoothing history.
    pub delta_outlier_threshold: Option<Duration>,
}

impl Default for TimeSmoothingConfig {
    fn default() -> Self {
        Self {
            average_count: NonZeroUsize::new(6).unwrap(),
            ignore_side_count: 2,
            time_constant: 0.1,
            delta_outlier_threshold: Some(Duration::from_millis(100)),
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
    raw: Duration,
    filtered_smoothed: Duration,
    smoothed: Duration,
}

impl TimeSmoothing {
    pub fn new(config: TimeSmoothingConfig) -> Self {
        let window_size = config.window_size();

        Self {
            samples: Vec::with_capacity(window_size.get()),
            sorted_samples: Vec::with_capacity(window_size.get()),
            config,
            raw: Duration::ZERO,
            filtered_smoothed: Duration::ZERO,
            smoothed: Duration::ZERO,
        }
    }

    pub fn update(&mut self, delta: Duration) -> Duration {
        self.raw = delta;

        if self
            .config
            .delta_outlier_threshold
            .is_some_and(|threshold| delta > threshold)
            || self.samples.is_empty() && delta.is_zero()
        {
            self.smoothed = delta;
            return self.raw_delta();
        }

        let startup_done = self.sorted_samples.len() >= self.config.window_size().get();

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
        } else if self.sorted_samples.len() > self.config.average_count.get() {
            let excess = self.sorted_samples.len() - self.config.average_count.get();
            let ignore_left = excess / 2;
            let ignore_right = excess - ignore_left;

            (ignore_left, self.sorted_samples.len() - ignore_right)
        } else {
            (0, self.sorted_samples.len())
        };

        let average = self.sorted_samples[start..end]
            .iter()
            .copied()
            .sum::<Duration>()
            / (end - start) as u32;

        self.filtered_smoothed = if self.config.time_constant == 0.0 || !startup_done {
            average
        } else {
            let delta_secs = average.as_secs_f64();
            let alpha = 1.0 - (-delta_secs / self.config.time_constant).exp();

            let current = self.filtered_smoothed.as_secs_f64();
            let target = average.as_secs_f64();

            Duration::from_secs_f64(current + (alpha * (target - current)))
        };
        self.smoothed = self.filtered_smoothed;

        self.smoothed
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

    pub fn raw_delta(&self) -> Duration {
        self.raw
    }

    pub fn smoothed_delta(&self) -> Duration {
        self.smoothed
    }
}

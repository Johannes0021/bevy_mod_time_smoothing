use bevy_ecs::resource::Resource;

#[derive(Resource, Debug, Clone)]
pub struct TimeSmoothing {
    samples: Vec<f64>,
    sorted_samples: Vec<f64>,
    window_size: usize,
    time_constant: f64,
    value: f64,
}

impl TimeSmoothing {
    /// `window_size` should be an odd number, e.g. 5 or 7.
    /// `time_constant` controls how quickly the smoothed value reacts, e.g. 50ms or 100ms.
    ///
    /// # Panics
    ///
    /// Panics if `window_size` is zero or even, or if `time_constant` is not greater than zero.
    pub fn new(window_size: usize, time_constant: f64) -> Self {
        assert!(
            window_size > 0 && !window_size.is_multiple_of(2),
            "window_size must be a positive odd number"
        );
        assert!(
            time_constant > 0.0,
            "time_constant must be greater than zero"
        );

        Self {
            samples: Vec::with_capacity(window_size),
            sorted_samples: Vec::with_capacity(window_size),
            window_size,
            value: 0.0,
            time_constant,
        }
    }

    pub fn update(&mut self, dt: f64) -> f64 {
        debug_assert!(
            dt.is_finite() && dt >= 0.0,
            "dt must be finite and non-negative"
        );

        if !dt.is_finite() || dt < 0.0 {
            return self.value;
        }

        if self.samples.is_empty() {
            self.samples.push(dt);
            self.sorted_samples.push(dt);
            self.value = dt;

            return self.value;
        }

        self.samples.push(dt);

        if self.samples.len() > self.window_size {
            self.samples.remove(0);
        }

        self.sorted_samples.clear();
        self.sorted_samples.extend(self.samples.iter().copied());
        self.sorted_samples.sort_unstable_by(f64::total_cmp);

        let median = self.sorted_samples[self.sorted_samples.len() / 2];

        let alpha = 1.0 - (-dt / self.time_constant).exp();

        self.value += alpha * (median - self.value);

        self.value
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn raw_delta_secs(&self) -> f64 {
        self.samples.last().copied().unwrap_or_default()
    }

    pub fn samples(&self) -> &[f64] {
        &self.samples
    }

    pub fn sorted_samples(&self) -> &[f64] {
        &self.sorted_samples
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }

    pub fn time_constant(&self) -> f64 {
        self.time_constant
    }
}

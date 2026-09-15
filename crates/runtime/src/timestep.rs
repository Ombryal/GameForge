// Accumulator-based fixed timestep. Wall-clock frame time is irregular —
// feed it in as `dt` and this converts it into a whole number of fixed
// steps, holding the remainder for next time. Keeps simulation
// deterministic across different frame rates instead of scaling physics
// by whatever dt happened to land.
pub struct FixedTimestep {
    step: f32,
    accumulator: f32,
}

impl FixedTimestep {
    pub fn new(step: f32) -> Self {
        Self { step, accumulator: 0.0 }
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn advance(&mut self, dt: f32) -> u32 {
        self.accumulator += dt;
        let mut steps = 0;
        while self.accumulator >= self.step {
            self.accumulator -= self.step;
            steps += 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_step_dt_produces_no_steps() {
        let mut ts = FixedTimestep::new(1.0 / 60.0);
        assert_eq!(ts.advance(0.001), 0);
    }

    #[test]
    fn dt_over_multiple_steps_runs_all_of_them() {
        let mut ts = FixedTimestep::new(0.1);
        assert_eq!(ts.advance(0.25), 2);
    }

    #[test]
    fn remainder_carries_into_the_next_advance() {
        let mut ts = FixedTimestep::new(0.1);
        ts.advance(0.05);
        assert_eq!(ts.advance(0.05), 1);
    }
}

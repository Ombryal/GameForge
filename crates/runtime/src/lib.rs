mod timestep;

pub use timestep::FixedTimestep;

// Drives the timestep and counts ticks. Deliberately takes dt as a plain
// argument rather than reading a clock internally — that's what makes
// this testable without sleeping, and it's also what a headless test
// runner or a replay system would want anyway. Wiring to an actual
// wall-clock and a window loop happens in platform, not here.
pub struct Runtime {
    timestep: FixedTimestep,
    tick_count: u64,
}

impl Runtime {
    pub fn new(fixed_dt: f32) -> Self {
        Self {
            timestep: FixedTimestep::new(fixed_dt),
            tick_count: 0,
        }
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    // Runs on_tick once per fixed step due this call, in order. Returns
    // how many steps actually ran, since callers driving a render after
    // the ticks may care whether anything happened at all.
    pub fn advance(&mut self, dt: f32, mut on_tick: impl FnMut(f32)) -> u32 {
        let steps = self.timestep.advance(dt);
        for _ in 0..steps {
            on_tick(self.timestep.step());
            self.tick_count += 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_step_dt_ticks_once() {
        let mut rt = Runtime::new(1.0 / 60.0);
        let mut calls = 0;
        rt.advance(1.0 / 60.0, |_| calls += 1);
        assert_eq!(calls, 1);
        assert_eq!(rt.tick_count(), 1);
    }

    #[test]
    fn large_dt_ticks_multiple_times_in_one_call() {
        let mut rt = Runtime::new(0.1);
        let mut calls = 0;
        rt.advance(0.35, |_| calls += 1);
        assert_eq!(calls, 3);
        assert_eq!(rt.tick_count(), 3);
    }

    #[test]
    fn sub_step_dt_does_not_tick() {
        let mut rt = Runtime::new(1.0 / 60.0);
        let mut calls = 0;
        rt.advance(0.001, |_| calls += 1);
        assert_eq!(calls, 0);
        assert_eq!(rt.tick_count(), 0);
    }
}

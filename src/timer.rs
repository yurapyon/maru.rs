use std::{
    thread,
    time::{
        self,
        Instant,
    }
};

// TODO
//   fixed update timer
//   .sleep() ?
//   average fps/delta needs ring buffer?

// timer should be stepped after initial load
// timer.start()?

pub struct Timer {
    last: Instant,
    time_dt: f64,
}

impl Timer {
    pub fn new() -> Self {
        let last = Instant::now();
        let time_dt = 0.;
        Self {
            last,
            time_dt,
        }
    }

    pub fn step(&mut self) -> f64 {
        self.time_dt = self.last.elapsed().as_secs_f64();
        self.last = Instant::now();
        self.time_dt
    }

    #[inline]
    pub fn delta_time(&self) -> f64 {
        self.time_dt
    }

    #[inline]
    pub fn sleep_millis(&self, millis: u64) {
        thread::sleep(time::Duration::from_millis(millis));
    }
}

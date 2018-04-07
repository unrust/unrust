use uni_app::now;
use std;
use std::collections::VecDeque;

pub struct DeltaTimeStats {
    pub dt_max: f64,
    pub dt_min: f64,
    pub dt_avg: f64,

    pub dt_history: VecDeque<f64>,
}

impl DeltaTimeStats {
    fn new() -> DeltaTimeStats {
        DeltaTimeStats {
            dt_max: std::f64::MIN,
            dt_min: std::f64::MAX,
            dt_avg: 0.0,
            dt_history: VecDeque::new(),
        }
    }

    fn update(&mut self, dt: f64) {
        self.dt_max =
            if self.dt_max > dt { self.dt_max } else { dt };
        self.dt_min =
            if self.dt_min < dt { self.dt_min } else { dt };

        self.dt_history.push_back(dt);
        if self.dt_history.len() > 60 {
            self.dt_history.pop_front();
        }

        self.dt_avg =
            self.dt_history.iter().fold(0.0, |acc, dt| acc + *dt) / (self.dt_history.len() as f64);
    }
}

pub struct FPS {
    counter: u32,
    delta_time: f64,

    delta_time_stats: DeltaTimeStats,
    last_delta_time_stats: DeltaTimeStats,

    last_second: f64,
    last_frame: f64,
    pub fps: u32,
}

impl FPS {
    pub fn new() -> FPS {
        let fps = FPS {
            counter: 0,
            last_second: now(),
            last_frame: now(),
            fps: 0,
            delta_time: 0.0,
            delta_time_stats: DeltaTimeStats::new(),
            last_delta_time_stats: DeltaTimeStats::new(),
        };

        fps
    }

    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn delta_time_stats(&self) -> &DeltaTimeStats {
        &self.last_delta_time_stats
    }

    pub fn step(&mut self) {
        self.counter += 1;
        let curr = now();
        self.delta_time = curr - self.last_frame;
        self.delta_time_stats.update(self.delta_time);

        if curr - self.last_second > 1.0 {
            self.last_second = curr;
            self.fps = self.counter;
            self.counter = 0;

            std::mem::swap(&mut self.last_delta_time_stats, &mut self.delta_time_stats);
            self.delta_time_stats = DeltaTimeStats::new();
        }

        self.last_frame = curr;
    }
}

use uni_app::now;

pub struct FPS {
    counter: u32,
    delta_time: f64,
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
        };

        fps
    }

    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn step(&mut self) {
        self.counter += 1;
        let curr = now();
        self.delta_time = curr - self.last_frame;

        if curr - self.last_second > 1.0 {
            self.last_second = curr;
            self.fps = self.counter;
            self.counter = 0;
        }

        self.last_frame = curr;
    }
}

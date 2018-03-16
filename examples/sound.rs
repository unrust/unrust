extern crate uni_app;
extern crate uni_snd;
extern crate unrust;

use uni_app::App;
use uni_snd::{SoundDriver, SoundGenerator};

use unrust::world::{Actor, Camera, World, WorldBuilder};
use unrust::engine::GameObject;
use unrust::world::events::AppEvent;

// GUI
use unrust::imgui;

struct SinGenerator {
    sample_rate: f32,
    frequency: f32,
    i: usize,
    channel: usize,
    volume: f32,
}

impl SinGenerator {
    pub fn new(volume: f32) -> Self {
        Self {
            sample_rate: 44000.0,
            frequency: 440.0,
            i: 0,
            channel: 0,
            volume,
        }
    }
}

impl SoundGenerator<f32> for SinGenerator {
    fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
    fn handle_event(&mut self, event: f32) {
        self.frequency = event;
        App::print(format!("new frequency: {}\n", event));
    }
    fn next_value(&mut self) -> f32 {
        if self.channel == 1 {
            self.i += 1;
            self.channel = 0;
        } else {
            self.channel = 1;
        }
        ((self.i as f32 % self.sample_rate) * self.frequency * 2.0 * 3.14159 / self.sample_rate)
            .sin() * self.volume
    }
}

struct SoundEngine {
    driver: SoundDriver<f32>,
    frequency: f32,
}

impl SoundEngine {
    pub fn new() -> Box<Actor> {
        Box::new(Self {
            driver: SoundDriver::new(Box::new(SinGenerator::new(0.2))),
            frequency: 440.0,
        })
    }
}

impl Actor for SoundEngine {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        self.driver.start();
        // add main camera to scene
        {
            let go = world.new_game_object();
            go.borrow_mut().add_component(Camera::default());
        }
    }
    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        self.driver.frame();
        for evt in world.events().iter() {
            match evt {
                &AppEvent::Click(_) => {
                    self.frequency = 600.0 - self.frequency;
                    App::print(format!("sending new frequency event: {}\n", self.frequency));
                    self.driver.send_event(self.frequency);
                }
                _ => (),
            }
        }
        // GUI
        use imgui::Metric::*;
        imgui::label(
            Native(0.5, 0.5),
            "Click of the canvas\nto change the sound frequency",
        );
    }
}

pub fn main() {
    let mut world = WorldBuilder::new("Sound demo")
        .with_size((640, 480))
        .with_stats(true)
        .build();

    // Add the main scene as component of scene game object
    let scene = world.new_game_object();
    scene.borrow_mut().add_component(SoundEngine::new());
    drop(scene);

    world.event_loop();
}

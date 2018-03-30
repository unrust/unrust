extern crate uni_app;
extern crate uni_snd;
extern crate unrust;

use uni_app::App;

use unrust::world::{Actor, Camera, World, WorldBuilder};
use unrust::engine::{GameObject, SoundHandle};
use unrust::world::events::AppEvent;

// GUI
use unrust::imgui;

struct SoundEmitter {
    flute_id: SoundHandle,
    sword_id: SoundHandle,
    mouse_pos: f64,
}

impl SoundEmitter {
    pub fn new(world: &mut World) -> Box<Actor> {
        let flute_id = world
            .sound
            .load_sound("static/sponza/sounds/flute_48000.wav");
        let sword_id = world.sound.load_sound("static/sponza/sounds/sword.wav");
        Box::new(Self {
            flute_id,
            sword_id,
            mouse_pos: 0.0,
        })
    }
}

impl Actor for SoundEmitter {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        let go = world.new_game_object();
        go.borrow_mut().add_component(Camera::default());
        // flute is priority 1 so that it doesn't get replaced by a sword if all channels are used
        world
            .sound
            .play_sound(self.flute_id, None, true, 1, 0.5, 0.5);
    }
    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        let mut sword = false;
        for evt in world.events().iter() {
            match evt {
                &AppEvent::MouseUp(_) => {
                    sword = true;
                }
                &AppEvent::MousePos((x, _)) => {
                    self.mouse_pos = x;
                }
                _ => (),
            }
        }
        if sword {
            // play sword sound on click, using mouse position to balance
            world.sound.play_sound(
                self.sword_id,
                None,
                false,
                0,
                1.0,
                self.mouse_pos as f32 / 640.0,
            );
        }
        use imgui::Metric::*;

        imgui::pivot((1.0, 1.0));
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "left click to hit with your sword!",
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
    scene
        .borrow_mut()
        .add_component(SoundEmitter::new(&mut world));
    drop(scene);

    world.event_loop();
}

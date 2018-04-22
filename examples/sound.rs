extern crate uni_app;
extern crate uni_snd;
extern crate unrust;

#[macro_use]
extern crate unrust_derive;

use unrust::world::{Actor, Camera, World, WorldBuilder};
use unrust::engine::{GameObject, SoundHandle};
use unrust::world::events::AppEvent;

// GUI
use unrust::imgui;

#[derive(Actor)]
struct SoundEmitter {
    flute_id: SoundHandle,
    sword_id: SoundHandle,
    mouse_pos: f64,
    flute_on: bool,
}

impl SoundEmitter {
    pub fn new(world: &mut World) -> SoundEmitter {
        let flute_id = world.sound.load_sound("sounds/flute_48000.wav");
        let sword_id = world.sound.load_sound("sounds/sword.wav");
        Self {
            flute_id,
            sword_id,
            mouse_pos: 0.0,
            flute_on: true,
        }
    }
}

impl Actor for SoundEmitter {
    fn start(&mut self, _go: &mut GameObject, world: &mut World) {
        // add main camera to scene
        let go = world.new_game_object();
        go.borrow_mut().add_component(Camera::default());
        // flute is priority 1 so that it doesn't get replaced by a sword if all channels are used
        // we also force the use of channel 0 to be able to stop the loop
        world
            .sound
            .play_sound(self.flute_id, Some(0), true, 1, 0.5, 0.5);
    }
    fn update(&mut self, _go: &mut GameObject, world: &mut World) {
        let mut sword = false;
        let mut flute = false;
        for evt in world.events().iter() {
            match evt {
                &AppEvent::MouseUp(ref event) => {
                    if event.button == 0 {
                        sword = true;
                    } else if event.button == 2 {
                        flute = true;
                    }
                }
                &AppEvent::MousePos((x, _)) => {
                    self.mouse_pos = x;
                }
                _ => (),
            }
        }
        if sword {
            // play sword sound on left click, using mouse position to balance
            world.sound.play_sound(
                self.sword_id,
                None,
                false,
                0,
                1.0,
                self.mouse_pos as f32 / 640.0,
            );
        }
        if flute {
            // start/stop flute on right click
            if self.flute_on {
                world.sound.stop_channel(0);
            } else {
                world
                    .sound
                    .play_sound(self.flute_id, Some(0), true, 1, 0.5, 0.5);
            }
            self.flute_on = !self.flute_on;
        }
        use imgui::Metric::*;

        imgui::pivot((1.0, 1.0));
        imgui::label(
            Native(1.0, 1.0) - Pixel(8.0, 8.0),
            "right click to start/stop playing flute\nleft click to hit with your sword!",
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

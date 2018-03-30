mod generator;
mod channel;

use std::collections::HashMap;
use uni_snd::SoundDriver;

use self::generator::Generator;

const CHANNEL_COUNT: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct SoundHandle(usize);

pub struct SoundSystem {
    cache: HashMap<String, SoundHandle>,
    next_handle: usize,
    driver: SoundDriver<SoundEvent>,
}

impl SoundSystem {
    pub fn new() -> Self {
        let mut driver = SoundDriver::new(Box::new(Generator::new(CHANNEL_COUNT)));
        driver.start();
        Self {
            cache: HashMap::new(),
            next_handle: 0,
            driver,
        }
    }
    pub fn load_sound(&mut self, filepath: &str) -> SoundHandle {
        let buffer = self.cache.get(&filepath.to_owned());
        match buffer {
            None => {
                let id = self.next_handle;
                self.next_handle += 1;
                self.driver
                    .send_event(SoundEvent::LoadBuffer(id, filepath.to_owned()));
                SoundHandle(id)
            }
            Some(buf) => *buf,
        }
    }
    pub fn play_sound(
        &mut self,
        id: SoundHandle,
        channel: Option<usize>,
        do_loop: bool,
        priority: usize,
        volume: f32,
        balance: f32,
    ) {
        self.driver.send_event(SoundEvent::Play(SoundPlayEvent {
            id: id.0,
            channel,
            do_loop,
            priority,
            volume,
            balance,
        }))
    }
}

enum SoundEvent {
    LoadBuffer(usize, String),
    Play(SoundPlayEvent),
}

#[derive(Clone, Copy)]
pub struct SoundPlayEvent {
    id: usize,
    channel: Option<usize>,
    do_loop: bool,
    priority: usize,
    volume: f32,
    balance: f32,
}

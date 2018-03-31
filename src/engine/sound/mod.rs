mod generator;
mod channel;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;
use uni_snd::SoundDriver;
use engine::{AssetError, AssetSystem};
use futures::Future;
use std::collections::BTreeSet;

use self::generator::Generator;

const CHANNEL_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct SoundHandle(usize);

pub struct SoundSystem {
    cache: HashMap<String, SoundHandle>,

    loading: Rc<RefCell<BTreeSet<SoundHandle>>>,
    pending_play: Vec<SoundPlayEvent>,

    next_handle: usize,
    driver: Rc<RefCell<SoundDriver<SoundEvent>>>,
    asys: Box<AssetSystem>,
}

impl SoundSystem {
    pub fn new(asys: Box<AssetSystem>) -> Self {
        let mut driver = SoundDriver::new(Box::new(Generator::new(CHANNEL_COUNT)));
        driver.start();
        Self {
            cache: HashMap::new(),
            next_handle: 0,
            driver: Rc::new(RefCell::new(driver)),
            loading: Rc::new(RefCell::new(BTreeSet::new())),
            pending_play: Vec::new(),
            asys,
        }
    }
    pub fn load_sound(&mut self, filepath: &str) -> SoundHandle {
        let filepath = filepath.to_owned();
        let buffer = self.cache.get(&filepath);
        match buffer {
            None => {
                let id = self.next_handle;
                self.next_handle += 1;
                let f = self.asys.new_file(&filepath);
                let driver = self.driver.clone();

                let load_f = f.and_then({
                    let filepath = filepath.clone();
                    let loading = self.loading.clone();
                    move |mut fdata| {
                        driver.borrow_mut().send_event(SoundEvent::LoadBuffer(
                            id,
                            fdata.read_binary()?,
                            filepath,
                        ));

                        loading.borrow_mut().remove(&SoundHandle(id));

                        Ok(())
                    }
                });

                self.asys
                    .execute(Box::new(load_f.map_err(|e| AssetError::FileIoError(e))));

                self.cache.insert(filepath.clone(), SoundHandle(id));
                self.loading.borrow_mut().insert(SoundHandle(id));

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
        let evt = SoundPlayEvent {
            id: id.0,
            channel,
            do_loop,
            priority,
            volume,
            balance,
        };

        if self.loading.borrow().contains(&id) {
            self.pending_play.push(evt);
            return;
        }

        self.driver.borrow_mut().send_event(SoundEvent::Play(evt))
    }

    pub fn step(&mut self) {
        let mut pending: Vec<_> = self.pending_play.drain(0..).collect();

        pending = pending
            .into_iter()
            .filter(|evt| {
                if self.loading.borrow().contains(&SoundHandle(evt.id)) {
                    return true;
                } else {
                    self.driver.borrow_mut().send_event(SoundEvent::Play(*evt));
                    return false;
                }
            })
            .collect();

        self.pending_play = pending;

        self.driver.borrow_mut().frame();
    }
}

enum SoundEvent {
    LoadBuffer(usize, Vec<u8>, String),
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

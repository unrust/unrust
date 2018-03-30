use std::collections::HashMap;
use std::sync::Arc;

use uni_snd::SoundGenerator;
use wavefile::WaveFile;

use super::{SoundEvent, SoundPlayEvent};
use super::channel::Channel;

pub struct SoundBuffer {
    /// number of channels. 1:mono, 2: stereo
    pub output_count: usize,
    /// audio samples per second
    pub sample_rate: usize,
    /// samples between -1.0 and 1.0
    pub samples: Vec<f32>,
}

pub struct Generator {
    cache: HashMap<usize, Arc<SoundBuffer>>,
    channels: Vec<Channel>,
    next_channel: usize,
}

impl Generator {
    pub fn new(channel_count: usize) -> Self {
        let mut channels = Vec::new();
        for _ in 0..channel_count {
            channels.push(Channel::new());
        }
        Self {
            cache: HashMap::new(),
            channels,
            next_channel: 0,
        }
    }
    fn handle_play_event(&mut self, evt: &SoundPlayEvent) {
        let mut free_channel_id = evt.channel;
        if free_channel_id.is_none() {
            // find a free channel
            for (id, channel) in self.channels.iter().enumerate() {
                if channel.is_free() {
                    free_channel_id = Some(id);
                    self.next_channel = id;
                    break;
                }
            }
        }
        if free_channel_id.is_none() {
            // no free channel. find a channel with a lower priority sound
            let last_channel = self.next_channel;
            // use a round robin algorithm to avoid using always the same channel
            self.next_channel = (self.next_channel + 1) % self.channels.len();
            while self.next_channel != last_channel
                && self.channels[self.next_channel].get_priority() > evt.priority
            {
                self.next_channel = (self.next_channel + 1) % self.channels.len();
            }
            if self.next_channel != last_channel {
                free_channel_id = Some(self.next_channel);
            }
        }
        match free_channel_id {
            None => return, // no channel available. skip this sound
            Some(id) => {
                self.channels[id].set_event(*evt, self.cache.get(&evt.id).unwrap().clone());
            }
        }
    }
    fn handle_load_buffer_event(&mut self, id: usize, filepath: String) {
        let new_buf = self.new_buffer(&filepath);
        self.cache.insert(id, Arc::new(new_buf));
    }
    fn new_buffer(&mut self, filepath: &str) -> SoundBuffer {
        let wav = WaveFile::open(filepath).expect(&format!("error cannot open {}", filepath));
        println!(
            "loading sound {} channels {} sample rate {} bits per sample {}",
            filepath,
            wav.channels(),
            wav.sample_rate(),
            wav.bits_per_sample()
        );

        let mut buffer = SoundBuffer {
            output_count: wav.channels(),
            sample_rate: wav.sample_rate(),
            samples: Vec::new(),
        };
        let coef = 2.0 / (1 << wav.bits_per_sample()) as f32;
        if wav.channels() == 1 {
            // mono sample
            for frame in wav.iter() {
                buffer.samples.push(frame[0] as f32 * coef);
            }
        } else {
            // stereo sample
            for frame in wav.iter() {
                buffer.samples.push(frame[0] as f32 * coef);
                buffer.samples.push(frame[1] as f32 * coef);
            }
        }
        buffer
    }
}

impl SoundGenerator<SoundEvent> for Generator {
    fn init(&mut self, sample_rate: f32) {
        for chan in self.channels.iter_mut() {
            chan.set_sample_rate(sample_rate);
        }
    }
    fn handle_event(&mut self, evt: SoundEvent) {
        match evt {
            SoundEvent::Play(ref play_evt) => self.handle_play_event(play_evt),
            SoundEvent::LoadBuffer(id, filepath) => self.handle_load_buffer_event(id, filepath),
        }
    }
    fn next_value(&mut self) -> f32 {
        let mut sample = 0.0;
        for chan in self.channels.iter_mut() {
            sample += chan.next_value();
        }
        sample / self.channels.len() as f32
    }
}

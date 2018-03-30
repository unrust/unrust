use std::sync::Arc;

use super::SoundPlayEvent;
use super::generator::SoundBuffer;

pub struct Channel {
    event: Option<SoundPlayEvent>,
    buffer: Option<Arc<SoundBuffer>>,
    sample_rate: f32,
    t: f32,
    delta_t: f32,
    cur_output: usize,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            event: None,
            buffer: None,
            t: 0.0,
            delta_t: 0.0,
            sample_rate: 1.0,
            cur_output: 0,
        }
    }
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
    pub fn set_event(&mut self, evt: SoundPlayEvent, buffer: Arc<SoundBuffer>) {
        self.event = Some(evt);
        self.delta_t = buffer.sample_rate as f32 / self.sample_rate;
        self.buffer = Some(buffer);
        self.cur_output = 0;
        self.t = 0.0;
    }
    pub fn is_free(&self) -> bool {
        self.event.is_none()
    }
    pub fn next_value(&mut self) -> f32 {
        let mut ret = 0.0;
        if let Some(ref buffer) = self.buffer {
            let sample_idx = self.t as usize;
            if buffer.output_count == 1 {
                ret = buffer.samples[sample_idx];
            } else {
                ret = buffer.samples[sample_idx * buffer.output_count + self.cur_output];
            }
            if self.delta_t != 1.0 {
                // interpolate samples when buffer sample rate is not equal to driver sample rate
                let interpol_coef = self.t - sample_idx as f32;
                if interpol_coef > 0.0 {
                    let next_sample =
                        if buffer.output_count == 1 && sample_idx + 1 < buffer.samples.len() {
                            buffer.samples[sample_idx + 1]
                        } else if (sample_idx + 1) * buffer.output_count + self.cur_output
                            < buffer.samples.len()
                        {
                            buffer.samples[(sample_idx + 1) * buffer.output_count + self.cur_output]
                        } else {
                            0.0
                        };
                    ret = (1.0 - interpol_coef) * ret + interpol_coef * next_sample;
                }
            }
            // balance and volume
            let event = self.event.unwrap();
            if event.balance != 0.5 {
                ret *= event.balance * self.cur_output as f32
                    + (1.0 - event.balance) * (1.0 - self.cur_output as f32);
            }
            ret *= event.volume;
            // alternate between left/right output channels
            self.cur_output = 1 - self.cur_output;
            if self.cur_output == 0 {
                self.t += self.delta_t;
                if self.t as usize * buffer.output_count >= buffer.samples.len() {
                    if self.event.unwrap().do_loop {
                        self.t = 0.0;
                    } else {
                        self.clear();
                    }
                }
            }
        }
        ret
    }
    pub fn clear(&mut self) {
        self.event = None;
        self.buffer = None;
    }
    pub fn get_priority(&self) -> usize {
        if let Some(ref event) = self.event {
            return event.priority;
        }
        0
    }
}

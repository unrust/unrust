use stdweb;
use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;

use uni_app::App;

use super::{SoundError, SoundGenerator};

pub struct SoundDriver<T> {
    generator: Option<Box<SoundGenerator<T>>>,
    ctx: stdweb::Value,
    start_audio: f64,
    buffer: [f32; BUFFER_SIZE as usize * 2],
    err: SoundError,
}

const BUFFER_SIZE: u32 = 2048;
const AUDIO_LATENCY: f64 = 0.1;

enum GameStatus {
    Running,
    Paused,
    Resumed(f64),
}

impl<T> SoundDriver<T> {
    pub fn get_error(&self) -> SoundError {
        self.err
    }

    pub fn new(generator: Box<SoundGenerator<T>>) -> Self {
        let ctx = js! {
            window.startPause=0;
            window.endPause=0;
            document.addEventListener("visibilitychange", (e) => {
                if (document.hidden) {
                    window.startPause = performance.now() / 1000.0;
                } else {
                    window.endPause = performance.now() / 1000.0;
                }
            }, false);
            if (AudioContext) {
                return new AudioContext();
            } else {
                return undefined;
            }
        };
        let err = if ctx == stdweb::Value::Undefined {
            SoundError::NoDevice
        } else {
            SoundError::NoError
        };
        Self {
            generator: Some(generator),
            ctx,
            start_audio: 0.0,
            buffer: [0.0; BUFFER_SIZE as usize * 2],
            err,
        }
    }
    // -1 => game paused
    // >0 => pause duration
    fn get_pause_status(&self) -> GameStatus {
        let value: Option<f64> = js! {
            var duration = window.endPause-window.startPause;
            if (duration > 0) {
                window.endPause = 0;
                window.startPause=0;
                return duration;
            } else if (window.startPause > 0) {
                return -1;
            } else {
                return undefined;
            }
        }.try_into()
            .unwrap();
        match value {
            None => GameStatus::Running,
            Some(v) => if v == -1.0 {GameStatus::Paused} else {GameStatus::Resumed(v)}
        }
    }
    pub fn send_event(&mut self, event: T) {
        if let Some(ref mut gen) = self.generator {
            gen.handle_event(event);
        }
    }
    pub fn frame(&mut self) {
        match self.get_pause_status() {
            GameStatus::Paused => {return;}
            GameStatus::Resumed(duration) =>  self.start_audio += duration,
            GameStatus::Running => (),
        }
        let now: f64 = js! {
            return @{&self.ctx}.currentTime;
        }.try_into()
            .unwrap();
        let now_latency = now + AUDIO_LATENCY;
        if self.start_audio == 0.0 {
            self.start_audio = now_latency;
        }
        if now >= self.start_audio - AUDIO_LATENCY {
            if let Some(ref mut gen) = self.generator {
                for i in 0..BUFFER_SIZE as usize * 2 {
                    self.buffer[i] = gen.next_value();
                }
            }
            let typed_array: TypedArray<f32> = (&self.buffer[..]).into();
            let samples: f64 = js! {
                var buffer=@{&self.ctx}.createBuffer(2,@{BUFFER_SIZE},@{&self.ctx}.sampleRate);
                var channel0 = buffer.getChannelData(0);
                var channel1 = buffer.getChannelData(1);
                var obuf=@{typed_array};
                for (var i=0,j=0; i < @{BUFFER_SIZE}; i++) {
                    channel0[i] = obuf[j++];
                    channel1[i] = obuf[j++];
                }
                var bufferSource = @{&self.ctx}.createBufferSource();
                bufferSource.buffer = buffer;
                bufferSource.connect(@{&self.ctx}.destination);
                bufferSource.start(@{&self.start_audio});
                var bufferSec = @{BUFFER_SIZE} / @{&self.ctx}.sampleRate;
                return bufferSec;
            }.try_into()
                .unwrap();
            self.start_audio += samples;
        }
    }
    pub fn start(&mut self) {
        if let Some(ref mut gen) = self.generator {
            let sample_rate: f64 = js!{
                return @{&self.ctx}.sampleRate;
            }.try_into()
                .unwrap();
            App::print(format!(
                "sound device : Web audio context. sample_rate: {}\n",
                sample_rate
            ));
            gen.init(sample_rate as f32);
        }
    }
}

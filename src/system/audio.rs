use std::{collections::VecDeque, time::Duration};

use rodio::{source::SineWave, OutputStream, OutputStreamHandle, Sink, Source};

pub const NES_AUDIO_FREQUENCY: u32 = 44100; // 44.1 KiHz

pub struct NesAudioHandler {
    output_stream: OutputStream,
    output_handle: OutputStreamHandle,
    /// Sink is like the sound output
    sink: Sink,

    audio_stream: NesAudioStream,
}

impl NesAudioHandler {
    pub fn new() -> Self {
        let (output_stream, output_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&output_handle).unwrap();



        Self {
            output_stream: output_stream,
            output_handle: output_handle,
            sink: sink,
            audio_stream: NesAudioStream::default(),
        }
    }

    pub fn play(&mut self) {
        let square_wave = SineWave::new(440.0)
            .take_duration(Duration::from_secs_f32(10.0))
            .map(|s| s.signum())
            .collect::<Vec<_>>();
        
        self.audio_stream.batch_queue_samples(square_wave);

        let source = self.audio_stream.drain_as_clone();

        self.sink.append(source);

        self.sink.sleep_until_end();
    }

    pub fn set_sound_samples(&mut self, samples: Vec<f32>) {
        self.audio_stream.batch_queue_samples(samples);
    }
}

#[derive(Debug, Default, Clone)]
pub struct NesAudioStream {
    sample_queue: VecDeque<f32>,
}

impl Iterator for NesAudioStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.sample_queue.pop_front()
    }
}

impl Source for NesAudioStream {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { NES_AUDIO_FREQUENCY }
    fn total_duration(&self) -> Option<Duration> { None }
}

impl NesAudioStream {
    pub fn queue_sample(&mut self, sample: f32) {
        self.sample_queue.push_back(sample);
    }

    pub fn batch_queue_samples(&mut self, samples: Vec<f32>) {
        self.sample_queue.extend(samples.iter());
    }

    pub fn drain_as_clone(&mut self) -> Self {
        Self {
            sample_queue: self.sample_queue.drain(..).collect()
        }
    }

    pub fn lowpass_filter(&mut self, cutoff_freq: f32) {
        // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
        let rc = 1.0 / (cutoff_freq * 2.0 * core::f32::consts::PI);
        // time per sample
        let dt = 1.0 / NES_AUDIO_FREQUENCY as f32;
        let alpha = dt / (rc + dt);

        let data = &mut self.sample_queue;

        data[0] *= alpha;
        for i in 1..data.len() {
            // https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter

            // we don't need a copy of the original data, because the original data is accessed
            // before it is overwritten: data[i] = ... data[i]
            data[i] = data[i - 1] + alpha * (data[i] - data[i - 1]);
        }
    }
}
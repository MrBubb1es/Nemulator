// use std::{collections::VecDeque, sync::Arc, time::Duration};

// use rodio::{source::SineWave, queue::SourcesQueueOutput, OutputStream, OutputStreamHandle, Sink, Source};
// use tokio::sync::mpsc::Receiver;

// use super::apu_util::{NesAudioStream, NES_AUDIO_FREQUENCY};

// pub struct NesAudioHandler {
//     output_stream: OutputStream,
//     output_handle: OutputStreamHandle,
//     /// Sink is like the sound output
//     sink: Sink,

//     nes_audio_receiver: Arc<SourcesQueueOutput<f32>>,
// }

// impl NesAudioHandler {
//     pub fn new(sound_input_channel: Receiver<Vec<f32>>) -> Self {
        // let (output_stream, output_handle) = OutputStream::try_default().unwrap();
        // let sink = Sink::try_new(&output_handle).unwrap();

//         Self {
//             output_stream: output_stream,
//             output_handle: output_handle,
//             sink: sink,
//             audio_stream: NesAudioStream::default(),
//         }
//     }

//     pub fn play(&mut self) {
//         let period: f32 = 1.0 / NES_AUDIO_FREQUENCY as f32;
//         let square_wave = SineWave::new(440.0)
//             .take_duration(Duration::from_secs_f32(period))
//             .map(|s| s.signum())
//             .collect::<Vec<_>>();

//         self.audio_stream.batch_queue_samples(square_wave);

//         let source = self.audio_stream.drain_as_clone();

//         self.sink.append(source);

//         self.sink.sleep_until_end();
//     }

//     pub fn set_sound_samples(&mut self, samples: Vec<f32>) {
//         self.audio_stream.batch_queue_samples(samples);
//     }
// }

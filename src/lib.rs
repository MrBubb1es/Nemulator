pub mod app;
pub mod cartridge;
pub mod system;


use rodio::{OutputStream, Sink};
use system::apu_util::NesAudioStream;
// use system::audio::NesAudioHandler;
use winit::event_loop::{ControlFlow, EventLoop};


#[derive(Default)]
pub struct RuntimeConfig {
    pub cart_path: String,
    pub limit_fps: bool,
    pub can_debug: bool,
}

pub fn run(config: RuntimeConfig) {
    env_logger::init();

    let (_output_stream, output_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&output_handle).unwrap();
    let (sound_stream, sample_queue) = NesAudioStream::new();

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::new();

    event_loop.set_control_flow(ControlFlow::Wait);

    nes_app.init(config, sample_queue);
    
    // Start the sound system
    sink.append(sound_stream);
    sink.play();

    nes_app.attatch_sound_sink(sink);

    // Run the application
    event_loop.run_app(&mut nes_app).unwrap();
}
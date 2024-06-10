pub mod app;
pub mod cartridge;
pub mod system;

use std::sync::Arc;

use system::audio::{self, NesAudioHandler, NES_AUDIO_FREQUENCY};
use winit::event_loop::{ControlFlow, EventLoop};

use tokio;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::channel;

const AUDIO_SLEEP_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / NES_AUDIO_FREQUENCY as u64);

// use app::window::Window;

pub async fn run(path: &str) {
    let path_thread_safe_thing = Arc::new(path.to_string());

    env_logger::init();

    let (audio_over_sender, audio_over_receiver) = channel::<bool>(1);  

    let audio_thread = tokio::spawn(async move {
        let mut audio_handler = NesAudioHandler::new();

        while audio_over_receiver.is_empty() {
            audio_handler.play();

            // sleep(AUDIO_SLEEP_INTERVAL).await;
        }
    });

    let event_loop = EventLoop::new().unwrap();
    let mut nes_app = app::NesApp::default();

    event_loop.set_control_flow(ControlFlow::Wait);

    let my_path = path_thread_safe_thing.as_ref();

    nes_app.init(my_path);
    
    event_loop.run_app(&mut nes_app).unwrap();

    audio_over_sender.send(true).await;
    audio_thread.await;
}

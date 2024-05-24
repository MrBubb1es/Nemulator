use std::sync::Arc;

use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Extra, Section as TextSection, Text}, BrushBuilder, TextBrush};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::system::nes::NES;

pub struct State {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    brush: TextBrush<FontRef<'static>>,
    section: wgpu_text::glyph_brush::Section<'static>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
               required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let font = include_bytes!("fonts/FiraCode-VariableFont_wght.ttf");

        let brush = BrushBuilder::using_font_bytes(font).unwrap()
        /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(&device, config.width, config.height, config.format);

        // Directly implemented from glyph_brush.
        let section = TextSection::default().add_text(Text::new("Hello World"));

        Self {
            instance,
            surface,
            device,
            queue,
            config,
            size,
            brush,
            section,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.brush.resize_view(self.config.width as f32, self.config.height as f32, &self.queue);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        //todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        println!("Here0");
        let output = self.surface.get_current_texture()?;
        println!("Here1");
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        println!("Here2");
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("NES Render Encoder"),
        });
        println!("Here3");

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NES Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        println!("Here4");

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        println!("Here5");

        output.present();

        println!("Here6");

        Ok(())
    }
    
        // Crashes if inner cache exceeds limits.
    //     self.brush.queue(&self.device, &self.queue, vec![&self.section]).unwrap();

    //     {
    //         self.brush.draw(&mut rpass);
    //     }

    //     queue.submit([encoder.finish()]);
    //     frame.present();
    // }
}

#[derive(Default)]
pub struct NesApp {
    window: Option<Arc<Window>>,
    state: Option<State>,

    nes: NES,
}

impl<'a> ApplicationHandler for NesApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("NEmulator");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(Arc::clone(&window)));

        window.request_redraw();

        println!("Anything abt window: {}", window.has_focus());
        println!("Anything abt state: {}", state.config.height);

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if state.input(&event) {
                return; // State handled event
            }

            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                },
                WindowEvent::RedrawRequested => {
                    // Draw.
                    state.update();
                    println!("Updated Ok");
                    match state.render() {
                        Ok(_) => { println!("Rendered Ok"); }
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw in
                    // applications which do not always need to. Applications that redraw continuously
                    // can render here instead.
                    self.window.as_ref().unwrap().request_redraw();
                }
                _ => (),
            }
        }
    }
}
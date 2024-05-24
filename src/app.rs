use std::sync::Arc;

use wgpu_text::{glyph_brush::{ab_glyph::FontRef, OwnedSection, OwnedText}, BrushBuilder, TextBrush};

use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::ElementState, keyboard::{Key, ModifiersState}, platform::modifier_supplement::KeyEventExtModifierSupplement};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::system::nes::NES;

// #[derive(Debug)]
pub struct State {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    brush: TextBrush<FontRef<'static>>,
    font_size: f32,
    zpage_sec: OwnedSection,
    cpu_sec: OwnedSection,
    instr_sec: OwnedSection,
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

        surface.configure(&device, &config); // Fuck you, God; You asshole.

        let font = include_bytes!("fonts/FiraCode-VariableFont_wght.ttf");

        let brush = BrushBuilder::using_font_bytes(font).unwrap()
        /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(&device, config.width, config.height, config.format);

        // Directly implemented from glyph_brush.
        let zpage_sec = OwnedSection::default()
            .with_screen_position((10.0, 10.0));
        let cpu_sec = OwnedSection::default()
            .with_screen_position((10.0, config.height as f32 * 0.65));
        let instr_sec = OwnedSection::default()
            .with_screen_position((config.width as f32 * 0.3, config.height as f32 * 0.65));

        Self {
            _instance: instance,
            surface,
            device,
            queue,
            config,
            size,

            font_size: 40.0,
            brush,
            zpage_sec,
            cpu_sec,
            instr_sec,
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

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self, nes: &NES) {
        let zpage_str = nes.zpage_str();
        self.zpage_sec.text.clear();
        self.zpage_sec.text.push(OwnedText::new(zpage_str)
            .with_scale(self.font_size));

        let mut cpu_str = String::with_capacity(200);
        cpu_str.push_str(&format!("     X: {:#06X}\n", nes.get_cpu().unwrap().get_x_reg()));
        cpu_str.push_str(&format!("     Y: {:#06X}\n", nes.get_cpu().unwrap().get_y_reg()));
        cpu_str.push_str(&format!("   ACC: {:#06X}\n", nes.get_cpu().unwrap().get_acc()));
        cpu_str.push_str(&format!("    SP: {:#06X}\n", nes.get_cpu().unwrap().get_sp()));
        cpu_str.push_str(&format!("    PC: {:#06X}\n", nes.get_cpu().unwrap().get_pc()));

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.cpu_sec.text.clear();
        self.cpu_sec.text.push(OwnedText::new("CPU Info:\n")
            .with_scale(self.font_size+5.0));
        self.cpu_sec.text.push(OwnedText::new(cpu_str)
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new(" Flags: ")
            .with_scale(self.font_size+5.0));
        
        self.cpu_sec.text.push(OwnedText::new("N")
            .with_color(if nes.get_cpu().unwrap().get_negative_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("V")
            .with_color(if nes.get_cpu().unwrap().get_overflow_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("U")
            .with_color(if nes.get_cpu().unwrap().get_unused_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("B")
            .with_color(if nes.get_cpu().unwrap().get_b_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("D")
            .with_color(if nes.get_cpu().unwrap().get_decimal_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("I")
            .with_color(if nes.get_cpu().unwrap().get_interrupt_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("Z")
            .with_color(if nes.get_cpu().unwrap().get_zero_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));
        self.cpu_sec.text.push(OwnedText::new("C")
            .with_color(if nes.get_cpu().unwrap().get_carry_flag() == 1 { GREEN } else { RED })
            .with_scale(self.font_size));


        let instr_str = nes.get_cpu().unwrap().current_instr_str();
        let clks_str = format!("\n\nClocks: {}", nes.get_clks());

        self.instr_sec.text.clear();
        self.instr_sec.text.push(OwnedText::new("Last Instr:\n")
            .with_scale(self.font_size+5.0));   
        self.instr_sec.text.push(OwnedText::new(instr_str)
            .with_scale(self.font_size));
        self. instr_sec.text.push(OwnedText::new(clks_str)
            .with_scale(self.font_size));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("NES Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            self.brush.draw(&mut render_pass);
        }

        // Crashes if inner cache exceeds limits.
        self.brush.queue(&self.device, &self.queue, vec![&self.zpage_sec, &self.cpu_sec, &self.instr_sec]).unwrap();

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
pub struct NesApp {
    window: Option<Arc<Window>>,
    state: Option<State>,
    modifiers: Option<ModifiersState>,

    nes: NES,
    paused: bool,
}

impl<'a> ApplicationHandler for NesApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("NEmulator")
            .with_inner_size(PhysicalSize::new(1920, 1080));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(Arc::clone(&window)));

        window.request_redraw();

        self.window = Some(window);
        self.state = Some(state);

        self.paused = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if state.input(&event) {
                return; // State handled event
            }

            match event {
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                },
                WindowEvent::ModifiersChanged(new) => {
                    self.modifiers = Some(new.state());
                },
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            Key::Named(winit::keyboard::NamedKey::Space) => { 
                                if self.modifiers.unwrap().shift_key() && self.paused {
                                    self.nes.cycle();
                                } else {
                                    self.paused = !self.paused; 
                                }
                            },
                            _ => (),
                        }
                    }
                },
                WindowEvent::RedrawRequested => {
                    // Draw.
                    state.update(&self.nes);
                    match state.render() {
                        Ok(_) => {}
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

        if !self.paused {
            self.nes.cycle();
        }
    }
}

impl NesApp {
    pub fn load_cart(&mut self, cart_path_str: &str) {
        self.nes.load_cart(cart_path_str);
    }

    pub fn nestest_init(&mut self) {
        self.nes.load_cart("prg_tests/nestest.nes");
        self.nes.set_cpu_state(Some(0xC000), None, None, None, None, None, None);
    }
}

use rusttype::{Font, Scale};
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::Window as SDLWindow;
use sdl2::event::EventPollIterator;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::{EventPump, Sdl};

use crate::system::bus::Bus;
use crate::system::cpu::CPU;

use super::util;

/// Window for the emulator application
pub struct Window<'a> {
    is_debug: bool,

    _context: Sdl,
    text_font: Font<'a>,
    canvas: Canvas<SDLWindow>,
    event_pump: EventPump,
}

impl<'a> Window<'a> {
    /// Return SDL context or panic if failure
    fn new_sdl_context() -> Sdl {
        let context_result = util::new_sdl_context();
        if let Err(e) = context_result {
            panic!("Error occured when getting SDL context: {e}");
        }
        
        context_result.unwrap()
    }

    /// Return new SDL canvas with given title or panic if failure to create window/canvas
    fn new_canvas(sdl_context: &Sdl, title: &str) -> Canvas<SDLWindow> {
        let canvas_result = util::new_canvas_with_context(&sdl_context, title, None, None);
        if let Err(e) = canvas_result {
            panic!("Error occured when creating canvas: {e}");
        }
        
        canvas_result.unwrap()
    }

    /// Create a new event pump from given context or panic if failure
    fn new_event_pump(sdl_context: &Sdl) -> EventPump {
        let event_pump_result = sdl_context.event_pump();
        if let Err(e) = event_pump_result {
            panic!("Error when creating event pump for SDL context: {e}");
        }
        
        event_pump_result.unwrap()
    }

    /// Create a new window. Uses the debug view if debug is true
    pub fn new(debug: bool) -> Self {
        let title = if debug {
            "NES Debug View"
        } else {
            "Nemulator"
        };

        let sdl_context = Window::new_sdl_context();
        let font = util::load_font();
        let canvas = Window::new_canvas(&sdl_context, title);
        let event_pump = Window::new_event_pump(&sdl_context);

        Window{
            is_debug: debug,
            _context: sdl_context,
            text_font: font,
            canvas: canvas,
            event_pump: event_pump,
        }
    }

    /// Draw the window
    pub fn draw(&mut self, cpu: &CPU, bus: &Bus) {
        if self.is_debug {
            self.show_debug(cpu, bus);
        } else {
            self.show();
        }
    }

    /// Draw the NES screen to the window
    fn show(&mut self) {
        self.canvas.set_draw_color(Color::CYAN);
        self.canvas.clear();
        self.canvas.present();
    }

    /// Create a zpage string for the debug view given the bus
    fn zpage_str(bus: &Bus) -> String {
        let mut mem_str: String = String::from("");

        for i in 0..16 {
            let prefix = format!("{:#06X}:", i*16);
            mem_str.push_str(&prefix);
            for j in 0..16 {
                let mem_val = bus.read(i * 16 + j);
                let val_str = format!("  {mem_val:02X}");
                mem_str.push_str(&val_str);
            }
            let suffix = "\n";
            mem_str.push_str(&suffix);
        }

        mem_str
    }

    /// Draw the debug view (very slow bc of calls to the write function)
    fn show_debug(&mut self, cpu: &CPU, bus: &Bus) {
        const DEBUG_BG_COLOR: Color = Color::RGB(0x09, 0x31, 0x45);
        const DEBUG_TXT_COLOR: Color = Color::RGB(0xEF, 0xD4, 0x69);
        const DEBUG_TXT_SIZE: f32 = 20.0;
        
        self.canvas.set_draw_color(DEBUG_BG_COLOR);
        self.canvas.clear();

        let zero_page_str = Window::zpage_str(&bus);

        self.write("Zero-Page:", 5, 5, DEBUG_TXT_SIZE+5.0, DEBUG_TXT_COLOR);
        self.write(&zero_page_str, 5, (DEBUG_TXT_SIZE as usize)+10, DEBUG_TXT_SIZE, DEBUG_TXT_COLOR);

        let low_height = 17*(DEBUG_TXT_SIZE as usize)+10;

        self.write_cpu(cpu, 5, low_height, DEBUG_TXT_SIZE, DEBUG_TXT_COLOR);

        self.write("Just Executed:", 200, low_height, DEBUG_TXT_SIZE+5.0, DEBUG_TXT_COLOR);
        self.write(&cpu.current_instr_str(), 210, low_height+(DEBUG_TXT_SIZE as usize)+10, DEBUG_TXT_SIZE, DEBUG_TXT_COLOR);

        self.canvas.present();
    }

    pub fn event_iter(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }

    /// Takes in text, an x and y pair to draw to on the window, and the height 
    /// in pixels to draw the text, and renders it to this window's SDL canvas.
    /// This function is very slow bc it loops over every pixel in the text box,
    /// and the text boxes get pretty large. Debug view runs at < 60 fps.
    pub fn write(&mut self, text: &str, x: usize, y: usize, height: f32, color: Color) {
        // 2x scale in x direction to counter the aspect ratio of monospace characters.
        let scale = Scale {
            x: height,
            y: height,
        };

        let pixel_height = height.ceil() as usize;

        let bg_col = self.canvas.draw_color();
        let r = color.r as f32;
        let g = color.g as f32;
        let b = color.b as f32;

        for (i, line_str) in text.split("\n").enumerate() {
            if line_str.len() == 0 {
                continue;
            }

            let v_metrics = self.text_font.v_metrics(scale);
            let offset = rusttype::point(0.0, v_metrics.ascent);

            let glyphs: Vec<_> = self.text_font.layout(line_str, scale, offset).collect();

            // Find the most visually pleasing width to display
            let width = glyphs
                .iter()
                .rev()
                .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                .next()
                .unwrap_or(0.0)
                .ceil() as usize;
        
            // let text_surface = Surface::new(width as u32, pixel_height, sdl2::pixels::PixelFormatEnum::RGB24).unwrap();
            // let text_s = Surface::from_data(data, width, height, pitch, format)
            let mut pixel_data = Vec::with_capacity(3 * width * pixel_height as usize);

            for _ in 0..width * pixel_height as usize {
                pixel_data.push(bg_col.r);
                pixel_data.push(bg_col.g);
                pixel_data.push(bg_col.b);
            }

            for glyph in glyphs {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        let col_r = r + (bg_col.r as f32 - r) * (1.0 - v);
                        let col_g = g + (bg_col.g as f32 - g) * (1.0 - v);
                        let col_b = b + (bg_col.b as f32 - b) * (1.0 - v);

                        let pixel_x = x as i32 + bb.min.x;
                        let pixel_y = y as i32 + bb.min.y;

                        let index = 3 * (pixel_y as usize * width + pixel_x as usize);
                        pixel_data[index] = col_r as u8;
                        pixel_data[index + 1] = col_g as u8;
                        pixel_data[index + 2] = col_b as u8;
                    });
                }
            }

            let text_surface = Surface::from_data(&mut pixel_data, 
                width as u32, 
                pixel_height as u32, 
                3 * width as u32, 
                sdl2::pixels::PixelFormatEnum::RGB24)
                .unwrap();

            let texture_creator = &self.canvas.texture_creator();
            let text_texture = text_surface.as_texture(texture_creator).unwrap();

            // 0.85 seems to be a good spacing for in between firacode lines of text.
            let row_height = 0.85 * pixel_height as f32;
            let y_offset = i as f32 * row_height;

            self.canvas.copy(
                &text_texture, 
                None, 
                Rect::new(
                    x as i32, 
                    y as i32 + y_offset as i32, 
                    width as u32, 
                    pixel_height as u32)
                ).unwrap();
        }
    }

    /// Write the CPU to the window for debug view. Displays registers and status flags
    fn write_cpu(&mut self, cpu: &CPU, x: usize, y: usize, height: f32, color: Color) {
        let x_str = format!("     X: {:#06X}", cpu.get_x_reg());
        let y_str = format!("     Y: {:#06X}", cpu.get_y_reg());
        let acc_str = format!("   ACC: {:#06X}", cpu.get_acc());
        let sp_str = format!("    SP: {:#06X}", cpu.get_sp());
        let pc_str = format!("    PC: {:#06X}", cpu.get_pc());

        let x_ypos = y + 10 + height as usize;
        let y_ypos = x_ypos + height as usize;
        let acc_ypos = y_ypos + height as usize;
        let sp_ypos = acc_ypos + height as usize;
        let pc_ypos = sp_ypos + height as usize;
        let flags_ypos = pc_ypos + height as usize;
        
        self.write("CPU Info:", x, y, height+5.0, color);
        self.write(&x_str, x, x_ypos, height, color);
        self.write(&y_str, x, y_ypos, height, color);
        self.write(&acc_str, x, acc_ypos, height, color);
        self.write(&sp_str, x, sp_ypos, height, color);
        self.write(&pc_str, x, pc_ypos, height, color);
        self.write(" Flags: ", x, flags_ypos, height, color);
        
        let char_width = (height as usize) / 2;
        
        self.write("N", x+char_width*7, flags_ypos, height, if cpu.get_negative_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("V", x+char_width*8, flags_ypos, height, if cpu.get_overflow_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("-", x+char_width*9, flags_ypos, height, color);
        self.write("B", x+char_width*10, flags_ypos, height, if cpu.get_b_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("D", x+char_width*11, flags_ypos, height, if cpu.get_decimal_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("I", x+char_width*12, flags_ypos, height, if cpu.get_interrupt_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("Z", x+char_width*13, flags_ypos, height, if cpu.get_zero_flag() == 1 { Color::GREEN } else { Color::RED });
        self.write("C", x+char_width*14, flags_ypos, height, if cpu.get_carry_flag() == 1 { Color::GREEN } else { Color::RED });
    }
}
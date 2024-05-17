use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

use rusttype::Font;

const SCREEN_WIDTH: u32  = 960;
const SCREEN_HEIGHT: u32 = 540;

// Create a new SDL context, or error if failure
pub fn new_sdl_context() -> Result<Sdl, String> {
    let sdl_context = sdl2::init()?;
    Ok(sdl_context)
}

// Create a new TTF context, or error if failure
// pub fn new_ttf_context() -> Result<Sdl2TtfContext, String> {
//     let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
//     Ok(ttf_context)
// }

// Initializes an SDL2 window and returns it in a result. Errors if there was
// a problem building the window.
pub fn new_window_with_context(sdl_context: &Sdl, title: &str, width: Option<u32>, height: Option<u32>) -> Result<Window, String> {
    let video_subsys = sdl_context.video()?;

    let w = width.unwrap_or(SCREEN_WIDTH);
    let h = height.unwrap_or(SCREEN_HEIGHT);

    let window = video_subsys
        .window(title, w, h)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(window)
}

// Initializes an SDL2 window wrapped in a canvas for easy drawing and returns 
// it in a result. Errors if there was a problem building the window or 
// canvas from window.
pub fn new_canvas_with_context(sdl_context: &Sdl, title: &str, width: Option<u32>, height: Option<u32>) -> Result<Canvas<Window>, String> {
    let window = new_window_with_context(sdl_context, title, width, height)?;
    let canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(canvas)
}

// Loads a font from path and returns a texture where each character in the font
// has been rendered. This texture will then be accessed to render text on the
// screen. Basically it just does all of the rendering up front and uses the
// rendered texture in the future.
pub fn load_font() -> Font<'static> {
    // if let Some(font_path) = std::env::args().nth(1) {
    //     let font_path = std::env::current_dir().unwrap().join(font_path);
    //     let data = std::fs::read(&font_path).unwrap();
    //     Font::try_from_vec(data).unwrap_or_else(|| {
    //         panic!("error constructing a Font from data at {:?}", font_path);
    //     })
    // } else {
    //     eprintln!("No font specified ... using FiraCode-VariableFont_wght.ttf");
    //     let font_data = include_bytes!("fonts/FiraCode-VariableFont_wght.ttf");
    //     Font::try_from_bytes(font_data as &[u8]).expect("error constructing a Font from bytes")
    // }

    eprintln!("No font specified ... using FiraCode-VariableFont_wght.ttf");
        let font_data = include_bytes!("fonts/FiraCode-VariableFont_wght.ttf");
        Font::try_from_bytes(font_data as &[u8]).expect("error constructing a Font from bytes")
}

// Chooses either black or white text based on given background color. Uses a
// formula ripped from stack overflow: https://stackoverflow.com/questions/1855884/determine-font-color-based-on-background-color
pub fn readable_text_color_from_bg_color(bg_color: Color) -> Color {
    let luminance = 
            (0.299 * bg_color.r as f64
            + 0.587 * bg_color.g as f64
            + 0.114 * bg_color.b as f64) / 255.0;

    let text_color = if luminance > 0.5 {
        Color::BLACK
    } else {
        Color::WHITE
    };

    text_color
}

// pub fn write(canvas: &mut Canvas<Window>, 
//              font: Font, 
//              text: &str, 
//              x: usize, y: usize, 
//              width: usize, height: usize,
//              color: Option<Color>) -> Result<(), String> {

//     let text_color = color.unwrap_or(
//         readable_text_color_from_bg_color(
//             canvas.draw_color()
//         )
//     );

//     let texture_creator = canvas.texture_creator();

//     let surface = font
//         .render(text)
//         .blended(text_color)
//         .map_err(|e| e.to_string())?;

//     println!("Here 0");
//     println!("w: {}, h: {}", surface.width(), surface.height());

//     let mut dest = Surface::new(width as u32, height as u32, 
//         sdl2::pixels::PixelFormatEnum::RGB24).map_err(|e| e.to_string())?;
//     let rect = Rect::new(0,0,width as u32, height as u32);

//     let scaled_surface = surface.as_ref()
//         .blit_scaled(surface.rect(), 
//         &mut dest, rect).map_err(|e| e.to_string())?; //SurfaceRef::blit_scaled(&self, src_rect, dst, dst_rect)

//     let texture = texture_creator
//         .create_texture_from_surface(&scaled_surface)
//         .map_err(|e| e.to_string())?;

//     println!("Here 1");

//     let target = Rect::new(x as i32, y as i32, width as u32, height as u32);

//     canvas.copy(&texture, None, Some(target))?;

//     Ok(())
// }
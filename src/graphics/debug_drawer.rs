use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;

pub fn draw(canvas: &mut Canvas<Window>, clocks: usize) {
    let col = match (clocks / 100) % 2 {
        0 => Color::CYAN,
        _ => Color::MAGENTA,
    };

    canvas.set_draw_color(col);

    canvas.clear();
    canvas.present();
}

pub fn draw_mem(canvas: &mut Canvas<Window>, clocks: usize) {

}
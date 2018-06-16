use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::gfx::primitives::ToColor;

pub fn measure_text(text: &str) -> (i16, i16) {
    let w = text.len() as i16 * 8 + 0;
    let h = 8;
    (w, h)
}

pub fn draw_text<C: ToColor + Copy>(canvas: &mut WindowCanvas, x: i16, y: i16, label: &str, color: C) -> Result<(), String> {
    let mut x = x + 1;
    let y = y + 1;
    for c in label.chars() {
        canvas.character(x, y, c, color)?;
        x += 8;
    }
    Ok(())
}


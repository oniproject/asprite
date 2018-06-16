use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::gfx::primitives::DrawRenderer;

use specs::prelude::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Line {
    pub start: (i16, i16),
    pub end: (i16, i16),
    pub color: Color,
}

impl Line {
    pub fn paint(&self, canvas: &mut ::render::Canvas) {
        let _ = canvas.canvas.borrow_mut().box_(
            self.start.0, self.start.1,
            self.end.0, self.end.1,
            self.color);
    }
}

pub struct Liner {
    start: Option<(i16, i16)>,
    end: Option<(i16, i16)>,
}

impl Liner {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }
    pub fn down(&mut self, x: i16, y: i16) {
        self.start = Some((x as i16, y as i16));
        self.end = None;
    }
    pub fn up(&mut self, _x: i16, _y: i16) {
        self.start = None;
        self.end = None;
    }
    pub fn mov(&mut self, x: i16, y: i16) {
        if self.end.is_some() {
            self.start = self.end;
        }
        if self.start.is_some() {
            self.end = Some((x, y));
        }
    }
}

impl Iterator for Liner {
    type Item = ((i16, i16), (i16, i16));

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            self.start = None;
            Some((start, end))
        } else {
            None
        }
    }
}

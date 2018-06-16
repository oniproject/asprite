use math::*;

pub trait MouseEvent {
    fn cursor(&self) -> Point2<f32>;
    fn was_pressed(&self) -> bool;
    fn was_released(&self) -> bool;
    fn is_cursor_in_rect(&self, r: &Rect<f32>) -> bool {
        r.contains(self.cursor())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mouse {
    pub cursor: Point2<f32>,
    pub pressed: [bool; 3],
    pub released: [bool; 3],
}

impl MouseEvent for Mouse {
    #[inline(always)]
    fn cursor(&self) -> Point2<f32> { self.cursor }
    #[inline(always)]
    fn was_pressed(&self) -> bool { self.pressed[0] }
    #[inline(always)]
    fn was_released(&self) -> bool { self.released[0] }
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            cursor: Point2::new(-14.0, -88.0),
            pressed: [false; 3],
            released: [false; 3],
        }
    }

    pub fn cleanup(&mut self) {
        self.pressed = [false; 3];
        self.released = [false; 3];
    }
}

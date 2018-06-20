use super::*;

use draw::CanvasFill;

pub struct Bucket<N: BaseNum, C: Copy> {
    pub start: Point2<N>,
    pub color: C,
}

impl Bucket<i32, u8> {
    pub fn new() -> Self {
        Self {
            start: Point2::new(0, 0),
            color: 0,
        }
    }
}

impl<N: BaseIntExt, C: Copy + Clone + Eq> Tool<N, C> for Bucket<N, C> {
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        let color = ctx.start();
        ctx.scanline_fill(p, color);
        ctx.commit();
    }
}

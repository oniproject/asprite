use super::*;

use draw::ScanlineFill;

pub struct Bucket;

impl Bucket {
    pub fn new() -> Self {
        Bucket
    }
}

impl<N: BaseIntExt, C: Copy + Eq> Tool<N, C> for Bucket {
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        ctx.start();
        let color = ctx.color();
        ctx.scanline_fill(p, color);
        ctx.commit();
    }
}

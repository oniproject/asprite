use super::*;

pub struct EyeDropper;

impl EyeDropper {
    pub fn new() -> Self {
        EyeDropper
    }
}

impl<N: BaseIntExt, C: Copy + Eq> Tool<N, C> for EyeDropper {
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        if let Some(color) = ctx.at(p.x, p.y) {
            ctx.change_color(color);
        }
    }
}

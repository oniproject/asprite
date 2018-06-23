use super::*;

use draw::draw_ellipse;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveMode {
    Rect,
    Ellipse,
}

pub struct Primitive<N: BaseNum> {
    pub start: Point2<N>,
    pub last: Point2<N>,
    pub active: bool,
    pub square: bool,
    pub mode: PrimitiveMode,
    pub fill: bool,
}

impl<N: BaseNum> Primitive<N> {
    pub fn new() -> Self {
        Self {
            start: Point2::new(N::zero(), N::zero()),
            last: Point2::new(N::zero(), N::zero()),
            active: false,
            square: false,
            mode: PrimitiveMode::Rect,
            fill: true,
        }
    }
}

impl<N: BaseIntExt, C: Copy + Eq> Tool<N, C> for Primitive<N> {

    fn special<Ctx: Context<N, C>>(&mut self, on: bool, _ctx: &mut Ctx) {
        self.square = on;
    }
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        ctx.start();
        self.active = true;
        self.start = p;
        self.last = p;
    }
    fn release<Ctx: Context<N, C>>(&mut self, _p: Point2<N>, ctx: &mut Ctx) {
        self.active = false;
        ctx.commit();
    }
    fn cancel<Ctx: Context<N, C>>(&mut self, ctx: &mut Ctx) {
        self.active = false;
        ctx.rollback();
    }
    fn movement<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        if self.active {
            ctx.sync();

            let min = if self.square {
                let delta = p - Vector2::new(self.start.x, self.start.y);
                let min = delta.x.abs().min(delta.y.abs());
                let signum = Vector2::new(delta.x.signum(), delta.y.signum());
                self.start - signum * min
            } else {
                p
            };

            let mut r = Rect::from_min_max(min, self.start).normalize();

            let color = ctx.color();
            match (self.fill, self.mode) {
                (true,  PrimitiveMode::Rect) => {
                    for y in r.min.y..=r.max.y {
                        for x in r.min.x..=r.max.x {
                            ctx.set(x, y, color);
                        }
                    }
                }
                (false, PrimitiveMode::Rect) => {
                    for x in r.min.x..=r.max.x {
                        ctx.set(x, r.min.y, color);
                        ctx.set(x, r.max.y, color);
                    }
                    for y in r.min.y+N::one()..r.max.y {
                        ctx.set(r.min.x, y, color);
                        ctx.set(r.max.x, y, color);
                    }
                }
                (false, PrimitiveMode::Ellipse) => {
                    draw_ellipse(r, |a, b| {
                        ctx.set(a.x, a.y, color);
                        ctx.set(b.x, b.y, color);
                    });
                }
                (true,  PrimitiveMode::Ellipse) => {
                    draw_ellipse(r, |a, b| {
                        for x in a.x..=b.x {
                            ctx.set(x, a.y, color)
                        }
                    });
                }
            }
        }
        self.last = p;
    }
}

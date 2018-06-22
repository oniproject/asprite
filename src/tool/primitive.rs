use super::*;

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

impl<N: BaseIntExt, C: Copy + Clone + Eq> Tool<N, C> for Primitive<N> {

    fn special<Ctx: Context<N, C>>(&mut self, on: bool, ctx: &mut Ctx) {
        self.square = on;
    }
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        ctx.start();
        self.active = true;
        self.start = p;
        self.last = p;
    }
    fn release<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
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
            r.max += Vector2::new(N::one(), N::one());

            let color = ctx.color();
            match (self.fill, self.mode) {
                (true,  PrimitiveMode::Rect) => ctx.rect_fill(r, color),
                (false, PrimitiveMode::Rect) => ctx.rect(r, color),
                (false, PrimitiveMode::Ellipse) => ctx.ellipse(r, color),
                (true,  PrimitiveMode::Ellipse) => ctx.ellipse_fill(r, color),
            }

            ctx.update(r.union_point(self.last));
        }
        self.last = p;
    }
}

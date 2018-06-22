use super::*;
use draw::*;

pub struct Freehand<N: BaseNum> {
    pub perfect: bool,
    pub line: bool,

    pub last: Point2<N>,
    pub pts: Vec<(Point2<N>, bool)>,
    pub active: bool,
}

impl<N: BaseNum> Freehand<N> {
    pub fn new() -> Self {
        Self {
            last: Point2::new(N::zero(), N::zero()),
            pts: Vec::new(),
            active: false,
            perfect: true,
            line: false,
        }
    }
}

impl<N: BaseIntExt, C: Copy + Clone + Eq> Tool<N, C> for Freehand<N> {
    fn cancel<Ctx: Context<N, C>>(&mut self, ctx: &mut Ctx) {
        self.active = false;
        self.pts.clear();
        ctx.rollback();
    }
    fn movement<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        let color = ctx.color();
        if self.line {
            ctx.sync();
            draw_line(p, self.last, |p| ctx.paint_brush(p, color));
        } else if self.active {
            if self.perfect {
                let last = self.last;
                self.update(p, last, ctx);
            } else {
                draw_line(p, self.last, |p| ctx.paint_brush(p, color));
            }
            self.last = p;
        }
    }
    fn press<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        if self.line {
            ctx.commit();
        } else {
            ctx.start();
            self.active = true;
            self.pts.push((p, true));
            self.last = p;
            let color = ctx.color();
            ctx.paint_brush(p, color);
        }
    }
    fn release<Ctx: Context<N, C>>(&mut self, p: Point2<N>, ctx: &mut Ctx) {
        let color = ctx.color();
        if self.active {
            while self.pts.len() > 0 {
                self.flatten_first_point(ctx, color);
            }
            ctx.commit();
        }
        self.active = false;
        self.last = p;
    }

    fn special<Ctx: Context<N, C>>(&mut self, on: bool, ctx: &mut Ctx) {
        self.line = on; && !self.active;
        println!("special: {} line: {}", on, self.line);
        if !self.active {
            ctx.start();
        }
    }

    fn preview<Ctx: PreviewContext<N, C>>(&self, mouse: Point2<N>, ctx: &mut Ctx) {
        let color = ctx.color();
        if self.active {
            self.pts.iter()
                .filter_map(|(p, active)| if *active { Some(p) } else { None })
                .for_each(|p| ctx.paint_brush(*p, color));
        }
        ctx.paint_brush(mouse, color);
    }
}

impl<N: BaseIntExt> Freehand<N> {
    pub fn update<C, Ctx>(&mut self, m: Point2<N>, last: Point2<N>, ctx: &mut Ctx)
        where Ctx: Context<N, C>, C: Copy + Clone + Eq
    {
        if self.point_exists(m.x, m.y) {
            return;
        }

        draw_line(last, m, |p: Point2<N>| {
            if !self.point_exists(p.x, p.y) {
                self.pts.push((p, true));
            }
        });

        let color = ctx.color();
        self.cleanup_points();
        while self.pts.len() > 4 {
            self.flatten_first_point(ctx, color);
        }
    }

    fn flatten_first_point<C, Ctx>(&mut self, ctx: &mut Ctx, color: C)
        where Ctx: Context<N, C>, C: Copy + Clone + Eq
    {
        let p = self.pts.remove(0);
        if p.1 {
            ctx.paint_brush(p.0, color);
        }
        while !self.pts.is_empty() && !self.pts[0].1 {
            self.pts.remove(0);
        }
    }

    fn cleanup_points(&mut self) {
        // XXX clone?
        let mut pts = self.pts.clone();
        pts.reverse();
        let o = N::one();
        let z = N::zero();
        for p in &mut pts {
            let pt = p.0;
            let n = self.active_point_exists(pt.x + z, pt.y - o);
            let s = self.active_point_exists(pt.x + z, pt.y + o);
            let w = self.active_point_exists(pt.x - o, pt.y + z);
            let e = self.active_point_exists(pt.x + o, pt.y + z);

            let count =
                self.point_exists(pt.x + z, pt.y - o) as isize +
                self.point_exists(pt.x + z, pt.y + o) as isize +
                self.point_exists(pt.x - o, pt.y + z) as isize +
                self.point_exists(pt.x + o, pt.y + z) as isize;

            p.1 = !(count == 2 && (n && w || n && e || s && w || s && e))
        }
        pts.reverse();
        self.pts = pts;
    }

    fn point_exists(&self, x: N, y: N) -> bool {
        self.pts.iter().any(|p| p.0.x == x && p.0.y == y)
    }

    fn active_point_exists(&self, x: N, y: N) -> bool {
        self.pts.iter().any(|p| p.1 && p.0.x == x && p.0.y == y)
    }
}

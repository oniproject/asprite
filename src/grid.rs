use math::*;
use theme::*;
use render::*;

#[derive(Clone, Copy)]
pub struct Grid {
    pub size: Vector2<i16>,
    pub offset: Vector2<i16>,
}

impl Grid {
    pub fn paint(&self, ctx: &mut Canvas, zoom: i16, rect: Rect<i32>) {
        let (pos, size) = {
            let min = rect.min;
            let pos = Point2::new(min.x as i16, min.y as i16);
            let size = Vector2::new(rect.dx() as i16, rect.dy() as i16);
            (pos, size)
        };

        let grid_color = GRID_COLOR.to_be();
        let corner_color = CORNER_COLOR.to_be();

        let (ox, oy) = (pos.x, pos.y);

        let (x1, x2) = (ox, ox + size.x * zoom);
        let (y1, y2) = (oy, oy + size.y * zoom);

        let ex = size.x / self.size.x;
        let ey = size.y / self.size.y;

        let ox = ox + (self.offset.x % self.size.x) * zoom;
        let oy = oy + (self.offset.y % self.size.y) * zoom;

        let zx = self.size.x * zoom;
        let zy = self.size.y * zoom;

        //ctx.clip(Rect::from_min_dim(pos, size * zoom));
        for x in 1..ex + 1 {
            let x = ox + x * zx;
            ctx.vline(x - 1, y1, y2, grid_color);
        }
        for y in 1..ey + 1 {
            let y = oy + y * zy;
            ctx.hline(x1, x2, y - 1, grid_color);
        }
        //ctx.unclip();

        // canvas border
        ctx.hline(x1-1, x2, y1-1, corner_color);
        ctx.hline(x1-1, x2, y2+0, corner_color);
        ctx.vline(x1-1, y1-1, y2, corner_color);
        ctx.vline(x2+0, y1-1, y2, corner_color);
    }
}

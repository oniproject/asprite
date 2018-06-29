use std::mem::size_of;
use math::{Rect, Point2, BaseNum, BaseIntExt};

pub fn mask<N, F>(r: Rect<N>, br: Rect<N>, brush: &[bool], mut f: F)
    where
        F: FnMut(N, N),
        N: BaseIntExt
{
    blit(r, br, brush, |x, y, pix| if pix { f(x, y) });
}

pub fn blit<N, F, C>(r: Rect<N>, br: Rect<N>, brush: &[C], mut f: F)
    where
        F: FnMut(N, N, C),
        N: BaseIntExt,
        C: Copy
{
    let w = br.dx();
    let start = r.min - br.min;
    let start = (start.x + start.y * w).to_usize().unwrap();
    let stride = (w - r.dx()).to_usize().unwrap();
    unsafe {
        let mut pix = brush.as_ptr().add(start);
        for y in r.min.y..r.max.y {
            for x in r.min.x..r.max.x {
                f(x, y, *pix);
                pix = pix.add(size_of::<C>());
            }
            pix = pix.add(stride);
        }
    }
}

pub fn fill_rect<N, F>(r: Rect<N>, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
    for y in r.min.y..r.max.y {
        for x in r.min.x..r.max.x {
            pixel(Point2::new(x, y))
        }
    }
}

pub fn draw_line<N, F>(start: Point2<N>, end: Point2<N>, pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
    super::Bresenham::new(start, end).for_each(pixel)

    /*
    let one = N::one();
    let two = N::one() + N::one();

    let dx = (start.x - end.x).abs();
    let dy = (start.y - end.y).abs();

    if dx >= one || dy >= one {
        let (incr, delta) = {
            let incr_x = if start.x < end.x { one } else { -one };
            let incr_y = if start.y < end.y { one } else { -one };
            (Point2::new(incr_x, incr_y), Point2::new(dx, dy))
        };

        let mut pos = start;
        if delta.y > delta.x {
            let mut cumul = delta.y / two;
            for _ in one..delta.y {
                pos.y += incr.y;
                cumul += delta.x;
                if cumul >= delta.y {
                    cumul -= delta.y;
                    pos.x += incr.x;
                }
                pixel(pos);
            }
        } else {
            let mut cumul = delta.x / two;
            for _ in one..delta.x {
                pos.x += incr.x;
                cumul += delta.y;
                if cumul >= delta.x {
                    cumul -= delta.x;
                    pos.y += incr.y;
                }
                pixel(pos);
            }
        }
    }

    if start != end {
        pixel(end);
    }
    */
}

pub fn draw_ellipse<N, F>(r: Rect<N>, mut seg: F)
    where
        N: BaseNum,
        F: FnMut(Point2<N>, Point2<N>),
{
    let (mut x0, mut y0, mut x1, mut y1) = (
        r.min.x.to_i64().unwrap(),
        r.min.y.to_i64().unwrap(),
        r.max.x.to_i64().unwrap(),
        r.max.y.to_i64().unwrap(),
    ); 

    let a = (x1 - x0).abs();
    let b = (y1 - y0).abs();

    // values of diameter
    let mut b1 = b & 1;

    // error increment
    let mut dx = 4 * ( 1 - a) * b * b;
    let mut dy = 4 * (b1 + 1) * a * a;
    let mut err = dx + dy + b1 * a * a;

    // if called with swapped points
    if x0 > x1 {
        x0 = x1;
        x1 += a;
    }

    // .. exchange them
    if y0 > y1 {
        y0 = y1;
    }

    // starting pixel
    y0 += (b + 1) / 2;
    y1 = y0 - b1;
    let a = 8 * a * a;
    b1 = 8 * b * b;

    while {
        let q1 = Point2::new(x1, y0).cast().unwrap();
        let q2 = Point2::new(x0, y0).cast().unwrap();
        let q3 = Point2::new(x0, y1).cast().unwrap();
        let q4 = Point2::new(x1, y1).cast().unwrap();
        seg(q2, q1);
        seg(q3, q4);
        let e2 = 2 * err; // error of 1.step
        // y step
        if e2 <= dy {
            y0 += 1;
            y1 -= 1;
            dy += a;
            err += dy;
        }

        // x step
        if e2 >= dx || 2 * err > dy {
            x0 += 1;
            x1 -= 1;
            dx += b1;
            err += dx;
        }

        x0 <= x1
    } {}

    // too early stop of flat ellipses a=1
    while y0 - y1 < b {
        // -> finish tip of ellipse 
        let a = Point2::new(x0-1, y0).cast().unwrap();
        let b = Point2::new(x1+1, y0).cast().unwrap();
        seg(a, b);
        y0 += 1;

        let a = Point2::new(x0-1, y1).cast().unwrap();
        let b = Point2::new(x1+1, y1).cast().unwrap();
        seg(a, b);
        y1 -= 1;
    }
}

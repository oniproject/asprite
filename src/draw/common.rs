use math::*;

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
    let start = (start.x + start.y * w).to_isize().unwrap();
    let stride = (w - r.dx()).to_isize().unwrap();
    unsafe {
        let mut pix = brush.as_ptr().offset(start);
        for y in r.min.y..r.max.y {
            for x in r.min.x..r.max.x {
                f(x, y, *pix);
                pix = pix.offset(1);
            }
            pix = pix.offset(stride);
        }
    }
}

fn _hline<N, F>(x1: N, x2: N, y: N, pixel: &mut F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
    for x in x1..x2 {
        pixel(Point2::new(x, y))
    }
}

fn _vline<N, F>(x: N, y1: N, y2: N, pixel: &mut F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
    for y in y1..y2 {
        pixel(Point2::new(x, y))
    }
}

pub fn draw_rect<N, F>(r: Rect<N>, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
    _hline(r.min.x, r.max.x, r.min.y, &mut pixel);
    _hline(r.min.x, r.max.x, r.max.y, &mut pixel);
    _vline(r.min.x, r.min.y, r.max.y, &mut pixel);
    _vline(r.max.x, r.min.y, r.max.y, &mut pixel);
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

pub fn draw_line<N, F>(start: Point2<N>, end: Point2<N>, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseIntExt
{
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

    let mut a = (x1-x0).abs();
    let b = (y1-y0).abs();
    // values of diameter
    let mut b1 = b & 1;

    // error increment
    let mut dx = 4*(1-a)*b*b;
    let mut dy = 4*(b1+1)*a*a;
    let mut err = dx+dy+b1*a*a;
    let mut e2; // error of 1.step

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
    y0 += (b+1)/2;
    y1 = y0-b1;
    a *= 8*a;
    b1 = 8*b*b;

    while {
        let q1 = Point2::new(N::from(x1).unwrap(), N::from(y0).unwrap());
        let q2 = Point2::new(N::from(x0).unwrap(), N::from(y0).unwrap());
        let q3 = Point2::new(N::from(x0).unwrap(), N::from(y1).unwrap());
        let q4 = Point2::new(N::from(x1).unwrap(), N::from(y1).unwrap());
        seg(q2, q1);
        seg(q3, q4);
        e2 = 2*err;
        // y step 
        if e2 <= dy {
            y0 += 1;
            y1 -= 1;
            dy += a;
            err += dy;
        }
        // x step
        if e2 >= dx || 2*err > dy {
            x0 += 1;
            x1 -= 1;
            dx += b1;
            err += dx;
        }

        x0 <= x1
    } {}

    // too early stop of flat ellipses a=1
    while y0-y1 < b {
        // -> finish tip of ellipse 
        let a = Point2::new(N::from(x0-1).unwrap(), N::from(y0).unwrap());
        let b = Point2::new(N::from(x1+1).unwrap(), N::from(y0).unwrap());
        seg(a, b);
        y0 += 1;

        let a = Point2::new(N::from(x0-1).unwrap(), N::from(y1).unwrap());
        let b = Point2::new(N::from(x1+1).unwrap(), N::from(y1).unwrap());
        seg(a, b);
        y1 -= 1;
    }
}

#![allow(dead_code)]

use math::*;
use std::iter::Step;

pub trait Canvas<C, N>
    where
        C: Copy + Clone + Eq,
        N: BaseNumExt + Step,
{
    unsafe fn set_pixel_unchecked(&mut self, x: N, y: N, color: C);
    unsafe fn get_pixel_unchecked(&self, x: N, y: N) -> C;

    fn bounds(&self) -> Rect<N>;

    fn at(&self, x: N, y: N) -> Option<C> {
        if self.bounds().contains_xy(x, y) {
            unsafe { Some(self.get_pixel_unchecked(x, y)) }
        } else {
            None
        }
    }

    fn intersect(&self, r: Rect<N>) -> Option<Rect<N>> {
        self.bounds().intersect(r)
    }

    fn rect(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            draw_rect(r, |p| unsafe {
                self.set_pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn rect_fill(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            fill_rect(r, |p| unsafe {
                self.set_pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn ellipse_fill(&mut self, r: Rect<N>, color: C) {
        draw_ellipse(r, |a, b| {
            hline_(a.x, b.x, a.y, |p| self.pixel(p.x, p.y, color));
        });
    }

    fn ellipse(&mut self, r: Rect<N>, color: C) {
        draw_ellipse(r, |a, b| {
            self.pixel(a.x, a.y, color);
            self.pixel(b.x, b.y, color);
        });
    }

    fn line(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            draw_line(r.min, r.max, |p| unsafe {
                self.set_pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn brush_line(&mut self, r: Rect<N>, size: Point2<N>, brush: &[bool], color: C) {
        draw_line(r.min, r.max, |p| {
            let br = Rect::from_coords_and_size(p.x, p.y, size.x, size.y);
            self.mask(br, brush, color);
        });
    }

    fn pixel(&mut self, x: N, y: N, color: C) {
        if self.bounds().contains_xy(x, y) {
            unsafe { self.set_pixel_unchecked(x, y, color) }
        }
    }

    fn mask(&mut self, br: Rect<N>, brush: &[bool], color: C) {
        if let Some(r) = self.intersect(br) {
            unsafe {
                mask(r, br, brush, |x, y| self.set_pixel_unchecked(x, y, color))
            }
        }
    }

    fn blit(&mut self, br: Rect<N>, brush: &[C]) {
        if let Some(r) = self.intersect(br) {
            unsafe {
                blit(r, br, brush, |x, y, color| self.set_pixel_unchecked(x, y, color))
            }
        }
    }

    /// see http://will.thimbleby.net/scanline-flood-fill/
    fn scanline_fill(&mut self, p: Point2<N>, color: C) {
        let x = p.x;
        let y = p.y;

        let one = N::one();
        let zero = N::zero();

        // x_min, x_max, y, down[true] / up[false], extend_left, extend_right
        let mut ranges = vec![(x, x, y, None, true, true)];
        let bounds = self.bounds();
        let width = bounds.dx();
        let height = bounds.dy();

        let test = self.at(x, y);
        if test.is_none() || test == Some(color) {
            return;
        }

        unsafe { self.set_pixel_unchecked(p.x, p.y, color) }

        while let Some((mut r0, mut r1, y, r3, extend_left, extend_right)) = ranges.pop() {
            let down = r3 == Some(true);
            let up =   r3 == Some(false);

            // extend left
            let mut min_x = r0;
            if extend_left {
                while min_x > zero && self.at(min_x-one, y) == test {
                    min_x -= one;
                    unsafe { self.set_pixel_unchecked(min_x, y, color) }
                }
            }

            // extend right
            let mut max_x = r1;
            if extend_right {
                while max_x < width-one && self.at(max_x+one, y) == test {
                    max_x += one;
                    unsafe { self.set_pixel_unchecked(max_x, y, color) }
                }
            }

            // extend range ignored from previous line
            r0 -= one;
            r1 += one;

            let mut line = |y, is_next, downwards: bool| {
                let mut rmin = min_x;
                let mut in_range = false;

                let mut x = min_x;

                while x <= max_x {
                    // skip testing, if testing previous line within previous range
                    let empty = (is_next || x < r0 || x > r1) && self.at(x, y) == test;

                    in_range = if !in_range && empty {
                        rmin = x;
                        true
                    } else if in_range && !empty {
                        ranges.push((rmin, x-one, y, Some(downwards), rmin == min_x, false));
                        false
                    } else {
                        in_range
                    };

                    if in_range {
                        unsafe { self.set_pixel_unchecked(x, y, color) }
                    }

                    // skip
                    if !is_next && x == r0 {
                        x = r1;
                    }

                    x += one;
                }

                if in_range {
                    ranges.push((rmin, x-one, y, Some(downwards), rmin == min_x, true));
                }
            };

            if y < height - one {
                line(y+one, !up, true);
            }
            if y > zero {
                line(y-one, !down, false);
            }
        }
    }
}

pub fn mask<N, F>(r: Rect<N>, br: Rect<N>, brush: &[bool], mut f: F)
    where
        F: FnMut(N, N),
        N: BaseNum + Step
{
    let w = br.dx();
    let start = r.min - br.min;
    let start = (start.x + start.y * w).to_isize().unwrap();
    let stride = (w - r.dx()).to_isize().unwrap();
    unsafe {
        let mut pix = brush.as_ptr().offset(start);
        for y in r.min.y..r.max.y {
            for x in r.min.x..r.max.x {
                if *pix {
                    f(x, y)
                }
                pix = pix.offset(1);
            }
            pix = pix.offset(stride);
        }
    }
}

pub fn blit<N, F, C>(r: Rect<N>, br: Rect<N>, brush: &[C], mut f: F)
    where
        F: FnMut(N, N, C),
        N: BaseNum + Step,
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

pub fn hline_<N, F>(x1: N, x2: N, y: N, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseNum + Step
{
    let one = N::one();
    for x in x1..x2+one {
        pixel(Point2::new(x, y))
    }
}

fn hline<N, F>(x1: N, x2: N, y: N, pixel: &mut F)
    where
        F: FnMut(Point2<N>),
        N: BaseNum + Step
{
    for x in x1..x2 {
        pixel(Point2::new(x, y))
    }
}

fn vline<N, F>(x: N, y1: N, y2: N, pixel: &mut F)
    where
        F: FnMut(Point2<N>),
        N: BaseNum + Step
{
    for y in y1..y2 {
        pixel(Point2::new(x, y))
    }
}

pub fn draw_rect<N, F>(r: Rect<N>, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseNum + Step
{
    hline(r.min.x, r.max.x, r.min.y, &mut pixel);
    hline(r.min.x, r.max.x, r.max.y, &mut pixel);
    vline(r.min.x, r.min.y, r.max.y, &mut pixel);
    vline(r.max.x, r.min.y, r.max.y, &mut pixel);
}

pub fn fill_rect<N, F>(r: Rect<N>, mut pixel: F)
    where
        F: FnMut(Point2<N>),
        N: BaseNum + Step
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
        N: BaseNumExt + Step
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

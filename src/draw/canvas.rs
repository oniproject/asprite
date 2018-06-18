#![allow(dead_code)]

use math::*;

use super::common::*;

pub trait Bounded<N> where N: BaseIntExt {
    fn bounds(&self) -> Rect<N>;

    fn intersect(&self, r: Rect<N>) -> Option<Rect<N>> {
        self.bounds().intersect(r)
    }
}

pub trait CanvasRead<C, N>: Bounded<N>
    where
        C: Copy + Clone + Eq,
        N: BaseIntExt,
{
    unsafe fn get_pixel_unchecked(&self, x: N, y: N) -> C;

    fn at(&self, x: N, y: N) -> Option<C> {
        if self.bounds().contains_xy(x, y) {
            unsafe { Some(self.get_pixel_unchecked(x, y)) }
        } else {
            None
        }
    }
}

pub trait CanvasWrite<C, N>: Bounded<N>
    where
        C: Copy + Clone + Eq,
        N: BaseIntExt,
{
    unsafe fn set_pixel_unchecked(&mut self, x: N, y: N, color: C);

    fn pixel(&mut self, x: N, y: N, color: C) {
        if self.bounds().contains_xy(x, y) {
            unsafe { self.set_pixel_unchecked(x, y, color) }
        }
    }

    fn rect(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            draw_rect(r, |p| unsafe {
                self.set_pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn clear(&mut self, color: C) {
        let r = self.bounds();
        self.rect_fill(r, color);
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
            for x in a.x..b.x + N::one() {
                self.pixel(x, a.y, color)
            }
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

    fn mask(&mut self, br: Rect<N>, brush: &[bool], color: C) {
        if let Some(r) = self.intersect(br) {
            mask(r, br, brush, |x, y| unsafe { self.set_pixel_unchecked(x, y, color) })
        }
    }

    fn blit(&mut self, br: Rect<N>, brush: &[C]) {
        if let Some(r) = self.intersect(br) {
            blit(r, br, brush, |x, y, color| unsafe { self.set_pixel_unchecked(x, y, color) })
        }
    }
}

impl<T, C, N> CanvasFill<C, N> for T
    where
        C: Copy + Clone + Eq,
        N: BaseIntExt,
        T: CanvasRead<C, N> + CanvasWrite<C, N>
{}

/// see http://will.thimbleby.net/scanline-flood-fill/
pub trait CanvasFill<C, N>: CanvasRead<C, N> + CanvasWrite<C, N>
    where
        C: Copy + Clone + Eq,
        N: BaseIntExt,
{
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

#![allow(dead_code)]
use common::*;

use math::*;

// see http://will.thimbleby.net/scanline-flood-fill/

use std::iter::Step;

pub trait Canvas<Color, N>
    where
        Color: Copy + Clone + Eq,
        N: BaseNumExt + Step,
{
    unsafe fn pixel_unchecked(&mut self, x: N, y: N, color: Color);
    fn update(&mut self, r: Rect<N>);
    fn bounds(&self) -> Rect<N>;

    fn at(&self, x: N, y: N) -> Option<Color>;

    fn update_point(&mut self, p: Point2<N>) {
        let one = N::one();
        let r = Rect::from_coords_and_size(p.x, p.y, one, one);
        self.update(r);
    }

    #[inline(always)]
    fn intersect(&self, r: Rect<N>) -> Option<Rect<N>> {
        self.bounds().intersect(r)
    }

    fn rect(&mut self, r: Rect<N>, color: Color) {
        if let Some(r) = self.intersect(r) {
            draw_rect(r, |p| unsafe {
                self.pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn rect_fill(&mut self, r: Rect<N>, color: Color) {
        if let Some(r) = self.intersect(r) {
            fill_rect(r, |p| unsafe {
                self.pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn ellipse_fill(&mut self, r: Rect<N>, color: Color) {
        draw_ellipse(r, |a, b| {
            hline_(a.x, b.x, a.y, |p| self.pixel(p.x, p.y, color));
        });
    }

    fn ellipse(&mut self, r: Rect<N>, color: Color) {
        draw_ellipse(r, |a, b| {
            self.pixel(a.x, a.y, color);
            self.pixel(b.x, b.y, color);
        });
    }

    fn line(&mut self, r: Rect<N>, color: Color) {
        if let Some(r) = self.intersect(r) {
            draw_line(r.min, r.max, |p| unsafe {
                self.pixel_unchecked(p.x, p.y, color)
            });
        }
    }

    fn brush_line(&mut self, r: Rect<N>, size: Point2<N>, brush: &[bool], color: Color) {
        draw_line(r.min, r.max, |p| {
            let br = Rect::from_coords_and_size(p.x, p.y, size.x, size.y);
            self.mask(br, brush, color);
        });
    }

    fn pixel(&mut self, x: N, y: N, color: Color) {
        if self.bounds().contains_xy(x, y) {
            unsafe { self.pixel_unchecked(x, y, color) }
        }
    }

    fn mask(&mut self, br: Rect<N>, brush: &[bool], color: Color) {
        if let Some(r) = self.intersect(br) {
            unsafe {
                mask(r, br, brush, |x, y| self.pixel_unchecked(x, y, color))
            }
        }
    }

    fn blit(&mut self, br: Rect<N>, brush: &[Color]) {
        if let Some(r) = self.intersect(br) {
            unsafe {
                blit(r, br, brush, |x, y, color| self.pixel_unchecked(x, y, color))
            }
        }
    }

    fn scanline_fill(&mut self, p: Point2<N>, color: Color) {
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

        unsafe { self.pixel_unchecked(p.x, p.y, color) }

        while let Some((mut r0, mut r1, y, r3, extend_left, extend_right)) = ranges.pop() {
            let down = r3 == Some(true);
            let up =   r3 == Some(false);

            // extend left
            let mut min_x = r0;
            if extend_left {
                while min_x > zero && self.at(min_x-one, y) == test {
                    min_x -= one;
                    unsafe { self.pixel_unchecked(min_x, y, color) }
                }
            }

            // extend right
            let mut max_x = r1;
            if extend_right {
                while max_x < width-one && self.at(max_x+one, y) == test {
                    max_x += one;
                    unsafe { self.pixel_unchecked(max_x, y, color) }
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
                        unsafe { self.pixel_unchecked(x, y, color) }
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

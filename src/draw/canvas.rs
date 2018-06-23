use math::{Rect, BaseIntExt};

use super::common::*;

pub trait Bounded<N> where N: BaseIntExt {
    fn bounds(&self) -> Rect<N>;

    fn intersect(&self, r: Rect<N>) -> Option<Rect<N>> {
        self.bounds().intersect(r)
    }
}

pub trait CanvasRead<C, N>: Bounded<N>
    where
        C: Copy + Eq,
        N: BaseIntExt,
{
    unsafe fn at_unchecked(&self, x: N, y: N) -> C;

    fn at(&self, x: N, y: N) -> Option<C> {
        if self.bounds().contains_xy(x, y) {
            unsafe { Some(self.at_unchecked(x, y)) }
        } else {
            None
        }
    }
}

pub trait CanvasWrite<C, N>: Bounded<N>
    where
        C: Copy + Eq,
        N: BaseIntExt,
{
    unsafe fn set_unchecked(&mut self, x: N, y: N, color: C);

    fn set(&mut self, x: N, y: N, color: C) {
        if self.bounds().contains_xy(x, y) {
            unsafe { self.set_unchecked(x, y, color) }
        }
    }
}

impl<T, C, N> CanvasWriteExt<C, N> for T
    where
        T: CanvasWrite<C, N>,
        C: Copy + Eq,
        N: BaseIntExt,
{}

pub trait CanvasWriteExt<C, N>: CanvasWrite<C, N>
    where
        C: Copy + Eq,
        N: BaseIntExt,
{
    /*
    fn rect(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            draw_rect(r, |p| unsafe {
                self.set_unchecked(p.x, p.y, color)
            });
        }
    }

    fn clear(&mut self, color: C) {
        let r = self.bounds();
        self.fill(r, color);
    }

    fn fill(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            fill_rect(r, |p| unsafe {
                self.set_unchecked(p.x, p.y, color)
            });
        }
    }

    fn line(&mut self, r: Rect<N>, color: C) {
        if let Some(r) = self.intersect(r) {
            draw_line(r.min, r.max, |p| unsafe {
                self.set_unchecked(p.x, p.y, color)
            });
        }
    }

    fn mask(&mut self, br: Rect<N>, brush: &[bool], color: C) {
        if let Some(r) = self.intersect(br) {
            mask(r, br, brush, |x, y| unsafe { self.set_unchecked(x, y, color) })
        }
    }

    fn blit(&mut self, br: Rect<N>, brush: &[C]) {
        if let Some(r) = self.intersect(br) {
            blit(r, br, brush, |x, y, color| unsafe { self.set_unchecked(x, y, color) })
        }
    }
    */
}

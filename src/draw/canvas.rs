use math::{Rect, BaseIntExt};

pub trait Bounded<N: BaseIntExt> {
    fn bounds(&self) -> Rect<N>;

    fn intersect(&self, r: Rect<N>) -> Option<Rect<N>> {
        self.bounds().intersect(r)
    }
}

pub trait CanvasRead<C, N: BaseIntExt>: Bounded<N> {
    unsafe fn at_unchecked(&self, x: N, y: N) -> C;

    fn at(&self, x: N, y: N) -> Option<C> {
        if self.bounds().contains_xy(x, y) {
            unsafe { Some(self.at_unchecked(x, y)) }
        } else {
            None
        }
    }
}

pub trait CanvasWrite<C, N: BaseIntExt>: Bounded<N> {
    unsafe fn set_unchecked(&mut self, x: N, y: N, color: C);

    fn set(&mut self, x: N, y: N, color: C) {
        if self.bounds().contains_xy(x, y) {
            unsafe { self.set_unchecked(x, y, color) }
        }
    }
}

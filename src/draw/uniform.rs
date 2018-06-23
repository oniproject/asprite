use math::{Rect, BaseIntExt};
use super::{CanvasRead, Bounded};

pub struct Uniform<C>(pub C);

impl<C, N> Bounded<N> for Uniform<C>
    where N: BaseIntExt
{
    fn bounds(&self) -> Rect<N> {
        let min = N::min_value();
        let max = N::max_value();
        Rect::from_coords(min, min, max, max)
    }
}

impl<C, N> CanvasRead<C, N> for Uniform<C>
    where C: Copy + Eq, N: BaseIntExt,
{
    unsafe fn at_unchecked(&self, _x: N, _y: N) -> C { self.0 }
    fn at(&self, _x: N, _y: N) -> Option<C> { Some(self.0) }
}

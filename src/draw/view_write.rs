use std::marker::PhantomData;
use std::mem::size_of;
use math::{Rect, BaseIntExt};
use super::{CanvasRead, CanvasWrite, Bounded};

pub struct ViewWrite<'a, C, N> {
    pub(in draw) pix: &'a mut [u8],
    pub(in draw) stride: usize,
    pub(in draw) rect: Rect<N>,
    _marker: PhantomData<C>,
}

impl<'a, C, N> ViewWrite<'a, C, N> where N: BaseIntExt {
    #[inline(always)]
    pub fn new(pix: &'a mut [u8], width: N, height: N) -> Self {
        Self {
            pix,
            stride: width.to_usize().unwrap(),
            rect: Rect::from_coords(N::zero(), N::zero(), width, height),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn pix_offset(&self, x: N, y: N) -> usize {
        let a = (y - self.rect.min.y).to_usize().unwrap() * self.stride;
        let b = (x - self.rect.min.x).to_usize().unwrap() * size_of::<C>();
        a + b
    }

    #[inline(always)]
    pub fn sub(&mut self, rect: Rect<N>) -> Option<ViewWrite<C, N>> {
        rect.intersect(self.rect).map(move |rect| {
            let i = self.pix_offset(rect.min.x, rect.min.y);
            ViewWrite {
                rect,
                pix:    &mut self.pix[i..],
                stride: self.stride,
                _marker: PhantomData,
            }
        })
    }
}

impl<'a, C, N> Bounded<N> for ViewWrite<'a, C, N> where N: BaseIntExt {
    #[inline(always)]
    fn bounds(&self) -> Rect<N> {
        let min = N::min_value();
        let max = N::max_value();
        Rect::from_coords(min, min, max, max)
    }
}

impl<'a, N> CanvasRead<u8, N> for ViewWrite<'a, u8, N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> u8 {
        let offset = self.pix_offset(x, y);
        *self.pix.get_unchecked(offset)
    }
}
impl<'a, N> CanvasWrite<u8, N> for ViewWrite<'a, u8, N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: N, y: N, c: u8) {
        let offset = self.pix_offset(x, y);
        *self.pix.get_unchecked_mut(offset) = c;
    }
}

impl<'a, N> CanvasRead<(u8, u8), N> for ViewWrite<'a, (u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> (u8, u8) {
        let offset = self.pix_offset(x, y);
        (
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
        )
    }
}
impl<'a, N> CanvasWrite<(u8, u8), N> for ViewWrite<'a, (u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: N, y: N, c: (u8, u8)) {
        let offset = self.pix_offset(x, y);
        *self.pix.get_unchecked_mut(offset) = c.0;
        *self.pix.get_unchecked_mut(offset) = c.1;
    }
}

impl<'a, N> CanvasRead<(u8, u8, u8), N> for ViewWrite<'a, (u8, u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> (u8, u8, u8) {
        let offset = self.pix_offset(x, y);
        (
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
        )
    }
}
impl<'a, N> CanvasWrite<(u8, u8, u8), N> for ViewWrite<'a, (u8, u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: N, y: N, c: (u8, u8, u8)) {
        let offset = self.pix_offset(x, y);
        *self.pix.get_unchecked_mut(offset) = c.0;
        *self.pix.get_unchecked_mut(offset) = c.1;
        *self.pix.get_unchecked_mut(offset) = c.2;
    }
}

impl<'a, N> CanvasRead<(u8, u8, u8, u8), N> for ViewWrite<'a, (u8, u8, u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> (u8, u8, u8, u8) {
        let offset = self.pix_offset(x, y);
        (
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
            *self.pix.get_unchecked(offset),
        )
    }
}
impl<'a, N> CanvasWrite<(u8, u8, u8, u8), N> for ViewWrite<'a, (u8, u8, u8, u8), N> where N: BaseIntExt {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: N, y: N, c: (u8, u8, u8, u8)) {
        let offset = self.pix_offset(x, y);
        *self.pix.get_unchecked_mut(offset) = c.0;
        *self.pix.get_unchecked_mut(offset) = c.1;
        *self.pix.get_unchecked_mut(offset) = c.2;
        *self.pix.get_unchecked_mut(offset) = c.3;
    }
}

use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use math::{Rect, Point2, BaseIntExt, SliceExt};

use super::{CanvasRead, CanvasWrite, Bounded};

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
    where C: Copy, N: BaseIntExt,
{
    unsafe fn at_unchecked(&self, _x: N, _y: N) -> C { self.0 }
    fn at(&self, _x: N, _y: N) -> Option<C> { Some(self.0) }
}

pub struct View<'a, C, N> {
    pix: &'a [u8],
    stride: usize,
    rect: Rect<N>,
    _marker: PhantomData<C>,
}

impl<'a, C, N> View<'a, C, N> where N: BaseIntExt {
    #[inline(always)]
    pub fn new(pix: &'a [u8], width: N, height: N) -> Self {
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
    pub fn sub(&self, rect: Rect<N>) -> Option<View<C, N>> {
        rect.intersect(self.rect).map(move |rect| {
            let i = self.pix_offset(rect.min.x, rect.min.y);
            View {
                rect,
                pix:    &self.pix[i..],
                stride: self.stride,
                _marker: PhantomData,
            }
        })
    }
}

impl<'a, C, N> Bounded<N> for View<'a, C, N> where N: BaseIntExt {
    #[inline(always)]
    fn bounds(&self) -> Rect<N> {
        let min = N::min_value();
        let max = N::max_value();
        Rect::from_coords(min, min, max, max)
    }
}

impl<'a, C, N> CanvasRead<C, N> for View<'a, C, N> where N: BaseIntExt, C: Copy {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> C {
        let offset = self.pix_offset(x, y);
        let ptr = self.pix.as_ptr().add(offset);
        (ptr as *const C).read()
    }
}


pub struct ViewMut<'a, C, N> {
    pix: &'a mut [u8],
    stride: usize,
    rect: Rect<N>,
    _marker: PhantomData<C>,
}

impl<'a, C, N> ViewMut<'a, C, N> where N: BaseIntExt {
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
    pub fn sub(&mut self, rect: Rect<N>) -> Option<ViewMut<C, N>> {
        rect.intersect(self.rect).map(move |rect| {
            let i = self.pix_offset(rect.min.x, rect.min.y);
            ViewMut {
                rect,
                pix:    &mut self.pix[i..],
                stride: self.stride,
                _marker: PhantomData,
            }
        })
    }
}

impl<'a, C, N> Bounded<N> for ViewMut<'a, C, N> where N: BaseIntExt {
    #[inline(always)]
    fn bounds(&self) -> Rect<N> {
        let min = N::min_value();
        let max = N::max_value();
        Rect::from_coords(min, min, max, max)
    }
}

impl<'a, C, N> CanvasRead<C, N> for ViewMut<'a, C, N> where N: BaseIntExt, C: Copy {
    #[inline(always)]
    unsafe fn at_unchecked(&self, x: N, y: N) -> C {
        let offset = self.pix_offset(x, y);
        let ptr = self.pix.as_ptr().add(offset);
        (ptr as *const C).read()
    }
}

impl<'a, C, N> CanvasWrite<C, N> for ViewMut<'a, C, N> where N: BaseIntExt, C: Copy {
    #[inline(always)]
    unsafe fn set_unchecked(&mut self, x: N, y: N, c: C) {
        let offset = self.pix_offset(x, y);
        let ptr = self.pix.as_ptr().add(offset);
        (ptr as *mut C).write(c)
    }
}

pub fn copy<C, N>(dst: &mut ViewMut<C, N>, r: Rect<N>, src: &View<C, N>, sp: Point2<N>)
    where C: Copy, N: BaseIntExt
{
    let n = r.dx().to_usize().unwrap() * size_of::<C>();
    let mut dy = r.dy().to_usize().unwrap();
    let mut d0 = dst.pix_offset(r.min.x, r.min.y);
    let mut s0 = src.pix_offset(sp.x, sp.y);

    if r.min.y <= sp.y {
        while dy > 0 {
            dst.pix[d0..d0+n].copy_from_slice(&src.pix[s0..s0+n]);
            d0 += dst.stride;
            s0 += src.stride;
            dy -= 1;
        }
    } else {
        d0 += (dy - 1) * dst.stride;
        s0 += (dy - 1) * src.stride;
        while dy > 0 {
            dst.pix[d0..d0+n].copy_from_slice(&src.pix[s0..s0+n]);
            d0 -= dst.stride;
            s0 -= src.stride;
            dy -= 1;
        }
    }
}

pub fn fill<C, N>(dst: &mut ViewMut<C, N>, r: Rect<N>, src: Uniform<C>)
    where C: Copy, N: BaseIntExt
{
    let mut i0 = dst.pix_offset(r.min.x, r.min.y);
    let mut i1 = i0 + r.dx().to_usize().unwrap() * size_of::<C>();

    {
        let color = src.0;
        let row = unsafe { dst.pix[i0..i1].cast_mut() };
        for c in row.iter_mut() {
            *c = color;
        }
    }

    let (src, len) = {
        let row = &dst.pix[i0..i1];
        (row.as_ptr(), row.len())
    };
    for _ in r.min.y + N::one()..r.max.y {
        i0 += dst.stride;
        i1 += dst.stride;
        let dst = dst.pix[i0..i1].as_mut_ptr();
        unsafe { copy_nonoverlapping(src, dst, len); }
    }
}

pub fn shade<V, F, C, N>(view: &mut V, mut shader: F)
    where C: Copy,
          N: BaseIntExt,
          F: FnMut(N, N, C) -> C,
          V: Bounded<N> + CanvasRead<C, N> + CanvasWrite<C, N>,
{
    let r = view.bounds();
    for y in r.min.y..r.max.y {
        for x in r.min.x..r.max.x {
            unsafe {
                let c = view.at_unchecked(x, y);
                view.set_unchecked(x, y, shader(x, y, c));
            }
        }
    }
}

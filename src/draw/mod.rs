pub mod gradient;
mod bresenham;

mod common;
mod canvas;
mod palette;
mod frame;

mod scanline;

mod view_read;
mod view_write;
mod uniform;

pub use self::view_read::ViewRead;
pub use self::view_write::ViewWrite;
pub use self::uniform::Uniform;

pub use self::bresenham::Bresenham;
pub use self::scanline::ScanlineFill;

pub use self::common::{
    blit,
    mask,
    draw_line,
    draw_ellipse,
    draw_rect,
    fill_rect,
};

pub use self::canvas::{
    Bounded,
    CanvasRead,
    CanvasWrite,
    CanvasWriteExt,
};
pub use self::palette::Palette;
pub use self::frame::Frame;

use math::{Rect, Point2, BaseIntExt};

pub fn copy_src<C, N>(dst: &mut ViewWrite<C, N>, r: Rect<N>, src: &ViewRead<C, N>, sp: Point2<N>)
    where C: Copy + Eq, N: BaseIntExt
{
    let n = r.dx().to_usize().unwrap() * ::std::mem::size_of::<C>();
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

pub fn fill_src<C, N>(dst: &mut ViewWrite<C, N>, r: Rect<N>, src: Uniform<C>, sp: Point2<N>)
    where C: Copy + Eq, N: BaseIntExt
{
    use std::slice::{from_raw_parts, from_raw_parts_mut};
    use std::mem::size_of;
    use math::SliceExt;

    let mut i0 = dst.pix_offset(r.min.x, r.min.y);
    let mut i1 = i0 + r.dx().to_usize().unwrap() * size_of::<C>();

    {
        let color = src.0;
        let row = unsafe { dst.pix[i0..i1].cast_mut() };
        for c in row.iter_mut() {
            *c = color;
        }
    }

    let row = unsafe {
        let row = &dst.pix[i0..i1];
        from_raw_parts(row.as_ptr(), row.len())
    };
    for y in r.min.y + N::one()..r.max.y {
        i0 += dst.stride;
        i1 += dst.stride;
        dst.pix[i0..i1].copy_from_slice(row)
    }
}

pub fn shade<V, F, C, N>(view: &mut V, mut shader: F)
    where C: Copy + Eq,
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

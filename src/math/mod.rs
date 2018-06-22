#![allow(dead_code)]

pub use cgmath::prelude::*;
pub use cgmath::{BaseNum, BaseFloat};

pub use cgmath::Transform;

pub use cgmath::Vector1;
pub use cgmath::Point2;
pub use cgmath::Vector2;

pub use cgmath::Matrix2;
pub use cgmath::Matrix4;

mod ext;
mod affine;
mod rect;
mod d8;
mod time;
mod stopwatch;

pub use self::ext::{BaseNumExt, BaseFloatExt, BaseIntExt};

pub use self::rect::Rect;
pub use self::affine::Affine;
pub use self::time::{Time, TimePair};
pub use self::stopwatch::Stopwatch;
pub use self::d8::D8;

pub fn lerp<S: BaseFloat>(min: S, max: S, t: S) -> S {
    (S::one() - t) * min + t * max
}

pub fn clamp01<S: BaseFloat>(v: S) -> S {
    if v < S::zero() {
        S::zero()
    } else if v > S::one() {
        S::one()
    } else {
        v
    }
}

#[inline(always)]
pub fn rect_align<S: BaseFloat>(rect: Rect<S>, align: Vector2<S>, size: Vector2<S>) -> Point2<S> {
    let v = rect.max - rect.min - size;
    rect.min + Vector2::new(v.x * align.x, v.y * align.y)
}

#[inline(always)]
fn lerp1<S: BaseFloat>(a: S, b: S, v: S) -> S {
    Vector1::new(a).lerp(Vector1::new(b), v).x
}

#[inline(always)]
fn lerp2<S: BaseFloat>(a: Vector2<S>, b: Vector2<S>, v: Vector2<S>) -> Vector2<S> {
    Vector2::new(lerp1(a.x, b.x, v.x), lerp1(a.y, b.y, v.y))
}

#[inline(always)]
pub fn rect_transform_point<S: BaseFloat>(base: Rect<S>, offset: Point2<S>, anchor: Vector2<S>) -> Point2<S> {
    offset + lerp2(base.min.to_vec(), base.max.to_vec(), anchor)
}

#[inline(always)]
pub fn rect_transform<S: BaseFloat>(base: Rect<S>, anchor: Rect<S>, offset: Rect<S>) -> Rect<S> {
    Rect {
        min: rect_transform_point(base, offset.min, anchor.min.to_vec()),
        max: rect_transform_point(base, offset.max, anchor.max.to_vec()),
    }
}

#[test]
fn transform_base() {
    let canvas = Rect {
        min: Point2::new(0.0, 0.0),
        max: Point2::new(100.0, 100.0),
    };
    let anchor = Rect {
        min: Point2::new(0.25, 0.25),
        max: Point2::new(0.75, 0.75),
    };
    let r = rect_transform(canvas, anchor, Rect {
        min: Point2::new(-10.0, -10.0),
        max: Point2::new(10.0, 10.0),
    });
    assert_eq!(r, Rect { min: Point2::new(15.0, 15.0), max: Point2::new(85.0, 85.0) });
}


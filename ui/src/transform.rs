use math::*;
use math::cgmath::Vector1;

pub fn lerp(min: f32, max: f32, t: f32) -> f32 {
	(1.0 - t) * min + t * max
}

pub fn clamp01(v: f32) -> f32 {
	if v < 0.0 {
		0.0
	} else if v > 1.0 {
		1.0
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
fn print() {
	let canvas = Rect {
		min: Point2::new(0.0, 0.0),
		max: Point2::new(100.0, 100.0),
	};
	let anchor = Rect {
		min: Point2::new(0.25, 0.25),
		max: Point2::new(0.75, 0.75),
	};
	println!("{:?}", rect_transform(canvas, anchor, Rect {
		min: Point2::new(-10.0, -10.0),
		max: Point2::new(10.0, 10.0),
	}));
}

use super::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Affine<T> {
	pub m: Matrix2<T>,
	pub t: Vector2<T>,
}

impl<T: BaseFloat> Default for Affine<T> {
	#[inline]
	fn default() -> Self {
		Self::one()
	}
}

impl<S: BaseFloat> Transform<Point2<S>> for Affine<S> {
	#[inline]
	fn one() -> Self {
		Self {
			m: Matrix2::one(),
			t: Vector2::zero(),
		}
	}

	#[inline]
	fn look_at(_eye: Point2<S>, _center: Point2<S>, _up: Vector2<S>) -> Self {
		unimplemented!();
		//let dir = center - eye;
		//Matrix3::from(Matrix2::look_at(dir, up))
	}

	#[inline]
	fn transform_vector(&self, vec: Vector2<S>) -> Vector2<S> {
		self.m * vec + self.t
	}

	#[inline]
	fn transform_point(&self, _point: Point2<S>) -> Point2<S> {
		unimplemented!();
		//Point2::from_vec((self * Point3::new(point.x, point.y, S::one()).to_vec()).truncate())
		//Point2::from_vec(self.m * Point2::new(point.x, point.y) + self.t)
	}

	#[inline]
	fn concat(&self, other: &Self) -> Self {
		Self {
			m: other.m * self.m,
			t: other.m * self.t + other.t,
		}
	}

	#[inline]
	fn inverse_transform(&self) -> Option<Self> {
		let a = self.a();
		let b = self.b();
		let c = self.c();
		let d = self.d();
		let tx = self.t.x;
		let ty = self.t.y;
		let n = (a * d) - (b * c);

		let m = Matrix2::new(
			d / n,
			-b / n,
			-c / n,
			a / n,
		);
		let t = Vector2::new(
			(c * ty - d * tx) / n,
			-(a * ty - b * tx) / n,
		);
		// FIXME:
		Some(Self { m, t })
	}
}

// transposed:
// mm00
// mm00
// tt10
// 0000
// non trnsposed
// mmt0
// mmt0
// 0010
// 0000

impl<T: BaseFloat> Into<[[T; 4]; 4]> for Affine<T> {
	#[inline]
	fn into(self) -> [[T; 4]; 4] {
		let o = T::one();
		let z = T::zero();
		[
			[self.a(), self.b(), o, o],
			[self.c(), self.d(), o, o],
			[self.t.x, self.t.y, z, o],
			[o, o, o, o],
		]
	}
}

impl<T: BaseFloat> Into<Matrix4<T>> for Affine<T> {
	#[inline]
	fn into(self) -> Matrix4<T> {
		let o = T::one();
		let z = T::zero();
		Matrix4::new(
			self.a(), self.b(), o, o,
			self.c(), self.d(), o, o,
			self.t.x, self.t.y, z, o,
			o, o, o, o,
		)
	}
}

impl<T: BaseFloat> Affine<T> {
	#[inline] fn a(&self) -> T { self.m.x.x }
	#[inline] fn b(&self) -> T { self.m.x.y }
	#[inline] fn c(&self) -> T { self.m.y.x }
	#[inline] fn d(&self) -> T { self.m.y.y }

	#[inline]
	pub fn projection(w: T, h: T) -> Self {
		let two = T::one() + T::one();
		let t = Vector2::new(-T::one(), -T::one());
		let m = Matrix2::new(
			T::one() / w * two,
			T::zero(),
			T::zero(),
			T::one() / h * two,
		);
		Self { m, t }
	}

	#[inline]
	pub fn apply_inv(&self, v: Vector2<T>) -> Vector2<T> {
		let a = self.a();
		let b = self.b();
		let c = self.c();
		let d = self.d();
		let tx = self.t.x;
		let ty = self.t.y;

		let id = T::one() / (a * d + c * (-b));

		let Vector2 { x, y } = v;

		Vector2::new(
			(d * id * x) + (-c * id * y) + ( ty * c - tx * d) * id,
			(a * id * y) + (-b * id * x) + (-ty * a + tx * b) * id,
		)
	}

	#[inline]
	pub fn translate(&mut self, x: T, y: T) {
		self.t.x += x;
		self.t.y += y;
	}
	#[inline]
	pub fn scale(&mut self, x: T, y: T) {
		self.m.x.x *= x;
		self.m.x.y *= y;
		self.m.y.x *= x;
		self.m.y.y *= y;
		self.t.x *= x;
		self.t.y *= y;
	}
	#[inline]
	pub fn rotate(&mut self, angle: T) {
		let (sin, cos) = angle.sin_cos();

		let a = self.a();
		let c = self.c();
		let tx = self.t.x;

		self.m.x.x = (a * cos) - (self.m.x.y * sin);
		self.m.x.y = (a * sin) + (self.m.x.y * cos);
		self.m.y.x = (c * cos) - (self.m.y.y * sin);
		self.m.y.y = (c * sin) + (self.m.y.y * cos);
		self.t.x = (tx * cos) - (self.t.y * sin);
		self.t.y = (tx * sin) + (self.t.y * cos);
	}

	#[inline]
	pub fn set_identity(&mut self) {
		self.m = Matrix2::one();
		self.t = Vector2::zero();
	}

	#[inline]
	pub fn compose(pos: Vector2<T>, pivot: Vector2<T>, scale: Vector2<T>, rotation: T, skew: Vector2<T>) -> Self {
		let (sr, cr) = rotation.sin_cos();
		let (sy, cy) = skew.y.sin_cos();
		let nsx = -skew.x.sin();
		let cx = skew.x.cos();

		let a =  cr * scale.x;
		let b =  sr * scale.x;
		let c = -sr * scale.y;
		let d =  cr * scale.y;

		let m = Matrix2::new(
			( cy * a) + (sy * c),
			( cy * b) + (sy * d),
			(nsx * a) + (cx * c),
			(nsx * b) + (cx * d),
		);
		let t = Vector2::new(
			pos.x + (pivot.x * a + pivot.y * c),
			pos.y + (pivot.x * b + pivot.y * d),
		);
		Self { m, t }
	}
}

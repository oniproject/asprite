use super::*;

pub fn mask<N, F>(r: Rect<N>, br: Rect<N>, brush: &[bool], mut f: F)
	where
		F: FnMut(N, N),
		N: SignedInt
{
	let w = br.dx();
	let start = r.min - br.min;
	let start = (start.x + start.y * w).to_isize().unwrap();
	let stride = (w - r.dx()).to_isize().unwrap();
	unsafe {
		let mut pix = brush.as_ptr().offset(start);
		for y in r.min.y..r.max.y {
			for x in r.min.x..r.max.x {
				if *pix {
					f(x, y)
				}
				pix = pix.offset(1);
			}
			pix = pix.offset(stride);
		}
	}
}

pub fn blit<N, F, C>(r: Rect<N>, br: Rect<N>, brush: &[C], mut f: F)
	where
		F: FnMut(N, N, C),
		N: SignedInt,
		C: Copy
{
	let w = br.dx();
	let start = r.min - br.min;
	let start = (start.x + start.y * w).to_isize().unwrap();
	let stride = (w - r.dx()).to_isize().unwrap();
	unsafe {
		let mut pix = brush.as_ptr().offset(start);
		for y in r.min.y..r.max.y {
			for x in r.min.x..r.max.x {
				f(x, y, *pix);
				pix = pix.offset(1);
			}
			pix = pix.offset(stride);
		}
	}
}

pub fn hline_<N, F>(x1: N, x2: N, y: N, mut pixel: F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	let one = N::one();
	for x in x1..x2+one {
		pixel(Point::new(x, y))
	}
}

fn hline<N, F>(x1: N, x2: N, y: N, pixel: &mut F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	for x in x1..x2 {
		pixel(Point::new(x, y))
	}
}

fn vline<N, F>(x: N, y1: N, y2: N, pixel: &mut F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	for y in y1..y2 {
		pixel(Point::new(x, y))
	}
}

pub fn draw_rect<N, F>(r: Rect<N>, mut pixel: F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	hline(r.min.x, r.max.x, r.min.y, &mut pixel);
	hline(r.min.x, r.max.x, r.max.y, &mut pixel);
	vline(r.min.x, r.min.y, r.max.y, &mut pixel);
	vline(r.max.x, r.min.y, r.max.y, &mut pixel);
}

pub fn fill_rect<N, F>(r: Rect<N>, mut pixel: F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	for y in r.min.y..r.max.y {
		for x in r.min.x..r.max.x {
			pixel(Point::new(x, y))
		}
	}
}

pub fn draw_line<N, F>(start: Point<N>, end: Point<N>, mut pixel: F)
	where
		F: FnMut(Point<N>),
		N: SignedInt
{
	let one = N::one();
	let two = N::one() + N::one();

	let dx = (start.x - end.x).abs();
	let dy = (start.y - end.y).abs();

	if dx >= one || dy >= one {
		let (incr, delta) = {
			let incr_x = if start.x < end.x { one } else { -one };
			let incr_y = if start.y < end.y { one } else { -one };
			(Point::new(incr_x, incr_y), Point::new(dx, dy))
		};

		let mut pos = start;
		if delta.y > delta.x {
			let mut cumul = delta.y / two;
			for _ in one..delta.y {
				pos.y += incr.y;
				cumul += delta.x;
				if cumul >= delta.y {
					cumul -= delta.y;
					pos.x += incr.x;
				}
				pixel(pos);
			}
		} else {
			let mut cumul = delta.x / two;
			for _ in one..delta.x {
				pos.x += incr.x;
				cumul += delta.y;
				if cumul >= delta.x {
					cumul -= delta.x;
					pos.y += incr.y;
				}
				pixel(pos);
			}
		}
	}

	if start != end {
		pixel(end);
	}
}

pub fn draw_ellipse<N, F>(r: Rect<N>, mut seg: F)
	where
		N: SignedInt,
		F: FnMut(Point<N>, Point<N>),
{
	let (mut x0, mut y0, mut x1, mut y1) = (
		r.min.x.to_i64().unwrap(),
		r.min.y.to_i64().unwrap(),
		r.max.x.to_i64().unwrap(),
		r.max.y.to_i64().unwrap(),
	); 

	let mut a = (x1-x0).abs();
	let b = (y1-y0).abs();
	// values of diameter
	let mut b1 = b & 1;

	// error increment
	let mut dx = 4*(1-a)*b*b;
	let mut dy = 4*(b1+1)*a*a;
	let mut err = dx+dy+b1*a*a;
	let mut e2; // error of 1.step

	// if called with swapped points
	if x0 > x1 {
		x0 = x1;
		x1 += a;
	}
	// .. exchange them
	if y0 > y1 {
		y0 = y1;
	}
	// starting pixel 
	y0 += (b+1)/2;
	y1 = y0-b1;
	a *= 8*a;
	b1 = 8*b*b;

	while {
		let q1 = Point::new(N::from(x1).unwrap(), N::from(y0).unwrap());
		let q2 = Point::new(N::from(x0).unwrap(), N::from(y0).unwrap());
		let q3 = Point::new(N::from(x0).unwrap(), N::from(y1).unwrap());
		let q4 = Point::new(N::from(x1).unwrap(), N::from(y1).unwrap());
		seg(q2, q1);
		seg(q3, q4);
		e2 = 2*err;
		// y step 
		if e2 <= dy {
			y0 += 1;
			y1 -= 1;
			dy += a;
			err += dy;
		}
		// x step
		if e2 >= dx || 2*err > dy {
			x0 += 1;
			x1 -= 1;
			dx += b1;
			err += dx;
		}

		x0 <= x1
	} {}
	
	// too early stop of flat ellipses a=1
	while y0-y1 < b {
		// -> finish tip of ellipse 
		let a = Point::new(N::from(x0-1).unwrap(), N::from(y0).unwrap());
		let b = Point::new(N::from(x1+1).unwrap(), N::from(y0).unwrap());
		seg(a, b);
		y0 += 1;

		let a = Point::new(N::from(x0-1).unwrap(), N::from(y1).unwrap());
		let b = Point::new(N::from(x1+1).unwrap(), N::from(y1).unwrap());
		seg(a, b);
		y1 -= 1;
	}
}



// Iterator-based Bresenham's line drawing algorithm
//
// [Bresenham's line drawing algorithm]
// (https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm) is fast
// algorithm to draw a line between two points. This crate implements the fast
// integer variant, using an iterator-based appraoch for flexibility. It
// calculates coordinates without knowing anything about drawing methods or
// surfaces.
//
// Example:
//
// ```rust
// extern crate bresenham;
// use bresenham::Bresenham;
//
// fn main() {
//	 for (x, y) in Bresenham::new((0, 1), (6, 4)) {
//		 println!("{}, {}", x, y);
//	 }
// }
// ```
//
// Will print:
//
// ```text
// (0, 1)
// (1, 1)
// (2, 2)
// (3, 2)
// (4, 3)
// (5, 3)
// ```

/// Line-drawing iterator
pub struct Bresenham<T> {
	x: T,
	y: T,
	dx: T,
	dy: T,
	x1: T,
	diff: T,
	octant: Octant,
}

struct Octant(u8);

impl Octant {
	/// adapted from http://codereview.stackexchange.com/a/95551
	#[inline]
	fn from_points<T>(start: Point<T>, end: Point<T>) -> Octant
		where T: SignedInt
	{
		let mut d = end - start;

		let mut octant = 0;

		if d.y < T::zero() {
			d.x = -d.x;
			d.y = -d.y;
			octant += 4;
		}

		if d.x < T::zero() {
			let tmp = d.x;
			d.x = d.y;
			d.y = -tmp;
			octant += 2
		}

		if d.x < d.y {
			octant += 1
		}

		Octant(octant)
	}

	#[inline]
	fn to_octant0<T>(&self, p: Point<T>) -> Point<T>
		where T: SignedInt
	{
		match self.0 {
			0 => Point::new(p.x, p.y),
			1 => Point::new(p.y, p.x),
			2 => Point::new(p.y, -p.x),
			3 => Point::new(-p.x, p.y),
			4 => Point::new(-p.x, -p.y),
			5 => Point::new(-p.y, -p.x),
			6 => Point::new(-p.y, p.x),
			7 => Point::new(p.x, -p.y),
			_ => unreachable!(),
		}
	}

	#[inline]
	fn from_octant0<T>(&self, p: Point<T>) -> Point<T>
		where T: SignedInt
	{
		match self.0 {
			0 => Point::new(p.x, p.y),
			1 => Point::new(p.y, p.x),
			2 => Point::new(-p.y, p.x),
			3 => Point::new(-p.x, p.y),
			4 => Point::new(-p.x, -p.y),
			5 => Point::new(-p.y, -p.x),
			6 => Point::new(p.y, -p.x),
			7 => Point::new(p.x, -p.y),
			_ => unreachable!(),
		}
	}
}

impl<T> Bresenham<T>
	where T: SignedInt
{
	/// Creates a new iterator.Yields intermediate points between `start`
	/// and `end`. Does include `start` but not `end`.
	#[inline]
	pub fn new(start: Point<T>, end: Point<T>) -> Bresenham<T> {
		let octant = Octant::from_points(start, end);

		let start = octant.to_octant0(start);
		let end = octant.to_octant0(end);

		let d = end - start;

		Bresenham {
			x: start.x,
			y: start.y,
			dx: d.x,
			dy: d.y,
			x1: end.x,
			diff: d.y - d.x,
			octant: octant,
		}
	}
}

impl<T> Iterator for Bresenham<T>
	where T: SignedInt
{
	type Item = Point<T>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.x >= self.x1 {
			return None;
		}

		let p = Point::new(self.x, self.y);

		if self.diff >= T::zero() {
			self.y += T::one();
			self.diff -= self.dx;
		}

		self.diff += self.dy;

		// loop inc
		self.x += T::one();

		Some(self.octant.from_octant0(p))
	}
}

#[test]
fn test_wp_example() {
	let bi = Bresenham::new(Point::new(0, 1), Point::new(6, 4));
	let res: Vec<_> = bi.collect();

	assert_eq!(
		res,
		[
			Point::new(0, 1),
			Point::new(1, 1),
			Point::new(2, 2),
			Point::new(3, 2),
			Point::new(4, 3),
			Point::new(5, 3),
		]
	)
}

#[test]
fn test_inverse_wp() {
	let bi = Bresenham::new(Point::new(6, 4), Point::new(0, 1));
	let res: Vec<_> = bi.collect();

	assert_eq!(
		res,
		[
			Point::new(6, 4),
			Point::new(5, 4),
			Point::new(4, 3),
			Point::new(3, 3),
			Point::new(2, 2),
			Point::new(1, 2),
		]
	)
}

#[test]
fn test_straight_hline() {
	let bi = Bresenham::new(Point::new(2, 3), Point::new(5, 3));
	let res: Vec<_> = bi.collect();

	assert_eq!(res, [Point::new(2, 3), Point::new(3, 3), Point::new(4, 3)]);
}

#[test]
fn test_straight_vline() {
	let bi = Bresenham::new(Point::new(2, 3), Point::new(2, 6));
	let res: Vec<_> = bi.collect();

	assert_eq!(res, [Point::new(2, 3), Point::new(2, 4), Point::new(2, 5)]);
}


/*
#[test]
fn test_br() {
	let pts = [
		Point::new(0, 0),
		Point::new(10, 10),
		Point::new(1, 2),
		Point::new(3, 4),
		Point::new(4, 6),
		Point::new(10, 10),
		Point::new(2, 3),
		Point::new(12, 5),
		Point::new(-1, -2),
		Point::new(0, 0),
		Point::new(-1, -2),
		Point::new(4, 6),
		Point::new(-10, -20),
		Point::new(30, 40),
		Point::new(8, 8),
		Point::new(8, 8),
		Point::new(88, 88),
		Point::new(88, 88),
		Point::new(6, 5),
		Point::new(4, 3),
	];

	println!("left - self, right - from grafx");
	for min in &pts {
		for max in &pts {
			//let (min, max) = (Point::new(0, 1), Point::new(6, 4));
			//let bi = Bresenham::new(*max, *min);

			let bi = Bresenham::new(*max, *min);
			let mut a: Vec<_> = bi.collect();
			a.reverse();

			let mut b = Vec::new();
			draw_line(*min, *max, |p| {
				b.push(p)
			});

			println!("min: {} max: {}", min, max);
			if a != b {
				println!("a");
				for v in &a {
					println!("\t{}", v);
				}
				println!("b");
				for v in &b {
					println!("\t{}", v);
				}
			}
			assert_eq!(a, b)
		}
	}
}
*/

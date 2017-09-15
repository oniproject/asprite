use super::*;

pub fn draw_line<N, F>(start: Point<N>, end: Point<N>, mut pixel: F)
	where
		F: FnMut(Point<N>),
		N: Signed
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
		where T: Signed
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
		where T: Signed
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
		where T: Signed
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
	where T: Signed
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
	where T: Signed
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

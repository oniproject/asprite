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
    fn from_points<T>(start: Point2<T>, end: Point2<T>) -> Octant
        where T: BaseNumExt
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
    fn to_octant0<T>(&self, p: Point2<T>) -> Point2<T>
        where T: BaseNumExt
    {
        match self.0 {
            0 => Point2::new(p.x, p.y),
            1 => Point2::new(p.y, p.x),
            2 => Point2::new(p.y, -p.x),
            3 => Point2::new(-p.x, p.y),
            4 => Point2::new(-p.x, -p.y),
            5 => Point2::new(-p.y, -p.x),
            6 => Point2::new(-p.y, p.x),
            7 => Point2::new(p.x, -p.y),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn from_octant0<T>(&self, p: Point2<T>) -> Point2<T>
        where T: BaseNumExt
    {
        match self.0 {
            0 => Point2::new(p.x, p.y),
            1 => Point2::new(p.y, p.x),
            2 => Point2::new(-p.y, p.x),
            3 => Point2::new(-p.x, p.y),
            4 => Point2::new(-p.x, -p.y),
            5 => Point2::new(-p.y, -p.x),
            6 => Point2::new(p.y, -p.x),
            7 => Point2::new(p.x, -p.y),
            _ => unreachable!(),
        }
    }
}

impl<T> Bresenham<T>
    where T: BaseNumExt
{
    /// Creates a new iterator.Yields intermediate points between `start`
    /// and `end`. Does include `start` but not `end`.
    #[inline]
    pub fn new(start: Point2<T>, end: Point2<T>) -> Bresenham<T> {
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
    where T: BaseNumExt
{
    type Item = Point2<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.x1 {
            return None;
        }

        let p = Point2::new(self.x, self.y);

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
    let bi = Bresenham::new(Point2::new(0, 1), Point2::new(6, 4));
    let res: Vec<_> = bi.collect();

    assert_eq!(
        res,
        [
            Point2::new(0, 1),
            Point2::new(1, 1),
            Point2::new(2, 2),
            Point2::new(3, 2),
            Point2::new(4, 3),
            Point2::new(5, 3),
        ]
    )
}

#[test]
fn test_inverse_wp() {
    let bi = Bresenham::new(Point2::new(6, 4), Point2::new(0, 1));
    let res: Vec<_> = bi.collect();

    assert_eq!(
        res,
        [
            Point2::new(6, 4),
            Point2::new(5, 4),
            Point2::new(4, 3),
            Point2::new(3, 3),
            Point2::new(2, 2),
            Point2::new(1, 2),
        ]
    )
}

#[test]
fn test_straight_hline() {
    let bi = Bresenham::new(Point2::new(2, 3), Point2::new(5, 3));
    let res: Vec<_> = bi.collect();

    assert_eq!(res, [Point2::new(2, 3), Point2::new(3, 3), Point2::new(4, 3)]);
}

#[test]
fn test_straight_vline() {
    let bi = Bresenham::new(Point2::new(2, 3), Point2::new(2, 6));
    let res: Vec<_> = bi.collect();

    assert_eq!(res, [Point2::new(2, 3), Point2::new(2, 4), Point2::new(2, 5)]);
}


/*
#[test]
fn test_br() {
    let pts = [
        Point2::new(0, 0),
        Point2::new(10, 10),
        Point2::new(1, 2),
        Point2::new(3, 4),
        Point2::new(4, 6),
        Point2::new(10, 10),
        Point2::new(2, 3),
        Point2::new(12, 5),
        Point2::new(-1, -2),
        Point2::new(0, 0),
        Point2::new(-1, -2),
        Point2::new(4, 6),
        Point2::new(-10, -20),
        Point2::new(30, 40),
        Point2::new(8, 8),
        Point2::new(8, 8),
        Point2::new(88, 88),
        Point2::new(88, 88),
        Point2::new(6, 5),
        Point2::new(4, 3),
    ];

    println!("left - self, right - from grafx");
    for min in &pts {
        for max in &pts {
            //let (min, max) = (Point2::new(0, 1), Point2::new(6, 4));
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

use math::*;

#[derive(Clone, Copy)]
pub enum Shape {
    Round,
    Square,
    HorizontalBar,
    VerticalBar,
    Slash,
    Antislash,
    Cross,
    Plus,
    Diamond,
    SieveRound,
    SieveSquare,

    //Random, // Random pixels in a circle shape, like an airbrush.

    /*
    Reserved1, // Reserved for future use
    Reserved2, // Reserved for future use
    Reserved3, // Reserved for future use
    Reserved4, // Reserved for future use
    Reserved5, // Reserved for future use
    Reserved6, // Reserved for future use
    Reserved7, // Reserved for future use
    Reserved8, // Reserved for future use

    Misc, // A raw monochrome bitmap, can't be resized. This must be the last of the preset paintbrush types.
    Point, // Used to reduce the paintbrush to a single pixel, during operations like floodfill.
    No, // Used to display no cursor at all (colorpicker)
    ColorBrush, // User's brush, in color mode
    MonoBrush, // User's brush, in mono mode
    Max, // Upper limit.
    */
}

impl Shape {
    pub fn gen(self, w: i32, h: i32) -> Vec<bool> {
        use self::Shape::*;
        match self {
            Round           => round(w, h).collect(),
            Square          => square(w, h).collect(),
            SieveRound      => sieve_round(w, h).collect(),
            SieveSquare     => sieve_square(w, h).collect(),
            Plus            => plus(w, h).collect(),
            Slash           => slash(w, h).collect(),
            Antislash       => antislash(w, h).collect(),
            HorizontalBar   => horizontal_bar(w, h).collect(),
            VerticalBar     => vertical_bar(w, h).collect(),
            Cross           => cross(w, h).collect(),
            Diamond         => diamond(w, h).collect(),
            //Random          => random(w, h),
            _ => unreachable!(),
        }
    }
}

/*
struct Mask<'a> {
    size: Point<i32>,
    data: &'a mut [bool],
}
*/

fn circle_squared_diameter(d: i32) -> i32 {
    let d2 = d * d;
    // Trick to make some circles rounder,
    // even though mathematically incorrect.
    match d {
        3 | 9 => d2 - 2,
        11 => d2 - 6,
        14 => d2 - 4,
        _ => d2,
    }
}

pub fn round(w: i32, h: i32) -> impl Iterator<Item=bool> {
    let r2 = circle_squared_diameter(w);
    SizeIter::new(w, h).map(move |(x, y)| {
        let x = 1 - w + x * 2;
        let y = 1 - h + y * 2;
        (x * x + y * y) <= r2
    })
}
pub fn square(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(_, _)| true)
}
pub fn sieve_round(w: i32, h: i32) -> impl Iterator<Item=bool> {
    let reminder = if w == 1 { 1 } else { 0 };
    let r2 = circle_squared_diameter(w);
    SizeIter::new(w, h).map(move |(xp, yp)| {
        let x = 1 - w + xp * 2;
        let y = 1 - h + yp * 2;
        ((xp + yp + reminder) & 1) != 0 && ((x * x + y * y) < r2)
    })
}
pub fn sieve_square(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| (xp + yp) & 1 == 0)
}
pub fn plus(w: i32, h: i32) -> impl Iterator<Item=bool> {
    let x = h / 2;
    SizeIter::new(w, h).map(move |(xp, yp)| xp == x || yp == x)
}
pub fn slash(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| xp == (w - (yp + 1)))
}
pub fn antislash(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| xp == yp)
}
pub fn horizontal_bar(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| yp == 0)
}
pub fn vertical_bar(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| xp == 0)
}
pub fn cross(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(xp, yp)| xp == yp || xp == (h - (yp + 1)))
}
pub fn diamond(w: i32, h: i32) -> impl Iterator<Item=bool> {
    let x = w / 2;
    SizeIter::new(w, h).map(move |(xp, yp)| {
        let a = if xp <= x { x - xp } else { xp - x };
        let b = if yp <= x { x - yp } else { yp - x };
        a + b <= x
    })
}

/*
fn random(w: i32, h: i32) {
    // init with blank
    for p in self.size.size_iter() {
        self.set(p.x, p.y, false);
    }
    let Point { x: w, y: h } = self.size;
    let r2 = circle_squared_diameter(w);
    for Point { x: xp, y: yp } in self.size.size_iter() {
        let x = 1 - w + xp * 2;
        let y = 1 - h + yp * 2;

        let rnd = rand::random::<u8>();

        // XXX if ((x*x) + (y*y) < r2 && !(rand() & 7)) {
        if x * x + y * y < r2 && rnd & 7 == 0 {
            self.set(xp, yp, true);
            // This prevents having a pixels that touch each other.
            if xp > 0 {
                self.set(xp - 1, yp, false);
            }
            if yp > 0 {
                self.set(xp, yp - 1, false);
            }
        }
    }
}
*/

struct SizeIter<T> where T: BaseIntExt {
    size: (T, T),
    pos: (T, T),
}

impl<T> SizeIter<T> where T: BaseIntExt {
    fn new(w: T, h: T) -> Self {
        Self {
            size: (w, h),
            pos: (T::zero(), T::zero()),
        }
    }
}

impl<T> Iterator for SizeIter<T> where T: BaseIntExt {
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.1 >= self.size.1 {
            return None;
        }
        let p = self.pos;
        self.pos.0 = self.pos.0 + T::one();
        if self.pos.0 >= self.size.0 {
            self.pos.0 = T::zero();
            self.pos.1 += T::one();
        }
        Some(p)
    }
}


#[test]
fn point_wh() {
    let v: Vec<_> = SizeIter::new(2, 3).collect();
    assert_eq!(v, &[
        (0, 0),
        (1, 0),
        (0, 1),
        (1, 1),
        (0, 2),
        (1, 2),
    ]);
}

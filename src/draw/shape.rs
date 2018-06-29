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

    Custom,
}

fn circle_squared_diameter<T>(d: T) -> T
    where T: BaseIntExt
{
    let d2 = d * d;
    // Trick to make some circles rounder,
    // even though mathematically incorrect.
    if d == T::from_i8(3).unwrap() || d == T::from_i8(9).unwrap() {
        d2 - T::from_i8(2).unwrap()
    } else if d == T::from_i8(11).unwrap() {
        d2 - T::from_i8(6).unwrap()
    } else if d == T::from_i8(14).unwrap() {
        d2 - T::from_i8(4).unwrap()
    } else {
        d2
    }
}

pub fn round<T>(w: T, h: T) -> impl Iterator<Item=bool>
    where T: BaseIntExt
{
    let two = T::from_i8(2).unwrap();
    let r2 = circle_squared_diameter(w);
    SizeIter::new(w, h).map(move |(x, y)| {
        let x = T::one() - w + x * two;
        let y = T::one() - h + y * two;
        (x * x + y * y) <= r2
    })
}
pub fn square<T>(w: T, h: T) -> impl Iterator<Item=bool>
    where T: BaseIntExt
{
    SizeIter::new(w, h).map(move |(_, _)| true)
}
pub fn sieve_round<T>(w: T, h: T) -> impl Iterator<Item=bool>
    where T: BaseIntExt
{
    let two = T::from_i8(2).unwrap();
    let reminder = if w == T::one() { T::one() } else { T::zero() };
    let r2 = circle_squared_diameter(w);
    SizeIter::new(w, h).map(move |(xp, yp)| {
        let x = T::one() - w + xp * two;
        let y = T::one() - h + yp * two;
        ((xp + yp + reminder) & T::one()) != T::zero() && ((x * x + y * y) < r2)
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
    SizeIter::new(w, h).map(move |(_, y)| y == 0)
}
pub fn vertical_bar(w: i32, h: i32) -> impl Iterator<Item=bool> {
    SizeIter::new(w, h).map(move |(x, _)| x == 0)
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

const MAX_SIZE: usize = 16;

pub struct Sieve {
    pub data: [[bool; MAX_SIZE]; MAX_SIZE],
    pub width: usize,
    pub height: usize,
}

impl Sieve {
    pub fn invert(&mut self) {
        for y in 0..MAX_SIZE {
            for x in 0..MAX_SIZE {
                self.data[x][y] = !self.data[x][y];
            }
        }
    }

    pub fn offset(&mut self, ox: isize, oy: isize) {
        let data = self.data;

        for y in 0..self.height {
            for x in 0..self.width {
                let ox = ((x as isize + ox) as usize) % self.width;
                let oy = ((y as isize + oy) as usize) % self.height;
                self.data[x][y] = data[ox][oy];
            }
        }
    }

    pub fn resize(&mut self, w: usize, h: usize, fill: bool) {
        let data = [[fill; MAX_SIZE]; MAX_SIZE];
    }

    pub fn filter(&self, x: usize, y: usize) -> bool {
        let x = x % self.width;
        let y = y % self.height;
        self.data[x][y]
    }
}

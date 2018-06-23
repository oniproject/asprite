use super::BaseIntExt;

pub struct SizeIter<T> where T: BaseIntExt {
    size: (T, T),
    pos: (T, T),
}

impl<T> SizeIter<T> where T: BaseIntExt {
    pub fn new(w: T, h: T) -> Self {
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

use math::Rect;

#[derive(Clone, Debug)]
pub struct Frame {
    pub page: Vec<u8>,
    pub transparent: Option<u8>,
    pub width: usize,
    pub height: usize,
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            page: vec![0; width * height],
            transparent: Some(0),
            width, height,
        }
    }
    pub fn copy_from(&mut self, other: &Frame) {
        self.width = other.width;
        self.height = other.height;
        self.transparent = other.transparent;
        self.page.resize(other.page.len(), 0);
        self.page.copy_from_slice(&other.page);
    }
}

/*
    pub fn view(&mut self, rect: Rect<isize>) -> Option<FrameView> {
        let w = self.width as isize;
        let h = self.height as isize;
        Rect::from_coords_and_size(0, 0, w, h)
            .intersect(rect)
            .map(move |rect| {
                let x = rect.min.x - w;
                let y = rect.min.y - h;
                let stride = w;
                let i = (y * w + x * 1) as usize;
                FrameView {
                    page: &mut self.page[i..],
                    stride: self.width,
                    rect,
                }
            })
    }
}

pub struct FrameView<'a> {
    page: &'a mut [u8],
    stride: usize,
    rect: Rect<isize>,
}

impl<'a> FrameView<'a> {
    #[inline(always)]
    pub fn pix_offset(&self, x: isize, y: isize) -> usize {
        let x = (x - self.rect.min.x) as usize;
        let y = (y - self.rect.min.y) as usize;
        y * self.stride + x * 1
    }
}

impl<'a> Canvas<u8, isize> for FrameView<'a> {
    #[inline(always)]
    unsafe fn set_pixel_unchecked(&mut self, x: isize, y: isize, color: u8) {
        let idx = self.pix_offset(x, y);
        *self.page.get_unchecked_mut(idx) = color;
    }

    #[inline(always)]
    unsafe fn get_pixel_unchecked(&self, x: isize, y: isize) -> u8 {
        let idx = self.pix_offset(x, y);
        *self.page.get_unchecked(idx)
    }

    #[inline(always)]
    fn bounds(&self) -> Rect<isize> {
        self.rect
    }
}
*/

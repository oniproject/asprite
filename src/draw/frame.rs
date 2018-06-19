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

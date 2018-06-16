#![allow(dead_code)]
use common::*;
use std::cell::Cell;

use std::path::Path;

use image::math::nq::NeuQuant as NQ;

pub struct Sprite {
    pub name: String,
    pub data: Vec<Layer>,
    pub palette: Palette<u32>,
    pub width: usize,
    pub height: usize,

    pub frame: Cell<usize>,
    pub layer: Cell<usize>,

    pub color: Cell<u8>,
}

impl Sprite {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            palette: Palette::new(0, None),
            width, height,
            frame: Cell::new(0),
            layer: Cell::new(0),
            color: Cell::new(1),
        }
    }

    pub fn is_lock(&self) -> bool {
        self.data[self.layer.get()].lock.get()
    }

    pub fn page(&self, layer: usize, frame: usize) -> &Frame {
        self.data[layer].get(frame)
    }
    pub fn page_mut(&mut self, layer: usize, frame: usize) -> &mut Frame {
        self.data[layer].get_mut(frame)
    }

    pub fn add_layer(&mut self, name: &str) {
        let mut layer = Layer::new(name);
        let page = Frame::new(self.width, self.height);
        layer.push(page);
        self.data.push(layer);
    }

    pub fn add_layer_page(&mut self, name: &str, page: Frame) {
        let mut layer = Layer::new(name);
        layer.push(page);
        self.data.push(layer);
    }
}

pub struct Layer {
    pub frames: Vec<Frame>,
    pub name: String,
    pub visible: Cell<bool>,
    pub lock: Cell<bool>,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            frames: Vec::new(),
            name: name.to_string(),
            visible: Cell::new(true),
            lock: Cell::new(false),
        }
    }

    pub fn get(&self, idx: usize) -> &Frame {
        &self.frames[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Frame {
        &mut self.frames[idx]
    }

    pub fn push(&mut self, page: Frame) {
        self.frames.push(page)
    }
    pub fn insert(&mut self, pos: usize, page: Frame) {
        self.frames.insert(pos, page)
    }
    pub fn remove(&mut self, pos: usize) -> Frame {
        self.frames.remove(pos)
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub page: Vec<u8>,
    pub transparent: Option<u8>,
    pub width: usize,
    pub height: usize,
}
impl Frame {
    pub fn new(width: usize, height: usize) -> Self {
        Frame {
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


fn get_pal(nq: &NQ) -> Vec<u32> {
    #[derive(Clone, Copy)]
    struct Quad<T> {
        r: T,
        g: T,
        b: T,
        a: T,
    }

    type Neuron = Quad<f64>;
    type Color = Quad<i32>;

    /// Neural network based color quantizer.
    pub struct NeuQuant {
        network: Vec<Neuron>,
        colormap: Vec<Color>,
        netindex: Vec<usize>,
        bias: Vec<f64>, // bias and freq arrays for learning
        freq: Vec<f64>,
        samplefac: i32,
        netsize: usize,
    }
    let nq: &NeuQuant = unsafe {
        ::std::mem::transmute(nq)
    };
    let pal: Vec<_> = nq.colormap.iter()
        .map(|c| {
            ((c.r as u32) << 24) |
            ((c.g as u32) << 16) |
            ((c.b as u32) << 8) |
            ((c.a as u32) << 0)
        })
        .collect();
    pal
}

pub fn load_sprite<P: AsRef<Path>>(filename: P) -> Option<Sprite> {
    use image::{load, ImageFormat};
    use image::imageops::{index_colors, dither};
    use image::math::nq::NeuQuant;
    use std::fs::File;
    use std::io::BufReader;

    let format = {
        let ext = filename.as_ref().extension()
            .and_then(|s| s.to_str())
            .map_or("".to_string(), |s| s.to_ascii_lowercase());
        match &*ext {
            "gif" => ImageFormat::GIF,
            "png" => ImageFormat::PNG,
            "jpeg" | "jpg" => ImageFormat::JPEG,
            _ => unimplemented!(),
        }
    };

    let name = filename
        .as_ref().file_name().unwrap()
        .to_str().unwrap()
        .to_string();

    let reader = File::open(filename).unwrap();
    let reader = BufReader::new(reader);

    let m = load(reader, format).unwrap();
    let mut m = m.to_rgba();

    let (w, h) = (m.width() as usize, m.height() as usize);

    let mut sprite = Sprite::new(&name, w, h);

    let data: Vec<u8> = m.pixels()
        .flat_map(|c| c.data.iter().map(|&u| u))
        .collect();
    let map = NeuQuant::new(10, 256, &data);

    dither(&mut m, &map);
    let m = index_colors(&m, &map);

    let mut page = Frame::new(w, h);
    for (i, p) in m.pixels().enumerate() {
        page.page[i] = p.data[0];
    }

    sprite.add_layer_page("load", page);

    let pal = get_pal(&map);
    for (i, &c) in pal.iter().enumerate() {
        if i < 256 {
            sprite.palette[i as u8] = c;
        }
    }

    Some(sprite)
}

pub fn open_file() -> Option<String> {
    use nfd::{self, Response};

    let result = nfd::dialog().filter("gif,png,jpg,jpeg").open().unwrap();

    let result = match result {
        Response::Okay(file) => Some(file),
        Response::OkayMultiple(files) => Some(files[0].clone()),
        Response::Cancel => None,
    };

        /*
    if let Some(filename) = result.clone() {


        // Open the file
        use std::fs::File;
        use gif;
        use gif::SetParameter;
        let file = File::open(filename).unwrap();

        let mut decoder = gif::Decoder::new(file);
        // Configure the decoder such that it will expand the image to RGBA.
        decoder.set(gif::ColorOutput::Indexed);
        // Read the file header
        let mut decoder = decoder.read_info().unwrap();
        println!("{}x{} has_pal: {} bg: {:?}",
            decoder.width(),
            decoder.height(),
            decoder.global_palette().is_some(),
            decoder.bg_color(),
        );
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            // Process every frame
            println!("frame[{}]: {}x{} {}x{}, transparent: {:?} dis: {:?}",
                frame.palette.is_some(),
                frame.top, frame.left, frame.width, frame.height, frame.transparent, frame.dispose);
        }
    }
        */

    result
}

use std::path::Path;

use image::math::nq::NeuQuant as NQ;

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

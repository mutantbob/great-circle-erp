pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub rgb_pixels: Vec<u8>,
}

impl RawImage {
    pub(crate) fn rgba_at(&self, x: usize, y: usize) -> [u8; 4] {
        let width = self.width as usize;
        let height = self.height as usize;
        if x < width && y < height {
            let base = 3 * (x + width * y);
            [
                self.rgb_pixels[base],
                self.rgb_pixels[base + 1],
                self.rgb_pixels[base + 2],
                0xff,
            ]
        } else {
            [0xff, 0, 0xff, 0xff]
        }
    }
}

impl RawImage {
    pub(crate) fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        if pixels.len() != (3 * width * height) as usize {
            panic!("not raw RGB?")
        }
        Self {
            width,
            height,
            rgb_pixels: pixels,
        }
    }
}

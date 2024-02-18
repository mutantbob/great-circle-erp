use crate::background_image::BackgroundImage;
use crate::raw_image::RawImage;
use crate::remapper::GreatCircleRemapper;
use eframe::emath::Vec2;
use egui::{
    Color32, ColorImage, Image, PointerButton, Pos2, Rect, Response, Sense, Shape, TextureHandle,
    Ui, Widget,
};
use std::io::Cursor;
use std::mem;
use std::sync::Arc;

pub struct WorldSampler {
    pub raw_image: RawImage,
}

impl WorldSampler {
    fn new() -> WorldSampler {
        let raw_image = Self::raw_world_map();
        Self { raw_image }
    }

    pub fn raw_world_map() -> RawImage {
        let raw = include_bytes!("world.png");
        let decoder = png::Decoder::new(Cursor::new(raw));

        let mut reader = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size()];
        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut buf).unwrap();
        // Grab the bytes of the image.
        buf.truncate(info.buffer_size());

        RawImage::new(info.width, info.height, buf)
    }

    pub(crate) fn get(&self, longitude_frac: f32, latitude_frac: f32) -> [u8; 4] {
        let x = (longitude_frac * self.raw_image.width as f32) as usize;
        let y = (latitude_frac * self.raw_image.height as f32) as usize;
        self.raw_image.rgba_at(x, y)
    }
}

//

pub struct WorldMap {
    texture: WorldMapCalculating,
    width: usize,
    height: usize,

    anchors: Vec<Vec2>,
    last_hover: Option<(f32, f32)>,
    world_sampler: Arc<WorldSampler>,
    remapper: Arc<GreatCircleRemapper>,
}

impl Default for WorldMap {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            world_sampler: Arc::new(WorldSampler::new()),
            texture: WorldMapCalculating::Nothing,
            width: 512,
            height: 512,
            anchors: vec![],
            last_hover: None,
            remapper: Arc::new(GreatCircleRemapper::new(&[])),
        }
    }

    pub(crate) fn set_anchor(&mut self, longitude_frac: f32, latitude_frac: f32) {
        self.anchors.push(Vec2::new(longitude_frac, latitude_frac));
        while self.anchors.len() > 2 {
            self.anchors.remove(0);
        }
        self.remapper = Arc::new(GreatCircleRemapper::new(&self.anchors));
    }

    fn get_texture(&mut self, ui: &mut Ui) -> Option<TextureHandle> {
        self.texture.maybe_get_texture(
            ui,
            self.width,
            self.height,
            self.world_sampler.clone(),
            self.remapper.clone(),
        )
    }

    pub fn calculate_replacement_image(&mut self, ui: &mut Ui) {
        let image_pipe = BackgroundImage::new(
            ui,
            self.width,
            self.height,
            self.world_sampler.clone(),
            self.remapper.clone(),
        );
        self.texture.start_recalculating(image_pipe);
    }
}

impl Widget for &mut WorldMap {
    fn ui(self, ui: &mut Ui) -> Response {
        self.width = ui.available_width() as _;
        self.height = ui.available_height() as _;
        let response = ui.allocate_response(
            Vec2::new(self.width as f32, self.height as f32),
            Sense::click(),
        );

        /*   println!(
            "widthxheight {}x{}",
            response.rect.width(),
            response.rect.height()
        );*/

        if let Some(pos) = response.hover_pos() {
            // println!("hover {:?}", pos);
            let Vec2 { x, y } = pos - response.rect.left_top();
            self.last_hover = Some((x / response.rect.width(), y / response.rect.height()));
        }

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let Vec2 { x, y } = pos - response.rect.left_top();
                println!("click {},{}", x, y);

                let u = x / response.rect.width();
                let v = y / response.rect.height();
                let Vec2 { x: u, y: v } = self.remapper.untwist(u, v);

                self.set_anchor(u, v);

                self.calculate_replacement_image(ui);
            }
        }

        let rect = &response.rect;

        // println!("enabled? {}", ui.is_enabled());

        #[allow(clippy::single_match)]
        match self.get_texture(ui) {
            Some(texture) => {
                let texture_size = texture.size_vec2();
                match 2 {
                    1 => {
                        let rect = Rect::from_min_max(rect.min, rect.min + texture_size);
                        let img = Image::new(&texture);
                        img.paint_at(ui, rect);
                    }
                    _ => {
                        let rect = Rect::from_min_max(rect.min, rect.min + texture_size);
                        let full_square =
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
                        ui.painter().add(Shape::image(
                            texture.id(),
                            rect,
                            full_square,
                            Color32::from_rgb(0xff, 0xff, 0xff),
                        ));
                    }
                }
            }
            None => {}
        }

        for anchor in &self.anchors {
            let Vec2 { x: u, y: v } = self.remapper.twist(anchor.x, anchor.y);

            let xy = Vec2::new(u * response.rect.width(), v * response.rect.height());
            let circle = Shape::circle_filled(rect.min + xy, 3.0, Color32::from_rgb(0xff, 0, 0));
            ui.painter().add(circle);
        }

        if false {
            let clicked: Vec<_> = [
                PointerButton::Primary,
                PointerButton::Secondary,
                PointerButton::Middle,
            ]
            .into_iter()
            .map(|button| response.clicked_by(button))
            .collect();
            println!("response {:?}", clicked);
            if response.clicked() {
                println!("clicked {:?}", response.interact_pointer_pos());
            }
        }

        response
    }
}

//

pub enum ImageProgress {
    None,
    Working(TextureHandle),
    Finished(TextureHandle),
}

//

enum WorldMapCalculating {
    Nothing,
    CalculatingNoTexture(BackgroundImage),
    Recalculating(BackgroundImage, TextureHandle),
    Stable(TextureHandle),
}

impl WorldMapCalculating {
    pub fn maybe_get_texture(
        &mut self,
        ui: &mut Ui,
        width: usize,
        height: usize,
        world_sampler: Arc<WorldSampler>,
        remapper: Arc<GreatCircleRemapper>,
    ) -> Option<TextureHandle> {
        let (new_val, rval) = mem::replace(self, Self::Nothing).inner_get_texture(
            ui,
            width,
            height,
            world_sampler,
            remapper,
        );
        *self = new_val;
        rval
    }

    fn inner_get_texture(
        self,
        ui: &mut Ui,
        width: usize,
        height: usize,
        world_sampler: Arc<WorldSampler>,
        remapper: Arc<GreatCircleRemapper>,
    ) -> (WorldMapCalculating, Option<TextureHandle>) {
        match self {
            WorldMapCalculating::Nothing => {
                let image_pipe = BackgroundImage::new(ui, width, height, world_sampler, remapper);
                (WorldMapCalculating::CalculatingNoTexture(image_pipe), None)
            }
            WorldMapCalculating::CalculatingNoTexture(mut image_pipe) => match image_pipe.get(ui) {
                ImageProgress::None => (Self::CalculatingNoTexture(image_pipe), None),
                ImageProgress::Working(tex) => {
                    (Self::Recalculating(image_pipe, tex.clone()), Some(tex))
                }
                ImageProgress::Finished(tex) => (Self::Stable(tex.clone()), Some(tex)),
            },
            WorldMapCalculating::Recalculating(mut image_pipe, tex) => match image_pipe.get(ui) {
                ImageProgress::None => (Self::Recalculating(image_pipe, tex.clone()), Some(tex)),
                ImageProgress::Working(tex) => {
                    (Self::Recalculating(image_pipe, tex.clone()), Some(tex))
                }
                ImageProgress::Finished(tex) => (Self::Stable(tex.clone()), Some(tex)),
            },
            WorldMapCalculating::Stable(tex) => (Self::Stable(tex.clone()), Some(tex)),
        }
    }

    pub fn start_recalculating(&mut self, image_pipe: BackgroundImage) {
        match self {
            WorldMapCalculating::Nothing => *self = Self::CalculatingNoTexture(image_pipe),
            WorldMapCalculating::CalculatingNoTexture(old_image) => {
                old_image.cancel();
                *self = Self::CalculatingNoTexture(image_pipe);
            }
            WorldMapCalculating::Recalculating(old_image, tex) => {
                old_image.cancel();
                *self = Self::Recalculating(image_pipe, (*tex).clone())
            }
            WorldMapCalculating::Stable(tex) => {
                *self = Self::Recalculating(image_pipe, (*tex).clone());
            }
        }
    }
}

//

pub fn world_map(
    width: usize,
    height: usize,
    world_sampler: &WorldSampler,
    remapper: &GreatCircleRemapper,
) -> ColorImage {
    println!("calculating new world map image");
    let solutions: Vec<[u8; 4]> = crate::rect_map(width, height, move |col, row| {
        let u0 = col as f32 / width as f32;
        let v0 = row as f32 / height as f32;

        let Vec2 { x: u, y: v } = remapper.untwist(u0, v0);

        if col == 0 && row == 50 {
            println!("{},{} -> {},{}", u0, v0, u, v)
        }

        world_sampler.get(u, v)
    })
    .collect();

    // debug_analyze(solutions.as_slice());

    let rgba: Vec<u8> = solutions.into_iter().flatten().collect();
    println!("world map image complete");
    ColorImage::from_rgba_unmultiplied([width, height], rgba.as_slice())
}

use crate::remapper::GreatCircleRemapper;
use crate::world_map::{world_map, ImageProgress, WorldSampler};
use egui::{ColorImage, Context, TextureHandle, TextureOptions, Ui};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct BackgroundImage {
    image_pipe: Arc<Mutex<ImagePipe>>,
    texture_handle: Option<TextureHandle>,
}

impl BackgroundImage {
    pub fn new(
        ui: &Ui,
        width: usize,
        height: usize,
        world_sampler: Arc<WorldSampler>,
        remapper: Arc<GreatCircleRemapper>,
    ) -> Self {
        let image_pipe = Arc::new(Mutex::new(ImagePipe::new()));
        let ctx = (*ui.ctx()).clone();
        let image_pipe2 = image_pipe.clone();
        thread::spawn(move || {
            calculate_image(ctx, image_pipe2, width, height, &world_sampler, &remapper);
        });

        Self {
            image_pipe,
            texture_handle: None,
        }
    }

    pub fn get(&mut self, ui: &mut Ui) -> ImageProgress {
        let mut ipo = self.image_pipe.lock().unwrap();
        match ipo.maybe_get() {
            None => match self.texture_handle.as_ref() {
                None => ImageProgress::None,
                Some(tex) => ImageProgress::Working((*tex).clone()),
            },
            Some(image) => {
                let tex = match self.texture_handle.as_mut() {
                    None => {
                        let tex = ui
                            .ctx()
                            .load_texture("my image", image, TextureOptions::LINEAR);
                        self.texture_handle = Some(tex.clone());
                        tex
                    }
                    Some(tex) => {
                        tex.set(image, TextureOptions::LINEAR);
                        (*tex).clone()
                    }
                };
                ImageProgress::Finished(tex)
            }
        }
    }

    pub fn cancel(&mut self) {
        self.image_pipe.lock().unwrap().cancelled = true;
    }
}

struct ImagePipe {
    read_cursor: usize,
    write_cursor: usize,
    cancelled: bool,
    img: Option<ColorImage>,
}

impl ImagePipe {
    pub fn new() -> Self {
        Self {
            read_cursor: 0,
            write_cursor: 0,
            cancelled: false,
            img: None,
        }
    }

    pub fn accept(&mut self, img: ColorImage) {
        self.img = Some(img);
        self.write_cursor = 1;
    }

    pub fn maybe_get(&mut self) -> Option<ColorImage> {
        if self.write_cursor > self.read_cursor {
            let rval = self.img.take();
            self.read_cursor = 0;
            rval
        } else {
            None
        }
    }
}

impl Default for ImagePipe {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_image(
    ctx: Context,
    sink: Arc<Mutex<ImagePipe>>,
    width: usize,
    height: usize,
    world_sampler: &WorldSampler,
    remapper: &GreatCircleRemapper,
) {
    thread::sleep(Duration::from_secs(3));

    let img = world_map(width, height, world_sampler, remapper);
    sink.lock().unwrap().accept(img);
    println!("repaint?");
    ctx.request_repaint()
}

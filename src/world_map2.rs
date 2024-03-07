use crate::remapper::{transform_ll_to_ll, GreatCircleRemapper};
use crate::world2::WorldGLSL;
use cgmath::{Matrix3, SquareMatrix};
use eframe::emath::Vec2;
use eframe::glow::Context;
use egui::{Color32, PaintCallback, PointerButton, Response, Sense, Shape, Ui, Widget};
use std::sync::Arc;

//

pub struct WorldMap2 {
    width: usize,
    height: usize,

    anchors: Vec<Vec2>,
    last_hover: Option<(f32, f32)>,
    world2: Arc<WorldGLSL<Context>>,
    matrix: Matrix3<f32>,
    matrix_inverse: Matrix3<f32>,
}

impl WorldMap2 {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.as_ref().unwrap();
        let world2 = Arc::new(WorldGLSL::new(gl));
        let matrix = Matrix3::identity();
        Self {
            width: 512,
            height: 512,
            anchors: vec![],
            last_hover: None,
            world2,
            matrix,
            matrix_inverse: matrix,
        }
    }

    pub(crate) fn set_anchor(&mut self, longitude_frac: f32, latitude_frac: f32) {
        self.anchors.push(Vec2::new(longitude_frac, latitude_frac));
        while self.anchors.len() > 2 {
            self.anchors.remove(0);
        }
        self.set_matrix(GreatCircleRemapper::matrix_from_anchors(&self.anchors));
    }

    pub fn set_matrix(&mut self, matrix: Matrix3<f32>) {
        self.matrix = matrix;
        self.matrix_inverse = matrix.invert().unwrap();
    }
}

impl Widget for &mut WorldMap2 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.width = ui.available_width() as _;
        self.height = ui.available_height() as _;
        let response = ui.allocate_response(
            Vec2::new(self.width as f32, self.height as f32),
            Sense::click(),
        );

        /*println!(
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
                let Vec2 { x: u, y: v } = transform_ll_to_ll(u, v, &self.matrix);

                self.set_anchor(u, v);

                // self.calculate_replacement_image(ui);
            }
        }

        let rect = &response.rect;

        // println!("enabled? {}", ui.is_enabled());

        let world2 = self.world2.clone();
        let slice: &[[f32; 3]; 3] = self.matrix.as_ref();
        let matrix: Vec<f32> = slice.iter().flat_map(|x| x.iter().copied()).collect();
        let cb = eframe::egui_glow::CallbackFn::new(move |_info, painter| {
            world2.paint(painter.gl(), &matrix)
        });
        //println!("painting for {:?}", rect);
        let callback = PaintCallback {
            rect: *rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(Shape::Callback(callback));

        for anchor in &self.anchors {
            let Vec2 { x: u, y: v } = transform_ll_to_ll(anchor.x, anchor.y, &self.matrix_inverse);

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

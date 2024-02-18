use crate::world_map::WorldMap;

pub struct App {
    world_map: WorldMap,
}

impl Default for App {
    fn default() -> Self {
        Self {
            world_map: WorldMap::new(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        /*
                // Load previous app state (if any).
                // Note that you must enable the `persistence` feature for this to work.
                if let Some(storage) = cc.storage {
                    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
                }
        */
        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| ui.add(&mut self.world_map));
    }
}

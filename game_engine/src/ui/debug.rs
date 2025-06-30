use egui::Window;

use super::Drawable;

pub struct DebugWindow {
    pub lines: Vec<String>,
}

impl Drawable for DebugWindow {
    fn show_window(&self, ctx: &egui::Context) {
        Window::new("Debug").default_open(false).show(ctx, |ui| {
            self.lines.iter().for_each(|t| {
                ui.label(t);
            });
        });
    }

    fn show_widget(&self, _ui: &mut egui::Ui) {}
}

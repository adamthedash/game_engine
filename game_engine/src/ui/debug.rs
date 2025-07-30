use std::sync::{LazyLock, Mutex};

use egui::Window;

use super::Drawable;

pub struct DebugWindow {
    lines: Mutex<Vec<String>>,
}

impl DebugWindow {
    pub fn add_line(&self, line: &str) {
        self.lines.lock().unwrap().push(line.to_string());
    }

    pub fn clear(&self) {
        self.lines.lock().unwrap().clear();
    }
}

pub static DEBUG_WINDOW: LazyLock<DebugWindow> = LazyLock::new(|| DebugWindow {
    lines: Default::default(),
});

impl Drawable for DebugWindow {
    fn show_window(&self, ctx: &egui::Context) {
        Window::new("Debug").default_open(false).show(ctx, |ui| {
            self.lines.lock().unwrap().iter().for_each(|t| {
                ui.label(t);
            });
        });
    }

    fn show_widget(&self, _ui: &mut egui::Ui) {}
}

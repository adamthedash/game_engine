use eframe::egui;
use egui::{ColorImage, TextureHandle, TextureOptions};
use game_engine::world_gen::Perlin;

struct PerlinViewer {
    generator: Perlin,
    texture: Option<TextureHandle>,
    map_size: usize,
    scale: f64,
    num_octaves: usize,
    amplitude: f64,
    persistence: f64,
    perlin_scale: f64,
    seed: u64,
    offset_x: f64,
    offset_y: f64,
    needs_update: bool,
    // Terrain thresholds
    water_threshold: f32,
    snow_threshold: f32,
}

impl Default for PerlinViewer {
    fn default() -> Self {
        let seed = 42;
        let num_octaves = 3;
        let amplitude = 1.0;
        let persistence = 0.5;
        let perlin_scale = 1.0;
        let generator = Perlin::new(seed, num_octaves, amplitude, persistence, perlin_scale);

        Self {
            generator,
            texture: None,
            map_size: 256,
            scale: 0.05,
            num_octaves,
            amplitude,
            persistence,
            perlin_scale,
            seed,
            offset_x: 0.0,
            offset_y: 0.0,
            needs_update: true,
            water_threshold: 0.0,
            snow_threshold: 0.5,
        }
    }
}

impl PerlinViewer {
    fn generate_noise_map(&self) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(self.map_size * self.map_size * 3);

        for y in 0..self.map_size {
            for x in 0..self.map_size {
                let world_x = (x as f64 * self.scale) + self.offset_x;
                let world_y = (y as f64 * self.scale) + self.offset_y;

                // Sample the noise (returns value in range [-1, 1])
                let noise_value = self.generator.sample(world_x, world_y, 0.0) as f32;

                // Determine terrain type based on thresholds
                let (r, g, b) = if noise_value < self.water_threshold {
                    // Water - blue
                    (30, 100, 200)
                } else if noise_value < self.snow_threshold {
                    // Rock/land - brown/green gradient
                    let land_factor = (noise_value - self.water_threshold)
                        / (self.snow_threshold - self.water_threshold);
                    let brown_r = (101.0 + land_factor * 20.0) as u8;
                    let brown_g = (67.0 + land_factor * 80.0) as u8;
                    let brown_b = (33.0 + land_factor * 20.0) as u8;
                    (brown_r, brown_g, brown_b)
                } else {
                    // Snow - white
                    (240, 240, 255)
                };

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
            }
        }

        pixels
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let pixels = self.generate_noise_map();
        let color_image = ColorImage::from_rgb([self.map_size, self.map_size], &pixels);

        if let Some(texture) = &mut self.texture {
            texture.set(color_image, TextureOptions::NEAREST);
        } else {
            self.texture =
                Some(ctx.load_texture("perlin_noise", color_image, TextureOptions::NEAREST));
        }

        self.needs_update = false;
    }

    fn update_generator(&mut self) {
        self.generator = Perlin::new(
            self.seed,
            self.num_octaves,
            self.amplitude,
            self.persistence,
            self.perlin_scale,
        );
        self.needs_update = true;
    }
}

impl eframe::App for PerlinViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.needs_update {
            self.update_texture(ctx);
        }

        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Perlin Noise Controls");
            ui.separator();
            ui.label("Terrain Thresholds:");

            ui.horizontal(|ui| {
                ui.label("Water (<):");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.water_threshold)
                            .speed(0.01)
                            .clamp_range(-1.0..=1.0),
                    )
                    .changed()
                {
                    // Ensure water threshold is always less than snow threshold
                    if self.water_threshold >= self.snow_threshold {
                        self.water_threshold = self.snow_threshold - 0.01;
                    }
                    self.needs_update = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Snow (>=):");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.snow_threshold)
                            .speed(0.01)
                            .clamp_range(-1.0..=1.0),
                    )
                    .changed()
                {
                    // Ensure snow threshold is always greater than water threshold
                    if self.snow_threshold <= self.water_threshold {
                        self.snow_threshold = self.water_threshold + 0.01;
                    }
                    self.needs_update = true;
                }
            });

            ui.label(format!(
                "Rock: {:.2} to {:.2}",
                self.water_threshold, self.snow_threshold
            ));

            // Color legend
            ui.separator();
            ui.label("Legend:");
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(30, 100, 200), "■ Water");
                ui.colored_label(egui::Color32::from_rgb(101, 67, 33), "■ Rock/Land");
                ui.colored_label(egui::Color32::from_rgb(240, 240, 255), "■ Snow");
            });

            ui.separator();

            ui.label("Noise Parameters:");

            let mut seed_changed = false;
            ui.horizontal(|ui| {
                ui.label("Seed:");
                seed_changed = ui.add(egui::DragValue::new(&mut self.seed)).changed();
            });

            let mut octaves_changed = false;
            ui.horizontal(|ui| {
                ui.label("Octaves:");
                octaves_changed = ui
                    .add(egui::DragValue::new(&mut self.num_octaves).clamp_range(1..=8))
                    .changed();
            });

            let mut amplitude_changed = false;
            ui.horizontal(|ui| {
                ui.label("Amplitude:");
                amplitude_changed = ui
                    .add(
                        egui::DragValue::new(&mut self.amplitude)
                            .speed(0.1)
                            .clamp_range(0.1..=5.0),
                    )
                    .changed();
            });

            let mut persistence_changed = false;
            ui.horizontal(|ui| {
                ui.label("Persistence:");
                persistence_changed = ui
                    .add(
                        egui::DragValue::new(&mut self.persistence)
                            .speed(0.01)
                            .clamp_range(0.0..=1.0),
                    )
                    .changed();
            });

            let mut perlin_scale_changed = false;
            ui.horizontal(|ui| {
                ui.label("Perlin Scale:");
                perlin_scale_changed = ui
                    .add(
                        egui::DragValue::new(&mut self.perlin_scale)
                            .speed(0.1)
                            .clamp_range(0.1..=10.0),
                    )
                    .changed();
            });

            if seed_changed
                || octaves_changed
                || amplitude_changed
                || persistence_changed
                || perlin_scale_changed
            {
                self.update_generator();
            }

            ui.separator();
            ui.label("View Controls:");

            ui.horizontal(|ui| {
                ui.label("Scale:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.scale)
                            .speed(0.001)
                            .clamp_range(0.001..=1.0),
                    )
                    .changed()
                {
                    self.needs_update = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Map Size:");
                if ui
                    .add(egui::DragValue::new(&mut self.map_size).clamp_range(64..=512))
                    .changed()
                {
                    self.needs_update = true;
                }
            });

            ui.separator();
            ui.label("Navigation:");

            let offset_speed = self.scale * 10.0;
            ui.horizontal(|ui| {
                ui.label("Offset X:");
                if ui
                    .add(egui::DragValue::new(&mut self.offset_x).speed(offset_speed))
                    .changed()
                {
                    self.needs_update = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Offset Y:");
                if ui
                    .add(egui::DragValue::new(&mut self.offset_y).speed(offset_speed))
                    .changed()
                {
                    self.needs_update = true;
                }
            });

            if ui.button("Reset View").clicked() {
                self.offset_x = 0.0;
                self.offset_y = 0.0;
                self.needs_update = true;
            }

            ui.separator();
            ui.label("Presets:");

            if ui.button("Terrain").clicked() {
                self.seed = 42;
                self.num_octaves = 6;
                self.amplitude = 1.0;
                self.persistence = 0.5;
                self.perlin_scale = 2.0;
                self.scale = 0.02;
                self.update_generator();
            }

            if ui.button("Clouds").clicked() {
                self.seed = 123;
                self.num_octaves = 4;
                self.amplitude = 0.8;
                self.persistence = 0.7;
                self.perlin_scale = 2.5;
                self.scale = 0.03;
                self.update_generator();
            }

            if ui.button("Island").clicked() {
                self.seed = 789;
                self.num_octaves = 5;
                self.amplitude = 1.2;
                self.persistence = 0.6;
                self.perlin_scale = 2.2;
                self.scale = 0.015;
                self.water_threshold = -0.2;
                self.snow_threshold = 0.6;
                self.update_generator();
            }

            if ui.button("Archipelago").clicked() {
                self.seed = 321;
                self.num_octaves = 4;
                self.amplitude = 0.9;
                self.persistence = 0.4;
                self.perlin_scale = 2.8;
                self.scale = 0.025;
                self.water_threshold = 0.1;
                self.snow_threshold = 0.7;
                self.update_generator();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Perlin Noise 2D Map");

            if let Some(texture) = &self.texture {
                let available_size = ui.available_size();
                let image_size = available_size.min_elem().min(600.0);

                ui.add(
                    egui::Image::from_texture(texture)
                        .fit_to_exact_size(egui::Vec2::splat(image_size))
                        .rounding(egui::Rounding::same(4)),
                );

                ui.separator();
                ui.label(format!("Map size: {}x{}", self.map_size, self.map_size));
                ui.label(format!("Scale: {:.4}", self.scale));
                ui.label(format!(
                    "Offset: ({:.2}, {:.2})",
                    self.offset_x, self.offset_y
                ));
                ui.label(format!("Water threshold: < {:.2}", self.water_threshold));
                ui.label(format!(
                    "Rock range: {:.2} to {:.2}",
                    self.water_threshold, self.snow_threshold
                ));
                ui.label(format!("Snow threshold: >= {:.2}", self.snow_threshold));
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Perlin Noise 2D Map Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "Perlin Noise Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(PerlinViewer::default()))),
    )
}

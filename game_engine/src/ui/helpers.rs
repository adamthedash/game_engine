use egui::{
    Align, Color32, FontId, Frame, Label, Layout, Rect, Response, Sense, Stroke, TextFormat, Ui,
    Vec2, text,
};
use egui_taffy::{
    TuiBuilderLogic,
    taffy::{self, AlignItems, prelude::percent},
    tui,
};
use enum_map::EnumMap;

use crate::{
    data::{item::ItemType, loader::ITEMS, recipe::Recipe},
    ui::Icon,
};

/// Draw a crafting recipe
pub fn draw_recipe(ui: &mut Ui, recipe: &Recipe, icon_size: f32, font_size: f32) -> Response {
    let items = ITEMS.get().expect("Items not initialised!");

    let resp =
        ui.allocate_ui_with_layout(Vec2::splat(1.), Layout::left_to_right(Align::Max), |ui| {
            Frame::default()
                .stroke(Stroke::new(1., Color32::DARK_GRAY))
                .show(ui, |ui| {
                    // Inputs on the left
                    recipe.inputs.iter().for_each(|(item, count)| {
                        Icon {
                            texture: &items[*item].texture,
                            size: icon_size,
                            count: Some(*count),
                            font_size,
                        }
                        .draw(ui);
                    });

                    // Space between input & output
                    let mut arrow = text::LayoutJob::default();
                    arrow.append(
                        ">",
                        0.,
                        TextFormat {
                            font_id: FontId::new(icon_size / 2., egui::FontFamily::Monospace),
                            ..Default::default()
                        },
                    );
                    ui.add_sized(Vec2::splat(icon_size), Label::new(arrow).selectable(false));

                    // Outputs on the right
                    let (item, count) = recipe.output;
                    Icon {
                        texture: &items[item].texture,
                        size: icon_size,
                        count: Some(count),
                        font_size,
                    }
                    .draw(ui);
                });
        });

    // Make frame clickable
    resp.response.interact(Sense::click())
}

/// Draw a grid of items
pub fn draw_item_grid(
    ui: &mut Ui,
    tui_id: &str,
    item_counts: &EnumMap<ItemType, usize>,
    icon_size: f32,
) -> EnumMap<ItemType, Option<Response>> {
    let items = ITEMS.get().expect("Items info not initialised!");

    // Use egui_taffy to create a grid layout
    let mut responses = EnumMap::default();
    tui(ui, ui.id().with(tui_id))
        .reserve_available_width()
        .style(taffy::Style {
            flex_direction: taffy::FlexDirection::Row,
            flex_wrap: taffy::FlexWrap::Wrap,
            align_items: Some(AlignItems::Start),
            size: taffy::Size {
                width: percent(1.),
                height: percent(1.),
            },
            ..Default::default()
        })
        .show(|ui| {
            ui.reuse_style().add(|ui| {
                // Draw each item icon if we have some
                item_counts
                    .iter()
                    .filter(|(_, count)| **count > 0)
                    .for_each(|(id, count)| {
                        // Create and draw the icon
                        let icon = Icon {
                            texture: &items[id].texture,
                            size: icon_size,
                            count: Some(*count),
                            font_size: icon_size / 2.,
                        };
                        let resp = ui.ui_add(icon);
                        responses[id] = Some(resp);
                    })
            })
        });

    responses
}

pub fn draw_progress_bar(ui: &mut Ui, width: f32, height: f32, progress: f32) {
    assert!(
        (0_f32..=1.).contains(&progress),
        "Progress must be between 0 and 1"
    );

    ui.allocate_ui(Vec2::new(width, height), |ui| {
        // Make sure the full bar size is taken up
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::empty());

        let painter = ui.painter();

        // Foreground
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(width * progress, height)),
            0.,
            Color32::WHITE,
        );

        // Background
        painter.rect_filled(
            Rect::from_min_size(
                rect.min + Vec2::new(width * progress, 0.),
                Vec2::new(width * (1. - progress), height),
            ),
            0.,
            Color32::BLACK,
        );
    });
}

use egui::Ui;

use crate::tools_data::{bbox_data::BboxSpecifics, ToolSpecifics, ToolsData};

pub fn bbox_menu(ui: &mut Ui, mut window_open: bool, mut data: BboxSpecifics) -> ToolsData {
    if ui.text_edit_singleline(&mut data.new_label).lost_focus() {
        data.push(data.new_label.clone(), None);
    }
    let mut to_be_removed = None;
    let mut new_idx = data.cat_id_current;
    for (label_idx, label) in data.labels().iter().enumerate() {
        let checked = label_idx == data.cat_id_current;
        ui.horizontal_top(|ui| {
            if ui.button("x").clicked() {
                to_be_removed = Some(label_idx);
            }
            if ui.selectable_label(checked, label).clicked() {
                new_idx = label_idx;
            }
        });
    }
    data.cat_id_current = new_idx;
    if let Some(idx) = to_be_removed {
        data.remove_cat(idx);
    }
    if ui.button("Close").clicked() {
        window_open = false;
    }
    ToolsData {
        specifics: ToolSpecifics::Bbox(data),
        menu_active: window_open,
    }
}

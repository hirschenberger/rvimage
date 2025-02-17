use crate::{
    cfg::{self, Cfg},
    control::{Control, Info, SortType},
    file_util::{self, RVPRJ_PREFIX},
    menu::{self, cfg_menu::CfgMenu, open_folder, picklist},
    paths_selector::PathsSelector,
    result::{to_rv, RvResult},
    tools::{ToolState, BBOX_NAME},
    tools_data::ToolSpecifics,
    world::ToolsDataMap,
};
use egui::{Area, Context, Frame, Id, Order, Response, Ui, Widget};
use std::mem;

use super::tools_menus::bbox_menu;

fn show_popup(
    ui: &mut Ui,
    msg: &str,
    icon: &str,
    popup_id: Id,
    info_message: Info,
    response: &Response,
) -> Info {
    ui.memory_mut(|m| m.open_popup(popup_id));
    let mut new_msg = Info::None;
    egui::popup_above_or_below_widget(ui, popup_id, response, egui::AboveOrBelow::Above, |ui| {
        let max_msg_len = 500;
        let shortened_msg = if msg.len() > max_msg_len {
            &msg[..max_msg_len]
        } else {
            msg
        };
        ui.label(format!("{icon} {shortened_msg}"));
        new_msg = if ui.button("close").clicked() {
            Info::None
        } else {
            info_message
        }
    });
    new_msg
}

pub(super) fn get_cfg() -> (Cfg, Info) {
    match cfg::get_cfg() {
        Ok(cfg) => (cfg, Info::None),
        Err(e) => (cfg::get_default_cfg(), Info::Error(format!("{e:?}"))),
    }
}
// evaluates an expression that is expected to return Result,
// passes unpacked value to effect function in case of Ok,
// sets according error message in case of Err
macro_rules! handle_error {
    ($effect:expr, $result:expr, $self:expr) => {
        match $result {
            Ok(r) => {
                #[allow(clippy::redundant_closure_call)]
                $effect(r);
            }
            Err(e) => {
                $self.info_message = Info::Error(e.to_string());
            }
        }
    };
    ($result:expr, $self:expr) => {
        handle_error!(|_| {}, $result, $self);
    };
}

pub struct ToolSelectMenu {
    window_open: bool,      // Only show the egui window when true.
    are_tools_active: bool, // can deactivate all tools, overrides activated_tool
    recently_activated_tool: Option<usize>,
}
impl ToolSelectMenu {
    fn new() -> Self {
        Self {
            window_open: true,
            are_tools_active: true,
            recently_activated_tool: None,
        }
    }
    pub fn recently_activated_tool(&self) -> Option<usize> {
        self.recently_activated_tool
    }
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        tools: &mut [ToolState],
        tools_menu_map: &mut ToolsDataMap,
    ) -> RvResult<()> {
        ui.horizontal_top(|ui| {
            self.recently_activated_tool = tools
                .iter_mut()
                .enumerate()
                .filter(|(_, t)| !t.is_always_active())
                .find(|(_, t)| ui.selectable_label(t.is_active(), t.button_label).clicked())
                .map(|(i, _)| i);
        });
        for v in tools_menu_map.values_mut().filter(|v| v.menu_active) {
            let tmp = match &mut v.specifics {
                ToolSpecifics::Bbox(x) => bbox_menu(ui, v.menu_active, mem::take(x)),
                ToolSpecifics::Brush(_) => Ok(mem::take(v)),
            };
            *v = tmp?;
        }
        Ok(())
    }

    pub fn toggle(&mut self) {
        if self.window_open {
            self.window_open = false;
            self.are_tools_active = true;
        } else {
            self.window_open = true;
        }
    }
}
impl Default for ToolSelectMenu {
    fn default() -> Self {
        Self::new()
    }
}

struct ImportBtnResp {
    pub resp: Option<Response>,
    pub popup_open: bool,
}

#[derive(Default)]
struct Stats {
    n_files_filtered_info: Option<String>,
    n_files_annotated_info: Option<String>,
}

struct About<'a> {
    id: Id,
    show_about: &'a mut bool,
}
impl<'a> About<'a> {
    pub fn new(id: Id, show_about: &'a mut bool) -> Self {
        Self { id, show_about }
    }
}
impl<'a> Widget for About<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let about_btn = ui.button("about");
        if about_btn.clicked() {
            *self.show_about = true;
        }
        if *self.show_about {
            ui.memory_mut(|m| m.open_popup(self.id));
            if ui.memory(|m| m.is_popup_open(self.id)) {
                let area = Area::new(self.id)
                    .order(Order::Foreground)
                    .default_pos(about_btn.rect.left_bottom());
                area.show(ui.ctx(), |ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        const VERSION: &str = env!("CARGO_PKG_VERSION");
                        const CODE: &str = env!("CARGO_PKG_REPOSITORY");
                        ui.label("RV Image\n");
                        ui.label(format!("version: {VERSION}"));
                        ui.hyperlink_to("license and code", CODE);
                        let resp_close = ui.button("close");
                        if resp_close.clicked() {
                            ui.memory_mut(|m| m.close_popup());
                            *self.show_about = false;
                        }
                    });
                });
            }
        }
        about_btn
    }
}

pub struct Menu {
    window_open: bool, // Only show the egui window when true.
    info_message: Info,
    filter_string: String,
    are_tools_active: bool,
    editable_ssh_cfg_str: String,
    scroll_offset: f32,
    open_folder_popup_open: bool,
    load_button_resp: ImportBtnResp,
    stats: Stats,
    filename_sort_type: SortType,
    show_about: bool,
}

impl Menu {
    fn new() -> Self {
        let (cfg, _) = get_cfg();
        let ssh_cfg_str = toml::to_string_pretty(&cfg.ssh_cfg).unwrap();
        Self {
            window_open: true,
            info_message: Info::None,
            filter_string: "".to_string(),
            are_tools_active: true,
            editable_ssh_cfg_str: ssh_cfg_str,
            scroll_offset: 0.0,
            open_folder_popup_open: false,
            load_button_resp: ImportBtnResp {
                resp: None,
                popup_open: false,
            },
            stats: Stats::default(),
            filename_sort_type: SortType::default(),
            show_about: false,
        }
    }
    pub fn sort_type(&self) -> SortType {
        self.filename_sort_type
    }
    pub fn popup(&mut self, info: Info) {
        self.info_message = info;
    }

    pub fn toggle(&mut self) {
        if self.window_open {
            self.are_tools_active = true;
            self.window_open = false;
        } else {
            self.window_open = true;
        }
    }

    pub fn reload_opened_folder(&mut self, ctrl: &mut Control) {
        if let Err(e) = ctrl.load_opened_folder_content(self.filename_sort_type) {
            self.info_message = Info::Error(format!("{e:?}"));
        }
    }

    pub fn show_info(&mut self, msg: Info) {
        self.info_message = msg;
    }

    /// Create the UI using egui.
    pub fn ui(&mut self, ctx: &Context, ctrl: &mut Control, tools_data_map: &mut ToolsDataMap) {
        egui::TopBottomPanel::top("top-menu-bar").show(ctx, |ui| {
            // Top row with open folder and settings button
            egui::menu::bar(ui, |ui| {
                let button_resp = open_folder::button(ui, ctrl, self.open_folder_popup_open);
                handle_error!(
                    |open| {
                        self.open_folder_popup_open = open;
                    },
                    button_resp,
                    self
                );
                let popup_id = ui.make_persistent_id("cfg-popup");
                self.load_button_resp.resp = Some(ui.button("load project"));

                if ui.button("save project").clicked() {
                    handle_error!(ctrl.save(tools_data_map), self);
                }

                let cfg_gui = CfgMenu::new(popup_id, &mut ctrl.cfg, &mut self.editable_ssh_cfg_str);
                ui.add(cfg_gui);
                let about_popup_id = ui.make_persistent_id("about-popup");
                ui.add(About::new(about_popup_id, &mut self.show_about));
            });
        });

        egui::SidePanel::left("left-main-menu").show(ctx, |ui| {
            if let Ok(folder) = ctrl.cfg.export_folder() {
                if let Some(load_btn_resp) = &self.load_button_resp.resp {
                    if load_btn_resp.clicked() {
                        self.load_button_resp.popup_open = true;
                    }
                    if self.load_button_resp.popup_open {
                        let mut filename_for_import = None;
                        let mut exports = || -> RvResult<()> {
                            let files = file_util::files_in_folder(folder, RVPRJ_PREFIX, "json")
                                .map_err(to_rv)?
                                .filter_map(|p| {
                                    p.file_name().map(|p| p.to_str().map(|p| p.to_string()))
                                })
                                .flatten()
                                .collect::<Vec<_>>();
                            if !files.is_empty() {
                                filename_for_import = picklist::pick(
                                    ui,
                                    files.iter().map(|s| s.as_str()),
                                    200.0,
                                    load_btn_resp,
                                )
                                .map(|s| s.to_string());
                            } else {
                                println!("no projects found that can be loaded")
                            }
                            Ok(())
                        };
                        handle_error!(exports(), self);
                        if let Some(filename) = filename_for_import {
                            handle_error!(
                                |tdm| {
                                    *tools_data_map = tdm;
                                },
                                ctrl.load(&filename),
                                self
                            );
                            self.load_button_resp.resp = None;
                            self.load_button_resp.popup_open = false;
                        }
                    }
                }
            }
            let mut connected = false;
            handle_error!(
                |con| {
                    connected = con;
                },
                ctrl.check_if_connected(self.filename_sort_type),
                self
            );
            if connected {
                ui.label(ctrl.opened_folder_label().unwrap_or(""));
            } else {
                ui.label("connecting...");
            }

            let filter_txt_field = ui.text_edit_singleline(&mut self.filter_string);
            if filter_txt_field.gained_focus() {
                self.are_tools_active = false;
            }
            if filter_txt_field.lost_focus() {
                self.are_tools_active = true;
            }
            if filter_txt_field.changed() {
                handle_error!(
                    ctrl.paths_navigator
                        .filter(&self.filter_string, tools_data_map),
                    self
                );
            }
            // Popup for error messages
            let popup_id = ui.make_persistent_id("info-popup");
            self.info_message = match &self.info_message {
                Info::Warning(msg) => show_popup(
                    ui,
                    msg,
                    "❕",
                    popup_id,
                    self.info_message.clone(),
                    &filter_txt_field,
                ),
                Info::Error(msg) => show_popup(
                    ui,
                    msg,
                    "❌",
                    popup_id,
                    self.info_message.clone(),
                    &filter_txt_field,
                ),
                Info::None => Info::None,
            };

            // scroll area showing image file names
            let scroll_to_selected = ctrl.paths_navigator.scroll_to_selected_label();
            let mut filtered_label_selected_idx = ctrl.paths_navigator.file_label_selected_idx();
            if let Some(ps) = &ctrl.paths_navigator.paths_selector() {
                self.scroll_offset = menu::scroll_area::scroll_area(
                    ui,
                    &mut filtered_label_selected_idx,
                    ps,
                    ctrl.file_info_selected.as_deref(),
                    scroll_to_selected,
                    self.scroll_offset,
                );
                ctrl.paths_navigator.deactivate_scroll_to_selected_label();
                if ctrl.paths_navigator.file_label_selected_idx() != filtered_label_selected_idx {
                    ctrl.paths_navigator
                        .select_label_idx(filtered_label_selected_idx);
                }
            }

            ui.separator();
            let clicked_nat = ui
                .radio_value(
                    &mut self.filename_sort_type,
                    SortType::Natural,
                    "natural sorting",
                )
                .clicked();
            let clicked_alp = ui
                .radio_value(
                    &mut self.filename_sort_type,
                    SortType::Alphabetical,
                    "alphabetical sorting",
                )
                .clicked();
            if clicked_nat || clicked_alp {
                handle_error!(
                    |_| {},
                    ctrl.sort(self.filename_sort_type, &self.filter_string, tools_data_map),
                    self
                );
                handle_error!(|_| {}, ctrl.reload(self.filename_sort_type), self);
            }
            if let Some(info) = &self.stats.n_files_filtered_info {
                ui.label(info);
            }
            if let Some(info) = &self.stats.n_files_annotated_info {
                ui.label(info);
            }
            let get_file_info = |ps: &PathsSelector| {
                let n_files_filtered = ps.len_filtered();
                Some(format!("{n_files_filtered} files"))
            };
            let get_annotation_info = |ps: &PathsSelector| {
                if let Some(bbox_data) = tools_data_map.get(BBOX_NAME) {
                    let n_files_annotated = bbox_data
                        .specifics
                        .bbox()
                        .n_annotated_images(&ps.filtered_file_paths());
                    Some(format!("{n_files_annotated} files with bbox annotations"))
                } else {
                    None
                }
            };
            if let Some(ps) = ctrl.paths_navigator.paths_selector() {
                if self.stats.n_files_filtered_info.is_none() {
                    self.stats.n_files_filtered_info = get_file_info(ps);
                }
                if self.stats.n_files_annotated_info.is_none() {
                    self.stats.n_files_annotated_info = get_annotation_info(ps);
                }
                if ui.button("re-compute stats").clicked() {
                    self.stats.n_files_filtered_info = get_file_info(ps);
                    self.stats.n_files_annotated_info = get_annotation_info(ps);
                }
            } else {
                self.stats.n_files_filtered_info = None;
                self.stats.n_files_annotated_info = None;
            }
        });
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

pub fn are_tools_active(menu: &Menu, tsm: &ToolSelectMenu) -> bool {
    menu.are_tools_active && tsm.are_tools_active
}

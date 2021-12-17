use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;

use eframe::egui::*;

use crate::backend::DemoBackend;
use crate::geometry::{SKILL_BOX, SKILL_IMAGE, SKILL_OFFSET_H, SKILL_TEXT};
use crate::skill::Skill;
use crate::spec::Spec;
use crate::utils::{selected_frame_around, RawImage};
use crate::WINDOW_SIZE;

fn toggle_popup(ui: &Ui, widget_response: &Response) -> Option<Id> {
    let popup_id = ui.make_persistent_id(widget_response.id.with("popup"));
    if widget_response.clicked() {
        ui.memory().toggle_popup(popup_id);
    }
    if !ui.memory().is_popup_open(popup_id) {
        None
    } else {
        Some(popup_id)
    }
}

fn popup_ui(
    ui: &Ui,
    popup_id: Id,
    pos: Pos2,
    width: f32,
    inner_ui: impl FnOnce(&mut Ui),
) -> Response {
    Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            let frame = Frame::popup(ui.style());
            let frame_margin = frame.margin;
            frame
                .show(ui, |ui| {
                    ui.set_width(width - 2. * frame_margin.x);
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), inner_ui)
                })
                .inner
        })
        .inner
        .response
}

fn type_checking_layouter<T: FromStr>(ui: &Ui, string: &str, _wrap_width: f32) -> Arc<Galley> {
    let color = if string.parse::<T>().is_ok() {
        Color32::WHITE
    } else {
        Color32::RED
    };
    let job = text::LayoutJob::simple_singleline(string.to_string(), text::TextStyle::Body, color);
    ui.fonts().layout_job(job)
}

fn right_to_left() -> Layout {
    Layout::right_to_left().with_cross_align(Align::LEFT)
}

pub fn show_pskill_popup(ui: &mut Ui, widget_response: Response, value: u8) -> Option<u8> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut mem_val = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || value.to_string())
        .clone();

    let mut layouter = type_checking_layouter::<u8>;
    let pos = widget_response.rect.left_bottom();
    let width = widget_response.rect.width();
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        let edit = TextEdit::singleline(&mut mem_val).layouter(&mut layouter);
        ui.add(edit).request_focus();
        ui.memory().data.insert_temp(popup_id, mem_val.clone());
        if ui.button("   ✅").clicked() || ui.input().key_pressed(Key::Enter) {
            let parsed_value = mem_val.parse::<u8>();
            if let Ok(new_value) = parsed_value {
                return_val = Some(new_value);
            }
        };
    });

    let close_condition = ui.input().key_pressed(Key::Escape)
        || (widget_response.clicked_elsewhere() && inner.clicked_elsewhere())
        || return_val.is_some();
    if close_condition {
        ui.memory().close_popup();
        ui.memory().data.remove::<String>(popup_id);
    }
    return_val
}

pub fn show_xp_popup(ui: &mut Ui, widget_response: Response, value: u16) -> Option<u16> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut mem_val = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || value.to_string())
        .clone();

    let mut layouter = type_checking_layouter::<u16>;
    let pos = SKILL_IMAGE.right_top() + vec2(4., -6.);
    let width = SKILL_TEXT.width();
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        let button_response = ui
            .horizontal(|ui| {
                ui.label("Опыт");
                ui.add_space(22.);
                ui.spacing_mut().button_padding.x += 2.0;
                ui.button("✅")
            })
            .inner;

        let edit = TextEdit::singleline(&mut mem_val).layouter(&mut layouter);
        ui.add(edit).request_focus();
        ui.memory().data.insert_temp(popup_id, mem_val.clone());
        if button_response.clicked() || ui.input().key_pressed(Key::Enter) {
            let parsed_value = mem_val.parse::<u16>();
            if let Ok(new_value) = parsed_value {
                return_val = Some(new_value);
            }
        }
    });

    let close_condition = ui.input().key_pressed(Key::Escape)
        || (widget_response.clicked_elsewhere() && inner.clicked_elsewhere())
        || return_val.is_some();
    if close_condition {
        ui.memory().close_popup();
        ui.memory().data.remove::<String>(popup_id);
    }
    return_val
}

pub fn show_mana_popup(
    ui: &mut Ui,
    widget_response: Response,
    current_value: u16,
    max_value: u16,
) -> Option<(u16, u16)> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;

    let mut mem_vals = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || {
            (current_value.to_string(), max_value.to_string())
        })
        .clone();

    let mut max_layouter = type_checking_layouter::<u16>;
    let pos = SKILL_IMAGE.translate(SKILL_OFFSET_H).right_top() + vec2(4., -6.);
    let width = SKILL_TEXT.width();
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        let button_response = ui
            .horizontal(|ui| {
                ui.label("Очки магии");
                ui.spacing_mut().button_padding.x += 2.;
                ui.button("✅")
            })
            .inner;
        ui.with_layout(right_to_left(), |ui| {
            ui.spacing_mut().item_spacing.x = 2.;
            let edit_max = TextEdit::singleline(&mut mem_vals.1)
                .layouter(&mut max_layouter)
                .desired_width(36.);
            ui.add(edit_max);

            ui.label("/");

            let mut current_layouter = |ui: &Ui, string: &str, _wrap_width: f32| {
                let color = if let Ok(parsed_current) = string.parse::<u16>() {
                    if let Ok(parsed_max) = mem_vals.1.parse::<u16>() {
                        if parsed_current > parsed_max {
                            Color32::RED
                        } else {
                            Color32::WHITE
                        }
                    } else {
                        Color32::RED
                    }
                } else {
                    Color32::RED
                };
                let job = text::LayoutJob::simple_singleline(
                    string.to_string(),
                    text::TextStyle::Body,
                    color,
                );
                ui.fonts().layout_job(job)
            };
            let edit_current = TextEdit::singleline(&mut mem_vals.0)
                .layouter(&mut current_layouter)
                .desired_width(36.);
            ui.add(edit_current);

            ui.memory().data.insert_temp(popup_id, mem_vals.clone());
        });
        if button_response.clicked() || ui.input().key_pressed(Key::Enter) {
            if let (Ok(current), Ok(max)) = (mem_vals.0.parse::<u16>(), mem_vals.1.parse::<u16>()) {
                if max > current {
                    return_val = Some((current, max));
                }
            }
        };
    });

    let close_condition = ui.input().key_pressed(Key::Escape)
        || (widget_response.clicked_elsewhere() && inner.clicked_elsewhere())
        || return_val.is_some();
    if close_condition {
        ui.memory().close_popup();
        ui.memory().data.remove::<String>(popup_id);
    }
    return_val
}

fn show_closable_window(
    ui: &mut Ui,
    response: Option<Response>,
    name: &str,
    force_open: bool,
    add_contents: impl FnOnce(&mut Ui, &mut bool),
) {
    let open_window_id = ui.make_persistent_id("open_window");
    let open_window_name = ui
        .memory()
        .data
        .get_temp_mut_or_default::<String>(open_window_id)
        .clone();
    let mut is_open = open_window_name == name;
    if force_open {
        is_open = true;
    }
    if let Some(response) = &response {
        if response.clicked() {
            is_open = !is_open;
        }
        if is_open {
            selected_frame_around(ui, response.rect);
        }
    }
    let response_side = if let Some(response) = &response {
        (response.rect.left_top().x / WINDOW_SIZE.x).round()
    } else {
        (WINDOW_SIZE.x - SKILL_BOX.width()) / 2.0
    };

    let window = Window::new(name)
        .open(&mut is_open)
        .default_pos(pos2((WINDOW_SIZE.x * (1. - response_side)) / 2.0, 0.0));

    let mut close_from_child = false;
    window.show(ui.ctx(), |ui| {
        ui.set_width(SKILL_BOX.width());
        add_contents(ui, &mut close_from_child)
    });

    if ui.input().key_pressed(Key::Escape) || close_from_child {
        is_open = false;
    }
    if is_open {
        ui.memory()
            .data
            .insert_temp(open_window_id, name.to_string());
    } else if !is_open && name == open_window_name {
        ui.memory().data.remove::<String>(open_window_id);
    }
}

pub fn show_selection_window(
    ui: &mut Ui,
    response: Response,
    name: &str,
    add_contents: impl FnOnce(&mut Ui),
) {
    show_closable_window(ui, Some(response), name, false, |ui, _| add_contents(ui))
}

pub fn show_selectable_block(
    ui: &mut Ui,
    image: &RawImage,
    text: impl Into<WidgetText>,
    selected: bool,
) -> (Response, Response) {
    ui.horizontal(|ui| {
        ui.set_height(SKILL_BOX.height());
        let background_shape = ui.painter().add(Shape::Noop);
        ui.image(image.texture_id.unwrap_or_default(), SKILL_IMAGE.size());
        let id = ui.label(text).id;
        let edit_reponse = ui.button("📝");
        let box_response = ui.interact(ui.max_rect(), ui.id().with(id), Sense::click());
        let visuals = ui.style().interact_selectable(&box_response, selected);
        let background_rect = epaint::RectShape {
            rect: ui.max_rect(),
            corner_radius: 0.,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        };
        ui.painter().set(background_shape, background_rect);
        (edit_reponse, box_response)
    })
    .inner
}

pub fn show_selectable_block_no_edit(
    ui: &mut Ui,
    image: &RawImage,
    text: impl Into<WidgetText>,
    selected: bool,
) -> Response {
    ui.horizontal(|ui| {
        ui.set_height(SKILL_BOX.height());
        let background_shape = ui.painter().add(Shape::Noop);
        ui.image(image.texture_id.unwrap_or_default(), SKILL_IMAGE.size());
        let id = ui.label(text).id;
        let box_response = ui.interact(ui.max_rect(), ui.id().with(id), Sense::click());
        let visuals = ui.style().interact_selectable(&box_response, selected);
        let background_rect = epaint::RectShape {
            rect: ui.max_rect(),
            corner_radius: 0.,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        };
        ui.painter().set(background_shape, background_rect);
        box_response
    })
    .inner
}

pub fn show_skill_edit_window(
    ui: &mut Ui,
    skill: Option<Option<Skill>>,
    backend: &mut DemoBackend,
    frame: &mut eframe::epi::Frame,
) {
    let force_open = skill.is_some();
    show_closable_window(ui, None, "skill_edit", force_open, |ui, close_window| {
        let (mut skill_in_edit, is_new_skill) = if let Some(skill) = skill {
            let is_new = skill.is_none();
            (skill.unwrap_or_default(), is_new)
        } else {
            ui.memory()
                .data
                .get_temp_mut_or_default::<(Skill, bool)>(ui.id())
                .clone()
        };

        Grid::new("grid").num_columns(2).show(ui, |ui| {
            if !is_new_skill {
                ui.label("ID");
                ui.label(&skill_in_edit.id.to_string());
                ui.end_row();
            }

            ui.label("Название");
            ui.text_edit_singleline(&mut skill_in_edit.name);
            ui.end_row();

            ui.label("Ступень");
            ui.add(DragValue::new(&mut skill_in_edit.level).clamp_range(1..=3));
            ui.end_row();

            ui.label("Картинка");
            if ui.add(skill_in_edit.image.image_button()).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let bytes = std::fs::File::open(path)
                        .unwrap()
                        .bytes()
                        .map(|b| b.unwrap())
                        .collect::<Vec<_>>();
                    skill_in_edit.image.load_bytes(&bytes, frame);
                }
            };
            ui.end_row()
        });

        if ui.button("🆗").clicked() {
            if is_new_skill {
                backend.create_skill(
                    &skill_in_edit.name,
                    skill_in_edit.level,
                    &skill_in_edit.image.bytes,
                );
            } else {
                backend.modify_skill(
                    skill_in_edit.id,
                    &skill_in_edit.name,
                    skill_in_edit.level,
                    &skill_in_edit.image.bytes,
                );
            }
            *close_window = true;
        }

        ui.memory()
            .data
            .insert_temp(ui.id(), (skill_in_edit, is_new_skill));
    });
}

pub fn show_spec_edit_window(
    ui: &mut Ui,
    spec: Option<Option<Spec>>,
    backend: &mut DemoBackend,
    frame: &mut eframe::epi::Frame,
) {
    let force_open = spec.is_some();
    show_closable_window(ui, None, "spec_edit", force_open, |ui, close_window| {
        let mut spec_in_edit = if let Some(spec) = spec {
            spec.unwrap_or_default()
        } else {
            ui.memory()
                .data
                .get_temp_mut_or_default::<Spec>(ui.id())
                .clone()
        };

        let classes = backend.get_classes();
        Grid::new("grid").num_columns(2).show(ui, |ui| {
            ui.label("Название");
            ui.text_edit_singleline(&mut spec_in_edit.name);
            ui.end_row();

            ui.label("Класс");
            if let Some(classes) = classes {
                ComboBox::from_id_source("")
                    .selected_text(&spec_in_edit.class)
                    .show_ui(ui, |ui| {
                        for class in classes {
                            ui.selectable_value(&mut spec_in_edit.class, class.to_string(), &class);
                        }
                    });
            }
            ui.end_row();

            ui.label("Картинка");
            if ui.add(spec_in_edit.image.image_button()).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let bytes = std::fs::File::open(path)
                        .unwrap()
                        .bytes()
                        .map(|b| b.unwrap())
                        .collect::<Vec<_>>();
                    spec_in_edit.image.load_bytes(&bytes, frame);
                }
            }
        });

        if ui.button("🆗").clicked() {
            backend.create_or_modify_spec(
                &spec_in_edit.name,
                &spec_in_edit.class,
                &spec_in_edit.image.bytes,
            );
            *close_window = true;
        }

        ui.memory().data.insert_temp(ui.id(), spec_in_edit);
    });
}

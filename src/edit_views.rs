use std::str::FromStr;
use std::sync::Arc;

use eframe::egui::*;

use crate::backpack::{Item, ItemSlot};
use crate::geometry::{SKILL_BOX, SKILL_OFFSET_H, SKILL_OFFSET_V, SKILL_TEXT};
use crate::skill::Skill;
use crate::spec::Spec;
use crate::unit::Unit;

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

fn checking_layouter<T: FromStr>(ui: &Ui, string: &str, _wrap_width: f32) -> Arc<Galley> {
    let color = if string.parse::<T>().is_ok() {
        Color32::WHITE
    } else {
        Color32::RED
    };
    let job = text::LayoutJob::simple_singleline(string.to_string(), text::TextStyle::Body, color);
    ui.fonts().layout_job(job)
}

pub fn edit_pskill_popup(ui: &mut Ui, widget_response: Response, value: u8) -> Option<u8> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut remembered_value = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || format!("{}", value))
        .clone();

    let mut layouter = checking_layouter::<u8>;
    let pos = widget_response.rect.left_bottom();
    let width = widget_response.rect.width();
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        let edit = TextEdit::singleline(&mut remembered_value).layouter(&mut layouter);
        ui.add(edit).request_focus();
        ui.memory()
            .data
            .insert_temp(popup_id, remembered_value.clone());
        if ui.button("   ✅").clicked() || ui.input().key_pressed(Key::Enter) {
            let parsed_value = remembered_value.parse::<u8>();
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

pub fn edit_xp_popup(ui: &mut Ui, widget_response: Response, value: u16) -> Option<u16> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut remembered_value = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || format!("{}", value))
        .clone();

    let mut layouter = checking_layouter::<u16>;
    let pos = SKILL_TEXT.left_top();
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

        let edit = TextEdit::singleline(&mut remembered_value).layouter(&mut layouter);
        ui.add(edit).request_focus();
        ui.memory()
            .data
            .insert_temp(popup_id, remembered_value.clone());
        if button_response.clicked() || ui.input().key_pressed(Key::Enter) {
            let parsed_value = remembered_value.parse::<u16>();
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

pub fn edit_mana_popup(
    ui: &mut Ui,
    widget_response: Response,
    current_value: u16,
    max_value: u16,
) -> Option<(u16, u16)> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;

    let mut remembered_values = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || {
            (current_value.to_string(), max_value.to_string())
        })
        .clone();

    let mut max_layouter = checking_layouter::<u16>;
    let pos = SKILL_TEXT.translate(SKILL_OFFSET_H).left_top();
    let width = SKILL_TEXT.width();
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        let button_response = ui
            .horizontal(|ui| {
                ui.label("Очки магии");
                ui.spacing_mut().button_padding.x += 2.0;
                ui.button("✅")
            })
            .inner;
        ui.with_layout(
            Layout::right_to_left().with_cross_align(Align::LEFT),
            |ui| {
                let edit_max = TextEdit::singleline(&mut remembered_values.1)
                    .layouter(&mut max_layouter)
                    .desired_width(36.);
                ui.add(edit_max);

                ui.label("/");

                let mut current_layouter = |ui: &Ui, string: &str, _wrap_width: f32| {
                    let color = if let Ok(parsed_current) = string.parse::<u16>() {
                        if let Ok(parsed_max) = remembered_values.1.parse::<u16>() {
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
                let edit_current = TextEdit::singleline(&mut remembered_values.0)
                    .layouter(&mut current_layouter)
                    .desired_width(36.);
                ui.add(edit_current);

                ui.memory()
                    .data
                    .insert_temp(popup_id, remembered_values.clone());
            },
        );
        if button_response.clicked() || ui.input().key_pressed(Key::Enter) {
            if let (Ok(current), Ok(max)) = (
                remembered_values.0.parse::<u16>(),
                remembered_values.1.parse::<u16>(),
            ) {
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

pub fn edit_spec_window(
    ui: &mut Ui,
    widget_response: Response,
    avalible_specs: &[Spec],
    current_spec: &Spec,
) -> Option<Spec> {
    let window_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;

    let first_run = ui.memory().data.get_temp::<String>(window_id).is_none();
    let mut remembered_search_query = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(window_id, || String::new())
        .clone();

    let pos = SKILL_BOX.translate(-SKILL_OFFSET_V).left_bottom();
    let width = SKILL_BOX.width() + ui.spacing().scroll_bar_width;
    let inner = popup_ui(ui, window_id, pos, width, |ui| {
        ui.with_layout(
            Layout::right_to_left().with_cross_align(Align::LEFT),
            |ui| {
                ui.spacing_mut().button_padding.x += 2.0;
                let button_response = ui.button("➕");
                let edit = TextEdit::singleline(&mut remembered_search_query);
                let search_response = ui.add(edit);
                if first_run {
                    search_response.request_focus();
                }
            },
        );
        ui.memory()
            .data
            .insert_temp(window_id, remembered_search_query.clone());
        ui.add_space(ui.spacing().item_spacing.y);
        ScrollArea::vertical().show_rows(ui, SKILL_BOX.height(), 100, |ui, range| {
            for x in 1..100 {
                // ui.set_width(width);
                if range.contains(&x) {
                    ui.horizontal(|ui| {
                        ui.add(current_spec.image.image());
                        ui.vertical(|ui| {
                            ui.label("Nameaaaaaaaaaa");
                        })
                    });
                }
            }
        });
    });

    let close_condition = ui.input().key_pressed(Key::Escape)
        || (widget_response.clicked_elsewhere() && inner.clicked_elsewhere())
        || return_val.is_some();
    if close_condition {
        ui.memory().close_popup();
        ui.memory().data.remove::<String>(window_id);
    }
    return_val
}

pub fn edit_skill_window(
    ui: &mut Ui,
    avalible_skills: &[Skill],
    idx: usize,
) -> Option<Option<Skill>> {
    unimplemented!()
}

pub fn edit_item_window(ui: &mut Ui, avaliable_items: &[Item], slot: ItemSlot) -> Option<Item> {
    unimplemented!()
}

pub fn edit_unit_window(
    ui: &mut Ui,
    avaliable_units: &[Unit],
    idx: usize,
    selected: &Unit,
    number: u16,
) -> Option<(Unit, u16)> {
    unimplemented!()
}

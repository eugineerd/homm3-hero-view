use std::str::FromStr;
use std::sync::Arc;

use eframe::egui::*;

use crate::backpack::{Item, ItemSlot};
use crate::geometry::{SKILL_BOX, SKILL_IMAGE, SKILL_OFFSET_H, SKILL_OFFSET_V, SKILL_TEXT};
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

fn right_to_left() -> Layout {
    Layout::right_to_left().with_cross_align(Align::LEFT)
}

pub fn edit_pskill_popup(ui: &mut Ui, widget_response: Response, value: u8) -> Option<u8> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut mem_val = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || value.to_string())
        .clone();

    let mut layouter = checking_layouter::<u8>;
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

pub fn edit_xp_popup(ui: &mut Ui, widget_response: Response, value: u16) -> Option<u16> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;
    let mut mem_val = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || value.to_string())
        .clone();

    let mut layouter = checking_layouter::<u16>;
    let pos = SKILL_IMAGE.right_top() + vec2(4., -6.); //SKILL_TEXT.left_top();
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

pub fn edit_mana_popup(
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

    let mut max_layouter = checking_layouter::<u16>;
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

pub fn edit_spec_popup(
    ui: &mut Ui,
    widget_response: Response,
    avalible_specs: &[Spec],
    current_spec: &Spec,
) -> Option<Spec> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;

    let first_run = ui.memory().data.get_temp::<String>(popup_id).is_none();
    let mut mem_query = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || String::new())
        .clone();

    let pos = SKILL_BOX.translate(-SKILL_OFFSET_V).left_bottom();
    let width = SKILL_BOX.width() + ui.spacing().scroll_bar_width;
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        ui.with_layout(right_to_left(), |ui| {
            let edit = TextEdit::singleline(&mut mem_query);
            let search_response = ui.add(edit);
            if first_run {
                search_response.request_focus();
            }
        });
        ui.memory().data.insert_temp(popup_id, mem_query.clone());

        ui.add_space(ui.spacing().item_spacing.y);

        let filtered_specs = avalible_specs
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&mem_query.to_lowercase()))
            .collect::<Vec<_>>();
        let rows = filtered_specs.len();
        if rows == 1 && ui.input().key_pressed(Key::Enter) {
            return_val = Some(filtered_specs[0].clone());
        }

        ScrollArea::vertical().show_rows(ui, SKILL_BOX.height(), rows, |ui, range| {
            for i in range {
                let resp = ui
                    .horizontal_wrapped(|ui| {
                        let background = ui.painter().add(Shape::Noop);
                        ui.add(filtered_specs[i].image.image());
                        ui.label(&filtered_specs[i].name);
                        if ui.rect_contains_pointer(ui.max_rect())
                            || filtered_specs[i] == current_spec
                        {
                            let bg_color = if filtered_specs[i] == current_spec {
                                Color32::DARK_GRAY
                            } else {
                                ui.visuals().widgets.hovered.bg_fill
                            };
                            let background_shape = epaint::RectShape {
                                rect: ui.max_rect(),
                                corner_radius: 0.,
                                fill: bg_color,
                                stroke: ui.visuals().widgets.open.bg_stroke,
                            };
                            ui.painter().set(background, background_shape);
                        }
                    })
                    .response;
                if ui
                    .interact(resp.rect, resp.id.with(i), Sense::click())
                    .clicked()
                {
                    return_val = Some(filtered_specs[i].clone())
                };
            }
        });
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

// pub fn edit_spec_window(
//     ui: &mut Ui,
//     widget_response: Response,
//     avalible_specs: &[Spec],
//     current_spec: &Spec,
// ) -> Option<Spec> {
//     let window_name = "Специальность";
//     let window_id = widget_response.id.with(window_name);
//     let mut is_open = ui.memory().data.get_temp_mut_or(window_id, true).clone();
//     let window = Window::new(window_name).id(window_id).open(&mut is_open);

//     let mut return_val = None;

//     let mut mem_query = ui
//         .memory()
//         .data
//         .get_temp_mut_or_insert_with(popup_id, || String::new())
//         .clone();

//     // let pos = SKILL_BOX.translate(-SKILL_OFFSET_V).left_bottom();
//     // let width = SKILL_BOX.width() + ui.spacing().scroll_bar_width;
//     // let inner = popup_ui(ui, popup_id, pos, width, |ui| {
//     window.show(ui.ctx(), |ui| {
//         ui.with_layout(right_to_left(), |ui| {
//             let edit = TextEdit::singleline(&mut mem_query);
//             let search_response = ui.add(edit);
//             if first_run {
//                 search_response.request_focus();
//             }
//         });
//         ui.memory().data.insert_temp(popup_id, mem_query.clone());

//         ui.add_space(ui.spacing().item_spacing.y);

//         let filtered_specs = avalible_specs
//             .iter()
//             .filter(|s| s.name.to_lowercase().contains(&mem_query.to_lowercase()))
//             .collect::<Vec<_>>();
//         let rows = filtered_specs.len();
//         if rows == 1 && ui.input().key_pressed(Key::Enter) {
//             return_val = Some(filtered_specs[0].clone());
//         }

//         ScrollArea::vertical().show_rows(ui, SKILL_BOX.height(), rows, |ui, range| {
//             for i in range {
//                 let resp = ui
//                     .horizontal_wrapped(|ui| {
//                         let background = ui.painter().add(Shape::Noop);
//                         ui.add(filtered_specs[i].image.image());
//                         ui.label(&filtered_specs[i].name);
//                         if ui.rect_contains_pointer(ui.max_rect())
//                             || filtered_specs[i] == current_spec
//                         {
//                             let bg_color = if filtered_specs[i] == current_spec {
//                                 Color32::DARK_GRAY
//                             } else {
//                                 ui.visuals().widgets.hovered.bg_fill
//                             };
//                             let background_shape = epaint::RectShape {
//                                 rect: ui.max_rect(),
//                                 corner_radius: 0.,
//                                 fill: bg_color,
//                                 stroke: ui.visuals().widgets.open.bg_stroke,
//                             };
//                             ui.painter().set(background, background_shape);
//                         }
//                     })
//                     .response;
//                 if ui
//                     .interact(resp.rect, resp.id.with(i), Sense::click())
//                     .clicked()
//                 {
//                     return_val = Some(filtered_specs[i].clone())
//                 };
//             }
//         });
//     });

//     let close_condition = ui.input().key_pressed(Key::Escape) || return_val.is_some();
//     if close_condition {
//         ui.memory().data.insert_temp(window_id, is_open);
//         ui.memory().data.remove::<String>(window_id);
//     }
//     return_val
// }

pub fn edit_skill_popup(
    ui: &mut Ui,
    widget_response: Response,
    avalible_skills: &[Skill],
    current_skill: &Option<Skill>,
    offset: Vec2,
) -> Option<Option<Skill>> {
    let popup_id = toggle_popup(ui, &widget_response)?;

    let mut return_val = None;

    let first_run = ui.memory().data.get_temp::<String>(popup_id).is_none();
    let mut mem_query = ui
        .memory()
        .data
        .get_temp_mut_or_insert_with(popup_id, || String::new())
        .clone();

    let pos = SKILL_BOX.translate(offset).left_bottom();
    let width = SKILL_BOX.width()
        + ui.spacing().scroll_bar_width
        + ui.spacing().item_spacing.x
        + ui.spacing().window_padding.x;
    let inner = popup_ui(ui, popup_id, pos, width, |ui| {
        ui.with_layout(right_to_left(), |ui| {
            ui.spacing_mut().button_padding.x += 2.0;
            let remove_button_response = ui.button("❌");
            let edit = TextEdit::singleline(&mut mem_query);
            let search_response = ui.add(edit);
            if first_run {
                search_response.request_focus();
            }
            if remove_button_response.clicked() {
                return_val = Some(None);
            }
        });
        ui.memory().data.insert_temp(popup_id, mem_query.clone());

        ui.add_space(ui.spacing().item_spacing.y);
        let filtered_skills = avalible_skills
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&mem_query.to_lowercase()))
            .collect::<Vec<_>>();
        let rows = filtered_skills.len();
        if rows == 1 && ui.input().key_pressed(Key::Enter) {
            return_val = Some(Some(filtered_skills[0].clone()));
        }

        let scroll_area = ScrollArea::vertical().auto_shrink([false, true]);
        scroll_area.show_rows(ui, SKILL_BOX.height(), rows, |ui, range| {
            for i in range {
                let resp = ui
                    .horizontal(|ui| {
                        let background = ui.painter().add(Shape::Noop);
                        ui.add(filtered_skills[i].image.image());
                        ui.vertical(|ui| {
                            ui.label(&filtered_skills[i].name);
                            ui.label(&format!("{} ступени", filtered_skills[i].level));
                        });
                        if ui.rect_contains_pointer(ui.max_rect()) {
                            let background_shape = epaint::RectShape {
                                rect: ui.max_rect(),
                                corner_radius: 0.,
                                fill: ui.visuals().widgets.hovered.bg_fill,
                                stroke: ui.visuals().widgets.open.bg_stroke,
                            };
                            ui.painter().set(background, background_shape);
                        }
                    })
                    .response;
                if ui
                    .interact(resp.rect, resp.id.with(i), Sense::click())
                    .clicked()
                {
                    return_val = Some(Some(filtered_skills[i].clone()))
                };
            }
        });
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
